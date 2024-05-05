use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    base::{Accumulator, MemoryCell},
    cli::{GlobalArgs, InstructionLimitingArgs},
    utils,
};

use super::RuntimeMemory;

/// Contains configuration values on how the memory layout should be configured, meaning what memory locations should be
/// available and pre initialized. Also stores if memory locations should be created if the are accessed but they don't exist already.
///
/// Can be used in the runtime builder to configure the memory values that should be available in the build runtime.
#[derive(PartialEq, Debug, Deserialize, Serialize, Default, Clone)]
pub struct MemoryConfig {
    pub accumulators: AccumulatorConfig,
    pub gamma_accumulator: GammaAccumulatorConfig,
    pub memory_cells: MemoryCellConfig,
    pub index_memory_cells: IndexMemoryCellConfig,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Default, Clone)]
pub struct AccumulatorConfig {
    pub values: HashMap<usize, Option<i32>>,
    pub autodetection: Option<bool>,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Default, Clone)]
pub struct GammaAccumulatorConfig {
    pub enabled: bool,
    pub value: Option<i32>,
    pub autodetection: Option<bool>,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Default, Clone)]
pub struct MemoryCellConfig {
    pub values: HashMap<String, Option<i32>>,
    pub autodetection: Option<bool>,
}

#[derive(PartialEq, Debug, Deserialize, Serialize, Default, Clone)]
pub struct IndexMemoryCellConfig {
    pub values: HashMap<usize, Option<i32>>,
    pub autodetection: Option<bool>,
}

impl MemoryConfig {
    /// Creates runtime memory from this memory config.
    #[deprecated(note = "This will be replaced by functions on the runtime builder")]
    pub fn into_runtime_memory(
        self,
        args: &GlobalArgs,
        ila: &InstructionLimitingArgs,
    ) -> RuntimeMemory {
        //let mut accumulators = HashMap::new();
        //for (idx, value) in self.accumulators {
        //    accumulators.insert(
        //        idx,
        //        Accumulator {
        //            id: idx,
        //            data: value,
        //        },
        //    );
        //}
        //let gamma = if self.gamma_accumulator.enabled {
        //    match self.gamma_accumulator.value {
        //        Some(value) => Some(Some(value)),
        //        None => Some(None),
        //    }
        //} else {
        //    None
        //};
        //let mut memory_cells = HashMap::new();
        //for (name, value) in self.memory_cells {
        //    memory_cells.insert(
        //        name.clone(),
        //        MemoryCell {
        //            label: name,
        //            data: value,
        //        },
        //    );
        //}
        //let mut index_memory_cells = HashMap::new();
        //for (idx, value) in self.index_memory_cells {
        //    index_memory_cells.insert(idx, value);
        //}
        //RuntimeMemory {
        //    accumulators,
        //    gamma,
        //    memory_cells,
        //    index_memory_cells,
        //    stack: Vec::new(),
        //    settings: RuntimeSettings::new(
        //        !ila.disable_memory_detection,
        //        args.disable_instruction_limit,
        //        !ila.disable_memory_detection,
        //    ),
        //}
        todo!("Move function into runtime builder subfunction, that should be named something like 'apply_memory_config' or 'from_memory_config'")
    }

    /// Tries to parse the provided file into a memory config.
    pub fn try_from_file(path: &str) -> miette::Result<Self, String> {
        match serde_json::from_str::<MemoryConfig>(&utils::read_file(path)?.join("\n")) {
            Ok(config) => return Ok(config),
            Err(e) => return Err(format!("json parse error: {e}")),
        };
    }
}
