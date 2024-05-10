use miette::Result;

use crate::{
    app::App,
    cli::{GlobalArgs, PlaygroundArgs},
    runtime::builder::RuntimeBuilder,
};

use super::load_instruction_history;

pub fn playground(global_args: &GlobalArgs, playground_args: &PlaygroundArgs) -> Result<()> {
    // check if command history is set
    let instruction_history =
        load_instruction_history(&playground_args.custom_instruction_history_file)?;

    println!("Building runtime");

    let dummy_instructions = Vec::new();
    let mut rb = RuntimeBuilder::new(&dummy_instructions, "playground")?;
    rb.apply_global_cli_args(global_args)?;
    let rt = rb.build()?;

    // setup terminal
    println!("Ready to run, launching tui");
    let mut terminal = super::setup_terminal()?;

    let mut app = App::from_runtime(
        rt,
        "Playground".to_string(),
        &Vec::new(),
        &None,
        instruction_history,
        None,
        playground_args.custom_instruction_history_file.clone(),
        true,
        !playground_args
            .load_playground_args
            .disable_syntax_highlighting,
        super::load_theme(&playground_args.load_playground_args)?,
    );
    let res = app.run(&mut terminal);

    // restore terminal
    super::restore_terminal(&mut terminal)?;

    res?;
    Ok(())
}
