use std::{ops::Deref, thread, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use miette::{miette, IntoDiagnostic, Result};
use ratatui::{backend::Backend, style::Color, Terminal};

use crate::runtime::{error_handling::RuntimeError, Runtime};

use self::{
    content::{InstructionListStates, MemoryListsManager},
    keybindings::KeybindingHints,
    ui::draw,
};

/// Content used to fill the tui elements.
mod content;
/// Everything related to keybindings.
mod keybindings;
/// Drawing of the ui.
mod ui;

// color config
const BREAKPOINT_ACCENT_COLOR: Color = Color::Magenta;
const ERROR_COLOR: Color = Color::Red;
const CODE_AREA_DEFAULT_COLOR: Color = Color::Green;
const LIST_ITEM_HIGHLIGHT_COLOR: Color = Color::Rgb(98, 114, 164);
const EXECUTION_FINISHED_POPUP_COLOR: Color = Color::Green;
const KEYBINDS_FG: Color = Color::White;
const KEYBINDS_DISABLED_FG: Color = Color::DarkGray;
const KEYBINDS_BG: Color = Color::Rgb(98, 114, 164);
const KEYBINDS_DISABLED_BG: Color = Color::Black;

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    Default,
    /// Indicates that the app is currently running.
    ///
    /// Boolean value is true, if at least one breakpoint is set.
    Running(bool),
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
    /// List of keybinding hints displayed at the bottom of the terminal
    keybinding_hints: KeybindingHints,
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
    /// Creates a new app from the provided runtime.
    ///
    /// ## Panics
    ///
    /// Panics when the keybinding hints are not properly initialized
    /// (if this happens it is a hardcoded issue, that needs to be fixed in the code)
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
            keybinding_hints: KeybindingHints::new()
                .expect("Keybinding hints should be properly initialized"),
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
                            self.state =
                                State::Running(self.instruction_list_states.breakpoints_set());
                            let idx = self
                                .instruction_list_states
                                .instruction_list_state_mut()
                                .selected()
                                .unwrap();
                            self.instruction_list_states.set_instruction(idx - 1);
                            self.runtime.set_next_instruction(idx);
                            _ = self.step();
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
                        State::Running(_) | State::Finished(_) => self.reset(),
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
                        if self.state == State::Default
                            || self.state == State::Running(true)
                            || self.state == State::Running(false)
                        {
                            if self.state != State::Running(true)
                                && self.state != State::Running(false)
                            {
                                self.instruction_list_states
                                    .set_start(self.runtime.next_instruction_index() as i32);
                            }
                            self.state =
                                State::Running(self.instruction_list_states.breakpoints_set());
                            _ = self.step();
                        }
                    }
                    KeyCode::Char('n') => {
                        // run to the next breakpoint
                        if self.state == State::Running(true) || self.state == State::Running(false)
                        {
                            _ = self.step();
                            while !self.instruction_list_states.is_breakpoint() {
                                match self.step() {
                                    Ok(bool) => {
                                        if bool {
                                            break;
                                        }
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                    }
                    KeyCode::Char('d') => match &self.state {
                        State::DebugSelect(s, i) => {
                            self.instruction_list_states.set_instruction_list_state(*i);
                            self.state = s.deref().clone();
                        }
                        State::Default | State::Running(_) => self.start_debug_select_mode(),
                        State::Finished(b) => {
                            if *b {
                                self.state = State::Finished(false);
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
            // update keybinding hints for next loop
            if let Err(e) = self.keybinding_hints.update(&self.state) {
                return Err(miette!("Error while updating keybinding hints:\n{e}"));
            }
        }
    }

    /// returns true when the execution finished in this step
    fn step(&mut self) -> Result<bool, ()> {
        let res = self.runtime.step();
        if let Err(e) = res {
            self.state = State::Errored(e);
            return Err(());
        }
        self.instruction_list_states
            .set(self.runtime.next_instruction_index() as i32);
        if self.runtime.finished() {
            match self.state {
                State::Errored(_) => (),
                _ => {
                    self.state = State::Finished(true);
                }
            }
            return Ok(true);
        }
        Ok(false)
    }

    fn start_debug_select_mode(&mut self) {
        let state = State::DebugSelect(
            Box::new(self.state.clone()),
            self.instruction_list_states.selected_line(),
        );
        match self.state {
            State::Running(_) => (),
            _ => {
                //self.instruction_list_states.set(self.runtime.next_instruction_index() as i32);
                self.instruction_list_states
                    .force_set(self.runtime.initial_instruction_index());
            }
        }
        self.state = state;
    }

    fn reset(&mut self) {
        self.runtime.reset();
        self.instruction_list_states.set(-1);
        self.instruction_list_states.deselect();
        self.state = State::Default;
    }
}
