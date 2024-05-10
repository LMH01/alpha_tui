use std::{
    fs::File,
    io::{self, Stdout},
    path::Path,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use directories::UserDirs;
use miette::{miette, IntoDiagnostic, Result};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    cli::LoadPlaygroundArgs,
    instructions::Instruction,
    utils::{self, remove_comment},
};

use super::ui::style::Theme;

/// Check command
pub mod check;
/// Load command
pub mod load;
/// Playground command
pub mod playground;

fn load_instruction_history(
    custom_instruction_history_file: &Option<String>,
) -> Result<Option<Vec<String>>> {
    let mut instruction_history = None;
    if let Some(file) = custom_instruction_history_file {
        let path = Path::new(file);
        if !path.exists() {
            // create file
            File::create(path).into_diagnostic()?;
        }

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

/// Uses the provided `load_playground_args` to select the theme that should be used.
///
/// Checks if theme file is located in config directory (`~/.config/alpha_tui` in linux
/// or `%APPDATA%/alpha_tui` in windows). This file is used to load the theme if
/// `--theme` or `--theme-file` are not set, if they are set, they will overwrite the selected theme.
///
/// If nothing is set, the default theme is loaded.
///
/// Returns error if theme file should be loaded that is invalid.
fn load_theme(load_playground_args: &LoadPlaygroundArgs) -> miette::Result<Theme> {
    // check if cli args are set
    if let Some(theme) = &load_playground_args.theme {
        return Ok(Theme::from(theme));
    }
    if let Some(theme_file) = &load_playground_args.theme_file {
        return Ok(load_theme_file(&theme_file)?);
    }
    // check if theme file exists
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(base_dir) = user_dirs.home_dir().to_str() {
            let file = format!("{base_dir}/.config/alpha_tui/theme.json");
            let path = Path::new(&file);
            // check if file exists
            if path.exists() && path.is_file() {
                return Ok(load_theme_file(
                    path.to_str().expect(" path should be valid unicode"),
                )?);
            }
        }
    }
    // return default
    Ok(Theme::default())
}

/// Loads the content of the file located at `path` and tries to parse it into a theme.
///
/// Returns error if file does not exist or content can not be parsed into a theme.
fn load_theme_file(path: &str) -> miette::Result<Theme> {
    match serde_json::from_str(&utils::read_file(path)?.join("\n")) {
        Ok(theme) => Ok(theme),
        Err(e) => Err(miette::miette!(
            "json parse error while loading theme file {path}: {e}"
        )),
    }
}
