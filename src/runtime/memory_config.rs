use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::utils;

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

    /// Tries to parse the provided file into a memory config.
    pub fn try_from_file(path: &str) -> miette::Result<Self, String> {
        match serde_json::from_str::<MemoryConfig>(&utils::read_file(path)?.join("\n")) {
            Ok(config) => return Ok(config),
            Err(e) => return Err(format!("json parse error: {e}")),
        };
    }
}
