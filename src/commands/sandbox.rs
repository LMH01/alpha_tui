use miette::Result;

use crate::{
    cli::{Cli, SandboxArgs}, runtime::{Runtime, RuntimeArgs}, tui::App
};

use super::load_instruction_history;

pub fn sandbox(cli: &Cli, sandbox_args: &SandboxArgs) -> Result<()> {
    // check if command history is set
    let instruction_history =
        load_instruction_history(&sandbox_args.custom_instruction_history_file)?;

    println!("Building runtime");
    
    let runtime_args = match RuntimeArgs::from_args(cli) {
        Ok(runtime_args) => runtime_args,
        Err(e) => return Err(miette::miette!("Unable to build runtime for sandbox: {e}")),
    };

    // TODO implement way that default values are set, if corresponding parameters are not set
    // this function could be put into RuntimeArgs and be named RuntimeArgs::from_args_with_defaults
    // this way accumulators, memory cells and gamma accumulator are available if needed, but can also be overwritten,
    // if the default does not provide enough elements

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
        None,
        true,
    );
    let res = app.run(&mut terminal);

    // restore terminal
    super::restore_terminal(&mut terminal)?;

    res?;
    Ok(())
}
