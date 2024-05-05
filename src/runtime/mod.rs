use std::collections::HashMap;

use miette::Result;

use crate::{
    base::{Accumulator, MemoryCell},
    instructions::Instruction,
};

use self::{
    error_handling::{RuntimeError, RuntimeErrorType},
    memory_config::MemoryConfig,
};

/// Structs related to building a runtime
pub mod builder;
pub mod error_handling;
pub mod memory_config;

const MAX_CALL_STACK_SIZE: usize = u16::MAX as usize;
const MAX_INSTRUCTION_RUNS: usize = 1_000_000;

#[derive(Debug, PartialEq)]
pub struct Runtime {
    memory: RuntimeMemory,
    instructions: Vec<Instruction>,
    control_flow: ControlFlow,
    /// Used to count how many instructions where executed.
    ///
    /// If the `MAX_INSTRUCTION_RUNS` instruction has been executed a runtime error is thrown to indicate
    /// that the runtime has reached its design limit. This is among other things to protect from misuse and infinite loops.
    instruction_runs: usize,
    settings: RuntimeSettings,
}

impl Runtime {
    /// Runs the complete program.
    #[allow(dead_code)]
    pub fn run(&mut self) -> Result<bool, RuntimeError> {
        while self.control_flow.next_instruction_index < self.instructions.len() {
            self.step()?;
        }
        Ok(true)
    }

    /// Runs the next instruction only.
    ///
    /// Returns true when no instruction was run because the last instruction was already run.
    pub fn step(&mut self) -> Result<bool, RuntimeError> {
        let current_instruction = self.control_flow.next_instruction_index;
        self.control_flow.next_instruction_index += 1;
        if let Some(i) = self.instructions.get(current_instruction) {
            if let Err(e) = i.run(&mut self.memory, &mut self.control_flow, &self.settings) {
                return Err(RuntimeError {
                    reason: e,
                    line_number: current_instruction + 1,
                })?;
            }
            self.verify(current_instruction + 1)?;
            self.instruction_runs += 1;
        } else {
            return Ok(true);
        }
        Ok(false)
    }

    /// Verifies that the current runtime is legal.
    ///
    /// The runtime is illegal, if specific conditions are met:
    /// - The maximum stack size is exceeded
    /// - 1mil instructions where executed (this is to protect from infinite loops and because the runtime is to build to run so many instructions)
    fn verify(&self, line_number: usize) -> Result<(), RuntimeError> {
        if self.control_flow.call_stack.len() >= MAX_CALL_STACK_SIZE {
            return Err(RuntimeError {
                reason: RuntimeErrorType::StackOverflowError,
                line_number,
            });
        }
        if !self.settings.disable_instruction_limit && self.instruction_runs > MAX_INSTRUCTION_RUNS
        {
            return Err(RuntimeError {
                reason: RuntimeErrorType::DesignLimitReached(MAX_INSTRUCTION_RUNS),
                line_number,
            });
        }
        Ok(())
    }

    /// Sets the instruction that should be executed next.
    ///
    /// Warning: using this may lead to runtime errors due to changed call stack.
    pub fn set_next_instruction(&mut self, idx: usize) {
        self.control_flow.next_instruction_index = idx;
    }

    /// Returns true when the execution is finished,
    pub fn finished(&self) -> bool {
        self.control_flow.next_instruction_index >= self.instructions.len()
    }

    /// Returns the index of the current instruction
    pub fn next_instruction_index(&self) -> usize {
        self.control_flow.next_instruction_index
    }

    /// Returns reference to **`runtime_args`**.
    pub fn runtime_memory(&self) -> &RuntimeMemory {
        &self.memory
    }

    /// Returns a reference to **`control_flow`**.
    pub fn control_flow(&self) -> &ControlFlow {
        &self.control_flow
    }

    /// Resets the current runtime to defaults, resets instruction pointer.
    pub fn reset(&mut self) {
        self.control_flow.reset_soft();
        self.memory.reset();
    }

    /// Returns the index of the instruction that is executed first
    pub fn initial_instruction_index(&self) -> usize {
        self.control_flow.initial_instruction
    }

    /// Runs the provided instruction in this runtime.
    ///
    /// Warning: depending on the instruction this may break things.
    pub fn run_foreign_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), RuntimeError> {
        if let Err(e) = instruction.run(&mut self.memory, &mut self.control_flow, &self.settings) {
            return Err(RuntimeError {
                reason: e,
                line_number: self.control_flow.next_instruction_index,
            })?;
        }
        Ok(())
    }

    /// Checks if this runtime contains at least one call instruction.
    pub fn contains_call_instruction(&self) -> bool {
        let mut res = false;
        for instruction in &self.instructions {
            if let Instruction::Call(_) = instruction {
                res = true
            };
        }
        res
    }
}

/// Used to control what instruction should be executed next.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlFlow {
    /// The index of the instruction that should be executed next in the **instructions** vector.
    pub next_instruction_index: usize,
    /// Stores label to instruction mappings.
    ///
    /// Key = label of the instruction
    ///
    /// Value = index of the instruction in the instructions vector
    pub instruction_labels: HashMap<String, usize>,
    /// Stores the index of the next instruction after a function returns
    pub call_stack: Vec<usize>,
    initial_instruction: usize,
}

impl ControlFlow {
    pub fn new() -> Self {
        Self {
            next_instruction_index: 0,
            instruction_labels: HashMap::new(),
            call_stack: Vec::new(),
            initial_instruction: 0,
        }
    }

    /// Updates **`next_instruction_index`** if **label** is contained in **`instruction_labels`**,
    /// otherwise returns an error.
    pub fn next_instruction_index(&mut self, label: &str) -> Result<(), RuntimeErrorType> {
        if let Some(index) = self.instruction_labels.get(label) {
            self.next_instruction_index = *index;
            Ok(())
        } else {
            Err(RuntimeErrorType::LabelMissing(label.to_string()))
        }
    }

    /// Updates the call stack with the instruction index from which the function was called
    /// and sets the next instruction index.
    /// Returns `StackOverflowError` when call stack exceeds size of `i16::max` elements (= the maximum size is ~2MB).
    pub fn call_function(&mut self, label: &str) -> Result<(), RuntimeErrorType> {
        self.call_stack.push(self.next_instruction_index);
        self.next_instruction_index(label)?;
        Ok(())
    }

    /// Resets the `next_instruction_index` to 0 and clears the call stack.
    pub fn reset_soft(&mut self) {
        self.next_instruction_index = self.initial_instruction;
        self.call_stack.clear();
    }
}

/// Used to store the values of the different memory spaces, while a program is run
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::module_name_repetitions, clippy::option_option)]
pub struct RuntimeMemory {
    /// Current values stored in accumulators
    pub accumulators: HashMap<usize, Accumulator>,
    /// The value of the gamma accumulator
    ///
    /// First option determines if gamma is active.
    /// Inner option determine if gamma contains a value.
    pub gamma: Option<Option<i32>>,
    /// All registers that are used to store data
    pub memory_cells: HashMap<String, MemoryCell>,
    /// All index registers that are used to store data,
    /// key is the index, value is the value of that register
    pub index_memory_cells: HashMap<usize, Option<i32>>,
    /// The stack of the runner
    pub stack: Vec<i32>,
}

impl Default for RuntimeMemory {
    /// Creates a runtime memory with 4 accumulators and 4 memory cells.
    fn default() -> Self {
        let mut accumulators = HashMap::new();
        for i in 0..4 {
            accumulators.insert(i, Accumulator::new(i));
        }
        let mut memory_cells: HashMap<String, MemoryCell> = HashMap::new();
        for i in 0..4 {
            let label = format!("h{i}");
            memory_cells.insert(label.clone(), MemoryCell::new(&label));
        }
        Self {
            accumulators,
            gamma: None,
            memory_cells,
            index_memory_cells: HashMap::new(),
            stack: Vec::new(),
        }
    }
}

impl RuntimeMemory {
    /// Checks if the accumulator with id exists.
    pub fn exists_accumulator(&self, id: usize) -> bool {
        for acc in &self.accumulators {
            if acc.0 == &id {
                return true;
            }
        }
        false
    }

    /// Resets all values back to None.
    pub fn reset(&mut self) {
        for acc in &mut self.accumulators {
            acc.1.data = None;
        }
        for cell in &mut self.memory_cells {
            cell.1.data = None;
        }
        for cell in &mut self.index_memory_cells {
            *cell.1 = None;
        }
        self.stack = Vec::new();
    }
}

impl From<MemoryConfig> for RuntimeMemory {
    fn from(value: MemoryConfig) -> Self {
        let mut accumulators = HashMap::new();
        for (idx, value) in value.accumulators.values {
            accumulators.insert(
                idx,
                Accumulator {
                    id: idx,
                    data: value,
                },
            );
        }
        let mut memory_cells = HashMap::new();
        for (label, value) in value.memory_cells.values {
            memory_cells.insert(label.clone(), MemoryCell { label, data: value });
        }
        let index_memory_cells = value.index_memory_cells.values;
        let gamma = if value.gamma_accumulator.enabled {
            match value.gamma_accumulator.value {
                Some(value) => Some(Some(value)),
                None => Some(None),
            }
        } else {
            None
        };
        Self {
            accumulators,
            gamma,
            memory_cells,
            index_memory_cells,
            stack: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Settings that may be required during runtime
pub struct RuntimeSettings {
    pub disable_instruction_limit: bool,
    // If true, accumulators will be created automatically, if they are accessed and the don't already exist.
    pub autodetect_accumulators: bool,
    // If true, accumulator gamma will be created automatically, if it is accessed it does not already exist.
    pub autodetect_gamma_accumulator: bool,
    // If true, memory cells will be created automatically, if they are accessed and the don't already exist.
    pub autodetect_memory_cells: bool,
    // If true, index memory cells will be created automatically, if they are accessed and the don't already exist.
    pub autodetect_index_memory_cells: bool,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            disable_instruction_limit: false,
            autodetect_accumulators: true,
            autodetect_gamma_accumulator: true,
            autodetect_memory_cells: true,
            autodetect_index_memory_cells: true,
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use std::collections::HashMap;

    use crate::base::{Accumulator, MemoryCell};

    use super::RuntimeMemory;

    impl<'a> RuntimeMemory {
        pub fn new_debug(memory_cells: &'a [&'static str]) -> Self {
            Self::new(
                4,
                memory_cells.iter().map(|f| (*f).to_string()).collect(),
                None,
                true,
            )
        }

        #[allow(dead_code)]
        pub fn new_empty() -> Self {
            Self {
                accumulators: HashMap::new(),
                gamma: None,
                memory_cells: HashMap::new(),
                index_memory_cells: HashMap::new(),
                stack: Vec::new(),
            }
        }

        pub fn new(
            acc: usize,
            m_cells: Vec<String>,
            idx_m_cells: Option<Vec<usize>>,
            enable_gamma: bool,
        ) -> Self {
            let mut accumulators = HashMap::new();
            for i in 0..acc {
                accumulators.insert(i, Accumulator::new(i));
            }
            let mut memory_cells: HashMap<String, MemoryCell> = HashMap::new();
            for i in m_cells {
                memory_cells.insert(i.clone(), MemoryCell::new(i.as_str()));
            }
            let gamma = if enable_gamma { Some(None) } else { None };
            let mut index_memory_cells = HashMap::new();
            if let Some(cells) = idx_m_cells {
                for c in cells {
                    index_memory_cells.insert(c, None);
                }
            }
            Self {
                accumulators,
                gamma,
                memory_cells,
                index_memory_cells,
                stack: Vec::new(),
            }
        }

        /// Creates a new memory cell with label **label** if it does not already exist
        /// and adds it to the **`memory_cells`* hashmap.
        #[allow(dead_code)]
        pub fn add_storage_cell(&mut self, label: &str) {
            if !self.memory_cells.contains_key(label) {
                self.memory_cells
                    .insert(label.to_string(), MemoryCell::new(label));
            }
        }

        /// Adds a new accumulator to the accumulators vector.
        #[allow(dead_code)]
        pub fn add_accumulator(&mut self) {
            let id = self.accumulators.len();
            self.accumulators.insert(id, Accumulator::new(id));
        }
    }
}
