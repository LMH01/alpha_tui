use std::process::exit;

use miette::{miette, Report};

use crate::{
    cli::{CheckArgs, GlobalArgs},
    runtime::builder::RuntimeBuilder,
    utils::build_instructions_with_whitelist,
};

pub fn check(
    global_args: &GlobalArgs,
    check_args: &CheckArgs,
    instructions: &[String],
    input: &str,
) {
    println!("Building program");
    let mut rb = match RuntimeBuilder::from_args(global_args, &check_args.instruction_limiting_args) {
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

    if let Some(file) = check_args.instruction_limiting_args.allowed_instructions_file.as_ref() {
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
