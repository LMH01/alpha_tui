use miette::Result;

use crate::{
    app::{commands::load_instruction_history, App},
    cli::{GlobalArgs, LoadArgs},
    instructions::instruction_config::InstructionConfig,
    runtime::builder,
    utils::{format_instructions, write_file},
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
    let instructions = &format_instructions(
        &instructions,
        !load_args.disable_alignment,
        !load_args.load_playground_args.disable_syntax_highlighting,
    );

    // TODO add in again
    //if load_args.write_alignment {
    //    // write new formatting to file if enabled
    //    println!("Writing alignment to source file");
    //    write_file(&instructions, &input)?;
    //}

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
    );
    let res = app.run(&mut terminal);

    // restore terminal
    super::restore_terminal(&mut terminal)?;

    res?;
    Ok(())
}

/// Removes all lines from the input vector that start with '#'
/// and returns the modified vector.
fn remove_special_commented_lines(mut instructions: Vec<String>) -> Vec<String> {
    instructions = instructions.into_iter().map(|f| f.to_string()).collect();
    instructions.retain(|f| !f.trim().starts_with('#'));
    instructions
}

#[cfg(test)]
mod tests {
    use super::remove_special_commented_lines;

    #[test]
    fn test_remove_special_commented_lines() {
        let input = vec![
            "a := 5".to_string(),
            "# a:= 5".to_string(),
            "       # a:= 5".to_string(),
            "// a := 5".to_string(),
            "       // a := 5".to_string(),
            "a := 5 # comment".to_string(),
            "a := 5 // comment".to_string(),
        ];
        let res = remove_special_commented_lines(input);
        assert_eq!(
            res,
            vec![
                "a := 5",
                "// a := 5",
                "       // a := 5",
                "a := 5 # comment",
                "a := 5 // comment"
            ]
        );
    }
}
