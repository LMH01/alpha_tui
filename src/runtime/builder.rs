use crate::{
    base::{Accumulator, MemoryCell},
    cli::{CliHint, GlobalArgs, InstructionLimitingArgs},
    instructions::{
        error_handling::{BuildProgramError, BuildProgramErrorTypes},
        instruction_config::InstructionConfig,
        Identifier, IndexMemoryCellIndexType, Instruction, TargetType, Value,
    },
};

use super::{
    error_handling::RuntimeBuildError, memory_config::MemoryConfig, ControlFlow, Runtime,
    RuntimeMemory, RuntimeSettings,
};

pub struct RuntimeBuilder {
    instructions: Vec<Instruction>,
    control_flow: ControlFlow,
    memory_config: Option<MemoryConfig>,
    runtime_settings: Option<RuntimeSettings>,
    instruction_config: InstructionConfig,
}

impl RuntimeBuilder {
    /// Creates a new runtime builder.
    ///
    /// The input instructions are build directly and this function returns an error if that failed.
    #[allow(clippy::result_large_err)]
    pub fn new<'a>(
        instructions_input: &'a [String],
        instructions_input_file_name: &'a str,
    ) -> Result<Self, BuildProgramError> {
        let mut control_flow = ControlFlow::new();

        // build instructions (also updated control flow with detected labels)
        let instructions = match build_instructions(
            instructions_input,
            instructions_input_file_name,
            &mut control_flow,
        ) {
            Ok(instructions) => instructions,
            Err(e) => return Err(*e),
        };

        Ok(Self {
            instructions,
            control_flow,
            memory_config: None,
            runtime_settings: None,
            instruction_config: InstructionConfig::default(),
        })
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
                        Ok(config) => config,
                        Err(e) => {
                            return Err(RuntimeBuildError::MemoryConfigFileInvalid(
                                path.to_string(),
                                e.to_string(),
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
        // update runtime settings
        let mut runtime_settings = match self.runtime_settings.take() {
            Some(settings) => settings,
            None => RuntimeSettings::default(),
        };
        if let Some(value) = memory_config.accumulators.autodetection {
            runtime_settings.autodetect_accumulators = value;
        }
        if let Some(value) = memory_config.gamma_accumulator.autodetection {
            runtime_settings.autodetect_gamma_accumulator = value;
        }
        if let Some(value) = memory_config.memory_cells.autodetection {
            runtime_settings.autodetect_memory_cells = value;
        }
        if let Some(value) = memory_config.index_memory_cells.autodetection {
            runtime_settings.autodetect_index_memory_cells = value;
        }
        self.runtime_settings = Some(runtime_settings);
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
        // if allowed instructions file is set, parse instructions and set the ids as allowed
        if let Some(path) = &instruction_limiting_args.allowed_instructions_file {
            match InstructionConfig::try_from_file(path) {
                Ok(config) => {
                    if let Some(instructions) = config.allowed_instruction_identifiers {
                        self.instruction_config.allowed_instruction_identifiers =
                            Some(instructions);
                    }
                    if let Some(comparisons) = config.allowed_comparisons {
                        self.instruction_config.allowed_comparisons = Some(comparisons);
                    }
                    if let Some(operations) = config.allowed_operations {
                        self.instruction_config.allowed_operations = Some(operations);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        if let Some(ac) = &instruction_limiting_args.allowed_comparisons {
            // if allowed_comparisons are already set, merge with additional allowed comparisons
            let mut allowed_comparisons = match self.instruction_config.allowed_comparisons.take() {
                Some(value) => value,
                None => Vec::new(),
            };
            allowed_comparisons.append(&mut ac.clone());
            self.instruction_config.allowed_comparisons = Some(allowed_comparisons);
        }
        if let Some(ao) = &instruction_limiting_args.allowed_operations {
            let mut allowed_operations = match self.instruction_config.allowed_operations.take() {
                Some(value) => value,
                None => Vec::new(),
            };
            allowed_operations.append(&mut ao.clone());
            self.instruction_config.allowed_operations = Some(allowed_operations);
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
            // update runtime settings
            let mut runtime_settings = match self.runtime_settings.take() {
                Some(runtime_settings) => runtime_settings,
                None => RuntimeSettings::default(),
            };
            runtime_settings.autodetect_accumulators = false;
            runtime_settings.autodetect_gamma_accumulator = false;
            runtime_settings.autodetect_memory_cells = false;
            runtime_settings.autodetect_index_memory_cells = false;
            self.runtime_settings = Some(runtime_settings);
        }
        Ok(self)
    }

    /// Builds a new runtime by consuming this `RuntimeBuilder`.
    ///
    /// Prints status messages into stdout.
    pub fn build(mut self) -> miette::Result<Runtime> {
        // set runtime settings
        let settings = self.runtime_settings.unwrap_or_default();

        // build memory
        let mut memory = match &self.memory_config {
            Some(memory_config) => RuntimeMemory::from(memory_config.to_owned()),
            None => RuntimeMemory::default(),
        };

        // check if gamma is used as index for index memory cell even though gamma is fully disabled
        // replace that gamma command with labeled memory cell access
        if let None = memory.gamma {
            // gamma is definitely currently disabled
            if !settings.autodetect_gamma_accumulator {
                // gamma can not be created automatically so gamma can not exist in runtime
                replace_gamma_as_index_instructions(&mut self.instructions);
            }
        }

        // check if instructions are used that are not allowed
        if let Err(e) = check_instructions(&self.instructions, &self.instruction_config) {
            return Err(miette::Report::new(*e));
        }

        // inject end labels to give option to end program using goto END
        inject_end_labels(&mut self.control_flow, self.instructions.len());

        if let Err(e) = check_labels(&self.control_flow, &self.instructions) {
            return Err(miette::Report::new(RuntimeBuildError::LabelUndefined(e)));
        }

        // Check if all used accumulators and memory_cells exist
        check_missing_vars(
            self.memory_config
                .as_ref()
                .unwrap_or(&MemoryConfig::default()),
            &self.instructions,
            &mut memory,
        )?;

        // check if main label is set and update instruction pointer if found
        if let Some(i) = self.control_flow.instruction_labels.get("main") {
            self.control_flow.next_instruction_index = *i;
            self.control_flow.initial_instruction = *i;
        }
        if let Some(i) = self.control_flow.instruction_labels.get("MAIN") {
            self.control_flow.next_instruction_index = *i;
            self.control_flow.initial_instruction = *i;
        }

        Ok(Runtime {
            memory,
            instructions: self.instructions,
            control_flow: self.control_flow,
            instruction_runs: 0,
            settings,
        })
    }
}

/// Builds the provided instructions.
///
/// Updates the provided control flow with labels.
fn build_instructions(
    instructions_input: &[String],
    file_name: &str,
    control_flow: &mut ControlFlow,
) -> Result<Vec<Instruction>, Box<BuildProgramError>> {
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
        return Err(Box::new(BuildProgramError {
            reason: BuildProgramErrorTypes::MainLabelDefinedMultipleTimes,
        }));
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

/// Checks instructions that are set by comparing them with the provided whitelist of instructions.
///
/// NOOP instructions are always allowed.
///
/// It is also checked if any comparisons or operations are used that are not allowed.
/// If `instructions` contains instructions, comparisons or operations that are not included in `instruction_config`,
/// an error is returned. If values are not set in `instruction_config` everything of that type is allowed.
pub fn check_instructions(
    instructions: &[Instruction],
    instruction_config: &InstructionConfig,
) -> Result<(), Box<BuildProgramError>> {
    for (idx, i) in instructions.iter().enumerate() {
        if let Some(whitelist) = &instruction_config.allowed_instruction_identifiers {
            if !whitelist.contains(&i.identifier()) && i.identifier() != "NOOP" {
                // Instruction found, that is forbidden
                let mut allowed_instructions = whitelist
                    .iter()
                    .map(String::to_string)
                    .collect::<Vec<String>>();
                allowed_instructions.sort();
                return Err(Box::new(BuildProgramError {
                    reason: BuildProgramErrorTypes::InstructionNotAllowed(
                        idx + 1,
                        format!("{i}"),
                        i.identifier(),
                        allowed_instructions.join("\n").to_string(),
                    ),
                }));
            }
        }
        // Check if all comparisons are allowed
        if let Some(ac) = &instruction_config.allowed_comparisons {
            if let Some(c) = i.comparison() {
                if !ac.contains(c) {
                    return Err(Box::new(BuildProgramError {
                        reason: BuildProgramErrorTypes::ComparisonNotAllowed(
                            idx + 1,
                            c.to_string(),
                            c.cli_hint(),
                        ),
                    }));
                }
            }
        }
        // Check if all operations are allowed
        if let Some(ao) = &instruction_config.allowed_operations {
            if let Some(o) = i.operation() {
                if !ao.contains(o) {
                    return Err(Box::new(BuildProgramError {
                        reason: BuildProgramErrorTypes::OperationNotAllowed(
                            idx + 1,
                            o.to_string(),
                            o.cli_hint(),
                        ),
                    }));
                }
            }
        }
    }
    Ok(())
}

/// Replaces all index accesses with gamma for memory cells with normal memory cell access.
///
/// So `p(y)` (where y is used as index for the index memory cell) is now changed to a normal
/// memory cell access, where `y` is the label of a specific memory cell.
fn replace_gamma_as_index_instructions(instructions: &mut Vec<Instruction>) {
    for instruction in instructions {
        match instruction {
            Instruction::Assign(target, value) => {
                let target = if target.is_imc_gamma() {
                    TargetType::MemoryCell("y".to_string())
                } else {
                    target.clone()
                };
                let value = if value.is_imc_gamma() {
                    Value::MemoryCell("y".to_string())
                } else {
                    value.clone()
                };
                *instruction = Instruction::Assign(target, value);
            }
            Instruction::Calc(target, value_a, op, value_b) => {
                let target = if target.is_imc_gamma() {
                    TargetType::MemoryCell("y".to_string())
                } else {
                    target.clone()
                };
                let value_a = if value_a.is_imc_gamma() {
                    Value::MemoryCell("y".to_string())
                } else {
                    value_a.clone()
                };
                let value_b = if value_b.is_imc_gamma() {
                    Value::MemoryCell("y".to_string())
                } else {
                    value_b.clone()
                };
                *instruction = Instruction::Calc(target, value_a, *op, value_b);
            }
            Instruction::JumpIf(value_a, cmp, value_b, label) => {
                let value_a = if value_a.is_imc_gamma() {
                    Value::MemoryCell("y".to_string())
                } else {
                    value_a.clone()
                };
                let value_b = if value_b.is_imc_gamma() {
                    Value::MemoryCell("y".to_string())
                } else {
                    value_b.clone()
                };
                *instruction = Instruction::JumpIf(value_a, *cmp, value_b, label.clone());
            }
            _ => (),
        }
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        instructions::{
            error_handling::{BuildProgramError, BuildProgramErrorTypes},
            IndexMemoryCellIndexType, Instruction,
        },
        runtime::{
            builder::{
                build_instructions, check_index_memory_cell, check_instructions, InstructionConfig,
            },
            error_handling::RuntimeBuildError,
            ControlFlow, RuntimeMemory,
        },
        utils::test_utils,
    };

    #[test]
    fn test_instruction_building_with_comments() {
        let instructions = r#"
            a0 := 4 // Set alpha to 4
            p(h1) := a0 # Set memory cell h1 to 4
            a0 := a1 # Just some stuff
            a1 := a2 // Just some more stuff
        "#;
        assert!(test_utils::runtime_from_str_with_default_cli_args(instructions).is_ok());
    }

    #[test]
    fn test_instruction_building_with_semicolons() {
        let instructions = r#"
            a0 := 4; // Set alpha to 4,
            p(h1) := a0; # Set memory cell h1 to 4,
            a0 := a1; # Just some stuff,
            a1 := a2; // Just some more stuff,
        "#;
        assert!(test_utils::runtime_from_str_with_default_cli_args(instructions).is_ok());
    }

    #[test]
    fn test_only_label_line() {
        let instructions = r#"
            a0 := 5
            my_label:
            a1 := 5
        "#;
        assert!(test_utils::runtime_from_str_with_default_cli_args(instructions).is_ok());
    }

    #[test]
    fn test_accumulator_auto_add_working() {
        let instructions = r#"
            a1 := a2 + a3
        "#;
        let rt = test_utils::runtime_from_str_with_default_cli_args(instructions).unwrap();
        assert!(rt.memory.accumulators.contains_key(&1));
        assert!(rt.memory.accumulators.contains_key(&2));
        assert!(rt.memory.accumulators.contains_key(&3));
        assert!(!rt.memory.accumulators.contains_key(&4));
    }

    #[test]
    fn test_check_index_memory_cell() {
        let mut args = RuntimeMemory::new_empty();
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

    // a simple helper function to make it easier to build test instructions
    fn build_instructions_test(input: &str) -> Result<Vec<Instruction>, Box<BuildProgramError>> {
        let lines = input
            .split('\n')
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        build_instructions(&lines, "test", &mut ControlFlow::new())
    }

    #[test]
    fn test_bpe_label_defined_multiple_times() {
        let res = build_instructions_test("loop:\n\nloop:");
        assert_eq!(
            res,
            Err(Box::new(BuildProgramError {
                reason: BuildProgramErrorTypes::LabelDefinedMultipleTimes("loop".to_string())
            }))
        )
    }

    #[test]
    fn test_bpe_main_label_defined_multiple_times() {
        let res = build_instructions_test("main:\n\nMAIN:");
        assert_eq!(
            res,
            Err(Box::new(BuildProgramError {
                reason: BuildProgramErrorTypes::MainLabelDefinedMultipleTimes
            }))
        );
        let res = build_instructions_test("main:\n\nmain:");
        assert_eq!(
            res,
            Err(Box::new(BuildProgramError {
                reason: BuildProgramErrorTypes::MainLabelDefinedMultipleTimes
            }))
        )
    }

    #[test]
    fn test_bpe_instruction_not_allowed() {
        let instructions = build_instructions_test("a := 5").unwrap();
        let mut allowed_instruction_identifiers = HashSet::new();
        allowed_instruction_identifiers.insert("A := H".to_string());
        let allowed_instructions = InstructionConfig {
            allowed_instruction_identifiers: Some(allowed_instruction_identifiers),
            allowed_comparisons: None,
            allowed_operations: None,
        };
        let res = check_instructions(&instructions, &allowed_instructions);
        assert_eq!(
            res,
            Err(Box::new(BuildProgramError {
                reason: BuildProgramErrorTypes::InstructionNotAllowed(
                    1,
                    "a0 := 5".to_string(),
                    "A := C".to_string(),
                    "A := H".to_string()
                )
            }))
        );
    }

    #[test]
    fn test_bpe_comparison_not_allowed() {
        let instructions = build_instructions_test("if a == a then goto loop").unwrap();
        let allowed_instructions = InstructionConfig {
            allowed_instruction_identifiers: None,
            allowed_comparisons: Some(Vec::new()),
            allowed_operations: None,
        };
        assert!(check_instructions(&instructions, &allowed_instructions).is_err());
    }

    #[test]
    fn test_bpe_operation_not_allowed() {
        let instructions = build_instructions_test("a := a + p(h1)").unwrap();
        let allowed_instructions = InstructionConfig {
            allowed_instruction_identifiers: None,
            allowed_comparisons: None,
            allowed_operations: Some(Vec::new()),
        };
        assert!(check_instructions(&instructions, &allowed_instructions).is_err());
    }
}
