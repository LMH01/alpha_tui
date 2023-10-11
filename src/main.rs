use std::{io, process::exit};

use ::ratatui::{backend::CrosstermBackend, Terminal};
use clap::Parser;
use cli::Cli;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use miette::{miette, Context, IntoDiagnostic, Report, Result};
use utils::read_file;

use crate::{
    cli::Commands,
    runtime::builder::RuntimeBuilder,
    tui::App,
    utils::{build_instructions_with_whitelist, pretty_format_instructions, write_file},
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
        Commands::Check(ref args) => args.file.clone(),
    };

    let instructions = match read_file(&input) {
        Ok(i) => i,
        Err(e) => {
            return Err(miette!("Unable to read file [{}]: {}", &input, e));
        }
    };

    match cli.command {
        Commands::Check(_) => cmd_check(&cli, &instructions, &input),
        Commands::Load(_) => cmd_load(&cli, instructions, input)?,
    }
    Ok(())
}

fn cmd_check(cli: &Cli, instructions: &[String], input: &str) {
    println!("Building program");
    let mut rb = match RuntimeBuilder::from_args(cli) {
        Ok(rb) => rb,
        Err(e) => {
            println!(
                "Check unsuccessful: {:?}",
                miette!(
                "Unable to create RuntimeBuilder, memory cells could not be loaded from file:\n{e}"
            )
            );
            exit(10);
        }
    };

    if let Some(file) = cli.allowed_instructions_file.as_ref() {
        match build_instructions_with_whitelist(&mut rb, instructions, input, file) {
            Ok(_) => (),
            Err(e) => {
                println!("Check unsuccessful: {:?}", miette!("Unable to create RuntimeBuilder, allowed instructions could not be loaded from file:\n{e}"));
            }
        }
    } else if let Err(e) =
        rb.build_instructions(&instructions.iter().map(String::as_str).collect(), input)
    {
        println!(
            "Check unsuccessful, program did not compile.\nError: {:?}",
            Report::new(e)
        );
        exit(1);
    }
    println!("Check successful");
}

#[allow(clippy::match_wildcard_for_single_variants)]
fn cmd_load(cli: &Cli, instructions: Vec<String>, input: String) -> Result<()> {
    println!("Building program");
    let mut rb = match RuntimeBuilder::from_args(cli) {
        Ok(rb) => rb,
        Err(e) => {
            return Err(miette!(
                "Unable to create RuntimeBuilder, memory cells could not be loaded from file:\n{e}"
            ));
        }
    };

    if let Some(file) = cli.allowed_instructions_file.as_ref() {
        build_instructions_with_whitelist(&mut rb, &instructions, &input, file)?;
    } else {
        rb.build_instructions(&instructions.iter().map(String::as_str).collect(), &input)?;
    }

    // format instructions pretty if cli flag is set
    let instructions = match cli.command {
        Commands::Load(ref args) => {
            if args.disable_alignment {
                instructions
            } else {
                pretty_format_instructions(&instructions)
            }
        }
        _ => pretty_format_instructions(&instructions),
    };

    println!("Building runtime");
    let rt = rb.build().wrap_err("while building runtime")?;

    if let Commands::Load(ref args) = cli.command {
        if args.write_alignment {
            // write new formatting to file if enabled
            println!("Writing alignment to source file");
            write_file(&instructions, &input)?;
        }
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
    let mut app = match cli.command {
        Commands::Load(ref args) => App::from_runtime(rt, input, &instructions, &args.breakpoints),
        _ => App::from_runtime(rt, input, &instructions, &None),
    };
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
