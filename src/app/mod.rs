use std::borrow::BorrowMut;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use miette::{miette, IntoDiagnostic, Result};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    text::Line,
    widgets::ListState,
    Terminal,
};

use crate::{
    instructions::{
        error_handling::{BuildProgramError, ParseSingleInstructionError},
        instruction_config::InstructionConfig,
        Instruction,
    },
    runtime::{self, error_handling::RuntimeError, Runtime},
    utils,
};

use self::{
    content::{InstructionListStates, MemoryListsManager},
    keybindings::KeybindingHints,
    run_instruction::SingleInstruction,
    ui::{
        style::SharedTheme,
        syntax_highlighting::{SyntaxHighlighter, ToSpans},
    },
};

/// Contains all commands that this app can run
pub mod commands;
/// Content used to fill the tui elements.
mod content;
/// Everything related to keybindings.
mod keybindings;
/// Everything related to running a single instruction while a program is loaded.
mod run_instruction;
/// Drawing of the ui.
pub mod ui;

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    Default,
    /// Indicates that the app is currently running.
    ///
    /// Boolean value is true, if at least one breakpoint is set.
    Running(bool),
    // 0 = state that the program was in when single instruction was called
    CustomInstruction(Box<State>, SingleInstruction),
    /// Indicates that parsing of the instruction failed.
    ///
    /// String contains the reason why it failed.
    ///
    /// Boolean value indicates if this error originates in the playground mode.
    CustomInstructionError(ParseSingleInstructionError, bool),
    /// Indicates that the custom instruction could not be build, because instructions, operations or comparisons
    /// where used that are not allowed.
    BuildProgramError(BuildProgramError),
    // 0 = state to restore to when debug mode is exited
    // 1 = index of instruction that was selected before debug mode was started
    DebugSelect(Box<State>, Option<usize>),
    // 0 = stores if the popup window is open
    Finished(bool),
    /// Indicates that an irrecoverable error occurred while a program was running.
    ///
    /// Boolean value indicates if this error originates in the playground mode.
    RuntimeError(RuntimeError, bool),
    /// Indicates that this app is in playground mode.
    /// Single instruction contains the state of the single instruction window.
    Playground(SingleInstruction),
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
    /// Contains instructions that where already executed using the custom instructions feature.
    executed_custom_instructions: Vec<String>,
    command_history_file: Option<String>,
    /// Determines if the call stack should be displayed in the tui
    show_call_stack: bool,
    /// Stores ids of instructions that are allowed and allowed comparisons/operations.
    ///
    /// Used to prevent forbidden instructions from getting executed in run custom instruction popup.
    instruction_config: Option<InstructionConfig>,
    /// Determines if syntax highlighting should be used.
    enable_syntax_highlighting: bool,
    /// Theme of the application.
    theme: SharedTheme,
}

#[allow(clippy::too_many_arguments)]
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
        instructions: &[Line<'static>], // The content of this array is purely cosmetical, it is just used to print the instructions inside the ui
        set_breakpoints: &Option<Vec<usize>>,
        custom_instructions: Option<Vec<String>>,
        instruction_config: Option<InstructionConfig>,
        command_history_file: Option<String>,
        playground: bool,
        enable_syntax_highlighting: bool,
        theme: SharedTheme,
    ) -> App {
        let mlm = MemoryListsManager::new(runtime.runtime_memory(), &theme);
        let show_call_stack = runtime.contains_call_instruction();
        let executed_custom_instructions = custom_instructions.unwrap_or_default();
        let state = if playground {
            State::Playground(SingleInstruction::new(
                &executed_custom_instructions,
                &theme,
            ))
        } else {
            State::Default
        };
        Self {
            runtime,
            filename,
            instruction_list_states: InstructionListStates::new(
                instructions,
                set_breakpoints.as_ref(),
            ),
            keybinding_hints: KeybindingHints::new(theme.clone())
                .expect("Keybinding hints should be properly initialized"),
            memory_lists_manager: mlm,
            state,
            executed_custom_instructions,
            command_history_file,
            show_call_stack,
            instruction_config,
            enable_syntax_highlighting,
            theme,
        }
    }

    #[allow(clippy::single_match)]
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        // update keybinding hints one to make sure that start keybinding hints are displayed properly
        if let Err(e) = self.keybinding_hints.update(&self.state) {
            return Err(miette!("Error while updating keybinding hints:\n{e}"));
        }
        loop {
            terminal.draw(|f| self.draw(f)).into_diagnostic()?;
            if let Event::Key(key) = event::read().into_diagnostic()? {
                if key.kind == KeyEventKind::Release {
                    // ignore when key is released, to prevent dual input
                    continue;
                }
                match &self.state {
                    State::CustomInstruction(_, _) | State::Playground(_) => {
                        if let KeyCode::Char(to_insert) = key.code {
                            self.any_char(to_insert)
                        }
                    }
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
                            KeyCode::Char('b') => {
                                if let State::DebugSelect(_, _) = &self.state {
                                    self.instruction_list_states.toggle_breakpoint();
                                }
                            }
                            KeyCode::Char('j') => {
                                if let State::DebugSelect(_, _) = &self.state {
                                    self.state = State::Running(
                                        self.instruction_list_states.breakpoints_set(),
                                    );
                                    let idx = self
                                        .instruction_list_states
                                        .instruction_list_state_mut()
                                        .selected()
                                        .unwrap();
                                    self.runtime.set_next_instruction(idx);
                                    _ = self.step();
                                }
                            }
                            KeyCode::Char('i') => match self.state {
                                State::Running(_) | State::Default => {
                                    self.state = State::CustomInstruction(
                                        Box::new(self.state.clone()),
                                        SingleInstruction::new(
                                            &self.executed_custom_instructions,
                                            &self.theme,
                                        ),
                                    )
                                }
                                _ => (),
                            },
                            KeyCode::Char('q') => match &self.state {
                                State::RuntimeError(e, _) => Err(e.clone())?,
                                State::CustomInstructionError(e, _) => Err(e.clone())?,
                                State::BuildProgramError(e) => Err(e.clone())?,
                                State::CustomInstruction(_, _) => (),
                                _ => return Ok(()),
                            },
                            KeyCode::Char('w') => {
                                if let State::DebugSelect(_, _) = self.state {
                                    self.instruction_list_states.set_prev_visual();
                                }
                            }
                            KeyCode::Char('t') => match self.state {
                                State::Running(_) | State::Finished(_) => self.reset(),
                                State::RuntimeError(_, false)
                                | State::CustomInstructionError(_, false) => {
                                    self.reset();
                                }
                                State::DebugSelect(_, _) => {
                                    self.instruction_list_states.set_next_visual();
                                }
                                _ => (),
                            },
                            KeyCode::Char('s') => match self.state {
                                State::Default => {
                                    self.instruction_list_states
                                        .set_start(self.runtime.next_instruction_index() as i32);
                                    self.state = State::Running(
                                        self.instruction_list_states.breakpoints_set(),
                                    );
                                    _ = self.step();
                                }
                                State::DebugSelect(_, _) => {
                                    self.instruction_list_states.set_next_visual();
                                }
                                _ => (),
                            },
                            KeyCode::Char('n') => {
                                match self.state {
                                    State::Running(_) => {
                                        _ = self.step();
                                    }
                                    _ => (),
                                };
                            }
                            KeyCode::Char('r') => {
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
                                State::DebugSelect(previous_state, i) => {
                                    self.instruction_list_states.set_instruction_list_state(*i);
                                    let state = match *previous_state.clone() {
                                        State::Default => State::Default,
                                        _ => State::Running(
                                            self.instruction_list_states.breakpoints_set(),
                                        ),
                                    };
                                    self.state = state;
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
                            KeyCode::Char('c') => match &self.state {
                                State::Default | State::Running(_) | State::DebugSelect(_, _) => {
                                    self.show_call_stack = !self.show_call_stack;
                                }
                                _ => (),
                            },
                            _ => (),
                        }
                    }
                }
                // keybinding actions that are always checked
                match key.code {
                    KeyCode::Esc => {
                        if self.escape_key()? {
                            return Ok(());
                        }
                    }
                    KeyCode::Backspace => self.backspace_key(),
                    KeyCode::Delete => self.delete_key(),
                    KeyCode::Left => self.left_key(),
                    KeyCode::Right => self.right_key(),
                    KeyCode::Down => self.down_key(),
                    KeyCode::Up => self.up_key(),
                    KeyCode::Enter => self.enter_key()?,
                    KeyCode::Tab => self.tab_key(),
                    _ => (),
                }
            }

            self.memory_lists_manager.update(&self.runtime);
            // update keybinding hints for next loop
            if let Err(e) = self.keybinding_hints.update(&self.state) {
                return Err(miette!("Error while updating keybinding hints:\n{e}"));
            }
        }
    }

    /// returns true when the execution finished in this step
    fn step(&mut self) -> Result<bool, ()> {
        // update instruction list states before running instruction to set the highlighted line correctly
        // in case jump to line or a call instruction was executed
        self.instruction_list_states
            .set(self.runtime.next_instruction_index() as i32);

        let res = self.runtime.step();
        if let Err(e) = res {
            self.state = State::RuntimeError(e, false);
            return Err(());
        }
        self.instruction_list_states
            .set(self.runtime.next_instruction_index() as i32);
        if self.runtime.finished() {
            match self.state {
                State::RuntimeError(_, _) => (),
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
        // recreate memory lists manager to remove set index memory cells from tui
        self.memory_lists_manager =
            MemoryListsManager::new(self.runtime.runtime_memory(), &self.theme);
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: exit custom instruction popup and resume running state
    /// Playground: exit the program
    ///
    /// Return value indicates if the program should be closed.
    fn escape_key(&mut self) -> Result<bool> {
        match &self.state {
            State::CustomInstruction(previous_state, _) => {
                let state = match *previous_state.clone() {
                    State::Default => State::Default,
                    State::Running(_) => {
                        State::Running(self.instruction_list_states.breakpoints_set())
                    }
                    _ => State::Running(false),
                };
                self.state = state
            }
            State::RuntimeError(e, _) => return Err(e.clone())?,
            State::CustomInstructionError(e, _) => return Err(e.clone())?,
            State::BuildProgramError(e) => return Err(e.clone())?,
            _ => return Ok(true),
        }
        Ok(false)
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Enter a char
    fn any_char(&mut self, to_insert: char) {
        match self.state.borrow_mut() {
            State::CustomInstruction(_, single_instruction)
            | State::Playground(single_instruction) => {
                insert_char_at_index(
                    &mut single_instruction.input,
                    single_instruction.cursor_position,
                    to_insert,
                );
                // check if selected item is still available in list
                if let Some(idx) = single_instruction.allowed_values_state.selected() {
                    let available_items = single_instruction.items_to_display();
                    if available_items.len() <= idx {
                        // index needs to be updated
                        if available_items.is_empty() {
                            single_instruction.allowed_values_state.select(None);
                        } else {
                            single_instruction
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
            State::CustomInstruction(_, single_instruction)
            | State::Playground(single_instruction) => {
                let is_not_cursor_leftmost = single_instruction.cursor_position != 0;
                if is_not_cursor_leftmost {
                    // Method "remove" is not used on the saved text for deleting the selected char.
                    // Reason: Using remove on String works on bytes instead of the chars.
                    // Using remove would require special care because of char boundaries.

                    let current_index = single_instruction.cursor_position;
                    let from_left_to_current_index = current_index - 1;

                    // Getting all characters before the selected character.
                    let before_char_to_delete = single_instruction
                        .input
                        .chars()
                        .take(from_left_to_current_index);
                    // Getting all characters after selected character.
                    let after_char_to_delete = single_instruction.input.chars().skip(current_index);

                    // Put all characters together except for the selected one.
                    // By leaving the selected one out, it is forgotten and therefore deleted.
                    single_instruction.input =
                        before_char_to_delete.chain(after_char_to_delete).collect();
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
            State::CustomInstruction(_, single_instruction)
            | State::Playground(single_instruction) => {
                // Method "remove" is not used on the saved text for deleting the selected char.
                // Reason: Using remove on String works on bytes instead of the chars.
                // Using remove would require special care because of char boundaries.

                let current_index = single_instruction.cursor_position;
                let from_left_to_current_index = current_index;

                // Getting all characters before the selected character.
                let before_char_to_delete = single_instruction
                    .input
                    .chars()
                    .take(from_left_to_current_index);
                // Getting all characters after selected character.
                let after_char_to_delete = single_instruction.input.chars().skip(current_index + 1);

                // Put all characters together except for the selected one.
                // By leaving the selected one out, it is forgotten and therefore deleted.
                single_instruction.input =
                    before_char_to_delete.chain(after_char_to_delete).collect();
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Move the cursor to the left.
    fn left_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(_, single_instruction)
            | State::Playground(single_instruction) => {
                let cursor_moved_left = single_instruction.cursor_position.saturating_sub(1);
                single_instruction.cursor_position =
                    cursor_moved_left.clamp(0, single_instruction.input.len());
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Move the cursor to the right.
    fn right_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(_, single_instruction)
            | State::Playground(single_instruction) => {
                let cursor_moved_right = single_instruction.cursor_position.saturating_add(1);
                single_instruction.cursor_position =
                    cursor_moved_right.clamp(0, single_instruction.input.len());
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: If not item is selected: Select first item, otherwise move down one item
    fn down_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(_, single_instruction)
            | State::Playground(single_instruction) => {
                let len = single_instruction.items_to_display().len();
                list_down(&mut single_instruction.allowed_values_state, &len);
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Moves the list up one item.
    fn up_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(_, single_instruction)
            | State::Playground(single_instruction) => {
                list_up(&mut single_instruction.allowed_values_state, true);
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction | Playground: If an element is selected in the list, it is filled in to the text area
    fn tab_key(&mut self) {
        match self.state.borrow_mut() {
            State::CustomInstruction(_, single_instruction)
            | State::Playground(single_instruction) => {
                if let Some(idx) = single_instruction.allowed_values_state.selected() {
                    let selected = &single_instruction.items_to_display()[idx];
                    single_instruction.input = selected.clone();
                    single_instruction.cursor_position = selected.chars().count();
                    single_instruction.allowed_values_state.select(None);
                }
            }
            _ => (),
        }
    }

    /// Performs an action. Action depends on current app state.
    ///
    /// CustomInstruction: Try to parse the text currently stored in the input field as instruction and run it
    /// CustomInstructionError: App state is set to running
    fn enter_key(&mut self) -> Result<()> {
        match &self.state.clone() {
            State::CustomInstruction(previous_state, single_instruction) => {
                self.custom_instruction_enter(previous_state, single_instruction)?
            }
            State::Playground(single_instruction) => self.custom_instruction_enter(
                &Box::new(State::Playground(single_instruction.clone())),
                single_instruction,
            )?,
            State::CustomInstructionError(_, is_playground) => {
                if *is_playground {
                    self.state = State::Playground(SingleInstruction::new(
                        &self.executed_custom_instructions,
                        &self.theme,
                    ))
                } else {
                    self.state = State::Running(self.instruction_list_states.breakpoints_set());
                }
            }
            State::BuildProgramError(_) => {
                self.state = State::Running(self.instruction_list_states.breakpoints_set());
            }
            State::RuntimeError(_, true) => {
                self.state = State::Playground(SingleInstruction::new(
                    &self.executed_custom_instructions,
                    &self.theme,
                ));
            }
            _ => (),
        }
        Ok(())
    }

    fn custom_instruction_enter(
        &mut self,
        previous_state: &Box<State>,
        single_instruction: &SingleInstruction,
    ) -> Result<()> {
        let instruction_str = match single_instruction.allowed_values_state.selected() {
            Some(idx) => single_instruction.items_to_display()[idx].clone(),
            None => single_instruction.input.clone(),
        };
        // check if something is entered
        if instruction_str.is_empty() {
            return Ok(());
        }
        let is_playground = match *previous_state.clone() {
            State::Playground(_) => true,
            _ => false,
        };
        let instruction = match Instruction::try_from(instruction_str.as_str()) {
            Ok(instruction) => instruction,
            Err(e) => {
                self.state = State::CustomInstructionError(
                    e.into_parse_single_instruction_error(
                        instruction_str.to_string(),
                        "input_field",
                        1,
                    ),
                    is_playground,
                );
                return Ok(());
            }
        };
        // check if instruction is allowed
        if let Some(ic) = &self.instruction_config {
            if let Err(e) = runtime::builder::check_instructions(&[instruction.clone()], ic) {
                // instruction could not be build, because instruction is forbidden
                self.state = State::BuildProgramError(*e);
                return Ok(());
            }
        }

        let instruction_line = Line::from(instruction.to_spans(&SyntaxHighlighter::new(
            &self.theme.syntax_highlighting_theme(),
        )));
        if let Err(e) = self.runtime.run_foreign_instruction(instruction) {
            self.state = State::RuntimeError(e, is_playground);
            return Ok(());
        }
        // instruction was executed successfully
        let instruction_run = single_instruction.input.clone();
        // add instruction to executed instructions, if it is not contained already and if it is not empty
        if !self.executed_custom_instructions.contains(&instruction_run)
            && !instruction_run.is_empty()
        {
            // write instruction to file, if it is set
            if let Some(path) = &self.command_history_file {
                utils::write_line_to_file(&instruction_run, path)?;
            }
            self.executed_custom_instructions.push(instruction_run);
        }
        // set new state
        if is_playground {
            // if in playground mode, add instruction to main window
            if self.enable_syntax_highlighting {
                self.instruction_list_states
                    .add_instruction(instruction_line);
            } else {
                self.instruction_list_states
                    .add_instruction(Line::from(instruction_str));
            }
            self.state = State::Playground(SingleInstruction::new(
                &self.executed_custom_instructions,
                &self.theme,
            ));
        } else {
            self.state = match *previous_state.clone() {
                State::Default => State::Default,
                _ => State::Running(self.instruction_list_states.breakpoints_set()),
            };
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

/// Helper function to create a centered rect, where the height and width of the rect are fixed.
pub fn centered_rect_abs(height: u16, width: u16, r: Rect) -> Rect {
    let margin_top_bottom = r.height.saturating_sub(height) / 2;
    let popup_layout = Layout::vertical([
        Constraint::Length(margin_top_bottom),
        Constraint::Length(height),
        Constraint::Length(margin_top_bottom),
    ])
    .split(r);

    let margin_left_right = r.width.saturating_sub(width) / 2;
    Layout::horizontal([
        Constraint::Length(margin_left_right),
        Constraint::Length(width),
        Constraint::Length(margin_left_right),
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
