use miette::{Context, Result};

use crate::{
    app::{commands::load_instruction_history, App},
    cli::{GlobalArgs, LoadArgs},
    utils::{build_instructions_with_whitelist, pretty_format_instructions, write_file},
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

    println!("Building program");
    let mut rb = super::create_runtime_builder(global_args, &load_args.instruction_limiting_args)?;

    if let Some(file) = load_args
        .instruction_limiting_args
        .allowed_instructions_file
        .as_ref()
    {
        build_instructions_with_whitelist(&mut rb, &instructions, &input, file)?;
    } else {
        rb.build_instructions(&instructions.iter().map(String::as_str).collect(), &input)?;
    }

    // format instructions pretty if cli flag is set
    let instructions = if load_args.disable_alignment {
        instructions
    } else {
        pretty_format_instructions(&instructions)
    };

    println!("Building runtime");
    let rt = rb.build().wrap_err("while building runtime")?;

    if load_args.write_alignment {
        // write new formatting to file if enabled
        println!("Writing alignment to source file");
        write_file(&instructions, &input)?;
    }

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
        load_args.custom_instruction_history_file.clone(),
        false,
    );
    let res = app.run(&mut terminal);

    // restore terminal
    super::restore_terminal(&mut terminal)?;

    res?;
    Ok(())
}
