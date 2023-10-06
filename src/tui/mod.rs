use std::{collections::HashMap, ops::Deref, thread, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use miette::{IntoDiagnostic, Result};
use ratatui::{
    backend::Backend,
    style::{Color, Style},
    text::{Line, Span},
    Terminal,
};

use crate::runtime::{error_handling::RuntimeError, Runtime};

use self::{
    content::{InstructionListStates, MemoryListsManager},
    ui::draw,
};

/// Content used to fill the tui elements
mod content;
/// Drawing of the ui
mod ui;

// color config
const BREAKPOINT_ACCENT_COLOR: Color = Color::Magenta;
const ERROR_COLOR: Color = Color::Red;
const CODE_AREA_DEFAULT_COLOR: Color = Color::Green;
const KEY_HINTS_COLOR: Color = Color::LightBlue;
const LIST_ITEM_HIGHLIGHT_COLOR: Color = Color::Rgb(98, 114, 164);
const EXECUTION_FINISHED_POPUP_COLOR: Color = Color::Green;

#[derive(Debug, PartialEq, Clone)]
enum State {
    Default,
    Running,
    // 0 = state to restore to when debug mode is exited
    // 1 = index of instruction that was selected before debug mode was started
    DebugSelect(Box<State>, Option<usize>),
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
    instruction_list_states: InstructionListStates,
    /// List of keybind hints displayed at the bottom of the terminal
    keybind_hints: HashMap<char, KeybindHint>,
    /// Manages accumulators, memory_cells and stack in the ui.
    memory_lists_manager: MemoryListsManager,
    // Don't set state directly, use set_state() to also update keybind hints
    state: State,
    /// Stores if the jump to line feature has been used, if it is used, a different error message is displayed,
    /// when a runtime error occurs and the option to reset the runtime is given.
    jump_to_line_used: bool,
}

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
impl App {
    pub fn from_runtime(
        runtime: Runtime,
        filename: String,
        instructions: &[String], // The content of this array is purely cosmetical, it is just used to print the instructions inside the ui
        set_breakpoints: &Option<Vec<usize>>,
    ) -> App {
        let mlm = MemoryListsManager::new(runtime.runtime_args());
        Self {
            runtime,
            filename,
            instruction_list_states: InstructionListStates::new(
                instructions,
                set_breakpoints.as_ref(),
            ),
            keybind_hints: init_keybind_hints(),
            memory_lists_manager: mlm,
            state: State::Default,
            jump_to_line_used: false,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| draw(f, self)).into_diagnostic()?;
            if let Event::Key(key) = event::read().into_diagnostic()? {
                match key.code {
                    KeyCode::Up => {
                        if let State::DebugSelect(_s, _i) = &self.state {
                            self.instruction_list_states.set_prev_visual();
                        }
                    }
                    KeyCode::Down => {
                        if let State::DebugSelect(_s, _i) = &self.state {
                            self.instruction_list_states.set_next_visual();
                        }
                    }
                    KeyCode::Char('t') => {
                        if let State::DebugSelect(_, _) = &self.state {
                            self.instruction_list_states.toggle_breakpoint();
                        }
                    }
                    KeyCode::Char('j') => {
                        if let State::DebugSelect(_, _) = &self.state {
                            self.jump_to_line_used = true;
                            self.set_state(State::Running);
                            let idx = self
                                .instruction_list_states
                                .instruction_list_state_mut()
                                .selected()
                                .unwrap();
                            self.instruction_list_states.set_instruction(idx - 1);
                            self.runtime.set_next_instruction(idx);
                            self.step();
                        }
                    }
                    KeyCode::Char('q') => match &self.state {
                        State::Errored(e) => Err(e.clone())?,
                        _ => return Ok(()),
                    },
                    KeyCode::Char('w') => {
                        if let State::DebugSelect(_, _) = self.state {
                            self.instruction_list_states.set_prev_visual();
                        }
                    }
                    KeyCode::Char('s') => match self.state {
                        State::Running | State::Finished(_) => self.reset(),
                        State::Errored(_) => {
                            if self.jump_to_line_used {
                                self.reset();
                            }
                        }
                        State::DebugSelect(_, _) => {
                            self.instruction_list_states.set_next_visual();
                        }
                        State::Default => (),
                    },
                    KeyCode::Char('r') => {
                        if self.state == State::Default || self.state == State::Running {
                            if self.state != State::Running {
                                self.instruction_list_states
                                    .set_start(self.runtime.next_instruction_index() as i32);
                            }
                            self.set_state(State::Running);
                            self.step();
                        }
                    }
                    KeyCode::Char('n') => {
                        // run to the next breakpoint
                        if self.state == State::Running {
                            self.step();
                            while !self.instruction_list_states.is_breakpoint() {
                                if self.step() {
                                    break;
                                }
                            }
                        }
                    }
                    KeyCode::Char('d') => match &self.state {
                        State::DebugSelect(s, i) => {
                            self.instruction_list_states.set_instruction_list_state(*i);
                            self.set_state(s.deref().clone());
                        }
                        State::Default | State::Running => self.start_debug_select_mode(),
                        State::Finished(b) => {
                            if *b {
                                self.set_state(State::Finished(false));
                            }
                        }
                        State::Errored(_) => (),
                    },
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
            )]));
        }
        spans
    }

    /// returns true when the execution finished in this step
    fn step(&mut self) -> bool {
        let res = self.runtime.step(); //TODO Move the two similar parts of this and the above function into a new function
        if let Err(e) = res {
            self.set_state(State::Errored(e));
        }
        self.instruction_list_states
            .set(self.runtime.next_instruction_index() as i32);
        if self.runtime.finished() {
            match self.state {
                State::Errored(_) => (),
                _ => {
                    self.set_state(State::Finished(true));
                }
            }
            return true;
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

    fn start_debug_select_mode(&mut self) {
        let state = State::DebugSelect(
            Box::new(self.state.clone()),
            self.instruction_list_states.selected_line(),
        );
        match self.state {
            State::Running => (),
            _ => {
                //self.instruction_list_states.set(self.runtime.next_instruction_index() as i32);
                self.instruction_list_states
                    .force_set(self.runtime.initial_instruction_index());
            }
        }
        self.set_state(state);
    }

    fn reset(&mut self) {
        self.runtime.reset();
        self.instruction_list_states.set(-1);
        self.instruction_list_states.deselect();
        self.set_state(State::Default);
    }

    // Sets a new state and updates keybind hints
    fn set_state(&mut self, state: State) {
        self.state = state;
        self.update_keybind_hints();
    }

    fn update_keybind_hints(&mut self) {
        self.reset_keybind_hints();
        match &self.state {
            State::Default => {
                self.set_keybind_hint('q', true);
                self.set_keybind_hint('d', true);
                self.set_keybind_hint('r', true);
                self.set_keybind_message('r', "Run");
            }
            State::Running => {
                self.set_keybind_hint('q', true);
                self.set_keybind_hint('d', true);
                self.set_keybind_hint('r', true);
                self.set_keybind_hint('s', true);
                self.set_keybind_hint('n', true);
                self.set_keybind_message('r', "Run next instruction");
            }
            State::DebugSelect(_s, _i) => {
                self.set_keybind_hint('q', true);
                self.set_keybind_hint('d', true);
                self.set_keybind_hint('↑', true);
                self.set_keybind_hint('↓', true);
                self.set_keybind_hint('t', true);
                self.set_keybind_hint('j', true);
                self.set_keybind_message('t', "Toggle breakpoint");
                self.set_keybind_message('d', "Exit debug select mode");
            }
            State::Finished(b) => {
                self.set_keybind_hint('d', *b);
                self.set_keybind_message('d', "Dismiss message");
                self.set_keybind_hint('q', true);
                self.set_keybind_hint('s', true);
            }
            State::Errored(_e) => {
                self.set_keybind_hint('q', true);
            }
        }
    }

    // sets all keybind hints to disabled
    fn reset_keybind_hints(&mut self) {
        for hint in &mut self.keybind_hints {
            hint.1.enabled = false;
        }
        self.set_keybind_message('d', "Enter debug select mode");
    }
}

/// Used organize hints to keybinds
pub struct KeybindHint {
    pub rank: usize,
    pub key: char,
    pub action: String,
    pub enabled: bool,
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

pub fn init_keybind_hints() -> HashMap<char, KeybindHint> {
    let mut map = HashMap::new();
    map.insert('q', KeybindHint::new(0, 'q', "Quit", true));
    map.insert('s', KeybindHint::new(1, 's', "Reset", false));
    map.insert('n', KeybindHint::new(2, 'n', "Next breakpoint", false));
    map.insert('r', KeybindHint::new(6, 'r', "Run", true));
    map.insert(
        'd',
        KeybindHint::new(7, 'd', "Enter debug select mode", true),
    );
    map.insert('t', KeybindHint::new(9, 't', "Toggle breakpoint", false));
    map.insert('j', KeybindHint::new(9, 'j', "Jump to line", false));
    map.insert('↑', KeybindHint::new(11, '↑', "Up", false));
    map.insert('↓', KeybindHint::new(12, '↓', "Down", false));
    map
}
