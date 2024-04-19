use miette::{miette, Result};

use crate::{
    cli::LoadArgs,
    instructions::Instruction,
    utils::{self, remove_comment},
};

/// Check command
pub mod check;
/// Load command
pub mod load;

fn load_instruction_history(load_args: &LoadArgs) -> Result<Option<Vec<String>>> {
    let mut instruction_history = None;
    if let Some(file) = &load_args.custom_instruction_history_file {
        // load content of file
        let content = match utils::read_file(&file) {
            Ok(content) => content,
            Err(e) => {
                return Err(miette!(
                    "Unable to read custom instruction history file:\n{e}"
                ))
            }
        };
        println!("Instruction history provided, checking validity of provided instructions");
        let mut checked_instructions = Vec::new();
        for (idx, instruction) in content.iter().enumerate() {
            // remove comment
            let instruction = remove_comment(instruction);
            // remove label if it exists
            let mut splits = instruction.split_whitespace().collect::<Vec<&str>>();
            if splits.is_empty() {
                continue;
            }
            if splits[0].ends_with(':') {
                splits.remove(0);
            }
            let instruction = splits.join(" ");
            if let Err(e) = Instruction::try_from(instruction.as_str()) {
                return Err(e
                    .into_build_program_error(content.join("\n"), &file, idx + 1)
                    .into());
            }
            // check if this instruction is not already contained
            if !checked_instructions.contains(&instruction) {
                checked_instructions.push(instruction);
            }
        }
        println!("Instruction history checked successfully");
        instruction_history = Some(checked_instructions);
    }
    Ok(instruction_history)
}
