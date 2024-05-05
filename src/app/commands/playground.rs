use miette::Result;

use crate::{
    app::App,
    cli::{GlobalArgs, InstructionLimitingArgs, PlaygroundArgs},
    runtime::{Runtime, RuntimeMemory},
};

use super::load_instruction_history;

pub fn playground(global_args: &GlobalArgs, playground_args: &PlaygroundArgs) -> Result<()> {
    // check if command history is set
    let instruction_history =
        load_instruction_history(&playground_args.custom_instruction_history_file)?;

    println!("Building runtime");

    let runtime_args = match RuntimeMemory::from_args_with_defaults(
        global_args,
        &InstructionLimitingArgs::default(),
        4,
        4,
        true,
    ) {
        Ok(runtime_args) => runtime_args,
        Err(e) => {
            return Err(miette::miette!(
                "Unable to build runtime for playground: {e}"
            ))
        }
    };

    let rt = Runtime::new_playground(runtime_args);

    // setup terminal
    println!("Ready to run, launching tui");
    let mut terminal = super::setup_terminal()?;

    let mut app = App::from_runtime(
        rt,
        "Playground".to_string(),
        &Vec::new(),
        &None,
        instruction_history,
        playground_args.custom_instruction_history_file.clone(),
        true,
    );
    let res = app.run(&mut terminal);

    // restore terminal
    super::restore_terminal(&mut terminal)?;

    res?;
    Ok(())
}
