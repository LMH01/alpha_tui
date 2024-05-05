use std::collections::HashSet;

use crate::{
    base::{Accumulator, Comparison, MemoryCell, Operation},
    cli::{GlobalArgs, InstructionLimitingArgs},
    instructions::{
        error_handling::{BuildProgramError, BuildProgramErrorTypes},
        IndexMemoryCellIndexType, Instruction, TargetType, Value,
    },
    utils,
};

use super::{
    error_handling::RuntimeBuildError, memory_config::MemoryConfig, ControlFlow, Runtime,
    RuntimeMemory, RuntimeSettings,
};

pub struct RuntimeBuilder<'a> {
    instructions_input: &'a Vec<String>,
    instructions_input_file_name: &'a str,
    memory_config: Option<MemoryConfig>,
    runtime_settings: Option<RuntimeSettings>,
    instruction_config: InstructionConfig,
}

impl<'a> RuntimeBuilder<'a> {
    pub fn new(instructions_input: &'a Vec<String>, instructions_input_file_name: &'a str) -> Self {
        Self {
            instructions_input,
            instructions_input_file_name,
            memory_config: None,
            runtime_settings: None,
            instruction_config: InstructionConfig::default(),
        }
    }

    /// Applies the parameters in global args to this runtime builder.
    ///
    /// Already existing values in the `MemoryConfig` and `RuntimeSettings` will be overwritten, if the provided value is not `None`.
    ///
    /// If a `MemoryConfig` and/or `RuntimeSettings` was not set already, a new memory config is generated from the cli args.
    pub fn apply_global_cli_args(
        &mut self,
        global_args: &GlobalArgs,
    ) -> miette::Result<&mut Self, RuntimeBuildError> {
        // set disable instruction limit value
        let mut settings = match self.runtime_settings.take() {
            Some(settings) => settings,
            None => RuntimeSettings::default(),
        };
        settings.disable_instruction_limit = global_args.disable_instruction_limit;
        self.runtime_settings = Some(settings);

        let mut memory_config = match self.memory_config.take() {
            Some(memory_config) => memory_config,
            None => {
                // check if memory config file is provided, from which the memory config can be build
                if let Some(path) = &global_args.memory_config_file {
                    match MemoryConfig::try_from_file(path) {
                        Ok(config) => {
                            self.memory_config = Some(config);
                            return Ok(self);
                        }
                        Err(e) => {
                            return Err(RuntimeBuildError::MemoryConfigFileInvalid(
                                path.to_string(),
                                e,
                            ))
                        }
                    }
                } else {
                    MemoryConfig::default()
                }
            }
        };
        // set/overwrite memory config values
        // set accumulator config
        if let Some(accumulators) = global_args.accumulators {
            for value in 0..=accumulators - 1 {
                memory_config
                    .accumulators
                    .values
                    .insert(value as usize, None);
            }
        }
        // set memory_cell config
        if let Some(memory_cells) = &global_args.memory_cells {
            for memory_cell in memory_cells {
                memory_config
                    .memory_cells
                    .values
                    .insert(memory_cell.clone(), None);
            }
        }
        // set index_memory_cell config
        if let Some(index_memory_cells) = &global_args.index_memory_cells {
            for imc in index_memory_cells {
                memory_config.index_memory_cells.values.insert(*imc, None);
            }
        }
        self.memory_config = Some(memory_config);
        Ok(self)
    }

    /// Applies the provided instruction limiting args to this runtime builder.
    ///
    /// If `MemoryConfig` is already set, the values for `autodetection` are overwritten to false,
    /// otherwise a new `MemoryConfig` is created where the values are set to false.
    ///
    /// All values previously set in `InstructionConfig` struct are replaced by the new values.
    pub fn apply_instruction_limiting_args(
        &mut self,
        instruction_limiting_args: &InstructionLimitingArgs,
    ) -> miette::Result<&mut Self> {
        if let Some(ac) = &instruction_limiting_args.allowed_comparisons {
            self.instruction_config.allowed_comparisons = Some(ac.clone());
        }
        if let Some(ao) = &instruction_limiting_args.allowed_operations {
            self.instruction_config.allowed_operations = Some(ao.clone());
        }
        // if allowed instructions file is set, parse instructions and set the ids as allowed
        if let Some(path) = &instruction_limiting_args.allowed_instructions_file {
            self.instruction_config.allowed_instruction_identifiers =
                Some(utils::build_instruction_whitelist(path)?);
        }
        // set/override memory autodetection values to false, if `--disable-memory-detection` is set
        if instruction_limiting_args.disable_memory_detection {
            let mut memory_config = match self.memory_config.take() {
                Some(memory_config) => memory_config,
                None => MemoryConfig::default(),
            };
            memory_config.accumulators.autodetection = Some(false);
            memory_config.gamma_accumulator.autodetection = Some(false);
            memory_config.memory_cells.autodetection = Some(false);
            memory_config.index_memory_cells.autodetection = Some(false);
            // enable gamma accumulator, if specified
            if instruction_limiting_args.enable_gamma_accumulator {
                memory_config.gamma_accumulator.enabled = true;
            }
            self.memory_config = Some(memory_config);
        }
        Ok(self)
    }

    /// Builds a new runtime by consuming this `RuntimeBuilder`.
    ///
    /// Prints status messages into stdout.
    pub fn build(self) -> miette::Result<Runtime> {
        // set runtime settings
        let settings = self.runtime_settings.unwrap_or(RuntimeSettings::default());

        // build memory
        let mut memory = match self.memory_config.as_ref() {
            Some(memory_config) => RuntimeMemory::from(memory_config.to_owned()),
            None => RuntimeMemory::default(),
        };

        // set control flow
        let mut control_flow = ControlFlow::new();

        // build instructions (also updated control flow with detected labels)
        println!("Building instructions");
        let instructions = build_instructions(
            &self.instructions_input,
            &self.instructions_input_file_name,
            &mut control_flow,
        )?;

        println!("Building runtime");

        // inject end labels to give option to end program using goto END
        inject_end_labels(&mut control_flow, instructions.len());

        if let Err(e) = check_labels(&control_flow, &instructions) {
            return Err(miette::miette!(RuntimeBuildError::LabelUndefined(e)));
        }

        // Check if all used accumulators and memory_cells exist
        check_missing_vars(
            &self
                .memory_config
                .as_ref()
                .unwrap_or(&MemoryConfig::default()),
            &instructions,
            &mut memory,
        )?;

        // check if main label is set and update instruction pointer if found
        if let Some(i) = control_flow.instruction_labels.get("main") {
            control_flow.next_instruction_index = *i;
            control_flow.initial_instruction = *i;
        }
        if let Some(i) = control_flow.instruction_labels.get("MAIN") {
            control_flow.next_instruction_index = *i;
            control_flow.initial_instruction = *i;
        }

        Ok(Runtime {
            memory,
            instructions,
            control_flow,
            instruction_runs: 0,
            settings,
        })
    }
}

/// Stores information that is used to limit what instructions should be allowed.
#[derive(Default)]
struct InstructionConfig {
    /// Stores the ids of instructions that are allowed.
    ///
    /// If the value is `None` all instructions are allowed.
    pub allowed_instruction_identifiers: Option<HashSet<String>>,
    /// Stores comparisons that are allowed, if value is `None`, all comparisons are allowed.
    pub allowed_comparisons: Option<Vec<Comparison>>,
    /// Stores operations that are allowed, if value is `None`, all operations are allowed.
    pub allowed_operations: Option<Vec<Operation>>,
}

/// Builds the provided instructions.
///
/// Updates the provided control flow with labels.
fn build_instructions(
    instructions_input: &[String],
    file_name: &str,
    control_flow: &mut ControlFlow,
) -> Result<Vec<Instruction>, BuildProgramError> {
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
            if control_flow
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
                Err(e.into_build_program_error(
                    instructions_input.join("\n"),
                    file_name,
                    index + 1,
                ))?;
            }
        }
    }
    if control_flow.instruction_labels.contains_key("main")
        && control_flow.instruction_labels.contains_key("MAIN")
    {
        return Err(BuildProgramError {
            reason: BuildProgramErrorTypes::MainLabelDefinedMultipleTimes,
        });
    }
    Ok(instructions)
}

/// Removes everything behind # or // from the string
pub fn remove_comment(instruction: &str) -> String {
    instruction
        .lines()
        .map(|line| {
            if let Some(index) = line.find("//") {
                line[..index].trim()
            } else if let Some(index) = line.find('#') {
                line[..index].trim()
            } else {
                line.trim()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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
    control_flow
        .instruction_labels
        .insert("End".to_string(), last_instruction_index);
    control_flow
        .instruction_labels
        .insert("Ende".to_string(), last_instruction_index);
}

fn check_label(control_flow: &ControlFlow, label: &str) -> Result<(), String> {
    if !control_flow.instruction_labels.contains_key(label) {
        return Err(label.to_string());
    }
    Ok(())
}

/// Checks if all labels that are called in the instructions exist in the control flow.
///
/// If label is missing the name of the label that is missing is returned.
fn check_labels(control_flow: &ControlFlow, instructions: &Vec<Instruction>) -> Result<(), String> {
    for instruction in instructions {
        match instruction {
            Instruction::Goto(label) | Instruction::JumpIf(_, _, _, label) => {
                check_label(control_flow, label)?;
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
fn check_missing_vars(
    memory_config: &MemoryConfig,
    instructions: &Vec<Instruction>,
    runtime_memory: &mut RuntimeMemory,
) -> Result<(), RuntimeBuildError> {
    for instruction in instructions {
        match instruction {
            Instruction::Assign(target, source) => {
                target.check_new(runtime_memory, memory_config)?;
                source.check_new(runtime_memory, memory_config)?;
            }
            Instruction::Calc(target, value_a, _, value_b) => {
                target.check_new(runtime_memory, memory_config)?;
                value_a.check_new(runtime_memory, memory_config)?;
                value_b.check_new(runtime_memory, memory_config)?;
            }
            _ => (),
        }
    }
    Ok(())
}

/// Checks if accumulators with id exist.
///
/// If `add_missing` is set, the accumulator is added with empty value instead of returning an error.
pub fn check_accumulator(
    runtime_args: &mut RuntimeMemory,
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
    runtime_args: &mut RuntimeMemory,
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
    runtime_args: &mut RuntimeMemory,
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
    runtime_args: &mut RuntimeMemory,
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
        runtime_args: &mut RuntimeMemory,
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

    /// Checks if this type is missing in `runtime_args`.
    ///
    /// If autodetection in memory_config is enabled, the type is added to runtime args instead of returning an error.
    pub fn check_new(
        &self,
        runtime_args: &mut RuntimeMemory,
        memory_config: &MemoryConfig,
    ) -> Result<(), RuntimeBuildError> {
        match self {
            Self::Accumulator(index) => check_accumulator(
                runtime_args,
                *index,
                memory_config.accumulators.autodetection.unwrap_or(true),
            )?,
            Self::MemoryCell(name) => check_memory_cell(
                runtime_args,
                name,
                memory_config.memory_cells.autodetection.unwrap_or(true),
            )?,
            Self::IndexMemoryCell(t) => check_index_memory_cell(
                runtime_args,
                t,
                memory_config
                    .index_memory_cells
                    .autodetection
                    .unwrap_or(true),
            )?,
            Self::Gamma => check_gamma(
                runtime_args,
                memory_config
                    .gamma_accumulator
                    .autodetection
                    .unwrap_or(true),
            )?,
        }
        Ok(())
    }
}

impl Value {
    /// Checks if this type is missing in `runtime_args`.
    ///
    /// If autodetection in memory_config is enabled, the type is added to runtime args instead of returning an error.
    pub fn check_new(
        &self,
        runtime_args: &mut RuntimeMemory,
        memory_config: &MemoryConfig,
    ) -> Result<(), RuntimeBuildError> {
        match self {
            Self::Accumulator(index) => check_accumulator(
                runtime_args,
                *index,
                memory_config.accumulators.autodetection.unwrap_or(true),
            )?,
            Self::MemoryCell(name) => check_memory_cell(
                runtime_args,
                name,
                memory_config.memory_cells.autodetection.unwrap_or(true),
            )?,
            Self::Constant(_) => (),
            Self::IndexMemoryCell(t) => check_index_memory_cell(
                runtime_args,
                t,
                memory_config
                    .index_memory_cells
                    .autodetection
                    .unwrap_or(true),
            )?,
            Self::Gamma => check_gamma(
                runtime_args,
                memory_config
                    .gamma_accumulator
                    .autodetection
                    .unwrap_or(true),
            )?,
        }
        Ok(())
    }
}

//#[cfg(test)]
//mod tests {
//    use crate::{
//        instructions::IndexMemoryCellIndexType,
//        runtime::{
//            builder::{check_index_memory_cell, RuntimeBuilder},
//            error_handling::RuntimeBuildError,
//            RuntimeMemory,
//        },
//    };
//
//    /// Used to set the available memory cells during testing.
//    const TEST_MEMORY_CELL_LABELS: &[&str] = &[
//        "a", "b", "c", "d", "e", "f", "w", "x", "y", "z", "h1", "h2", "h3", "h4",
//    ];
//
//    #[test]
//    fn test_instruction_building_with_comments() {
//        let instructions = vec![
//            "a0 := 4 // Set alpha to 4",
//            "p(h1) := a0 # Set memory cell h1 to 4",
//            "a0 := a1 # Just some stuff",
//            "a1 := a2 // Just some more stuff",
//        ];
//        let mut rb = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
//        assert!(rb.build_instructions(&instructions, "test").is_ok());
//    }
//
//    #[test]
//    fn test_instruction_building_with_semicolons() {
//        let instructions = vec![
//            "a0 := 4; // Set alpha to 4",
//            "p(h1) := a0; # Set memory cell h1 to 4",
//            "a0 := a1; # Just some stuff",
//            "a1 := a2; // Just some more stuff",
//        ];
//        let mut rb = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
//        assert!(rb.build_instructions(&instructions, "test").is_ok());
//    }
//
//    #[test]
//    fn test_only_label_line() {
//        let mut rb = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
//        assert!(rb
//            .build_instructions(&vec!["a0 := 5", "my_label:", "a1 := 5"], "")
//            .is_ok());
//    }
//
//    #[test]
//    fn test_accumulator_auto_add_working() {
//        let instructions = vec!["a1 := a2 + a3"];
//        let mut rb = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
//        assert!(rb.build_instructions(&instructions, "test").is_ok());
//        let rt = rb.build();
//        assert!(rt.is_ok());
//        let rt = rt.unwrap();
//        assert!(rt.memory.accumulators.contains_key(&1));
//        assert!(rt.memory.accumulators.contains_key(&2));
//        assert!(rt.memory.accumulators.contains_key(&3));
//        assert!(!rt.memory.accumulators.contains_key(&4));
//    }
//
//    #[test]
//    fn test_check_index_memory_cell() {
//        let mut args = RuntimeMemory::new_empty();
//        assert_eq!(
//            check_index_memory_cell(&mut args, &IndexMemoryCellIndexType::Accumulator(0), false),
//            Err(RuntimeBuildError::AccumulatorMissing("0".to_string()))
//        );
//        assert_eq!(
//            check_index_memory_cell(&mut args, &IndexMemoryCellIndexType::Gamma, false),
//            Err(RuntimeBuildError::GammaDisabled)
//        );
//        assert_eq!(
//            check_index_memory_cell(
//                &mut args,
//                &IndexMemoryCellIndexType::MemoryCell("h1".to_string()),
//                false
//            ),
//            Err(RuntimeBuildError::MemoryCellMissing("h1".to_string()))
//        );
//        assert_eq!(
//            check_index_memory_cell(&mut args, &IndexMemoryCellIndexType::Accumulator(0), true),
//            Ok(())
//        );
//        assert_eq!(
//            check_index_memory_cell(&mut args, &IndexMemoryCellIndexType::Gamma, true),
//            Ok(())
//        );
//        assert_eq!(
//            check_index_memory_cell(
//                &mut args,
//                &IndexMemoryCellIndexType::MemoryCell("h1".to_string()),
//                true
//            ),
//            Ok(())
//        );
//        assert!(args.accumulators.contains_key(&0));
//        assert!(args.gamma.is_some());
//        assert!(args.memory_cells.contains_key("h1"));
//    }
//}
