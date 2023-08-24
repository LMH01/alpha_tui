use ratatui::widgets::ListState;


/// Used to store the instructions and to remember what instruction should currently be highlighted.
#[derive(Debug, Clone)]
pub struct InstructionListStates {
    instruction_list_state: ListState,
    breakpoint_list_state: ListState,
    instructions: Vec<(usize, String, bool)>, // third argument specifies if a breakpoint is set for this line
    last_index: i32,
    current_index: i32,
}

impl InstructionListStates {
    pub fn new(instructions: Vec<String>, set_breakpoints: Option<&Vec<usize>>) -> Self {
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
        self.set(current_instruction_index -1);
        self.current_index = current_instruction_index;
    }

    /// Used to set the line that should be highlighted
    pub fn set(&mut self, current_instruction_idx: i32) {
        self.current_index = current_instruction_idx - 1 as i32;
        if current_instruction_idx as i32 - self.last_index as i32 != 1 {
            // line jump detected, only increase state by one
            self.instruction_list_state.select(Some((self.last_index +1 ) as usize));
            self.breakpoint_list_state.select(Some((self.last_index +1 ) as usize));
        } else {
            self.instruction_list_state.select(Some(current_instruction_idx as usize));
            self.breakpoint_list_state.select(Some(current_instruction_idx as usize));
        }
        self.last_index = current_instruction_idx as i32;
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
        self.instructions[self.instruction_list_state.selected().unwrap()].2
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
        self.instructions == other.instructions && self.last_index == other.last_index && self.current_index == other.current_index
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