use std::{time::Duration, collections::HashMap, thread, io};

use crossterm::event::{Event, KeyCode, self};
use tui::{backend::Backend, Frame, layout::{Layout, Direction, Constraint, Alignment}, widgets::{Tabs, Block, Borders, BorderType, ListItem, List, ListState}, style::{Style, Color, Modifier}, text::{Spans, Span}, Terminal};

use crate::runtime::Runtime;

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Wrapping block for a group
    // Just draw the block and the group on the same area and build the group
    // with at least a margin of 1
    //let size = f.size();

    let global_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(95), Constraint::Percentage(5)])
        .split(f.size());

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
        .split(global_chunks[0]);

    let key_hints = Tabs::new(app.active_keybind_hints())
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(Color::Cyan));
        //.highlight_style(
        //    Style::default()
        //        .add_modifier(Modifier::BOLD)
        //        .bg(Color::Black),
        //);
    f.render_widget(key_hints, global_chunks[1]);

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

struct KeybindHint {
    key: char,
    action: String,
    enabled: bool,
}

impl KeybindHint {
    fn new (key: char, action: &str, enabled: bool) -> Self {
        Self {
            key,
            action: action.to_string(),
            enabled,
        }
    }
}

pub struct App<'a> {
    runtime: Runtime<'a>,
    /// Filename of the file that contains the code
    filename: String,
    /// The code that is compiled and run
    instructions: StatefulInstructions,
    /// List of keybind hints displayed at the bottom of the terminal
    keybind_hints: HashMap<char, KeybindHint>,
}

impl<'a> App<'a> {
    pub fn from_runtime(runtime: Runtime<'a>, filename: String, instructions: Vec<String>) -> App<'a> {
        Self {
            runtime,
            filename,
            instructions: StatefulInstructions::new(instructions),
            keybind_hints: init_keybinds(),
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| ui(f, self))?;
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => self.instructions.next(),//TODO remove manual list control
                    KeyCode::Up => self.instructions.previous(),
                    KeyCode::Char('n')  => {
                        self.keybind_hints.get_mut(&'r').unwrap().enabled = false;
                    }
                    KeyCode::Char('m')  => {
                        self.keybind_hints.get_mut(&'r').unwrap().enabled = true;
                    }
                    _ => (),
                }
            }
            thread::sleep(Duration::from_millis(30));
        }
    }

    fn active_keybind_hints(&self) -> Vec<Spans> {
        let mut spans = Vec::new();
        for (k, v) in &self.keybind_hints {
            if !v.enabled {
                continue;
            }
            spans.push(Spans::from(vec![
                Span::styled(format!("{} [{}]", v.action, v.key), Style::default()),
            ]))
        }
        spans
    }
}

fn init_keybinds() -> HashMap<char, KeybindHint> {
    let mut map = HashMap::new();
    map.insert('q', KeybindHint::new('q', "Quit", true));
    map.insert('r', KeybindHint::new('r', "Run", true));
    map
}