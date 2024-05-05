use miette::Result;

use crate::{
    app::{commands::load_instruction_history, App},
    cli::{GlobalArgs, LoadArgs},
    runtime::builder::{self, InstructionConfig},
    utils::{self, pretty_format_instructions, write_file},
};

#[allow(clippy::match_wildcard_for_single_variants)]
pub fn load(
    global_args: &GlobalArgs,
    load_args: &LoadArgs,
    instructions: Vec<String>,
    input: String,
) -> Result<()> {
    // check if command history is set
    let instruction_history = load_instruction_history(&load_args.custom_instruction_history_file)?;

    // create runtime builder and apply cli args
    println!("Building instructions");
    let mut rb = builder::RuntimeBuilder::new(&instructions, &input)?;
    rb.apply_global_cli_args(global_args)?
        .apply_instruction_limiting_args(&load_args.instruction_limiting_args)?;
    // build runtime
    println!("Building runtime");
    let rt = rb.build()?;

    // format instructions pretty if cli flag is set
    let instructions = if load_args.disable_alignment {
        instructions
    } else {
        pretty_format_instructions(&instructions)
    };

    if load_args.write_alignment {
        // write new formatting to file if enabled
        println!("Writing alignment to source file");
        write_file(&instructions, &input)?;
    }

    // check if allowed instructions are restricted
    let allowed_instructions = match &load_args
        .instruction_limiting_args
        .allowed_instructions_file
    {
        Some(path) => Some(InstructionConfig {
            allowed_instruction_identifiers: Some(utils::build_instruction_whitelist(path)?),
            allowed_comparisons: load_args
                .instruction_limiting_args
                .allowed_comparisons
                .clone(),
            allowed_operations: load_args
                .instruction_limiting_args
                .allowed_operations
                .clone(),
        }),
        None => None,
    };

    // tui
    // setup terminal
    println!("Ready to run, launching tui");
    let mut terminal = super::setup_terminal()?;

    // create app
    let mut app = App::from_runtime(
        rt,
        input,
        &instructions,
        &load_args.breakpoints,
        instruction_history,
        allowed_instructions,
        load_args.custom_instruction_history_file.clone(),
        false,
    );
    let res = app.run(&mut terminal);

    // restore terminal
    super::restore_terminal(&mut terminal)?;

    res?;
    Ok(())
}
