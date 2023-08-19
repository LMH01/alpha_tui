use std::{error::Error, io, process::exit, thread, time::Duration};

use clap::Parser;
use cli::Args;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Modifier},
    text::{Spans, Text, Span},
    widgets::{Block, BorderType, Borders, ListItem, Paragraph, List, ListState},
    Frame, Terminal,
};
use utils::read_file;

use crate::{
    base::{Comparison, Operation},
    instructions::Instruction,
    runtime::{Runtime, RuntimeArgs, RuntimeBuilder},
};

/// Contains all required data types used to run programs
mod base;
/// Command line parsing
mod cli;
/// Supported instructions
mod instructions;
/// Program execution
mod runtime;
/// Utility functions
mod utils;

/// Used to set the maximum number of accumulators.
///
/// Should be at least 1.
const ACCUMULATORS: usize = 4;
/// Used to set the available memory cells.
const MEMORY_CELL_LABELS: &'static [&'static str] = &[
    "a", "b", "c", "d", "e", "f", "w", "x", "y", "z", "h1", "h2", "h3", "h4",
];

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let instructions = match read_file(&args.input) {
        Ok(i) => i,
        Err(e) => {
            println!("Unable to read file: {}", e);
            exit(-1);
        }
    };
    println!("Building program");
    let mut rb = RuntimeBuilder::new_default();
    match rb.build_instructions(&instructions.iter().map(|s| s.as_str()).collect()) {
        Ok(_) => (),
        Err(e) => {
            println!("{e}");
            exit(-1);
        }
    };
    println!("Building runtime");
    let mut rt = match rb.build() {
        Ok(rt) => rt,
        Err(e) => {
            println!("Unable to build runtime: {:?}", e);
            exit(-1);
        }
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
    let mut app = App::from_runtime(rt, args.input, instructions);
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

struct StatefulInstructions {
    state: ListState,
    instructions: Vec<(usize, String)>,
}

impl StatefulInstructions {
    fn new(instructions: Vec<String>) -> Self {
        let mut i = Vec::new();
        for (index, s) in instructions.iter().enumerate() {
            i.push((index, s.clone()));
        }
        StatefulInstructions {
            state: ListState::default(),
            instructions: i,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.instructions.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.instructions.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App<'a> {
    runtime: Runtime<'a>,
    /// Filename of the file that contains the code
    filename: String,
    /// The code that is compiled and run
    instructions: StatefulInstructions,
}

impl<'a> App<'a> {
    fn from_runtime(runtime: Runtime<'a>, filename: String, instructions: Vec<String>) -> App<'a> {
        Self {
            runtime,
            filename,
            instructions: StatefulInstructions::new(instructions),
        }
    }

    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| ui(f, self))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => self.instructions.next(),//TODO remove manual list control
                    KeyCode::Up => self.instructions.previous(),
                    _ => (),
                }
            }
            thread::sleep(Duration::from_millis(30));
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
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
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    // Code area
    let code_area = Block::default()
        .borders(Borders::ALL)
        .title(app.filename.clone())
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green));

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .instructions
        .instructions
        .iter()
        .map(|i| {
            let content = vec![Spans::from(Span::raw(format!("{:2}: {}", i.0+1, i.1)))];
            ListItem::new(content).style(Style::default())
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(code_area)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.instructions.state);
    //f.render_widget(code_area, chunks[0]);

    //let code_area_text = List::new(app.instructions.clone()).block(code_area);

    //let code_area_text = Paragraph::new("Some Text")
    //    .block(code_area)
    //    .style(Style::default().fg(Color::White))
    //    .alignment(Alignment::Left);
    //f.render_widget(code_area_text, chunks[0]);

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
