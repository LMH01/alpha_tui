use std::{collections::HashMap, thread, time::Duration, ops::Deref};

use crossterm::event::{self, Event, KeyCode};
use miette::{IntoDiagnostic, Result};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};

use crate::runtime::{error_handling::RuntimeError, Runtime, RuntimeArgs};

/// Used to store the instructions and to remember what instruction should currently be highlighted.
#[derive(Debug, Clone)]
struct StatefulInstructions {
    instruction_list_state: ListState,
    breakpoint_list_state: ListState,
    instructions: Vec<(usize, String, bool)>, // third argument specifies if a breakpoint is set for this line
    last_index: i32,
    current_index: i32,
}

impl StatefulInstructions {
    fn new(instructions: Vec<String>, set_breakpoints: Option<&Vec<usize>>) -> Self {
        let mut i = Vec::new();
        for (index, s) in instructions.iter().enumerate() {
            if let Some(v) = set_breakpoints {
                if v.contains(&(index+1)) {
                    i.push((index, s.clone(), true));
                } else {
                    i.push((index, s.clone(), false));
                }
            } else {
                i.push((index, s.clone(), false));
            }
        }
        StatefulInstructions {
            instruction_list_state: ListState::default(),
            breakpoint_list_state: ListState::default(),
            instructions: i,
            last_index: -1,
            current_index: -1,
        }
    }

    /// Used to set the line that should be highlighted
    fn set_last(&mut self, index: i32) {
        if index as i32 - self.last_index as i32 != 1 {
            // line jump detected, only increase state by one
            self.instruction_list_state.select(Some((self.last_index +1 ) as usize));
        } else {
            self.instruction_list_state.select(Some(index as usize));
        }
        self.last_index = index as i32;
    }
}

impl PartialEq for StatefulInstructions {
    fn eq(&self, other: &Self) -> bool {
        self.instructions == other.instructions && self.last_index == other.last_index && self.current_index == other.current_index
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
            accumulators.insert(*acc.0, (format!("{}", acc.1), false));
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
            let a = self.accumulators.get_mut(&acc.0).unwrap();
            let update = format!("{}", acc.1);
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
        let mut new_stack: Vec<ListItem<'_>> = runtime_args
            .stack
            .iter()
            .map(|f| ListItem::new(f.to_string()))
            .collect();
        if stack_changed && !new_stack.is_empty() {
            let last_stack = new_stack
                .pop()
                .unwrap()
                .style(Style::default().bg(Color::DarkGray));
            new_stack.push(last_stack);
        }
        self.stack = new_stack;
    }

    /// Returns the current accumulators as list
    fn accumulator_list(&self) -> Vec<ListItem<'static>> {
        let mut list = Vec::new();
        for acc in &self.accumulators {
            let mut item = ListItem::new(acc.1 .0.clone());
            if acc.1 .1 {
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
            let mut item = ListItem::new(cell.1 .0.clone());
            if cell.1 .1 {
                item = item.style(Style::default().bg(Color::DarkGray));
            }
            list.push((item, cell.0))
        }
        list.sort_by(|a, b| a.1.cmp(b.1));
        list.iter().map(|f| f.0.clone()).collect()
    }

    /// Returns the stack items as list
    fn stack_list(&self) -> Vec<ListItem<'static>> {
        let mut list = self.stack.clone();
        list.reverse();
        list
    }
}

/// Used organize hints to keybinds
struct KeybindHint {
    rank: usize,
    key: char,
    action: String,
    enabled: bool,
}

impl KeybindHint {
    fn new(rank: usize, key: char, action: &str, enabled: bool) -> Self {
        Self {
            rank,
            key,
            action: action.to_string(),
            enabled,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum State {
    Default,
    Running,
    // 0 = state to restore to when breakpoint mode is exited
    // 1 = index of instruction that was selected before breakpoint mode was started
    Breakpoints(Box<State>, Option<usize>),
    // 0 = stores if the popup window is open
    Finished(bool),
    Errored(RuntimeError),
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
    // Don't set state directly, use set_state() to also update keybind hints
    state: State,
}

impl App {
    pub fn from_runtime(runtime: Runtime, filename: String, instructions: Vec<String>, set_breakpoints: Option<Vec<usize>>) -> App {
        let mlm = MemoryListsManager::new(runtime.runtime_args());
        Self {
            runtime,
            filename,
            instructions: StatefulInstructions::new(instructions, set_breakpoints.as_ref()),
            keybind_hints: init_keybind_hints(),
            memory_lists_manager: mlm,
            state: State::Default,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| ui(f, self)).into_diagnostic()?;
            if let Event::Key(key) = event::read().into_diagnostic()? {
                match key.code {
                    KeyCode::Up => {
                        if let State::Breakpoints(s, i) = &self.state {
                            list_prev(&mut self.instructions.instruction_list_state, self.instructions.instructions.len());
                            list_prev(&mut self.instructions.breakpoint_list_state, self.instructions.instructions.len());
                            //TODO See if it is a good idea to make the breakpoint list move too
                        }
                    },
                    KeyCode::Down => {
                        if let State::Breakpoints(s, i) = &self.state {
                            list_next(&mut self.instructions.instruction_list_state, self.instructions.instructions.len());
                            list_next(&mut self.instructions.breakpoint_list_state, self.instructions.instructions.len());
                        }
                    },
                    KeyCode::Char('b') => {
                        match &self.state {
                            State::Breakpoints(s, i) => {
                                self.instructions.instruction_list_state.select(*i);
                                self.set_state(s.deref().clone());
                            }
                            State::Default => self.start_breakpoint_mode(),
                            State::Running => self.start_breakpoint_mode(),
                            _ => (),
                        }
                    }
                    KeyCode::Char('t') => {
                        // toggle keybind
                        match &self.state {
                            State::Breakpoints(s, _) => {
                                let val = self.instructions.instructions[self.instructions.instruction_list_state.selected().unwrap()].2;
                                self.instructions.instructions[self.instructions.instruction_list_state.selected().unwrap()].2 = !val;
                            },
                            _ => (),
                        }
                    }
                    KeyCode::Char('q') => match &self.state {
                        State::Errored(e) => Err(e.clone())?,
                        _ => return Ok(()),
                    },
                    KeyCode::Char('w') => {
                        match self.state {
                            State::Breakpoints(_, _) => {
                                list_prev(&mut self.instructions.instruction_list_state, self.instructions.instructions.len());
                                list_prev(&mut self.instructions.breakpoint_list_state, self.instructions.instructions.len());
                            }
                            _ => (),
                        }
                    }
                    KeyCode::Char('s') => {
                        match self.state {
                            State::Finished(_) => self.reset(),
                            State::Running => self.reset(),
                            State::Breakpoints(_, _) => {
                                list_next(&mut self.instructions.instruction_list_state, self.instructions.instructions.len());
                                list_next(&mut self.instructions.breakpoint_list_state, self.instructions.instructions.len());
                            }
                            _ => (),
                        }
                    }
                    KeyCode::Char('r') => {
                        if self.state == State::Default || self.state == State::Running {
                            if self.state != State::Running {
                                self.instructions.set_last(self.runtime.current_instruction_index() as i32 -1);
                                self.instructions.current_index = self.runtime.current_instruction_index() as i32;
                            }
                            self.set_state(State::Running);
                            self.step();
                        }
                    }
                    KeyCode::Char('n') => {
                        // run to the next breakpoint
                        if self.state == State::Running {
                            self.step();
                            while !self.instructions.instructions[self.instructions.instruction_list_state.selected().unwrap()].2 {
                                if self.step() {
                                    break;
                                }
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        // dismiss execution finished popup
                        if self.state == State::Finished(true) {
                            self.set_state(State::Finished(false));
                        }
                    }
                    _ => (),
                }
            }
            self.memory_lists_manager
                .update(self.runtime.runtime_args());
            thread::sleep(Duration::from_millis(30));
        }
    }

    fn active_keybind_hints(&self) -> Vec<Line> {
        let mut spans = Vec::new();
        let mut hints: Vec<&KeybindHint> = self.keybind_hints.values().collect();
        hints.sort_by(|a, b| a.rank.cmp(&b.rank));
        for v in hints {
            if !v.enabled {
                continue;
            }
            spans.push(Line::from(vec![Span::styled(
                format!("{} [{}]", v.action, v.key),
                Style::default(),
            )]))
        }
        spans
    }

    fn step(&mut self) -> bool {
        let res = self.runtime.step();//TODO Move the two similar parts of this and the above function into a new function
        if let Err(e) = res {
            self.set_state(State::Errored(e));
        }
        self.instructions.current_index = self.runtime.current_instruction_index() as i32;
        self.instructions.set_last(self.instructions.current_index -1);
        if self.runtime.finished() {
            match self.state {
                State::Errored(_) => (),
                _ => {
                    self.set_state(State::Finished(true));
                },
            }
            return true
        }
        false
    }

    /// Set whether the keybind hint should be shown or not.
    fn set_keybind_hint(&mut self, key: char, value: bool) {
        if let Some(h) = self.keybind_hints.get_mut(&key) {
            h.enabled = value;
        }
    }

    /// Sets the message for the keybind.
    fn set_keybind_message(&mut self, key: char, message: &str) {
        if let Some(h) = self.keybind_hints.get_mut(&key) {
            h.action = message.to_string();
        }
    }

    fn start_breakpoint_mode(&mut self) {
        let state = State::Breakpoints(Box::new(self.state.clone()), self.instructions.instruction_list_state.selected());
        match self.state {
            State::Running => (),
            _ => {
                self.instructions.breakpoint_list_state.select(Some(0));
                self.instructions.instruction_list_state.select(Some(0));
            }
        }
        self.set_state(state);
    }

    fn reset(&mut self) {
        self.runtime.reset();
        self.instructions.set_last(-1);
        self.instructions.instruction_list_state.select(None);
        self.set_state(State::Default);
    }

    // Sets a new state and updates keybind hints
    fn set_state(&mut self, state: State) {
        self.state = state;
        self.update_keybind_hints()
    }

    fn update_keybind_hints(&mut self) {
        self.reset_keybind_hints();
        match &self.state {
            State::Default => {//TODO Move all keybind set instructions to here
                self.set_keybind_hint('q', true);
                self.set_keybind_hint('b', true);
                self.set_keybind_hint('r', true);
                self.set_keybind_message('r', "Run");
            },
            State::Running => {
                self.set_keybind_hint('q', true);
                self.set_keybind_hint('b', true);
                self.set_keybind_hint('r', true);
                self.set_keybind_hint('s', true);
                self.set_keybind_hint('n', true);
                self.set_keybind_message('r', "Run next instruction");
            },
            State::Breakpoints(s, i) => {
                self.set_keybind_hint('q', true);
                self.set_keybind_hint('b', true);
                self.set_keybind_hint('↑', true);
                self.set_keybind_hint('↓', true);
                self.set_keybind_hint('t', true);
                self.set_keybind_message('t', "Toggle breakpoint");
                self.set_keybind_message('b', "Exit breakpoint mode");
            }
            State::Finished(b) => {
                self.set_keybind_hint('d', *b);
                self.set_keybind_hint('q', true);
                self.set_keybind_hint('s', true); 
            }
            State::Errored(e) => {
                self.set_keybind_hint('q', true);
            }
        }
    }

    // sets all keybind hints to disabled
    fn reset_keybind_hints(&mut self) {
        for hint in self.keybind_hints.iter_mut() {
            hint.1.enabled = false;
        }
        self.set_keybind_message('b', "Enter breakpoint mode");
    }
}

fn init_keybind_hints() -> HashMap<char, KeybindHint> {
    let mut map = HashMap::new();
    map.insert('q', KeybindHint::new(0, 'q', "Quit", true));
    map.insert('s', KeybindHint::new(1, 's', "Reset", false));
    map.insert('n', KeybindHint::new(2, 'n', "Next breakpoint", false));
    map.insert('r', KeybindHint::new(6, 'r', "Run", true));
    map.insert('d', KeybindHint::new(7, 'd', "Dismiss message", false));
    map.insert('b', KeybindHint::new(8, 'b', "Enter breakpoint mode", true));
    map.insert('t', KeybindHint::new(9, 't', "Toggle breakpoint", false));
    map.insert('↑', KeybindHint::new(10, '↑', "Up", false));
    map.insert('↓', KeybindHint::new(11, '↓', "Down", false));
    map
}

fn list_next(list_state: &mut ListState, instruction_length: usize) {
    let i = match list_state.selected() {
        Some(i) => {
            if i >= instruction_length - 1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    };
    list_state.select(Some(i));
}

fn list_prev(list_state: &mut ListState, max_index: usize) {
    let i = match list_state.selected() {
        Some(i) => {
            if i == 0 {
                max_index - 1
            } else {
                i - 1
            }
        }
        None => 0,
    };
    list_state.select(Some(i));
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
                Constraint::Percentage(5),
                Constraint::Percentage(65),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(global_chunks[0]);

    // Key hints
    let key_hints = Tabs::new(app.active_keybind_hints())
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(key_hints, global_chunks[1]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[2]);

    // Code area
    let mut code_area = Block::default()
        .borders(Borders::ALL)
        .title(app.filename.clone())
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    if let State::Errored(_) = app.state {
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
            let content = vec![Line::from(Span::raw(format!("{:2}: {}", i.0 + 1, i.1)))];
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
    f.render_stateful_widget(items, chunks[1], &mut app.instructions.instruction_list_state);

    // Breakpoint list
    let breakpoint_list_items: Vec<ListItem> = app.instructions.instructions.iter().map(|f| {
        let v = match f.2 {
            false => format!(" "),
            true => format!("*"),
        };
        ListItem::new(v).style(Style::default())
    }).collect();

    let breakpoint_list = Block::default()
        .borders(Borders::ALL)
        .title("BPs")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let breakpoints = List::new(breakpoint_list_items).block(breakpoint_list);
    f.render_widget(breakpoints, chunks[0]);

    // Accumulator block
    let accumulator = Block::default()
        .borders(Borders::ALL)
        .title("Accumulators")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let accumulator_list =
        List::new(app.memory_lists_manager.accumulator_list()).block(accumulator);
    f.render_widget(accumulator_list, right_chunks[0]);

    // Memory cell block
    let memory_cells = Block::default()
        .borders(Borders::ALL)
        .title("Memory cells")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let memory_cell_list =
        List::new(app.memory_lists_manager.memory_cell_list()).block(memory_cells);
    f.render_widget(memory_cell_list, right_chunks[1]);

    // Stack block
    let stack = Block::default()
        .borders(Borders::ALL)
        .title("Stack")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    let stack_list = List::new(app.memory_lists_manager.stack_list()).block(stack);
    f.render_widget(stack_list, chunks[3]);

    // Popup if execution has finished
    if app.state == State::Finished(true) {
        let block = Block::default()
            .title("Execution finished!")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let area = centered_rect(60, 20, f.size());
        let text = Paragraph::new("Press [q] to exit.\nPress [s] to reset to start.\nPress [d] to dismiss this message.").block(block);
        f.render_widget(Clear, area); //this clears out the background
        f.render_widget(text, area);
    }

    // Popup if runtime error
    if let State::Errored(e) = &app.state {
        let block = Block::default()
            .title("Runtime error!")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        let area = centered_rect(60, 30, f.size());
        let text = Paragraph::new(format!(
            "Execution can not continue due to the following problem:\n{}\n\nPress [q] to exit.",
            e.reason
        ))
        .block(block);
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
