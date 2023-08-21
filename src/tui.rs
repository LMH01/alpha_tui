use std::{time::Duration, collections::HashMap, thread};

use crossterm::event::{Event, KeyCode, self};
use miette::{Result, IntoDiagnostic};
use ratatui::{backend::Backend, Frame, layout::{Layout, Direction, Constraint, Alignment, Rect}, widgets::{Tabs, Block, Borders, BorderType, ListItem, List, ListState, Clear, Paragraph}, style::{Style, Color, Modifier}, text::{Line, Span}, Terminal};

use crate::runtime::{Runtime, RuntimeArgs, error_handling::RuntimeError};

/// Used to store the instructions and to remember what instruction should currently be highlighted.
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

    fn set(&mut self, index: usize) {
        self.state.select(Some(index));
    }

}

/// Used to update and set the lists for accumulators, memory cells and stack.
struct MemoryListsManager {
    accumulators: HashMap<usize, (String, bool)>,
    memory_cells: HashMap<String, (String, bool)>,
    stack: Vec<ListItem<'static>>,
}

impl MemoryListsManager {
    /// Creates a new MemoryListsManager with the current values of the runtime arguments.
    fn new(runtime_args: &RuntimeArgs) -> Self {
        let mut accumulators = HashMap::new();
        for acc in &runtime_args.accumulators {
            accumulators.insert(acc.id, (format!("{}", acc), false));
        }
        //accumulators.sort_by(|a, b| a.0.cmp(&b.0));
        let mut memory_cells = HashMap::new();
        for cell in &runtime_args.memory_cells {
            memory_cells.insert(cell.1.label.clone(), (format!("{}", cell.1), false));
        }
        Self {
            accumulators,
            memory_cells,
            stack: Vec::new(),
        }
    }

    /// Updates the lists values.
    /// The old values are compared against the new values, if a value has changed the background color
    /// of that list item is changed.
    fn update(&mut self, runtime_args: &RuntimeArgs) {
        // Update accumulators
        for acc in &runtime_args.accumulators {
            let a = self.accumulators.get_mut(&acc.id).unwrap();
            let update = format!("{}", acc);
            if update == *a.0 {
                a.1 = false;
            } else {
                *a = (update, true);
            }
        }
        // Update memory_cells
        for acc in &runtime_args.memory_cells {
            let a = self.memory_cells.get_mut(&acc.1.label).unwrap();
            let update = format!("{}", acc.1);
            if update == *a.0 {
                a.1 = false;
            } else {
                *a = (update, true);
            }
        }
        // Update stack
        let stack_changed = self.stack.len() != runtime_args.stack.len();
        let mut new_stack: Vec<ListItem<'_>> = runtime_args.stack.iter().map(|f| ListItem::new(f.to_string())).collect();
        if stack_changed && !new_stack.is_empty() {
            let last_stack = new_stack.pop().unwrap().style(Style::default().bg(Color::DarkGray));
            new_stack.push(last_stack);
        }
        self.stack = new_stack;
    }

    /// Returns the current accumulators as list
    fn accumulator_list(&self) -> Vec<ListItem<'static>> {
        let mut list = Vec::new();
        for acc in &self.accumulators {
            let mut item = ListItem::new(acc.1.0.clone());
            if acc.1.1 {
                item = item.style(Style::default().bg(Color::DarkGray));
            }
            list.push((item, acc.0));
        }
        list.sort_by(|a, b| a.1.cmp(b.1));
        list.iter().map(|f| f.0.clone()).collect()
    }

    /// Returns the current memory cells as list
    fn memory_cell_list(&self) -> Vec<ListItem<'static>> {
        let mut list = Vec::new();
        for cell in &self.memory_cells {
            let mut item = ListItem::new(cell.1.0.clone());
            if cell.1.1 {
                item = item.style(Style::default().bg(Color::DarkGray));
            }
            list.push((item, cell.0))
        }
        list.sort_by(|a, b| a.1.cmp(b.1));
        list.iter().map(|f| f.0.clone()).collect()
    }

    /// Returns the stack items as list
    fn stack_list(&self) -> Vec<ListItem<'static>> {
        self.stack.clone()
    }
}

/// Used organize hints to keybinds
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

/// App holds the state of the application
pub struct App {
    runtime: Runtime,
    /// Filename of the file that contains the code
    filename: String,
    /// The code that is compiled and run
    instructions: StatefulInstructions,
    /// List of keybind hints displayed at the bottom of the terminal
    keybind_hints: HashMap<char, KeybindHint>,
    /// Manages accumulators, memory_cells and stack in the ui.
    memory_lists_manager: MemoryListsManager,
    finished: bool,
    running: bool,
    errored: Option<RuntimeError>,
}

impl App {
    pub fn from_runtime(runtime: Runtime, filename: String, instructions: Vec<String>) -> App {
        let mlm = MemoryListsManager::new(runtime.runtime_args());
        Self {
            runtime,
            filename,
            instructions: StatefulInstructions::new(instructions),
            keybind_hints: init_keybinds(),
            memory_lists_manager: mlm,
            finished: false,
            running: false,
            errored: None,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| ui(f, self)).into_diagnostic()?;
            if let Event::Key(key) = event::read().into_diagnostic()? {
                match key.code {
                    KeyCode::Char('q') => match self.errored.as_ref() {
                        None => return Ok(()),
                        Some(e) => Err(e.clone())?,
                    },
                    KeyCode::Char('s') => {
                        if self.finished {
                            self.runtime.reset();
                            self.finished = false;
                            self.running = false;
                            self.set_keybind_hint('s', false);
                            self.set_keybind_hint('r', true);
                            self.set_keybind_message('r', "Run".to_string());
                            self.instructions.set(0);
                        }
                    }
                    KeyCode::Char('r') => {
                        if !self.finished && self.errored.is_none(){
                            self.running = true;
                            self.set_keybind_message('r', "Run next instruction".to_string());
                            let res = self.runtime.step();
                            if let Err(e) = res {
                                self.running = false;
                                self.errored = Some(e);
                                self.set_keybind_hint('r', false);
                            }
                            self.instructions.set(self.runtime.current_instruction_index());
                            if self.runtime.finished() && self.errored.is_none() {
                                self.finished = true;
                                self.set_keybind_hint('s', true);
                                self.set_keybind_hint('r', false);
                            }
                        }
                    }
                    _ => (),
                }
            }
            self.memory_lists_manager.update(self.runtime.runtime_args());
            thread::sleep(Duration::from_millis(30));
        }
    }

    fn active_keybind_hints(&self) -> Vec<Line> {
        let mut spans = Vec::new();
        for v in self.keybind_hints.values() {
            if !v.enabled {
                continue;
            }
            spans.push(Line::from(vec![
                Span::styled(format!("{} [{}]", v.action, v.key), Style::default()),
            ]))
        }
        spans
    }

    /// Set whether the keybind hint should be shown or not.
    fn set_keybind_hint(&mut self, key: char, value: bool) {
        if let Some(h) = self.keybind_hints.get_mut(&key) {
            h.enabled = value;
        }
    }

    /// Sets the message for the keybind.
    fn set_keybind_message(&mut self, key: char, message: String) {
        if let Some(h) = self.keybind_hints.get_mut(&key) {
            h.action = message;
        }
    }

}

fn init_keybinds() -> HashMap<char, KeybindHint> {
    let mut map = HashMap::new();
    map.insert('q', KeybindHint::new('q', "Quit", true));
    map.insert('r', KeybindHint::new('r', "Run", true));
    map.insert('n', KeybindHint::new('n', "Next instruction", false));
    map.insert('s', KeybindHint::new('s', "Reset", false));
    map
}

/// Draw the ui
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    let global_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(99), Constraint::Percentage(1)])
        .split(f.size());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
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
    f.render_widget(key_hints, global_chunks[1]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    // Code area
    let mut code_area = Block::default()
        .borders(Borders::ALL)
        .title(app.filename.clone())
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    if app.errored.is_some() {
        code_area = code_area.border_style(Style::default().fg(Color::Red));
    } else {
        code_area = code_area.border_style(Style::default().fg(Color::Green));
    }

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .instructions
        .instructions
        .iter()
        .map(|i| {
            let content = vec![Line::from(Span::raw(format!("{:2}: {}", i.0+1, i.1)))];
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

    // Accumulator block
    let accumulator = Block::default()
        .borders(Borders::ALL)
        .title("Accumulators")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let accumulator_list = List::new(app.memory_lists_manager.accumulator_list()).block(accumulator);
    f.render_widget(accumulator_list, right_chunks[0]);

    // Memory cell block
    let memory_cells = Block::default()
        .borders(Borders::ALL)
        .title("Memory cells")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let memory_cell_list = List::new(app.memory_lists_manager.memory_cell_list()).block(memory_cells);
    f.render_widget(memory_cell_list, right_chunks[1]);

    // Stack block
    let stack = Block::default()
        .borders(Borders::ALL)
        .title("Stack")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let stack_list = List::new(app.memory_lists_manager.stack_list()).block(stack);
    f.render_widget(stack_list, chunks[2]);

    // Popup if execution has finished
    if app.finished {
        let block = Block::default().title("Execution finished!").borders(Borders::ALL).border_style(Style::default().fg(Color::Green));
        let area = centered_rect(60, 20, f.size());
        let text = Paragraph::new("Press [q] to exit.\nPress [s] to reset to start.").block(block);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(text, area);
    }

    // Popup if runtime error
    if app.errored.is_some() {
        let block = Block::default().title("Runtime error!").borders(Borders::ALL).border_style(Style::default().fg(Color::Red));
        let area = centered_rect(60, 30, f.size());
        let text = Paragraph::new(format!("Execution can not continue due to the following problem:\n{}\n\nPress [q] to exit.", app.errored.as_ref().unwrap().reason)).block(block);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(text, area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`.
/// Copied from tui examples.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
