use std::collections::HashMap;

use crate::{instructions::Instruction, base::{Accumulator, MemoryCell}, ACCUMULATORS, MEMORY_CELL_LABELS};

//TODO make fields private and add access functions, move into separate module
pub struct Runner<'a> {
    runtime_args: RuntimeArgs<'a>,
    instructions: Vec<Instruction<'a>>,
    control_flow: ControlFlow<'a>,
}

impl<'a> Runner<'a> {
    pub fn new(instructions: Vec<Instruction<'a>>) -> Self {
        Self {
            runtime_args: RuntimeArgs::new(),
            instructions,
            control_flow: ControlFlow::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.control_flow.next_instruction_index < self.instructions.len() {
            if let Err(e) = self.instructions[self.control_flow.next_instruction_index].run(&mut self.runtime_args, &mut self.control_flow) {
                println!("Unable to continue execution, an irrecoverable error occured: {}", e);
                return Err(format!("Execution terminated: {}", e));
            } else {
                self.control_flow.next_instruction_index += 1;
            }
        }
        Ok(())
    }

    /// Adds an instruction to the end of the instruction vector with a label mapping.
    pub fn add_instruction_with_label(&mut self, instruction: Instruction<'a>, label: &'a str) {
        self.instructions.push(instruction);
        self.control_flow.instruction_labels.insert(label, self.instructions.len()-1);
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
}