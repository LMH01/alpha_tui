use app::commands;
use clap::Parser;
use cli::Cli;
use miette::Result;

use crate::cli::Commands;

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
        Commands::Load(ref args) => Some(args.file.clone()),
        Commands::Check(ref args) => Some(args.file.clone()),
        Commands::Sandbox(_) => None,
    };

    if cli.global_args.disable_instruction_limit {
        println!(
            "Warning: instruction limit is disabled, this might lead to performance problems!"
        );
    }

    match &cli.command {
        Commands::Check(check_args) => commands::check::check(
            &cli.global_args,
            check_args,
            &read_file(&input_file.as_ref().unwrap())?,
            &input_file.unwrap(),
        ),
        Commands::Load(load_args) => commands::load::load(
            &cli.global_args,
            load_args,
            read_file(&input_file.as_ref().unwrap())?,
            input_file.unwrap(),
        )?,
        Commands::Sandbox(sandbox_args) => {
            commands::sandbox::sandbox(&cli.global_args, sandbox_args)?
        }
    }
    Ok(())
}

fn read_file(path: &str) -> Result<Vec<String>> {
    match utils::read_file(&path) {
        Ok(i) => Ok(i),
        Err(e) => Err(miette::miette!("Unable to read file [{}]: {}", &path, e)),
    }
}
