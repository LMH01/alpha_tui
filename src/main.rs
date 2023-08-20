use std::{error::Error, io, process::exit, thread, time::Duration, collections::HashMap, hash::Hash};

use clap::Parser;
use cli::Args;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use runtime::ControlFlow;
use ::tui::{backend::CrosstermBackend, Terminal};
use utils::read_file;

use crate::{
    base::{Comparison, Operation},
    instructions::Instruction,
    runtime::{Runtime, RuntimeArgs, RuntimeBuilder}, tui::App,
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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let instructions = match read_file(&args.input) {
        Ok(i) => i,
        Err(e) => {
            println!("Unable to read file: {}", e);
            exit(-1);
        }
    };
    println!("Building program");
    let mut rb = RuntimeBuilder::from_args(&args);
    match rb.build_instructions(&instructions.iter().map(|s| s.as_str()).collect()) {
        Ok(_) => (),
        Err(e) => {
            println!("{e}");
            exit(-1);
        }
    };
    println!("Building runtime");
    let mut rt = match rb.build() {
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
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // create app
    let mut app = App::from_runtime(rt, args.input, instructions);
    let res = app.run(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match res {
        Ok(()) => exit(0),
        Err(e) => {
            println!("{e}");
            exit(-1);
        }
    }

}
