use std::rc::Rc;

use miette::Result;

use crate::{
    app::{
        commands::load_instruction_history,
        ui::{style::Theme, syntax_highlighting::SyntaxHighlighter},
        App,
    },
    cli::{GlobalArgs, LoadArgs},
    instructions::instruction_config::InstructionConfig,
    runtime::builder,
    utils::write_file,
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

    let theme = Rc::new(Theme::default_old());

    // format instructions pretty if cli flag is set
    let instructions = SyntaxHighlighter::new(&theme.syntax_highlighting_theme()).input_to_lines(
        &instructions,
        !load_args.disable_alignment,
        !load_args.load_playground_args.disable_syntax_highlighting,
    )?;

    if load_args.write_alignment {
        // write new formatting to file if enabled
        println!("Writing alignment to source file");
        write_file(
            &instructions.iter().map(|f| f.to_string()).collect(),
            &input,
        )?;
    }

    // check if allowed instructions are restricted
    let allowed_instructions = match &load_args
        .instruction_limiting_args
        .allowed_instructions_file
    {
        Some(path) => match InstructionConfig::try_from_file(path) {
            Ok(config) => Some(config),
            Err(e) => return Err(e),
        },
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
        //&remove_special_commented_lines(instructions),
        &instructions,
        &load_args.breakpoints,
        instruction_history,
        allowed_instructions,
        load_args.custom_instruction_history_file.clone(),
        false,
        !load_args.load_playground_args.disable_syntax_highlighting,
        Theme::default_old(),
    );
    let res = app.run(&mut terminal);

    // restore terminal
    super::restore_terminal(&mut terminal)?;

    res?;
    Ok(())
}
