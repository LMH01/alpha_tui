use std::collections::{HashMap, BinaryHeap};

use miette::Result;

use crate::{
    base::{Accumulator, MemoryCell, IndexMemoryCell},
    cli::Args,
    instructions::Instruction,
    utils::read_file,
};

use self::error_handling::{RuntimeError, RuntimeErrorType};

/// Structs related to building a runtime
pub mod builder;
pub mod error_handling;

#[derive(Debug, PartialEq)]
pub struct Runtime {
    runtime_args: RuntimeArgs,
    instructions: Vec<Instruction>,
    control_flow: ControlFlow,
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
            if let Err(e) = i.run(&mut self.runtime_args, &mut self.control_flow) {
                return Err(RuntimeError {
                    reason: e,
                    line_number: current_instruction + 1,
                })?;
            }
        } else {
            return Ok(true);
        }
        Ok(false)
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
    pub fn runtime_args(&self) -> &RuntimeArgs {
        &self.runtime_args
    }

    /// Resets the current runtime to defaults, resets instruction pointer.
    pub fn reset(&mut self) {
        self.control_flow.reset_soft();
        self.runtime_args.reset();
    }

    /// Returns the index of the instruction that is executed first
    pub fn initial_instruction_index(&self) -> usize {
        self.control_flow.initial_instruction
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
    /// Returns StackOverflowError when call stack exceeds size of i16::max elements (= the maximum size is ~2MB).
    pub fn call_function(&mut self, label: &str) -> Result<(), RuntimeErrorType> {
        self.call_stack
            .push(self.next_instruction_index);
        if self.call_stack.len() > i16::MAX as usize {
            return Err(RuntimeErrorType::StackOverflowError)
        }
        self.next_instruction_index(label)?;
        Ok(())
    }

    /// Resets the `next_instruction_index` to 0 and clears the `instruction_labels` map.
    pub fn reset(&mut self) {
        self.next_instruction_index = 0;
        self.instruction_labels.clear();
        self.call_stack.clear();
    }

    /// Resets the `next_instruction_index` to 0 and clears the call stack.
    pub fn reset_soft(&mut self) {
        self.next_instruction_index = self.initial_instruction;
        self.call_stack.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct RuntimeArgs {
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
    pub index_memory_cells: HashMap<usize, i32>,
    /// The stack of the runner
    pub stack: Vec<i32>,
}

impl<'a> RuntimeArgs {
    /// Creates a new runtimes args struct with empty lists.
    ///
    /// Errors if option is set to parse memory cells from file and the parsing fails.
    pub fn from_args(args: &Args) -> Result<Self, String> {
        if let Some(path) = &args.memory_cell_file {
            match read_memory_cells_from_file(path) {
                Ok(memory_cells) => {
                    let accumulators = match args.accumulators {
                        None => HashMap::new(),
                        Some(v) => {
                            let mut accumulators = HashMap::new();
                            for i in 0..v {
                                accumulators.insert(i as usize, Accumulator::new(i as usize));
                            }
                            accumulators
                        }
                    };
                    return Ok(Self {
                        accumulators,
                        gamma: None,
                        memory_cells,
                        index_memory_cells: HashMap::new(),
                        stack: Vec::new(),
                    });
                }
                Err(e) => return Err(e),
            };
        }
        let accumulators = args.accumulators.unwrap_or(0);
        let memory_cells = match args.memory_cells.as_ref() {
            None => Vec::new(),
            Some(value) => value.clone(),
        };
        Ok(Self::new(accumulators as usize, memory_cells))
    }

    pub fn new_debug(memory_cells: &'a [&'static str]) -> Self {
        Self::new(4, memory_cells.iter().map(|f| (*f).to_string()).collect())
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

    fn new(acc: usize, m_cells: Vec<String>) -> Self {
        let mut accumulators = HashMap::new();
        for i in 0..acc {
            accumulators.insert(i, Accumulator::new(i));
        }
        let mut memory_cells: HashMap<String, MemoryCell> = HashMap::new();
        for i in m_cells {
            memory_cells.insert(i.clone(), MemoryCell::new(i.as_str()));
        }
        Self {
            accumulators,
            gamma: None,
            memory_cells,
            index_memory_cells: HashMap::new(),
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
        self.stack = Vec::new();
    }
}

/// Reads memory cells from file and returns map of memory cells.
///
/// Each line contains a single memory cell in the following formatting: NAME=VALUE
///
/// If value is missing an empty memory cell will be created.
///
/// Errors when file could not be read.
#[allow(clippy::unnecessary_unwrap)]
fn read_memory_cells_from_file(path: &str) -> Result<HashMap<String, MemoryCell>, String> {
    let contents = read_file(path)?;
    let mut map = HashMap::new();
    for (index, line) in contents.iter().enumerate() {
        let chunks = line.split('=').collect::<Vec<&str>>();
        let v = chunks.get(1);
        if v.is_some() && !v.unwrap().is_empty() {
            let v = v.unwrap();
            let value = match v.parse::<i32>() {
                Ok(num) => num,
                Err(e) => {
                    return Err(format!(
                        "{}: [Line {}] Unable to parse int: {} \"{}\"",
                        path,
                        index + 1,
                        e,
                        v
                    ))
                }
            };
            map.insert(
                chunks[0].to_string(),
                MemoryCell {
                    label: chunks[0].to_string(),
                    data: Some(value),
                },
            );
        } else {
            map.insert(chunks[0].to_string(), MemoryCell::new(chunks[0]));
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{Comparison, Operation},
        instructions::{Instruction, TargetType, Value},
        runtime::{builder::RuntimeBuilder, error_handling::RuntimeBuildError},
    };

    use super::RuntimeArgs;

    /// Used to set the available memory cells during testing.
    const TEST_MEMORY_CELL_LABELS: &[&str] = &[
        "a", "b", "c", "d", "e", "f", "w", "x", "y", "z", "h1", "h2", "h3", "h4",
    ];

    #[test]
    fn test_label_missing() {
        test_label_instruction(Instruction::Goto("loop".to_string()), "loop");
        test_label_instruction(
            Instruction::JumpIf(
                Value::Accumulator(0),
                Comparison::Eq,
                Value::Accumulator(0),
                "loop".to_string(),
            ),
            "loop",
        );
        test_label_instruction(
            Instruction::JumpIf(
                Value::Accumulator(0),
                Comparison::Eq,
                Value::MemoryCell("h1".to_string()),
                "loop".to_string(),
            ),
            "loop",
        );
    }

    fn test_label_instruction(instruction: Instruction, label: &str) {
        let instructions = vec![instruction];
        let mut rb = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
        rb.set_instructions(instructions.clone());
        assert!(rb.add_label(label.to_string(), 0).is_ok());
        assert!(rb.build().is_ok());
        rb.reset();
        rb.set_instructions(instructions);
        rb.set_runtime_args(RuntimeArgs::new_debug(TEST_MEMORY_CELL_LABELS));
        assert_eq!(
            rb.build(),
            Err(RuntimeBuildError::LabelUndefined(label.to_string()))
        );
    }

    #[test]
    fn test_accumulator_missing() {
        test_accumulator_instruction(
            Instruction::Assign(TargetType::Accumulator(0), Value::Accumulator(1)),
            vec![&0_usize, &1_usize],
        );
        test_accumulator_instruction(
            Instruction::Calc(
                TargetType::Accumulator(0),
                Value::Accumulator(1),
                Operation::Add,
                Value::Accumulator(2),
            ),
            vec![&0_usize, &1_usize, &2_usize],
        );
    }

    fn test_accumulator_instruction(instruction: Instruction, to_test: Vec<&usize>) {
        let mut rb = RuntimeBuilder::new();
        rb.set_instructions(vec![instruction]);
        _ = rb.add_label("loop".to_string(), 0);
        // Test if ok works
        let mut runtime_args = RuntimeArgs::new_empty();
        runtime_args.add_storage_cell("h1");
        runtime_args.add_storage_cell("h2");
        for _ in &to_test {
            runtime_args.add_accumulator();
        }
        rb.set_runtime_args(runtime_args);
        let build = rb.build();
        assert!(build.is_ok());
        // Test if missing accumulators are detected
        for (_, s) in to_test.iter().enumerate() {
            let mut runtime_args = RuntimeArgs::new_empty();
            runtime_args.add_storage_cell("h1");
            runtime_args.add_storage_cell("h2");
            for _ in 0..(to_test.len() - *s - 1) {
                runtime_args.add_accumulator();
            }
            rb.set_runtime_args(runtime_args);
            let b = rb.build();
            assert_eq!(
                b,
                Err(RuntimeBuildError::AccumulatorMissing(
                    (to_test.len() - *s - 1).to_string()
                ))
            );
        }
    }

    #[test]
    fn test_memory_cell_missing() {
        test_memory_cell_instruction(
            Instruction::Assign(
                TargetType::MemoryCell("h1".to_string()),
                Value::MemoryCell("h2".to_string()),
            ),
            vec!["h1", "h2"],
        );
        test_memory_cell_instruction(
            Instruction::Calc(
                TargetType::MemoryCell("h1".to_string()),
                Value::MemoryCell("h2".to_string()),
                Operation::Add,
                Value::MemoryCell("h3".to_string()),
            ),
            vec!["h1", "h2", "h3"],
        );
    }

    fn test_memory_cell_instruction(instruction: Instruction, to_test: Vec<&str>) {
        let mut rb = RuntimeBuilder::new();
        rb.set_instructions(vec![instruction]);
        _ = rb.add_label("loop".to_string(), 0);
        // Test if ok works
        let mut runtime_args = RuntimeArgs::new_empty();
        runtime_args.add_accumulator();
        for s in &to_test {
            runtime_args.add_storage_cell(s);
        }
        rb.set_runtime_args(runtime_args);
        let build = rb.build();
        println!("{:?}", build);
        assert!(build.is_ok());
        // Test if missing memory cells are detected
        for (i1, s) in to_test.iter().enumerate() {
            let mut runtime_args = RuntimeArgs::new_empty();
            runtime_args.add_accumulator();
            for (i2, s2) in to_test.iter().enumerate() {
                if i1 == i2 {
                    continue;
                }
                runtime_args.add_storage_cell(s2);
            }
            rb.set_runtime_args(runtime_args);
            let b = rb.build();
            assert_eq!(b, Err(RuntimeBuildError::MemoryCellMissing(s.to_string())));
        }
    }
}
