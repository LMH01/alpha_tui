use std::collections::HashSet;

use crate::{
    base::{Comparison, Operation},
    cli::GlobalArgs,
    instructions::Instruction,
};

use super::{
    error_handling::RuntimeBuildError, memory_config::MemoryConfig, ControlFlow, RuntimeMemory,
    RuntimeSettings,
};

pub struct RuntimeBuilder<'a> {
    instructions_input: &'a Vec<&'a str>,
    memory_config: Option<MemoryConfig>,
    runtime_settings: Option<RuntimeSettings>,
    instruction_config: InstructionConfig<'a>,
}

impl<'a> RuntimeBuilder<'a> {
    pub fn new(instructions_input: &'a Vec<&'a str>) -> Self {
        Self {
            instructions_input,
            memory_config: None,
            runtime_settings: None,
            instruction_config: InstructionConfig::default(),
        }
    }

    pub fn memory_config(&mut self, memory_config: MemoryConfig) -> &mut Self {
        self.memory_config = Some(memory_config);
        self
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
            for value in 1..=accumulators {
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
}

#[derive(Default)]
struct InstructionConfig<'a> {
    /// Stores the ids of instructions that are allowed.
    ///
    /// If the value is `None` all instructions are allowed.
    pub allowed_instruction_identifiers: Option<&'a HashSet<String>>,
    /// Stores comparisons that are allowed, if value is `None`, all comparisons are allowed.
    pub allowed_comparisons: Option<Vec<Comparison>>,
    /// Stores operations that are allowed, if value is `None`, all operations are allowed.
    pub allowed_operations: Option<Vec<Operation>>,
}
