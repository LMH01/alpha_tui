use std::process::exit;

use miette::{miette, Context, Report};

use crate::{
    cli::{CheckArgs, GlobalArgs},
    runtime::builder_new::RuntimeBuilder,
};

pub fn check(
    global_args: &GlobalArgs,
    check_args: &CheckArgs,
    instructions: Vec<String>,
    input: &str,
) {
    // create runtime builder and apply cli args
    println!("Building instructions");
    let mut rb = match RuntimeBuilder::new(&instructions, &input) {
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
    if let Err(e) = rb.apply_instruction_limiting_args(&check_args.instruction_limiting_args) {
        println!(
            "Check unsuccessful: {:?}",
            miette!("Unable to create RuntimeBuilder:\n{:?}", e)
        );
        exit(1);
    }
    // build runtime
    if let Err(e) = rb.build() {
        println!(
            "Check unsuccessful, program did not compile.\nError: {:?}",
            miette!(e)
        );
        exit(1);
    }

    //let rt = match super::build_runtime(instructions, input, global_args, &check_args.instruction_limiting_args) {
    //    Ok(rt) => rt,
    //    Err(e) => {
    //    }
    //}

    //if let Some(file) = check_args
    //    .instruction_limiting_args
    //    .allowed_instructions_file
    //    .as_ref()
    //{
    //    match build_instructions_with_whitelist(&mut rb, instructions, input, file) {
    //        Ok(_) => (),
    //        Err(e) => {
    //            println!(
    //                "Check unsuccessful: {:?}",
    //                miette!("Unable to create RuntimeBuilder:\n{:?}", e)
    //            );
    //            exit(1);
    //        }
    //    }
    //} else if let Err(e) =
    //    rb.build_instructions(&instructions.iter().map(String::as_str).collect(), input)
    //{
    //    println!(
    //        "Check unsuccessful, program did not compile.\nError: {:?}",
    //        Report::new(e)
    //    );
    //    exit(1);
    //}
    println!("Check successful");
}
