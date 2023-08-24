use std::collections::HashMap;

use ratatui::{
    style::{Color, Style},
    widgets::{ListItem, ListState},
};

use crate::runtime::RuntimeArgs;

/// Used to store the instructions and to remember what instruction should currently be highlighted.
#[derive(Debug, Clone)]
pub struct InstructionListStates {
    instruction_list_state: ListState,
    breakpoint_list_state: ListState,
    instructions: Vec<(usize, String, bool)>, // third argument specifies if a breakpoint is set for this line
    last_index: i32,
    current_index: i32,
}

#[allow(clippy::cast_sign_loss)]
impl InstructionListStates {
    pub fn new(instructions: &[String], set_breakpoints: Option<&Vec<usize>>) -> Self {
        let mut i = Vec::new();
        for (index, s) in instructions.iter().enumerate() {
            if let Some(v) = set_breakpoints {
                if v.contains(&(index + 1)) {
                    i.push((index, s.clone(), true));
                } else {
                    i.push((index, s.clone(), false));
                }
            } else {
                i.push((index, s.clone(), false));
            }
        }
        InstructionListStates {
            instruction_list_state: ListState::default(),
            breakpoint_list_state: ListState::default(),
            instructions: i,
            last_index: -1,
            current_index: -1,
        }
    }

    /// Selects the line in which the program starts
    pub fn set_start(&mut self, current_instruction_index: i32) {
        self.set(current_instruction_index - 1);
        self.current_index = current_instruction_index;
    }

    /// Used to set the line that should be highlighted
    pub fn set(&mut self, current_instruction_idx: i32) {
        self.current_index = current_instruction_idx - 1_i32;
        if current_instruction_idx - self.last_index == 1 {
            self.instruction_list_state
                .select(Some(current_instruction_idx as usize));
            self.breakpoint_list_state
                .select(Some(current_instruction_idx as usize));
        } else {
            // line jump detected, only increase state by one
            self.instruction_list_state
                .select(Some((self.last_index + 1) as usize));
            self.breakpoint_list_state
                .select(Some((self.last_index + 1) as usize));
        }
        self.last_index = current_instruction_idx;
    }

    pub fn deselect(&mut self) {
        self.instruction_list_state.select(None);
        self.breakpoint_list_state.select(None);
    }

    /// Updates instruction list and breakpoint list to select the next value
    pub fn set_next_visual(&mut self) {
        list_next(&mut self.instruction_list_state, self.instructions.len());
        list_next(&mut self.breakpoint_list_state, self.instructions.len());
    }

    /// Updates the instructions list and breakpoint list to select the previous value
    pub fn set_prev_visual(&mut self) {
        list_prev(&mut self.instruction_list_state, self.instructions.len());
        list_prev(&mut self.breakpoint_list_state, self.instructions.len());
    }

    pub fn set_instruction_list_state(&mut self, index: Option<usize>) {
        self.instruction_list_state.select(index);
    }

    /// Toggles the breakpoint in the current line
    pub fn toggle_breakpoint(&mut self) {
        let val = self.instructions[self.instruction_list_state.selected().unwrap()].2;
        self.instructions[self.instruction_list_state.selected().unwrap()].2 = !val;
    }

    /// Checks if the current line contains a breakpoint
    pub fn is_breakpoint(&self) -> bool {
        if let Some(idx) = self.instruction_list_state.selected() {
            match self.instructions.get(idx) {
                Some(i) => return i.2,
                None => return false,
            };
        }
        false
        // self.instructions[self.instruction_list_state.selected().unwrap()].2
    }

    pub fn selected_line(&self) -> Option<usize> {
        self.instruction_list_state.selected()
    }

    pub fn instructions(&self) -> &Vec<(usize, String, bool)> {
        &self.instructions
    }

    pub fn instruction_list_state_mut(&mut self) -> &mut ListState {
        &mut self.instruction_list_state
    }

    pub fn breakpoint_list_state_mut(&mut self) -> &mut ListState {
        &mut self.breakpoint_list_state
    }
}

impl PartialEq for InstructionListStates {
    fn eq(&self, other: &Self) -> bool {
        self.instructions == other.instructions
            && self.last_index == other.last_index
            && self.current_index == other.current_index
    }
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

/// Used to update and set the lists for accumulators, memory cells and stack.
pub struct MemoryListsManager {
    accumulators: HashMap<usize, (String, bool)>,
    memory_cells: HashMap<String, (String, bool)>,
    stack: Vec<ListItem<'static>>,
}

impl MemoryListsManager {

    /// Creates a new `MemoryListsManager` with the current values of the runtime arguments.
    pub fn new(runtime_args: &RuntimeArgs) -> Self {
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
    pub fn update(&mut self, runtime_args: &RuntimeArgs) {
        // Update accumulators
        for acc in &runtime_args.accumulators {
            let a = self.accumulators.get_mut(acc.0).unwrap();
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
    pub fn accumulator_list(&self) -> Vec<ListItem<'static>> {
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
    pub fn memory_cell_list(&self) -> Vec<ListItem<'static>> {
        let mut list = Vec::new();
        for cell in &self.memory_cells {
            let mut item = ListItem::new(cell.1 .0.clone());
            if cell.1 .1 {
                item = item.style(Style::default().bg(Color::DarkGray));
            }
            list.push((item, cell.0));
        }
        list.sort_by(|a, b| a.1.cmp(b.1));
        list.iter().map(|f| f.0.clone()).collect()
    }

    /// Returns the stack items as list
    pub fn stack_list(&self) -> Vec<ListItem<'static>> {
        let mut list = self.stack.clone();
        list.reverse();
        list
    }
}
