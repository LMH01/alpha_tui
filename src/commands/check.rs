use std::process::exit;

use miette::{miette, Report};

use crate::{cli::Cli, runtime::builder::RuntimeBuilder, utils::build_instructions_with_whitelist};

pub fn check(cli: &Cli, instructions: &[String], input: &str) {
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
