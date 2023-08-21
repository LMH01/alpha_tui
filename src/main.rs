use std::io;

use clap::Parser;
use cli::Args;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use miette::{Result, IntoDiagnostic, Context, miette};
use ::ratatui::{backend::CrosstermBackend, Terminal};
use utils::read_file;

use crate::{
    runtime::builder::RuntimeBuilder, tui::App,
};

/// Contains all required data types used to run programs
mod base;
/// Command line parsing
mod cli;
/// Supported instructions
mod instructions;
/// Program execution
mod runtime;
/// Utility functions
mod utils;
/// Terminal user interface
mod tui;

fn main() -> Result<()> {
    let args = Args::parse();

    let instructions = match read_file(&args.input) {
        Ok(i) => i,
        Err(e) => {
            return Err(miette!("Unable to read file [{}]: {}", args.input, e));
        }
    };
    println!("Building program");
    let mut rb = match RuntimeBuilder::from_args(&args) {
        Ok(rb) => rb,
        Err(e) => {
            return Err(miette!("Unable to create RuntimeBuilder, memory cells could not be loaded from file:\n{e}"));
        },
    };
    rb.build_instructions(&instructions.iter().map(|s| s.as_str()).collect(), &args.input)?;
    
    println!("Building runtime");
    let rt = rb.build().wrap_err("while building runtime")?;

    // tui
    // setup terminal
    enable_raw_mode().into_diagnostic()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).into_diagnostic()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // create app
    println!("Ready to run, launching tui");
    let mut app = App::from_runtime(rt, args.input, instructions);
    let res = app.run(&mut terminal);

    // restore terminal
    disable_raw_mode().into_diagnostic()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).into_diagnostic()?;
    terminal.show_cursor().into_diagnostic()?;

    res?;
    Ok(())
}
