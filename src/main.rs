use std::{io, process::exit};

use clap::Parser;
use cli::Args;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use miette::{Result, IntoDiagnostic, NamedSource};
use ::ratatui::{backend::CrosstermBackend, Terminal};
use utils::read_file;

use crate::{
    runtime::RuntimeBuilder, tui::App, instructions::BuildProgramError,
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
            println!("Unable to read file [{}]: {}", args.input, e);
            exit(-1);
        }
    };
    println!("Building program");
    let mut rb = match RuntimeBuilder::from_args(&args) {
        Ok(rb) => rb,
        Err(e) => {
            println!("Unable to create RuntimeBuilder, memory cells could not be loaded from file:\n{e}");
            exit(-1);
        },
    };
    //rb.build_instructions(&instructions.iter().map(|s| s.as_str()).collect())?;
    match rb.build_instructions(&instructions.iter().map(|s| s.as_str()).collect()) {
        Ok(_) => (),
        Err(e) => {
            Err(BuildProgramError {
                src: NamedSource::new(&args.input, instructions.clone().join("\n")),
                bad_bit: e.position(1),
                reason: e,
            })?
        }
    };
    println!("Building runtime");
    let rt = match rb.build() {
        Ok(rt) => rt,
        Err(e) => {
            println!("Unable to build runtime: {:?}", e);
            exit(-1);
        }
    };
    println!("Ready to run, launching tui");
    //println!("----- Program start -----");
    //match rt.run() {
    //    Ok(_) => {
    //        println!("----- Program end -----");
    //        println!("Program run successfully")
    //    },
    //    Err(e) => {
    //        println!("Runtime error: {}", e);
    //        exit(-1);
    //    }
    //};

    //tui
    // setup terminal
    enable_raw_mode().into_diagnostic()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).into_diagnostic()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // create app
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

    match res {
        Ok(()) => exit(0),
        Err(e) => {
            println!("{e}");
            exit(-1);
        }
    }

}
