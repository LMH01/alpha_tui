use std::{borrow::BorrowMut, ops::Deref, thread, time::Duration};

use crossterm::event::{self, Event, KeyCode};
use miette::{miette, IntoDiagnostic, Result};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::ListState,
    Terminal,
};

use crate::{
    instructions::Instruction,
    runtime::{error_handling::RuntimeError, Runtime}, utils,
};

use self::{
    content::{InstructionListStates, MemoryListsManager},
    keybindings::KeybindingHints,
    run_instruction::SingleInstruction,
    ui::draw,
};

/// Content used to fill the tui elements.
mod content;
/// Everything related to keybindings.
mod keybindings;
/// Everything related to running a single instruction while a program is loaded.
mod run_instruction;
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
const CUSTOM_INSTRUCTION_ACCENT_FG: Color = Color::Cyan;

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    Default,
    /// Indicates that the app is currently running.
    ///
    /// Boolean value is true, if at least one breakpoint is set.
    Running(bool),
    CustomInstruction(SingleInstruction),
    /// Indicates that parsing of the instruction failed.
    ///
    /// String contains the reason why it failed.
    CustomInstructionError(String),
    // 0 = state to restore to when debug mode is exited
    // 1 = index of instruction that was selected before debug mode was started
    DebugSelect(Box<State>, Option<usize>),
    // 0 = stores if the popup window is open
    Finished(bool),
    /// Indicates that an irrecoverable error occurred while a program was running.
    RuntimeError(RuntimeError),
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
    state: State,
    /// Stores if the jump to line feature has been used, if it is used, a different error message is displayed,
    /// when a runtime error occurs and the option to reset the runtime is given.
    jump_to_line_used: bool,
    /// Contains instructions that where already executed using the custom instructions feature.
    executed_custom_instructions: Vec<String>,
    command_history_file: Option<String>,
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
        custom_instructions: Option<Vec<String>>,
        command_history_file: Option<String>,
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
            executed_custom_instructions: custom_instructions.unwrap_or(Vec::new()),
            command_history_file,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| draw(f, self)).into_diagnostic()?;
            if let Event::Key(key) = event::read().into_diagnostic()? {
                match &self.state {
                    State::CustomInstruction(_) => match key.code {
                        KeyCode::Char(to_insert) => self.any_char(to_insert),
                        _ => (),
                    },
                    _ => {
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
                                    self.state = State::Running(
                                        self.instruction_list_states.breakpoints_set(),
                                    );
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
                            KeyCode::Char('i') => match self.state {
                                State::Running(_) => {
                                    self.state = State::CustomInstruction(SingleInstruction::new(
                                        &self.executed_custom_instructions,
                                    ))
                                }
                                _ => (),
                            },
                            KeyCode::Char('q') => match &self.state {
                                State::RuntimeError(e) => Err(e.clone())?,
                                State::CustomInstruction(_) => (),
                                _ => return Ok(()),
                            },
                            KeyCode::Char('w') => {
                                if let State::DebugSelect(_, _) = self.state {
                                    self.instruction_list_states.set_prev_visual();
                                }
                            }
                            KeyCode::Char('s') => match self.state {
                                State::Running(_) | State::Finished(_) => self.reset(),
                                State::RuntimeError(_) => {
                                    if self.jump_to_line_used {
                                        self.reset();
                                    }
                                }
                                State::DebugSelect(_, _) => {
                                    self.instruction_list_states.set_next_visual();
                                }
                                _ => (),
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
                                    self.state = State::Running(
                                        self.instruction_list_states.breakpoints_set(),
                                    );
                                    _ = self.step();
                                }
                            }
                            KeyCode::Char('n') => {
                                // run to the next breakpoint
                                if self.state == State::Running(true)
                                    || self.state == State::Running(false)
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
                                State::Default | State::Running(_) => {
                                    self.start_debug_select_mode()
                                }
                                State::Finished(b) => {
                                    if *b {
                                        self.state = State::Finished(false);
                                    }
                                }
                                _ => (),
                            },
                            _ => (),
                        }
                    }
                }
                // keybinding actions that are always checked
                match key.code {
                    KeyCode::Esc => self.escape_key(),
                    KeyCode::Backspace => self.backspace_key(),
                    KeyCode::Delete => self.delete_key(),
                    KeyCode::Left => self.left_key(),
                    KeyCode::Right => self.right_key(),
                    KeyCode::Down => self.down_key(),
                    KeyCode::Up => self.up_key(),
                    KeyCode::Enter => self.enter_key()?,
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
            self.state = State::RuntimeError(e);
            return Err(());
        }
        self.instruction_list_states
            .set(self.runtime.next_instruction_index() as i32);
        if self.runtime.finished() {
            match self.state {
                State::RuntimeError(_) => (),
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

    fn escape_key(&mut self) {
        match self.state {
            State::CustomInstruction(_) => {
                self.state = State::Running(self.instruction_list_states.breakpoints_set())
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Enter a char
    fn any_char(&mut self, to_insert: char) {
        match self.state.borrow_mut() {
            State::CustomInstruction(state) => {
                insert_char_at_index(&mut state.input, state.cursor_position, to_insert);
                // check if selected item is still available in list
                if let Some(idx) = state.allowed_values_state.selected() {
                    let available_items = state.items_to_display();
                    if available_items.len() <= idx {
                        // index needs to be updated
                        if available_items.len() == 0 {
                            state.allowed_values_state.select(None);
                        } else {
                            state
                                .allowed_values_state
                                .select(Some(available_items.len() - 1));
                        }
                    }
                }

                self.right_key();
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Deletes a char
    fn backspace_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(state) => {
                let is_not_cursor_leftmost = state.cursor_position != 0;
                if is_not_cursor_leftmost {
                    // Method "remove" is not used on the saved text for deleting the selected char.
                    // Reason: Using remove on String works on bytes instead of the chars.
                    // Using remove would require special care because of char boundaries.

                    let current_index = state.cursor_position;
                    let from_left_to_current_index = current_index - 1;

                    // Getting all characters before the selected character.
                    let before_char_to_delete =
                        state.input.chars().take(from_left_to_current_index);
                    // Getting all characters after selected character.
                    let after_char_to_delete = state.input.chars().skip(current_index);

                    // Put all characters together except for the selected one.
                    // By leaving the selected one out, it is forgotten and therefore deleted.
                    state.input = before_char_to_delete.chain(after_char_to_delete).collect();
                    self.left_key()
                }
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Deletes the char behind the cursor.
    fn delete_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(state) => {
                // Method "remove" is not used on the saved text for deleting the selected char.
                // Reason: Using remove on String works on bytes instead of the chars.
                // Using remove would require special care because of char boundaries.

                let current_index = state.cursor_position;
                let from_left_to_current_index = current_index;

                // Getting all characters before the selected character.
                let before_char_to_delete = state.input.chars().take(from_left_to_current_index);
                // Getting all characters after selected character.
                let after_char_to_delete = state.input.chars().skip(current_index + 1);

                // Put all characters together except for the selected one.
                // By leaving the selected one out, it is forgotten and therefore deleted.
                state.input = before_char_to_delete.chain(after_char_to_delete).collect();
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Move the cursor to the left.
    fn left_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(state) => {
                let cursor_moved_left = state.cursor_position.saturating_sub(1);
                state.cursor_position = cursor_moved_left.clamp(0, state.input.len());
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Move the cursor to the right.
    fn right_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(state) => {
                let cursor_moved_right = state.cursor_position.saturating_add(1);
                state.cursor_position = cursor_moved_right.clamp(0, state.input.len());
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: If not item is selected: Select first item, otherwise move down one item
    fn down_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(state) => {
                let len = state.items_to_display().len();
                list_down(&mut state.allowed_values_state, &len);
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Moves the list up one item.
    fn up_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(state) => {
                list_up(&mut state.allowed_values_state, true);
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Try to parse the text currently stored in the input field as instruction and run it
    /// CustomInstructionError: App state is set to running
    fn enter_key(&mut self) -> Result<()> {
        match &self.state {
            State::CustomInstruction(state) => {
                let instruction_str = match state.allowed_values_state.selected() {
                    Some(idx) => state.items_to_display()[idx].clone(),
                    None => state.input.clone(),
                };
                // check if something is entered
                if instruction_str.is_empty() {
                    return Ok(());
                }
                let instruction = match Instruction::try_from(instruction_str.as_str()) {
                    Ok(instruction) => instruction,
                    Err(e) => {
                        self.state = State::CustomInstructionError(format!("{}", e));
                        return Ok(());
                    }
                };
                if let Err(e) = self.runtime.run_foreign_instruction(instruction) {
                    self.state = State::RuntimeError(e);
                    return Ok(());
                }
                // instruction was executed successfully
                let instruction_run = state.input.clone();
                self.state = State::Running(self.instruction_list_states.breakpoints_set());
                // add instruction to executed instructions, if it is not contained already and if it is not empty
                if !self.executed_custom_instructions.contains(&instruction_run)
                    && !instruction_run.is_empty()
                {
                    // write instruction to file, if it is set
                    if let Some(path) = &self.command_history_file {
                        utils::write_line_to_file(&instruction_run, &path)?;
                    }
                    self.executed_custom_instructions.push(instruction_run);
                }
            }
            State::CustomInstructionError(_) => {
                self.state = State::Running(self.instruction_list_states.breakpoints_set());
            }
            _ => (),
        }
        Ok(())
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`.
pub fn centered_rect(percent_x: u16, percent_y: u16, height: Option<u16>, r: Rect) -> Rect {
    let center_constraint = match height {
        Some(value) => Constraint::Length(value),
        None => Constraint::Percentage(percent_y),
    };
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        center_constraint,
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

/// Scrolls the provided list down.
pub fn list_down(state: &mut ListState, len: &usize) {
    if let Some(idx) = state.selected() {
        // check if we are at the bottom of the list
        if idx != len - 1 {
            state.select(Some(idx + 1));
        }
    } else if len > &0 {
        state.select(Some(0));
    }
}

/// Scrolls the provided list up.
pub fn list_up(state: &mut ListState, deselect: bool) {
    if let Some(idx) = state.selected() {
        // check if we are at the top of the list
        if idx > 0 {
            state.select(Some(idx - 1));
        } else if deselect {
            state.select(None)
        }
    }
}

pub fn insert_char_at_index(s: &mut String, idx: usize, to_insert: char) {
    let mut chars = s.chars().collect::<Vec<char>>();
    chars.insert(idx, to_insert);
    *s = chars.into_iter().collect()
}
