use clap::Parser;
use cli::Cli;
use commands::{check, load, sandbox};
use miette::Result;

use crate::cli::Commands;

/// Contains all required data types used to run programs
mod base;
/// Command line parsing
mod cli;
/// Contains all commands that this app can run
mod commands;
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
    let input_file = match cli.command {
        Commands::Load(ref args) => Some(args.file.clone()),
        Commands::Check(ref args) => Some(args.file.clone()),
        Commands::Sandbox(_) => None
    };

    if cli.disable_instruction_limit {
        println!(
            "Warning: instruction limit is disabled, this might lead to performance problems!"
        );
    }

    match &cli.command {
        Commands::Check(_) => check::check(&cli, &read_file(&input_file.as_ref().unwrap())?, &input_file.unwrap()),
        Commands::Load(args) => load::load(&cli, read_file(&input_file.as_ref().unwrap())?, input_file.unwrap(), args.clone())?,
        Commands::Sandbox(args) => sandbox::sandbox(&cli, args)?,
    }
    Ok(())
}

fn read_file(path: &str) -> Result<Vec<String>> {
    match utils::read_file(&path) {
        Ok(i) => Ok(i),
        Err(e) => {
            Err(miette::miette!("Unable to read file [{}]: {}", &path, e))
        }
    }
}
