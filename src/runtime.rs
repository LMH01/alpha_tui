use std::collections::HashMap;

use crate::{instructions::{Instruction, InstructionParseError}, base::{Accumulator, MemoryCell}, ACCUMULATORS, MEMORY_CELL_LABELS};

/// Type that is used to build a new runtime environment.
/// 
/// This runtime can be configured to only allow a selected amount of accumulators and memory cells.
/// When a runtime is build from this builder compatibility checks are performed.
pub struct RuntimeBuilder<'a> {
    runtime_args: Option<RuntimeArgs<'a>>,
    instructions: Option<Vec<Instruction>>,
    control_flow: ControlFlow<'a>,
}

impl<'a> RuntimeBuilder<'a> {
    /// Creates a new runtime builder with no values set.
    pub fn new() -> Self {
        Self {
            runtime_args: None,
            instructions: None,
            control_flow: ControlFlow::new(),
        }
    }

    /// Creates a new runtime builder with default values.
    pub fn new_default() -> Self {
        Self {
            runtime_args: Some(RuntimeArgs::new()),
            instructions: None,
            control_flow: ControlFlow::new(),
        }
    }

    /// Resets the current values to none.
    pub fn reset(&mut self) {
        self.runtime_args = None;
        self.instructions = None;
        self.control_flow = ControlFlow::new();
    }

    pub fn set_runtime_args(&mut self, runtime_args: RuntimeArgs<'a>) {
        self.runtime_args = Some(runtime_args);
    }

    /// Builds instructions from the string and sets them as current instructions.
    /// 
    /// Each line has to contain a single instruction.
    /// 
    /// Control flow is updated accordingly.
    /// 
    /// If an instruction could not be parsed, an error is returned containing the reason.
    pub fn build_instructions(&mut self, instructions_input: &Vec<&str>) -> Result<(), String> {
        let mut instructions = Vec::new();
        for instruction in instructions_input {
            match Instruction::try_from(*instruction) {
                Ok(i) => instructions.push(i),
                Err(e) => {
                    let mut message = String::from(*instruction);
                    message.push('\n');
                    match e {
                        InstructionParseError::UnknownOperation(idx, str) => {
                            append_char_indicator(&mut message, idx);
                            message.push_str(&format!("Error: Unknown operation: {}", str));
                        },
                        InstructionParseError::UnknownComparison(idx, str) => {
                            append_char_indicator(&mut message, idx);
                            message.push_str(&format!("Error: Unknown comparison: {}", str));
                        },
                        InstructionParseError::NotANumber(idx, str) => {
                            append_char_indicator(&mut message, idx);
                            message.push_str(&format!("Error: Not a number: {}", str));
                        },
                        InstructionParseError::InvalidExpression(idx, str) => {
                            append_char_indicator(&mut message, idx);
                            message.push_str(&format!("Error: Invalid expression: {}", str));
                        },
                        InstructionParseError::NoMatch => {
                            message.push_str("^\n");
                            message.push_str("Error: No matching instruction found!");
                        },
                        InstructionParseError::NoMatchSuggestion(str) => {
                            append_char_indicator(&mut message, 0);
                            message.push_str(&format!("Error: No matching instruction found, did you mean: {} ?", str));

                        }
                    }
                    return Err(message);
                },
            }
        }
        Ok(())
    }

}

/// Prints a pointer at index.
fn append_char_indicator(str: &mut String, idx: usize) {
    for i in 0..idx {
        str.push(' ');
    }
    str.push_str("^\n");
}

//TODO make fields private and add access functions, move into separate module
pub struct Runner<'a> {
    runtime_args: RuntimeArgs<'a>,
    instructions: Vec<Instruction>,
    control_flow: ControlFlow<'a>,
}

impl<'a> Runner<'a> {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            runtime_args: RuntimeArgs::new(),
            instructions,
            control_flow: ControlFlow::new(),
        }
    }

    /// Creates a new runner that can be initialized with different runtime args.
    pub fn new_custom(instructions: Vec<Instruction>, runtime_args: RuntimeArgs<'a>) -> Self {
        Self {
            runtime_args,
            instructions,
            control_flow: ControlFlow::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.control_flow.next_instruction_index < self.instructions.len() {
            let current_instruction = self.control_flow.next_instruction_index;
            self.control_flow.next_instruction_index += 1;
            if let Err(e) = self.instructions[current_instruction].run(&mut self.runtime_args, &mut self.control_flow) {
                println!("Unable to continue execution, an irrecoverable error occured: {}", e);
                return Err(format!("Execution terminated: {}", e));
            }
        }
        Ok(())
    }

    /// Adds an instruction to the end of the instruction vector with a label mapping.
    pub fn add_instruction_with_label(&mut self, instruction: Instruction, label: &'a str) {
        self.instructions.push(instruction);
        self.control_flow.instruction_labels.insert(label, self.instructions.len()-1);
    }

    /// Adds label to instruction labels.
    /// 
    /// Errors when **instruction_index** is out of bounds.
    /// 
    /// Note: Make sure that you start counting at 0 and not 1!
    pub fn add_label(&mut self, label: &'a str, instruction_index: usize) -> Result<(), String> {
        if self.instructions.len() <= instruction_index {
            Err(format!("Unable to add label {}, index {} is out of bounds!", label, instruction_index))
        } else {
            self.control_flow.instruction_labels.insert(label, instruction_index);
            Ok(())
        }
    }

    /// Returns reference to **runtime_args**.
    pub fn runtime_args(&self) -> &RuntimeArgs {
        &self.runtime_args
    }

}

/// Used to control what instruction should be executed next.
pub struct ControlFlow<'a> {
    /// The index of the instruction that should be executed next in the **instructions** vector.
    pub next_instruction_index: usize,
    /// Stores label to instruction mappings.
    /// 
    /// Key = label of the instruction
    /// 
    /// Value = index of the instruction in the instructions vector
    pub instruction_labels: HashMap<&'a str, usize>,
}

impl<'a> ControlFlow<'a> {

    pub fn new() -> Self {
        Self {
            next_instruction_index: 0,
            instruction_labels: HashMap::new(),
        }
    }

    /// Updates **next_instruction_index** if **label** is contained in **instruction_labels**,
    /// otherwise returns an error.
    pub fn next_instruction_index(&mut self, label: &str) -> Result<(), String> {
        if let Some(index) = self.instruction_labels.get(label) {
            self.next_instruction_index = *index;
            Ok(())
        } else {
            Err(format!("Unable to update instruction index: no index found for label {}", label))
        }
    }
}

pub struct RuntimeArgs<'a> {
    /// Current values stored in accumulators
    pub accumulators: Vec<Accumulator>,
    /// All registers that are used to store data
    pub memory_cells: HashMap<&'a str, MemoryCell>,
    /// The stack of the runner
    pub stack: Vec<i32>,
}

impl<'a> RuntimeArgs<'a> {
    pub fn new() -> Self {
        let mut accumulators = Vec::new();
        for i in 0..ACCUMULATORS {
            accumulators.push(Accumulator::new(i));
        }
        if ACCUMULATORS <= 0 {
            accumulators.push(Accumulator::new(0));
        }
        let mut memory_cells: HashMap<&str, MemoryCell> = HashMap::new();
        for i in MEMORY_CELL_LABELS {
            memory_cells.insert(i, MemoryCell::new(i));
        }
        Self {
            accumulators,
            memory_cells,
            stack: Vec::new(),
        }
    }

    /// Creates a new runtimes args struct with empty lists.
    pub fn new_empty() -> Self {
        Self {
            accumulators: Vec::new(),
            memory_cells: HashMap::new(),
            stack: Vec::new(),
        }
    }

    /// Creates a new memory cell with label **label** if it does not already exist
    /// and adds it to the **memory_cells* hashmap.
    pub fn add_storage_cell(&mut self, label: &'a str) {
        if !self.memory_cells.contains_key(label) {
            self.memory_cells.insert(label, MemoryCell::new(label));
        }
    }

    /// Adds a new accumulator to the accumulators vector.
    pub fn add_accumulator(&mut self) {
        let id = self.accumulators.len();
        self.accumulators.push(Accumulator::new(id as i32));
    }
}