use std::{io, process::exit};

use ::ratatui::{backend::CrosstermBackend, Terminal};
use clap::Parser;
use cli::{Cli, LoadArgs};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use miette::{miette, Context, IntoDiagnostic, Report, Result};
use utils::read_file;

use crate::{
    cli::Commands,
    instructions::Instruction,
    runtime::builder::RuntimeBuilder,
    tui::App,
    utils::{
        build_instructions_with_whitelist, pretty_format_instructions, remove_comment, write_file,
    },
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

    if cli.disable_instruction_limit {
        println!(
            "Warning: instruction limit is disabled, this might lead to performance problems!"
        );
    }

    let instructions = match read_file(&input) {
        Ok(i) => i,
        Err(e) => {
            return Err(miette!("Unable to read file [{}]: {}", &input, e));
        }
    };

    match &cli.command {
        Commands::Check(_) => cmd_check(&cli, &instructions, &input),
        Commands::Load(args) => cmd_load(&cli, instructions, input, args.clone())?,
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
                "Unable to create RuntimeBuilder, memory config could not be loaded from file:\n{e}"
            )
            );
            exit(10);
        }
    };

    if let Some(file) = cli.allowed_instructions_file.as_ref() {
        match build_instructions_with_whitelist(&mut rb, instructions, input, file) {
            Ok(_) => (),
            Err(e) => {
                println!(
                    "Check unsuccessful: {:?}",
                    miette!("Unable to create RuntimeBuilder:\n{:?}", e)
                );
                exit(1);
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
fn cmd_load(
    cli: &Cli,
    instructions: Vec<String>,
    input: String,
    load_args: LoadArgs,
) -> Result<()> {
    // check if command history is set
    let mut instruction_history = None;
    if let Some(file) = load_args.custom_instruction_history_file {
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

    println!("Building program");
    let mut rb = match RuntimeBuilder::from_args(cli) {
        Ok(rb) => rb,
        Err(e) => {
            return Err(miette!(
                "Unable to create RuntimeBuilder, memory config could not be loaded from file:\n{e}"
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
        Commands::Load(ref args) => App::from_runtime(
            rt,
            input,
            &instructions,
            &args.breakpoints,
            instruction_history,
            args.custom_instruction_history_file.clone(),
        ),
        _ => App::from_runtime(rt, input, &instructions, &None, instruction_history, None),
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
