use std::collections::HashSet;

use miette::Result;
use serde::{Deserialize, Serialize};

use crate::{
    base::{Comparison, Operation},
    utils,
};

/// Stores information that is used to limit what instructions should be allowed.
#[derive(Default)]
pub struct InstructionConfig {
    /// Stores the ids of instructions that are allowed.
    ///
    /// If the value is `None` all instructions are allowed.
    pub allowed_instruction_identifiers: Option<HashSet<String>>,
    /// Stores comparisons that are allowed, if value is `None`, all comparisons are allowed.
    pub allowed_comparisons: Option<Vec<Comparison>>,
    /// Stores operations that are allowed, if value is `None`, all operations are allowed.
    pub allowed_operations: Option<Vec<Operation>>,
}

impl InstructionConfig {
    /// Tries to parse the provided file into a instruction config.
    ///
    /// Uses `RawInstructionConfig` to initially parse the file and if allowed instructions are set, they are parsed and
    /// the ids are stored.
    pub fn try_from_file(path: &str) -> miette::Result<Self> {
        let raw = match serde_json::from_str::<RawInstructionConfig>(
            &utils::read_file_new(path)?.join("\n"),
        ) {
            Ok(config) => config,
            // TODO change error return type to return RuntimeBuildError
            Err(e) => return Err(miette::miette!("json parse error: {e}")),
        };
        return Ok(raw.into_instruction_config()?);
    }
}

/// Data transfer object to parse the instruction config file.
#[derive(Serialize, Deserialize)]
struct RawInstructionConfig {
    /// Instructions that should be allowed
    instructions: Option<Vec<String>>,
    /// Comparisons that should be allowed
    comparisons: Option<Vec<Comparison>>,
    /// Operations that should be allowed
    operations: Option<Vec<Operation>>,
}

impl RawInstructionConfig {
    /// Converts this instruction config file into an instruction config.
    fn into_instruction_config(self) -> Result<InstructionConfig> {
        let allowed_instruction_identifiers = match self.instructions {
            Some(aii) => Some(utils::build_instruction_whitelist_new(aii, "")?),
            None => None,
        };
        Ok(InstructionConfig {
            allowed_instruction_identifiers,
            allowed_comparisons: self.comparisons,
            allowed_operations: self.operations,
        })
    }
}
