use std::process::exit;

use miette::miette;

use crate::{
    cli::{CheckArgs, CheckCommand, GlobalArgs},
    runtime::builder::RuntimeBuilder,
};

pub fn check(
    global_args: &GlobalArgs,
    check_args: &CheckArgs,
    instructions: Vec<String>,
    input: &str,
) {
    // create runtime builder and apply cli args
    println!("Building instructions");
    let mut rb = match RuntimeBuilder::new(&instructions, input) {
        Ok(rb) => rb,
        Err(e) => {
            println!(
                "Check unsuccessful, program did not compile.\nError: {:?}",
                miette!(e)
            );
            exit(1);
        }
    };

    println!("Building runtime");
    if let Err(e) = rb.apply_global_cli_args(global_args) {
        println!(
            "Check unsuccessful: {:?}",
            miette!(
                "Unable to create RuntimeBuilder, memory config could not be loaded from file:\n{e}"
            )
        );
        exit(10);
    }
    if let Err(e) =
        rb.apply_instruction_limiting_args(&check_args.check_load_args.instruction_limiting_args)
    {
        println!(
            "Check unsuccessful: {:?}",
            miette!("Unable to create RuntimeBuilder:\n{:?}", e)
        );
        exit(1);
    }
    if let Err(e) = rb.apply_check_load_args(&check_args.check_load_args) {
        println!(
            "Check unsuccessful: {:?}",
            miette!("Unable to create RuntimeBuilder:\n{:?}", e)
        );
        exit(1);
    }
    // build runtime
    let mut rt = match rb.build() {
        Ok(rt) => rt,
        Err(e) => {
            println!(
                "Check unsuccessful, program did not compile.\nError: {:?}",
                miette!(e)
            );
            exit(1);
        }
    };

    match check_args.command {
        CheckCommand::Compile => {
            println!("Check successful");
            return;
        }
        CheckCommand::Run => (),
    }

    // run runtime
    if let Err(e) = rt.run() {
        println!(
            "Check unsuccessful, runtime error while running program.\nError: {:?}",
            miette!(e)
        );
        exit(1);
    }

    println!("Check successful");
}
