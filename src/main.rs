use std::{io, thread, time::Duration, process::exit, error::Error};

use clap::Parser;
use cli::Args;
use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, execute, event::{EnableMouseCapture, DisableMouseCapture, Event, self, KeyCode}};
use tui::{backend::{CrosstermBackend, Backend}, Terminal, widgets::{Block, Borders, Paragraph, BorderType}, Frame, layout::{Layout, Direction, Constraint, Alignment}, text::{Text, Spans}};
use utils::read_file;

use crate::{instructions::Instruction, runtime::{Runtime, RuntimeArgs, RuntimeBuilder}, base::{Operation, Comparison}};

/// Contains all required data types used to run programs
mod base;
/// Program execution
mod runtime;
/// Supported instructions
mod instructions;
/// Command line parsing
mod cli;
/// Utility functions
mod utils;

/// Used to set the maximum number of accumulators.
///
/// Should be at least 1.
const ACCUMULATORS: usize = 4;
/// Used to set the available memory cells.
const MEMORY_CELL_LABELS: &'static [&'static str] = &["a", "b", "c", "d", "e", "f", "w", "x", "y", "z", "h1", "h2", "h3", "h4"];

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    
    let instructions = match read_file(&args.input) {
        Ok(i) => i,
        Err(e) => {
            println!("Unable to read file: {}", e);
            exit(-1);
        },
    };
    println!("Building program");
    let mut rb = RuntimeBuilder::new_default();
    match rb.build_instructions(&instructions.iter().map(|s| s.as_str()).collect()) {
        Ok(_) => (),
        Err(e) => {
            println!("{e}");
            exit(-1);
        },
    };
    println!("Building runtime");
    let mut rt = match rb.build() {
        Ok(rt) => rt,
        Err(e) => {
            println!("Unable to build runtime: {:?}", e);
            exit(-1);
        },
    };
    println!("Ready to run, launching tui");
    //println!("----- Program start -----");
    //match rt.run() {
    //    Ok(_) => {
    //        println!("----- Program end -----");
    //        println!("Program run successfully")
    //    },
    //    Err(e) => {
    //        println!("Runtime error: {}", e);
    //        exit(-1);
    //    }
    //};

    //tui
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // create app
    let app = App::from_runtime(rt);
    let res = app.run(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

struct App<'a> {
    runtime: Runtime<'a>,
}

impl<'a> App<'a> {
    fn from_runtime(runtime: Runtime<'a>) -> App<'a> {
        Self {
            runtime,
        }
    }

    fn run<B: Backend>(&self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| ui(f, &self))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(())
                    }
                    _ => (),
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    // Wrapping block for a group
    // Just draw the block and the group on the same area and build the group
    // with at least a margin of 1
    //let size = f.size();

    let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(70),
                    Constraint::Percentage(20),
                    Constraint::Percentage(10)
                ].as_ref()
            )
            .split(f.size());

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint:: Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    // Surrounding block
    let code_area = Block::default()
        .borders(Borders::ALL)
        .title("Code Area")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    f.render_widget(code_area, chunks[0]);

    // Accumulator block
    let accumulator = Block::default()
        .borders(Borders::ALL)
        .title("Accumulators")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    f.render_widget(accumulator, right_chunks[0]);
    
    // Memory cell block
    let memory_cells = Block::default()
        .borders(Borders::ALL)
        .title("Memory cells")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    f.render_widget(memory_cells, right_chunks[1]);

    // Stack block
    let stack = Block::default()
        .borders(Borders::ALL)
        .title("Stack")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    f.render_widget(stack, chunks[2]);
}
