use clap::Parser;
use cli::Cli;
use commands::{check, load};
use miette::Result;
use utils::read_file;

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
            return Err(miette::miette!("Unable to read file [{}]: {}", &input, e));
        }
    };

    match &cli.command {
        Commands::Check(_) => check::check(&cli, &instructions, &input),
        Commands::Load(args) => load::load(&cli, instructions, input, args.clone())?,
    }
    Ok(())
}
