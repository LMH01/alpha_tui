use std::{time::Duration, collections::{HashMap, HashSet}, thread, io};

use crossterm::event::{Event, KeyCode, self};
use tui::{backend::Backend, Frame, layout::{Layout, Direction, Constraint, Alignment}, widgets::{Tabs, Block, Borders, BorderType, ListItem, List, ListState}, style::{Style, Color, Modifier}, text::{Spans, Span}, Terminal};

use crate::runtime::{Runtime, RuntimeArgs};

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
            accumulators.insert(acc.id, (format!("{:2}: {:?}", acc.id, acc.data), false));
        }
        //accumulators.sort_by(|a, b| a.0.cmp(&b.0));
        let mut memory_cells = HashMap::new();
        for cell in &runtime_args.memory_cells {
            memory_cells.insert(cell.1.label.clone(), (format!("{:2}: {:?}", cell.1.label, cell.1.data), false));
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
            let update = format!("{:2}: {:?}", acc.id, acc.data);
            if update == *a.0 {
                a.1 = false;
            } else {
                *a = (update, true);
            }
        }
        // Update memory_cells
        for acc in &runtime_args.memory_cells {
            let a = self.memory_cells.get_mut(&acc.1.label).unwrap();
            let update = format!("{:2}: {:?}", acc.1.label, acc.1.data);
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
pub struct App<'a> {
    runtime: Runtime<'a>,
    /// Filename of the file that contains the code
    filename: String,
    /// The code that is compiled and run
    instructions: StatefulInstructions,
    /// List of keybind hints displayed at the bottom of the terminal
    keybind_hints: HashMap<char, KeybindHint>,
    /// Manages accumulators, memory_cells and stack in the ui.
    memory_lists_manager: MemoryListsManager,
}

impl<'a> App<'a> {
    pub fn from_runtime(runtime: Runtime<'a>, filename: String, instructions: Vec<String>) -> App<'a> {
        let mlm = MemoryListsManager::new(runtime.runtime_args());
        Self {
            runtime,
            filename,
            instructions: StatefulInstructions::new(instructions),
            keybind_hints: init_keybinds(),
            memory_lists_manager: mlm,
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
                    KeyCode::Char('r') => {
                        let res = self.runtime.step();
                        self.instructions.next();
                    }
                    _ => (),
                }
            }
            self.memory_lists_manager.update(self.runtime.runtime_args());
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
    map.insert('n', KeybindHint::new('n', "Next instruction", false));
    map
}

/// Example how how I can highlight list items.
/// 
/// (should i keep the way I use the main List or should I change it to use such system?)
fn test_list_items() -> Vec<ListItem<'static>> {
    let mut items = Vec::new();
    for i in 1..=10 {
        let mut item = ListItem::new(format!("Item {}", i));
        if i % 2 == 0 {
            item = item.style(Style::default().bg(Color::DarkGray));
        }
        items.push(item);
    }
    items
}

/// Draw the ui
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Wrapping block for a group
    // Just draw the block and the group on the same area and build the group
    // with at least a margin of 1
    //let size = f.size();

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
}