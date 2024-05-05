use app::commands;
use clap::Parser;
use cli::Cli;
use miette::Result;

use crate::cli::Command;

/// The application itself
mod app;
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    // perform additional validation checks on provided cli arguments
    cli::validate_arguments(&cli)?;

    let input_file = match cli.command {
        Command::Load(ref args) => Some(args.file.clone()),
        Command::Check(ref args) => Some(args.file.clone()),
        Command::Playground(_) => None,
    };

    if cli.global_args.disable_instruction_limit {
        println!(
            "Warning: instruction limit is disabled, this might lead to performance problems!"
        );
    }

    match &cli.command {
        Command::Check(check_args) => commands::check::check(
            &cli.global_args,
            check_args,
            read_file(input_file.as_ref().unwrap())?,
            &input_file.unwrap(),
        ),
        Command::Load(load_args) => commands::load::load(
            &cli.global_args,
            load_args,
            read_file(input_file.as_ref().unwrap())?,
            input_file.unwrap(),
        )?,
        Command::Playground(playground_args) => {
            commands::playground::playground(&cli.global_args, playground_args)?
        }
    }
    Ok(())
}

fn read_file(path: &str) -> Result<Vec<String>> {
    match utils::read_file(path) {
        Ok(i) => Ok(i),
        Err(e) => Err(miette::miette!("Unable to read file [{}]: {}", &path, e)),
    }
}
