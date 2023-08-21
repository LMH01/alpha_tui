use std::collections::HashMap;

use miette::Result;

use crate::{
    base::{Accumulator, MemoryCell},
    instructions::Instruction, cli::Args, utils::read_file,
};

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
    pub fn run(&mut self) -> Result<(), String> {
        while self.control_flow.next_instruction_index < self.instructions.len() {
            self.step()?;
        }
        Ok(())
    }

    /// Runs the next instruction only.
    pub fn step(&mut self) -> Result<(), String> {
        let current_instruction = self.control_flow.next_instruction_index;
        self.control_flow.next_instruction_index += 1;
        if let Err(e) = self.instructions[current_instruction]
            .run(&mut self.runtime_args, &mut self.control_flow)
        {
            return Err(format!("[Line {}] {}", current_instruction+1, e));
        }
        Ok(())
    }

    /// Returns true when the execution is finished,
    pub fn finished(&self) -> bool {
        self.control_flow.next_instruction_index >= self.instructions.len()
    }

    /// Returns the index of the current instruction
    pub fn current_instruction_index(&self) -> usize {
        self.control_flow.next_instruction_index - 1
    }

    /// Returns reference to **runtime_args**.
    pub fn runtime_args(&self) -> &RuntimeArgs {
        &self.runtime_args
    }

    /// Resets the current runtime to defaults, resets instruction pointer.
    pub fn reset(&mut self) {
        self.control_flow.reset_soft();
        self.runtime_args.reset();
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
}

impl ControlFlow {
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
            Err(format!(
                "Unable to update instruction index: no index found for label {}",
                label
            ))
        }
    }

    /// Resets the `next_instruction_index` to 0 and clears the `instruction_labels` map.
    pub fn reset(&mut self) {
        self.next_instruction_index = 0;
        self.instruction_labels.clear();
    }

    /// Only resets the `next_instruction_index` to 0.
    pub fn reset_soft(&mut self) {
        self.next_instruction_index = 0;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeArgs {
    /// Current values stored in accumulators
    pub accumulators: Vec<Accumulator>,
    /// All registers that are used to store data
    pub memory_cells: HashMap<String, MemoryCell>,
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
                        None => Vec::new(),
                        Some(v) => {
                            let mut accumulators = Vec::new();
                            for i in 0..v {
                                accumulators.push(Accumulator::new(i as usize));
                            }
                            accumulators
                        },
                    };
                    return Ok(Self {accumulators, memory_cells, stack: Vec::new()})
                },
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
        Self::new(4, memory_cells.iter().map(|f| f.to_string()).collect())
    }

    #[allow(dead_code)]
    pub fn new_empty() -> Self {
        Self {
            accumulators: Vec::new(),
            memory_cells: HashMap::new(),
            stack: Vec::new(),
        }
    }

    fn new(acc: usize, m_cells: Vec<String>) -> Self {
        let mut accumulators = Vec::new();
        for i in 0..acc {
            accumulators.push(Accumulator::new(i));
        }
        let mut memory_cells: HashMap<String, MemoryCell> = HashMap::new();
        for i in m_cells {
            memory_cells.insert(i.clone(), MemoryCell::new(i.as_str()));
        }
        Self {
            accumulators,
            memory_cells,
            stack: Vec::new(),
        }
    }

    /// Creates a new memory cell with label **label** if it does not already exist
    /// and adds it to the **memory_cells* hashmap.
    #[allow(dead_code)]
    pub fn add_storage_cell(&mut self, label: &str) {
        if !self.memory_cells.contains_key(label) {
            self.memory_cells.insert(label.to_string(), MemoryCell::new(label));
        }
    }

    /// Adds a new accumulator to the accumulators vector.
    #[allow(dead_code)]
    pub fn add_accumulator(&mut self) {
        let id = self.accumulators.len();
        self.accumulators.push(Accumulator::new(id));
    }

    /// Checks if the accumulator with id exists.
    pub fn exists_accumulator(&self, id: &usize) -> bool {
        for acc in &self.accumulators {
            if acc.id == *id {
                return true;
            }
        }
        false
    }

    /// Resets all values back to None.
    pub fn reset(&mut self) {
        for acc in self.accumulators.iter_mut() {
            acc.data = None;
        }
        for cell in self.memory_cells.iter_mut() {
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
                Err(e) => return Err(format!("{}: [Line {}] Unable to parse int: {} \"{}\"", path, index+1, e, v)),
            };
            map.insert(chunks[0].to_string(), MemoryCell {label: chunks[0].to_string(), data: Some(value)});
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
        instructions::Instruction,
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
            Instruction::GotoIfAccumulator(Comparison::Equal, "loop".to_string(), 0, 0),
            "loop",
        );
        test_label_instruction(
            Instruction::GotoIfConstant(Comparison::Equal, "loop".to_string(), 0, 0),
            "loop",
        );
        test_label_instruction(
            Instruction::GotoIfMemoryCell(
                Comparison::Equal,
                "loop".to_string(),
                0,
                "a".to_string(),
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
            Err(RuntimeBuildError::LabelMissing(label.to_string()))
        );
    }

    #[test]
    fn test_accumulator_missing() {
        test_accumulator_instruction(
            Instruction::AssignAccumulatorValue(0, 1),
            vec![&0_usize],
        );
        test_accumulator_instruction(
            Instruction::AssignAccumulatorValueFromAccumulator(0, 1),
            vec![&0_usize, &1_usize],
        );
        test_accumulator_instruction(
            Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string()),
            vec![&0_usize],
        );
        test_accumulator_instruction(
            Instruction::AssignAccumulatorValue(0, 1),
            vec![&0_usize],
        );
        test_accumulator_instruction(
            Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 0),
            vec![&0_usize],
        );
        test_accumulator_instruction(
            Instruction::CalcAccumulatorWithAccumulator(Operation::Plus, 0, 1),
            vec![&0_usize, &1_usize],
        );
        test_accumulator_instruction(
            Instruction::CalcAccumulatorWithAccumulators(Operation::Plus, 0, 1, 2),
            vec![&0_usize, &1_usize, &2_usize],
        );
        test_accumulator_instruction(
            Instruction::CalcAccumulatorWithConstant(Operation::Plus, 0, 0),
            vec![&0_usize],
        );
        test_accumulator_instruction(
            Instruction::CalcAccumulatorWithMemoryCell(Operation::Plus, 0, "a".to_string()),
            vec![&0_usize],
        );
        test_accumulator_instruction(
            Instruction::CalcAccumulatorWithMemoryCells(
                Operation::Plus,
                0,
                "a".to_string(),
                "b".to_string(),
            ),
            vec![&0_usize],
        );
        test_accumulator_instruction(
            Instruction::CalcMemoryCellWithMemoryCellAccumulator(
                Operation::Plus,
                "a".to_string(),
                "b".to_string(),
                0,
            ),
            vec![&0_usize],
        );
        test_accumulator_instruction(
            Instruction::GotoIfAccumulator(Comparison::Equal, "loop".to_string(), 0, 0),
            vec![&0_usize],
        );
    }

    fn test_accumulator_instruction(instruction: Instruction, to_test: Vec<&usize>) {
        let mut rb = RuntimeBuilder::new();
        rb.set_instructions(vec![instruction]);
        _ = rb.add_label("loop".to_string(), 0);
        // Test if ok works
        let mut runtime_args = RuntimeArgs::new_empty();
        runtime_args.add_storage_cell("a");
        runtime_args.add_storage_cell("b");
        for _ in &to_test {
            runtime_args.add_accumulator();
        }
        rb.set_runtime_args(runtime_args);
        let build = rb.build();
        println!("{:?}", build);
        assert!(build.is_ok());
        // Test if missing accumulators are detected
        for (_, s) in to_test.iter().enumerate() {
            let mut runtime_args = RuntimeArgs::new_empty();
            runtime_args.add_storage_cell("a");
            runtime_args.add_storage_cell("b");
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
            Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string()),
            vec!["a"],
        );
        test_memory_cell_instruction(
            Instruction::AssignMemoryCellValue("a".to_string(), 0),
            vec!["a"],
        );
        test_memory_cell_instruction(
            Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 0),
            vec!["a"],
        );
        test_memory_cell_instruction(
            Instruction::AssignMemoryCellValueFromMemoryCell("a".to_string(), "b".to_string()),
            vec!["a", "b"],
        );
        test_memory_cell_instruction(
            Instruction::CalcAccumulatorWithMemoryCell(Operation::Plus, 0, "a".to_string()),
            vec!["a"],
        );
        test_memory_cell_instruction(
            Instruction::CalcAccumulatorWithMemoryCells(
                Operation::Plus,
                0,
                "a".to_string(),
                "b".to_string(),
            ),
            vec!["a", "b"],
        );
        test_memory_cell_instruction(
            Instruction::CalcMemoryCellWithMemoryCellAccumulator(
                Operation::Plus,
                "a".to_string(),
                "b".to_string(),
                0,
            ),
            vec!["a", "b"],
        );
        test_memory_cell_instruction(
            Instruction::CalcMemoryCellWithMemoryCellConstant(
                Operation::Plus,
                "a".to_string(),
                "b".to_string(),
                0,
            ),
            vec!["a", "b"],
        );
        test_memory_cell_instruction(
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Plus,
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
            ),
            vec!["a", "b", "c"],
        );
        test_memory_cell_instruction(
            Instruction::GotoIfMemoryCell(
                Comparison::Equal,
                "loop".to_string(),
                0,
                "a".to_string(),
            ),
            vec!["a"],
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