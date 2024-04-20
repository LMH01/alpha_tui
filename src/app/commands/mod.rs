use std::io::{self, Stdout};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use miette::{miette, IntoDiagnostic, Result};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    cli::{GlobalArgs, InstructionLimitingArgs},
    instructions::Instruction,
    runtime::builder::RuntimeBuilder,
    utils::{self, remove_comment},
};

/// Check command
pub mod check;
/// Load command
pub mod load;
/// Sandbox command
pub mod sandbox;

fn load_instruction_history(
    custom_instruction_history_file: &Option<String>,
) -> Result<Option<Vec<String>>> {
    let mut instruction_history = None;
    if let Some(file) = custom_instruction_history_file {
        // load content of file
        let content = match utils::read_file(file) {
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
                    .into_build_program_error(content.join("\n"), file, idx + 1)
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

/// Creates a runtime builder and returns miette error if it fails.
fn create_runtime_builder(
    global_args: &GlobalArgs,
    ila: &InstructionLimitingArgs,
) -> Result<RuntimeBuilder> {
    match RuntimeBuilder::from_args(global_args, ila) {
        Ok(rb) => Ok(rb),
        Err(e) => Err(miette!(
            "Unable to create RuntimeBuilder, memory config could not be loaded from file:\n{e}"
        )),
    }
}

/// Setup the terminal and returns it.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    terminal::enable_raw_mode().into_diagnostic()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture).into_diagnostic()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).unwrap();
    Ok(terminal)
}

/// Restores the terminal.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal::disable_raw_mode().into_diagnostic()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .into_diagnostic()?;
    terminal.show_cursor().into_diagnostic()?;
    Ok(())
}
