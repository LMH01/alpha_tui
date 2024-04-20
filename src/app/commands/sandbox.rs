use miette::Result;

use crate::{
    app::App,
    cli::{GlobalArgs, InstructionLimitingArgs, SandboxArgs},
    runtime::{Runtime, RuntimeArgs},
};

use super::load_instruction_history;

pub fn sandbox(global_args: &GlobalArgs, sandbox_args: &SandboxArgs) -> Result<()> {
    // check if command history is set
    let instruction_history =
        load_instruction_history(&sandbox_args.custom_instruction_history_file)?;

    println!("Building runtime");

    let runtime_args = match RuntimeArgs::from_args_with_defaults(
        global_args,
        &InstructionLimitingArgs::default(),
        4,
        4,
        true,
    ) {
        Ok(runtime_args) => runtime_args,
        Err(e) => return Err(miette::miette!("Unable to build runtime for sandbox: {e}")),
    };

    let rt = Runtime::new_sandbox(runtime_args);

    // setup terminal
    println!("Ready to run, launching tui");
    let mut terminal = super::setup_terminal()?;

    let mut app = App::from_runtime(
        rt,
        "Sandbox".to_string(),
        &Vec::new(),
        &None,
        instruction_history,
        sandbox_args.custom_instruction_history_file.clone(),
        true,
    );
    let res = app.run(&mut terminal);

    // restore terminal
    super::restore_terminal(&mut terminal)?;

    res?;
    Ok(())
}
