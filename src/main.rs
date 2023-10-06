use std::io;

use ::ratatui::{backend::CrosstermBackend, Terminal};
use clap::Parser;
use cli::Cli;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use miette::{miette, Context, IntoDiagnostic, Result};
use utils::read_file;

use crate::{
    runtime::builder::RuntimeBuilder,
    tui::App,
    utils::{pretty_format_instructions, write_file}, cli::{LoadArgs, Commands},
};

/// Contains all required data types used to run programs
mod base;
/// Command line parsing
mod cli;
/// Supported instructions
mod instructions;
//mod instructions_new;
/// Program execution
mod runtime;
/// Terminal user interface
mod tui;
/// Utility functions
mod utils;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let input = match cli.command {
        Commands::Load(ref args) => args.file.clone(),
    };

    let instructions = match read_file(&input) {
        Ok(i) => i,
        Err(e) => {
            return Err(miette!("Unable to read file [{}]: {}", &input, e));
        }
    };
    println!("Building program");
    let mut rb = match RuntimeBuilder::from_args(&cli) {
        Ok(rb) => rb,
        Err(e) => {
            return Err(miette!(
                "Unable to create RuntimeBuilder, memory cells could not be loaded from file:\n{e}"
            ));
        }
    };
    rb.build_instructions(
        &instructions.iter().map(String::as_str).collect(),
        &input,
    )?;

    // format instructions pretty if cli flag is set
    let instructions = match cli.disable_alignment {
        false => pretty_format_instructions(&instructions),
        true => instructions,
    };

    println!("Building runtime");
    let rt = rb.build().wrap_err("while building runtime")?;

    // write new formatting to file if enabled
    if cli.write_alignment {
        println!("Writing alignment to source file");
        write_file(&instructions, &input)?;
    }

    // tui
    // setup terminal
    println!("Ready to run, launching tui");
    enable_raw_mode().into_diagnostic()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).into_diagnostic()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // create app
    let mut app = App::from_runtime(rt, input, &instructions, &cli.breakpoints);
    let res = app.run(&mut terminal);

    // restore terminal
    disable_raw_mode().into_diagnostic()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .into_diagnostic()?;
    terminal.show_cursor().into_diagnostic()?;

    res?;
    Ok(())
}
