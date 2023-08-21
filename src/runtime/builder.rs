use miette::{NamedSource, Result, SourceOffset, SourceSpan};

use crate::{
    base::{Accumulator, MemoryCell},
    cli::Args,
    instructions::{
        error_handling::{BuildProgramError, BuildProgramErrorTypes, InstructionParseError},
        Instruction,
    },
};

use super::{
    error_handling::{AddLabelError, RuntimeBuildError},
    ControlFlow, Runtime, RuntimeArgs,
};

/// Type that is used to build a new runtime environment.
///
/// This runtime can be configured to only allow a selected amount of accumulators and memory cells.
/// When a runtime is build from this builder compatibility checks are performed.
#[derive(Debug)]
pub struct RuntimeBuilder {
    runtime_args: Option<RuntimeArgs>,
    instructions: Option<Vec<Instruction>>,
    control_flow: ControlFlow,
    add_missing: bool,
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
        }
    }

    /// Creates a new runtime builder from the cli arguments.
    pub fn from_args(args: &Args) -> Result<Self, String> {
        Ok(Self {
            runtime_args: Some(RuntimeArgs::from_args(args)?),
            instructions: None,
            control_flow: ControlFlow::new(),
            add_missing: !args.disable_memory_detection,
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
            &self.instructions.as_ref().unwrap().len(),
        );
        if let Err(e) = self.check_labels() {
            return Err(RuntimeBuildError::LabelUndefined(e));
        }
        if let Err(e) = self.check_accumulators(self.add_missing) {
            return Err(RuntimeBuildError::AccumulatorMissing(e));
        }
        if let Err(e) = self.check_memory_cells(self.add_missing) {
            return Err(RuntimeBuildError::MemoryCellMissing(e));
        }
        Ok(Runtime {
            runtime_args: self.runtime_args.clone().unwrap(),
            instructions: self.instructions.clone().unwrap(),
            control_flow: self.control_flow.clone(),
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
    #[allow(clippy::ptr_arg)]
    pub fn build_instructions(
        &mut self,
        instructions_input: &Vec<&str>,
        file_name: &str,
    ) -> Result<()> {
        self.control_flow.reset();
        let mut instructions = Vec::new();
        for (index, instruction) in instructions_input.iter().enumerate() {
            // Remove comments
            let instruction = instruction
                .lines()
                .map(|line| {
                    if let Some(index) = line.find("//") {
                        &line[..index]
                    } else if let Some(index) = line.find("#") {
                        &line[..index]
                    } else {
                        line
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            // Check for labels
            let mut splits = instruction.split_whitespace().collect::<Vec<&str>>();
            if splits.is_empty() {
                // Line is empty / line contains comment, add dummy instruction
                instructions.push(Instruction::Sleep());
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
                    // label defined multiple times
                    Err(BuildProgramError {
                        reason: BuildProgramErrorTypes::LabelDefinedMultipleTimes(label),
                    })?;
                }
            }
            //instructions.push(Instruction::try_from(&splits).wrap_err("when building instructions")?)
            //instructions.push(Instruction::try_from(&splits)?)
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
                        InstructionParseError::UnknownInstructionSuggestion {
                            range: _,
                            help: _,
                            src: _,
                        } => e.range().1 - e.range().0 + 1,
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
                    })?
                    //})?
                }
            }
        }
        self.instructions = Some(instructions);
        Ok(())
    }

    /// Sets the instructions to the provided instructions.
    ///
    /// If loops and labels are used, they have to be set manually by using [RuntimeBuilder::add_label](#add_label).
    #[allow(dead_code)]
    pub fn set_instructions(&mut self, instructions: Vec<Instruction>) {
        self.instructions = Some(instructions);
    }

    /// Adds label to instruction labels.
    ///
    /// Errors when **instruction_index** is out of bounds.
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
                Instruction::Goto(label) => check_label(&self.control_flow, label)?,
                Instruction::GotoIfAccumulator(_, label, _, _) => {
                    check_label(&self.control_flow, label)?
                }
                Instruction::GotoIfConstant(_, label, _, _) => {
                    check_label(&self.control_flow, label)?
                }
                Instruction::GotoIfMemoryCell(_, label, _, _) => {
                    check_label(&self.control_flow, label)?
                }
                _ => (),
            };
        }
        Ok(())
    }

    /// Checks if all accumulators that are used in the instructions exist in the runtime args.
    ///
    /// If accumulator is missing, the id of the missing accumulator is returned.
    ///
    /// Panics if runtime_args is `None`.
    fn check_accumulators(&mut self, add_missing: bool) -> Result<(), String> {
        if self.instructions.is_none() {
            return Ok(());
        }
        for instruction in self.instructions.as_ref().unwrap() {
            match instruction {
                Instruction::AssignAccumulatorValue(id, _) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::AssignAccumulatorValueFromAccumulator(id_a, id_b) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_a, add_missing)?;
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_b, add_missing)?;
                }
                Instruction::AssignAccumulatorValueFromMemoryCell(id, _) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::AssignMemoryCellValueFromAccumulator(_, id) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::CalcAccumulatorWithAccumulator(_, id_a, id_b) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_a, add_missing)?;
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_b, add_missing)?;
                }
                Instruction::CalcAccumulatorWithAccumulators(_, id_a, id_b, id_c) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_a, add_missing)?;
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_b, add_missing)?;
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_c, add_missing)?;
                }
                Instruction::CalcAccumulatorWithConstant(_, id, _) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::CalcAccumulatorWithMemoryCell(_, id, _) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::CalcAccumulatorWithMemoryCells(_, id, _, _) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::CalcMemoryCellWithMemoryCellAccumulator(_, _, _, id) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::GotoIfAccumulator(_, _, id_a, id_b) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_a, add_missing)?;
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_b, add_missing)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    /// Checks if all memory cells that are used in the instructions exist in the runtime args.
    ///
    /// If memory cell is missing, the name of the missing memory cell is returned.
    ///
    /// Panics if runtime_args is `None`.
    fn check_memory_cells(&mut self, add_missing: bool) -> Result<(), String> {
        if self.instructions.is_none() {
            return Ok(());
        }
        for instruction in self.instructions.as_ref().unwrap() {
            match instruction {
                Instruction::AssignAccumulatorValueFromMemoryCell(_, name) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name, add_missing)?
                }
                Instruction::AssignMemoryCellValue(name, _) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name, add_missing)?
                }
                Instruction::AssignMemoryCellValueFromAccumulator(name, _) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name, add_missing)?
                }
                Instruction::AssignMemoryCellValueFromMemoryCell(name_a, name_b) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_a, add_missing)?;
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_b, add_missing)?;
                }
                Instruction::CalcAccumulatorWithMemoryCell(_, _, name) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name, add_missing)?
                }
                Instruction::CalcAccumulatorWithMemoryCells(_, _, name_a, name_b) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_a, add_missing)?;
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_b, add_missing)?;
                }
                Instruction::CalcMemoryCellWithMemoryCellConstant(_, name_a, name_b, _) => {
                    if !self
                        .runtime_args
                        .as_ref()
                        .unwrap()
                        .memory_cells
                        .contains_key(name_a.as_str())
                    {
                        return Err(name_a.clone());
                    }
                    if !self
                        .runtime_args
                        .as_ref()
                        .unwrap()
                        .memory_cells
                        .contains_key(name_b.as_str())
                    {
                        return Err(name_b.clone());
                    }
                }
                Instruction::CalcMemoryCellWithMemoryCellAccumulator(_, name_a, name_b, _) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_a, add_missing)?;
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_b, add_missing)?;
                }
                Instruction::CalcMemoryCellWithMemoryCells(_, name_a, name_b, name_c) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_a, add_missing)?;
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_b, add_missing)?;
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_c, add_missing)?;
                }
                Instruction::GotoIfMemoryCell(_, _, _, name) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name, add_missing)?
                }
                _ => (),
            }
        }
        Ok(())
    }
}

fn inject_end_labels(control_flow: &mut ControlFlow, last_instruction_index: &usize) {
    control_flow
        .instruction_labels
        .insert("END".to_string(), *last_instruction_index);
    control_flow
        .instruction_labels
        .insert("ENDE".to_string(), *last_instruction_index);
    control_flow
        .instruction_labels
        .insert("end".to_string(), *last_instruction_index);
    control_flow
        .instruction_labels
        .insert("ende".to_string(), *last_instruction_index);
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
fn check_accumulators(
    runtime_args: &mut RuntimeArgs,
    id: &usize,
    add_missing: bool,
) -> Result<(), String> {
    if !runtime_args.exists_accumulator(id) {
        if add_missing {
            runtime_args.accumulators.push(Accumulator::new(*id));
        } else {
            return Err(id.to_string());
        }
    }
    Ok(())
}

/// Checks if the memory cell with name exists.
///
/// If `add_missing` is set, the memory cell is added with empty value instead of returning an error.
fn check_memory_cell(
    runtime_args: &mut RuntimeArgs,
    name: &str,
    add_missing: bool,
) -> Result<(), String> {
    if !runtime_args.memory_cells.contains_key(name) {
        if add_missing {
            runtime_args
                .memory_cells
                .insert(name.to_string(), MemoryCell::new(name));
        } else {
            return Err(name.to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::runtime::builder::RuntimeBuilder;

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
}
