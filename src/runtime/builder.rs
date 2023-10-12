use std::collections::HashSet;

use miette::{NamedSource, Result, SourceOffset, SourceSpan};

use crate::{
    base::{Accumulator, Comparison, MemoryCell, Operation},
    cli::Cli,
    instructions::{
        error_handling::{BuildProgramError, BuildProgramErrorTypes, InstructionParseError},
        Identifier, IndexMemoryCellIndexType, Instruction, TargetType, Value,
    },
    utils::remove_comment,
};

use super::{
    error_handling::{AddLabelError, RuntimeBuildError},
    ControlFlow, Runtime, RuntimeArgs,
};

/// Type that is used to build a new runtime environment.
///
/// This runtime can be configured to only allow a selected amount of accumulators memory cells, operations and comparisons.
/// When a runtime is build from this builder compatibility checks are performed.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct RuntimeBuilder {
    runtime_args: Option<RuntimeArgs>,
    instructions: Option<Vec<Instruction>>,
    control_flow: ControlFlow,
    add_missing: bool,
    /// If set to `None` all comparisons are allowed.
    limit_comparisons_to: Option<Vec<Comparison>>,
    /// If set to 'None' all operations are allowed.
    limit_operations_to: Option<Vec<Operation>>,
}

impl RuntimeBuilder {
    /// Creates a new runtime builder with no values set.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            runtime_args: None,
            instructions: None,
            control_flow: ControlFlow::new(),
            add_missing: false,
            limit_comparisons_to: None,
            limit_operations_to: None,
        }
    }

    /// Creates a new runtime builder from the cli arguments.
    pub fn from_args(args: &Cli) -> Result<Self, String> {
        Ok(Self {
            runtime_args: Some(RuntimeArgs::from_args(args)?),
            instructions: None,
            control_flow: ControlFlow::new(),
            add_missing: !args.disable_memory_detection,
            limit_comparisons_to: args.allowed_comparisons.clone(),
            limit_operations_to: args.allowed_operations.clone(),
        })
    }

    /// Creates a new runtime builder with default values.
    #[allow(dead_code)]
    pub fn new_debug(memory_cells: &[&'static str]) -> Self {
        Self {
            runtime_args: Some(RuntimeArgs::new_debug(memory_cells)),
            instructions: None,
            control_flow: ControlFlow::new(),
            add_missing: false,
            limit_comparisons_to: None,
            limit_operations_to: None,
        }
    }

    /// Constructs a new runtime.
    ///
    /// Performs some compatibility checks.
    /// Set `add_missing` to true to automatically add missing accumulators and memory cells.
    ///
    /// Returns `RuntimeBuildError` when the runtime could not be constructed due to missing information.
    pub fn build(&mut self) -> Result<Runtime, RuntimeBuildError> {
        if self.runtime_args.is_none() {
            return Err(RuntimeBuildError::RuntimeArgsMissing);
        }
        if self.instructions.is_none() || self.instructions.as_ref().unwrap().is_empty() {
            return Err(RuntimeBuildError::InstructionsMissing);
        }
        // Inject end labels to give option to end program by using goto END
        inject_end_labels(
            &mut self.control_flow,
            self.instructions.as_ref().unwrap().len(),
        );
        if let Err(e) = self.check_labels() {
            return Err(RuntimeBuildError::LabelUndefined(e));
        }
        // Check if all used accumulators and memory_cells exist
        self.check_missing_vars(self.add_missing)?;
        // Check if main label is set and update instruction pointer if found
        if let Some(i) = self.control_flow.instruction_labels.get("main") {
            self.control_flow.next_instruction_index = *i;
            self.control_flow.initial_instruction = *i;
        }
        if let Some(i) = self.control_flow.instruction_labels.get("MAIN") {
            self.control_flow.next_instruction_index = *i;
            self.control_flow.initial_instruction = *i;
        }
        Ok(Runtime {
            runtime_args: self.runtime_args.clone().unwrap(),
            instructions: self.instructions.clone().unwrap(),
            control_flow: self.control_flow.clone(),
            instruction_runs: 0,
        })
    }

    /// Resets the current values to none.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.runtime_args = None;
        self.instructions = None;
        self.control_flow.reset();
    }

    #[allow(dead_code)]
    pub fn set_runtime_args(&mut self, runtime_args: RuntimeArgs) {
        self.runtime_args = Some(runtime_args);
    }

    /// Builds instructions from the vector.
    ///
    /// Each element is a single instruction.
    ///
    /// Control flow is reset and updated accordingly.
    ///
    /// If an instruction could not be parsed, an error is returned containing the reason.
    ///
    /// This function was written to outsource the common code from `RuntimeBuilder::build_instructions` and `RuntimeBuilder::build_instructions_whitelist`.
    #[allow(clippy::match_same_arms)]
    fn build_instructions_internal(
        &mut self,
        instructions_input: &[&str],
        file_name: &str,
    ) -> Result<Vec<Instruction>, BuildProgramError> {
        self.control_flow.reset();
        let mut instructions = Vec::new();
        for (index, instruction) in instructions_input.iter().enumerate() {
            // Remove comments
            let instruction = remove_comment(instruction);
            // Check for labels
            let mut splits = instruction.split_whitespace().collect::<Vec<&str>>();
            if splits.is_empty() {
                // Line is empty / line contains comment, add dummy instruction
                instructions.push(Instruction::Noop);
                continue;
            }
            if splits[0].ends_with(':') {
                let label = splits.remove(0).replace(':', "");
                if self
                    .control_flow
                    .instruction_labels
                    .insert(label.clone(), index)
                    .is_some()
                {
                    // main label defined multiple times
                    if label == "main" || label == "MAIN" {
                        Err(BuildProgramError {
                            reason: BuildProgramErrorTypes::MainLabelDefinedMultipleTimes,
                        })?;
                    }
                    // label defined multiple times
                    Err(BuildProgramError {
                        reason: BuildProgramErrorTypes::LabelDefinedMultipleTimes(label),
                    })?;
                }
                if splits.is_empty() {
                    // line contains only label
                    instructions.push(Instruction::Noop);
                    continue;
                }
            }

            match Instruction::try_from(&splits) {
                Ok(i) => instructions.push(i),
                Err(e) => {
                    // Workaround for wrong end_range value depending on error.
                    // For the line to be printed when more then one character is affected for some reason the range needs to be increased by one.
                    let end_range = match e {
                        InstructionParseError::InvalidExpression(_, _) => {
                            e.range().1 - e.range().0 + 1
                        }
                        InstructionParseError::UnknownInstruction(_, _) => {
                            e.range().1 - e.range().0 + 1
                        }
                        InstructionParseError::NotANumber(_, _) => e.range().1 - e.range().0,
                        InstructionParseError::UnknownComparison(_, _) => e.range().1 - e.range().0,
                        InstructionParseError::UnknownOperation(_, _) => e.range().1 - e.range().0,
                        InstructionParseError::MissingExpression { range: _, help: _ } => {
                            e.range().1 - e.range().0
                        }
                    };
                    let file_contents = instructions_input.join("\n");
                    Err(BuildProgramError {
                        reason: BuildProgramErrorTypes::ParseError {
                            src: NamedSource::new(file_name, instructions_input.clone().join("\n")),
                            bad_bit: SourceSpan::new(
                                SourceOffset::from_location(
                                    file_contents.clone(),
                                    index + 1,
                                    e.range().0 + 1,
                                ),
                                SourceOffset::from(end_range),
                            ),
                            reason: e,
                        },
                    })?;
                    //})?
                }
            }
        }
        if self.control_flow.instruction_labels.contains_key("main")
            && self.control_flow.instruction_labels.contains_key("MAIN")
        {
            return Err(BuildProgramError {
                reason: BuildProgramErrorTypes::MainLabelDefinedMultipleTimes,
            });
        }
        Ok(instructions)
    }

    /// Builds instructions from the vector.
    ///
    /// Each element is a single instruction.
    ///
    /// Control flow is reset and updated accordingly.
    ///
    /// If an instruction could not be parsed, an error is returned containing the reason.
    #[allow(clippy::ptr_arg)]
    #[allow(clippy::match_same_arms)]
    pub fn build_instructions(
        &mut self,
        instructions_input: &Vec<&str>,
        file_name: &str,
    ) -> Result<(), BuildProgramError> {
        let instructions = self.build_instructions_internal(instructions_input, file_name)?;
        self.check_instructions(&instructions, None)?;
        self.instructions = Some(instructions);
        Ok(())
    }

    /// Builds instructions from the vector and compares them with a provided whitelist of instructions.
    /// If instructions are found that are not contained in the whitelist, the build will fail and an error is returned.
    ///
    /// The whitelist contains the return value of `Instruction::identifier()`.
    ///
    /// `RuntimeBuilder::check_instructions()` is used for the check if instructions are allowed.
    pub fn build_instructions_whitelist(
        &mut self,
        instructions_input: &[&str],
        file_name: &str,
        whitelist: &HashSet<String>,
    ) -> Result<(), BuildProgramError> {
        let instructions = self.build_instructions_internal(instructions_input, file_name)?;
        self.check_instructions(&instructions, Some(whitelist))?;
        self.instructions = Some(instructions);
        Ok(())
    }

    /// Sets the instructions to the provided instructions.
    ///
    /// If loops and labels are used, they have to be set manually by using [`RuntimeBuilder::add_label`](#add_label).
    #[allow(dead_code)]
    pub fn set_instructions(&mut self, instructions: Vec<Instruction>) {
        self.instructions = Some(instructions);
    }

    /// Adds label to instruction labels.
    ///
    /// Errors when **`instruction_index`** is out of bounds.
    ///
    /// Note: Make sure that you start counting at 0 and not 1!
    #[allow(dead_code)]
    pub fn add_label(
        &mut self,
        label: String,
        instruction_index: usize,
    ) -> Result<(), AddLabelError> {
        if self.instructions.is_none() {
            return Err(AddLabelError::InstructionsNotSet);
        }
        if self.instructions.as_ref().unwrap().len() <= instruction_index {
            Err(AddLabelError::IndexOutOfBounds)
        } else {
            self.control_flow
                .instruction_labels
                .insert(label, instruction_index);
            Ok(())
        }
    }

    /// Checks if all labels that are called in the instructions exist in the control flow.
    ///
    /// If label is missing the name of the label that is missing is returned.
    fn check_labels(&self) -> Result<(), String> {
        if self.instructions.is_none() {
            return Ok(());
        }
        for instruction in self.instructions.as_ref().unwrap() {
            match instruction {
                Instruction::Goto(label) | Instruction::JumpIf(_, _, _, label) => {
                    check_label(&self.control_flow, label)?;
                }
                _ => (),
            };
        }
        Ok(())
    }

    /// Checks if any accumulators or memory cells are missing in the runtime args that are used.
    ///
    /// If something missing is found, a runtime build error is returned.
    ///
    /// If `add_missing` is true, the missing `accumulator/memory_cell` is added with empty value to the runtime args instead of returning an error.
    fn check_missing_vars(&mut self, add_missing: bool) -> Result<(), RuntimeBuildError> {
        if self.instructions.is_none() {
            return Ok(());
        }
        for instruction in self.instructions.as_ref().unwrap() {
            match instruction {
                Instruction::Assign(target, source) => {
                    target.check(self.runtime_args.as_mut().unwrap(), add_missing)?;
                    source.check(self.runtime_args.as_mut().unwrap(), add_missing)?;
                }
                Instruction::Calc(target, value_a, _, value_b) => {
                    target.check(self.runtime_args.as_mut().unwrap(), add_missing)?;
                    value_a.check(self.runtime_args.as_mut().unwrap(), add_missing)?;
                    value_b.check(self.runtime_args.as_mut().unwrap(), add_missing)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    /// Checks instructions that are set by comparing them with the provided whitelist of instructions.
    /// It is also checked if any comparisons or operations are used that are not allowed.
    /// If this runtime builder contains instructions that are not contained within the whitelist or comparisons
    /// or operations that are not allowed, an error is returned.
    ///
    /// The whitelist contains the return value of `Instruction::identifier()`.
    pub fn check_instructions(
        //TODO if I rework the builder pattern this function should be called when the runtime is build and when the instructions are added
        &self,
        instructions: &[Instruction],
        whitelist: Option<&HashSet<String>>,
    ) -> Result<(), BuildProgramError> {
        for (idx, i) in instructions.iter().enumerate() {
            if let Some(whitelist) = whitelist {
                if !whitelist.contains(&i.identifier()) {
                    // Instruction found, that is forbidden
                    let mut allowed_instructions = whitelist
                        .iter()
                        .map(String::to_string)
                        .collect::<Vec<String>>();
                    allowed_instructions.sort();
                    return Err(BuildProgramError {
                        reason: BuildProgramErrorTypes::InstructionNotAllowed(
                            idx + 1,
                            format!("{i}"),
                            i.identifier(),
                            allowed_instructions.join("\n").to_string(),
                        ),
                    });
                }
            }
            // Check if all comparisons are allowed
            if let Some(ac) = &self.limit_comparisons_to {
                if let Some(c) = i.comparison() {
                    if !ac.contains(c) {
                        return Err(BuildProgramError {
                            reason: BuildProgramErrorTypes::ComparisonNotAllowed(
                                idx + 1,
                                c.to_string(),
                            ),
                        });
                    }
                }
            }
            // Check if all operations are allowed
            if let Some(ao) = &self.limit_operations_to {
                if let Some(o) = i.operation() {
                    if !ao.contains(o) {
                        return Err(BuildProgramError {
                            reason: BuildProgramErrorTypes::OperationNotAllowed(
                                idx + 1,
                                o.to_string(),
                            ),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

fn inject_end_labels(control_flow: &mut ControlFlow, last_instruction_index: usize) {
    control_flow
        .instruction_labels
        .insert("END".to_string(), last_instruction_index);
    control_flow
        .instruction_labels
        .insert("ENDE".to_string(), last_instruction_index);
    control_flow
        .instruction_labels
        .insert("end".to_string(), last_instruction_index);
    control_flow
        .instruction_labels
        .insert("ende".to_string(), last_instruction_index);
}

fn check_label(control_flow: &ControlFlow, label: &str) -> Result<(), String> {
    if !control_flow.instruction_labels.contains_key(label) {
        return Err(label.to_string());
    }
    Ok(())
}

/// Checks if accumulators with id exist.
///
/// If `add_missing` is set, the accumulator is added with empty value instead of returning an error.
pub fn check_accumulator(
    runtime_args: &mut RuntimeArgs,
    id: usize,
    add_missing: bool,
) -> Result<(), RuntimeBuildError> {
    if !runtime_args.exists_accumulator(id) {
        if add_missing {
            runtime_args.accumulators.insert(id, Accumulator::new(id));
        } else {
            return Err(RuntimeBuildError::AccumulatorMissing(id.to_string()));
        }
    }
    Ok(())
}

/// Checks if the memory cell with name exists.
///
/// If `add_missing` is set, the memory cell is added with empty value instead of returning an error.
pub fn check_memory_cell(
    runtime_args: &mut RuntimeArgs,
    name: &str,
    add_missing: bool,
) -> Result<(), RuntimeBuildError> {
    if !runtime_args.memory_cells.contains_key(name) {
        if add_missing {
            runtime_args
                .memory_cells
                .insert(name.to_string(), MemoryCell::new(name));
        } else {
            return Err(RuntimeBuildError::MemoryCellMissing(name.to_string()));
        }
    }
    Ok(())
}

/// Checks if the accumulator or `memory_cell` exists that is used inside an `index_memory_cell`.
pub fn check_index_memory_cell(
    runtime_args: &mut RuntimeArgs,
    t: &IndexMemoryCellIndexType,
    add_missing: bool,
) -> Result<(), RuntimeBuildError> {
    match t {
        IndexMemoryCellIndexType::Accumulator(idx) => {
            check_accumulator(runtime_args, *idx, add_missing)
        }
        IndexMemoryCellIndexType::Direct(_) | IndexMemoryCellIndexType::Index(_) => Ok(()),
        IndexMemoryCellIndexType::Gamma => check_gamma(runtime_args, add_missing),
        IndexMemoryCellIndexType::MemoryCell(name) => {
            check_memory_cell(runtime_args, name, add_missing)
        }
    }
}

/// Checks if gamma is enabled in runtime args.
///
/// If `add_missing` is set, gamma is enabled, instead of returning an error.
pub fn check_gamma(
    runtime_args: &mut RuntimeArgs,
    add_missing: bool,
) -> Result<(), RuntimeBuildError> {
    if runtime_args.gamma.is_none() {
        if add_missing {
            runtime_args.gamma = Some(None);
            return Ok(());
        }
        return Err(RuntimeBuildError::GammaDisabled);
    }
    Ok(())
}

impl TargetType {
    /// Checks if this type is missing in `runtime_args`.
    ///
    /// If `add_missing` is set, the type is added to runtime args instead of returning an error.
    pub fn check(
        &self,
        runtime_args: &mut RuntimeArgs,
        add_missing: bool,
    ) -> Result<(), RuntimeBuildError> {
        match self {
            Self::Accumulator(index) => check_accumulator(runtime_args, *index, add_missing)?,
            Self::MemoryCell(name) => check_memory_cell(runtime_args, name, add_missing)?,
            Self::IndexMemoryCell(t) => check_index_memory_cell(runtime_args, t, add_missing)?,
            Self::Gamma => check_gamma(runtime_args, add_missing)?,
        }
        Ok(())
    }
}

impl Value {
    /// Checks if this type is missing in `runtime_args`.
    ///
    /// If `add_missing` is set, the type is added to runtime args instead of returning an error.
    pub fn check(
        &self,
        runtime_args: &mut RuntimeArgs,
        add_missing: bool,
    ) -> Result<(), RuntimeBuildError> {
        match self {
            Self::Accumulator(index) => check_accumulator(runtime_args, *index, add_missing)?,
            Self::MemoryCell(name) => check_memory_cell(runtime_args, name, add_missing)?,
            Self::Constant(_) => (),
            Self::IndexMemoryCell(t) => check_index_memory_cell(runtime_args, t, add_missing)?,
            Self::Gamma => check_gamma(runtime_args, add_missing)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        instructions::IndexMemoryCellIndexType,
        runtime::{
            builder::{check_index_memory_cell, RuntimeBuilder},
            error_handling::RuntimeBuildError,
            RuntimeArgs,
        },
    };

    /// Used to set the available memory cells during testing.
    const TEST_MEMORY_CELL_LABELS: &[&str] = &[
        "a", "b", "c", "d", "e", "f", "w", "x", "y", "z", "h1", "h2", "h3", "h4",
    ];

    #[test]
    fn test_instruction_building_with_comments() {
        let instructions = vec![
            "a0 := 4 // Set alpha to 4",
            "p(h1) := a0 # Set memory cell h1 to 4",
            "a0 := a1 # Just some stuff",
            "a1 := a2 // Just some more stuff",
        ];
        let mut rb = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
        assert!(rb.build_instructions(&instructions, "test").is_ok());
    }

    #[test]
    fn test_instruction_building_with_semicolons() {
        let instructions = vec![
            "a0 := 4; // Set alpha to 4",
            "p(h1) := a0; # Set memory cell h1 to 4",
            "a0 := a1; # Just some stuff",
            "a1 := a2; // Just some more stuff",
        ];
        let mut rb = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
        assert!(rb.build_instructions(&instructions, "test").is_ok());
    }

    #[test]
    fn test_only_label_line() {
        let mut rb = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
        assert!(rb
            .build_instructions(&vec!["a0 := 5", "my_label:", "a1 := 5"], "")
            .is_ok());
    }

    #[test]
    fn test_accumulator_auto_add_working() {
        let instructions = vec!["a1 := a2 + a3"];
        let mut rb = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
        assert!(rb.build_instructions(&instructions, "test").is_ok());
        let rt = rb.build();
        assert!(rt.is_ok());
        let rt = rt.unwrap();
        assert!(rt.runtime_args.accumulators.contains_key(&1));
        assert!(rt.runtime_args.accumulators.contains_key(&2));
        assert!(rt.runtime_args.accumulators.contains_key(&3));
        assert!(!rt.runtime_args.accumulators.contains_key(&4));
    }

    #[test]
    fn test_check_index_memory_cell() {
        let mut args = RuntimeArgs::new_empty();
        assert_eq!(
            check_index_memory_cell(&mut args, &IndexMemoryCellIndexType::Accumulator(0), false),
            Err(RuntimeBuildError::AccumulatorMissing("0".to_string()))
        );
        assert_eq!(
            check_index_memory_cell(&mut args, &IndexMemoryCellIndexType::Gamma, false),
            Err(RuntimeBuildError::GammaDisabled)
        );
        assert_eq!(
            check_index_memory_cell(
                &mut args,
                &IndexMemoryCellIndexType::MemoryCell("h1".to_string()),
                false
            ),
            Err(RuntimeBuildError::MemoryCellMissing("h1".to_string()))
        );
        assert_eq!(
            check_index_memory_cell(&mut args, &IndexMemoryCellIndexType::Accumulator(0), true),
            Ok(())
        );
        assert_eq!(
            check_index_memory_cell(&mut args, &IndexMemoryCellIndexType::Gamma, true),
            Ok(())
        );
        assert_eq!(
            check_index_memory_cell(
                &mut args,
                &IndexMemoryCellIndexType::MemoryCell("h1".to_string()),
                true
            ),
            Ok(())
        );
        assert!(args.accumulators.contains_key(&0));
        assert!(args.gamma.is_some());
        assert!(args.memory_cells.contains_key("h1"));
    }
}
