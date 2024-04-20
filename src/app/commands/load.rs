use miette::{Context, Result};

use crate::{
    app::{commands::load_instruction_history, App},
    cli::{Cli, Commands, LoadArgs},
    utils::{build_instructions_with_whitelist, pretty_format_instructions, write_file},
};

#[allow(clippy::match_wildcard_for_single_variants)]
pub fn load(
    cli: &Cli,
    instructions: Vec<String>,
    input: String,
    load_args: LoadArgs,
) -> Result<()> {
    // check if command history is set
    let instruction_history = load_instruction_history(&load_args.custom_instruction_history_file)?;

    println!("Building program");
    let mut rb = super::create_runtime_builder(cli)?;

    if let Some(file) = cli.allowed_instructions_file.as_ref() {
        build_instructions_with_whitelist(&mut rb, &instructions, &input, file)?;
    } else {
        rb.build_instructions(&instructions.iter().map(String::as_str).collect(), &input)?;
    }

    // format instructions pretty if cli flag is set
    let instructions = match cli.command {
        Commands::Load(ref args) => {
            if args.disable_alignment {
                instructions
            } else {
                pretty_format_instructions(&instructions)
            }
        }
        _ => pretty_format_instructions(&instructions),
    };

    println!("Building runtime");
    let rt = rb.build().wrap_err("while building runtime")?;

    if let Commands::Load(ref args) = cli.command {
        if args.write_alignment {
            // write new formatting to file if enabled
            println!("Writing alignment to source file");
            write_file(&instructions, &input)?;
        }
    }

    // tui
    // setup terminal
    println!("Ready to run, launching tui");
    let mut terminal = super::setup_terminal()?;

    // create app
    let mut app = match cli.command {
        Commands::Load(ref args) => App::from_runtime(
            rt,
            input,
            &instructions,
            &args.breakpoints,
            instruction_history,
            args.custom_instruction_history_file.clone(),
            false,
        ),
        _ => App::from_runtime(
            rt,
            input,
            &instructions,
            &None,
            instruction_history,
            None,
            false,
        ),
    };
    let res = app.run(&mut terminal);

    // restore terminal
    super::restore_terminal(&mut terminal)?;

    res?;
    Ok(())
}
