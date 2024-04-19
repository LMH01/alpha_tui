use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use miette::{miette, Context, IntoDiagnostic, Result};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    cli::{Cli, Commands, LoadArgs},
    commands::load_instruction_history,
    runtime::builder::RuntimeBuilder,
    tui::App,
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
    let instruction_history = load_instruction_history(&load_args)?;

    println!("Building program");
    let mut rb = match RuntimeBuilder::from_args(cli) {
        Ok(rb) => rb,
        Err(e) => {
            return Err(miette!(
                "Unable to create RuntimeBuilder, memory config could not be loaded from file:\n{e}"
            ));
        }
    };

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
    enable_raw_mode().into_diagnostic()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).into_diagnostic()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

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
    disable_raw_mode().into_diagnostic()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .into_diagnostic()?;
    terminal.show_cursor().into_diagnostic()?;

    res?;
    Ok(())
}
