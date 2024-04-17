use std::collections::HashMap;

use miette::Result;

use crate::{
    base::{Accumulator, MemoryCell},
    cli::Cli,
    instructions::Instruction, utils,
};

use self::error_handling::{RuntimeError, RuntimeErrorType};

/// Structs related to building a runtime
pub mod builder;
pub mod error_handling;

const MAX_CALL_STACK_SIZE: usize = u16::MAX as usize;
const MAX_INSTRUCTION_RUNS: usize = 1_000_000;

#[derive(Debug, PartialEq)]
pub struct Runtime {
    runtime_args: RuntimeArgs,
    instructions: Vec<Instruction>,
    control_flow: ControlFlow,
    /// Used to count how many instructions where executed.
    ///
    /// If the `MAX_INSTRUCTION_RUNS` instruction has been executed a runtime error is thrown to indicate
    /// that the runtime has reached its design limit. This is among other things to protect from misuse and infinite loops.
    instruction_runs: usize,
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
        if !self.runtime_args.settings.disable_instruction_limit
            && self.instruction_runs > MAX_INSTRUCTION_RUNS
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

    /// Runs the provided instruction in this runtime.
    ///
    /// Warning: depending on the instruction this may break things.
    pub fn run_foreign_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), RuntimeError> {
        if let Err(e) = instruction.run(&mut self.runtime_args, &mut self.control_flow) {
            return Err(RuntimeError {
                reason: e,
                line_number: self.control_flow.next_instruction_index,
            })?;
        }
        Ok(())
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
#[allow(clippy::module_name_repetitions, clippy::option_option)]
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
    pub index_memory_cells: HashMap<usize, Option<i32>>,
    /// The stack of the runner
    pub stack: Vec<i32>,
    pub settings: Settings,
}

impl<'a> RuntimeArgs {
    /// Creates a new runtimes args struct with empty lists.
    ///
    /// Errors if option is set to parse memory cells from file and the parsing fails.
    pub fn from_args(args: &Cli) -> Result<Self, String> {
        if let Some(path) = &args.memory_config_file {
            let config = match MemoryConfig::from_file_contents(&utils::read_file(path)?, path) {
                Ok(config) => config,
                Err(e) => return Err(e),
            };
            return Ok(Self {
                accumulators: config.accumulators,
                memory_cells: config.memory_cells,
                index_memory_cells: config.index_memory_cells,
                gamma: config.gamma_accumulator,
                stack: Vec::new(),
                settings: Settings::from(args)
            });
        }
        let accumulators = args.accumulators.unwrap_or(0);
        let memory_cells = match args.memory_cells.as_ref() {
            None => Vec::new(),
            Some(value) => value.clone(),
        };
        let idx_memory_cells = args.index_memory_cells.as_ref().cloned();
        Ok(Self::new(
            accumulators as usize,
            memory_cells,
            idx_memory_cells,
            args.enable_gamma_accumulator,
            Settings::from(args),
        ))
    }

    pub fn new_debug(memory_cells: &'a [&'static str]) -> Self {
        Self::new(
            4,
            memory_cells.iter().map(|f| (*f).to_string()).collect(),
            None,
            true,
            Settings::new_default(),
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
            settings: Settings::new_default(),
        }
    }

    fn new(
        acc: usize,
        m_cells: Vec<String>,
        idx_m_cells: Option<Vec<usize>>,
        enable_gamma: bool,
        settings: Settings,
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
            settings,
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
        for cell in &mut self.index_memory_cells {
            *cell.1 = None;
        }
        self.stack = Vec::new();
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Settings that may be required during runtime
pub struct Settings {
    /// If true, index memory cells are generated when they are receiving a value and if the don't already exist.
    ///
    /// If false, they will not be generated and a runtime error is thrown.
    pub enable_imc_auto_creation: bool,
    pub disable_instruction_limit: bool,
}

impl Settings {
    fn new_default() -> Self {
        Self {
            enable_imc_auto_creation: true,
            disable_instruction_limit: false,
        }
    }
}

impl From<&Cli> for Settings {
    fn from(value: &Cli) -> Self {
        Self {
            enable_imc_auto_creation: !value.disable_memory_detection,
            disable_instruction_limit: value.disable_instruction_limit,
        }
    }
}

/// Used to read in memory config from a file and then construct runtime args.
#[derive(PartialEq, Debug)]
struct MemoryConfig {
    pub accumulators: HashMap<usize, Accumulator>,
    /// The value of the gamma accumulator
    ///
    /// First option determines if gamma is active.
    /// Inner option determine if gamma contains a value.
    pub gamma_accumulator: Option<Option<i32>>,
    pub memory_cells: HashMap<String, MemoryCell>,
    pub index_memory_cells: HashMap<usize, Option<i32>>,
}

impl MemoryConfig {
    /// Creates the memory configuration by taking the values in `contents`.
    /// `path` is used to print a helpful error message.
    ///
    /// Supports empty values, to make it possible to declare existing memory locations with this file.
    ///
    /// Each line contains either a single memory cell, a single accumulator or the gamma accumulator:
    ///
    /// ## File formattings
    ///
    /// ### Accumulators
    ///
    /// a<ID>=VALUE or a<ID>
    ///
    /// ### Gamma
    ///
    /// y if gamma should be enabled but without a value or
    /// y=VALUE if gamma should be enabled and contain a value
    ///
    /// If multiple gamma values are placed in the file, the last value in the file is used.
    ///
    /// ### Memory cell
    ///
    /// NAME=VALUE, NAME, [INDEX]=VALUE or [INDEX]
    ///
    /// Memory cells can not be named y or begin with a because these are used to identify accumulators and the gamma accumulator.
    fn from_file_contents(contents: &Vec<String>, path: &str) -> Result<MemoryConfig, String> {
        let mut accumulators = HashMap::new();
        let mut gamma_accumulator = None;
        let mut memory_cells = HashMap::new();
        let mut index_memory_cells = HashMap::new();
        for (index, line) in contents.iter().enumerate() {
            let chunks = line.split('=').collect::<Vec<&str>>();
            let v = chunks.get(1);
            let idx = parse_index(&chunks);

            // check if line is a comment or empty
            if line.is_empty() || line.starts_with("//") || line.starts_with("#") {
                continue;
            }

            // check if line is accumulator
            if line.starts_with("a") {
                let idx = match idx {
                    Some(idx) => idx,
                    None => {
                        return Err(format!(
                            "{}: [Line {}] Unable to parse index: {}",
                            path,
                            index + 1,
                            line
                        ))
                    }
                };
                if v.is_none() || v.unwrap().is_empty() {
                    accumulators.insert(idx, Accumulator::new(idx));
                    continue;
                }
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
                accumulators.insert(
                    idx,
                    Accumulator {
                        id: idx,
                        data: Some(value),
                    },
                );
                continue;
            }

            // check if line is gamma
            if line.starts_with("y") {
                if v.is_none() || v.unwrap().is_empty() {
                    // gamma is active, but without a value
                    gamma_accumulator = Some(None);
                    continue;
                }
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
                gamma_accumulator = Some(Some(value));
                continue;
            }

            // Check if line is index memory cell
            if v.is_some() && !v.unwrap().is_empty() {
                let v = v.unwrap();
                if v.is_empty() {
                    memory_cells.insert(
                        chunks[0].to_string(),
                        MemoryCell {
                            label: chunks[0].to_string(),
                            data: None,
                        },
                    );
                    continue;
                }
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
                if let Some(idx) = idx {
                    index_memory_cells.insert(idx, Some(value));
                } else {
                    memory_cells.insert(
                        chunks[0].to_string(),
                        MemoryCell {
                            label: chunks[0].to_string(),
                            data: Some(value),
                        },
                    );
                }
            } else if let Some(idx) = idx {
                index_memory_cells.insert(idx, None);
            } else {
                memory_cells.insert(chunks[0].to_string(), MemoryCell::new(chunks[0]));
            }
        }
        Ok(Self {
            accumulators,
            gamma_accumulator,
            memory_cells,
            index_memory_cells,
        })
    }
}

/// Tries to parse an index from the first chunk.
///
/// In order to parse the index the formatting has to be "[INDEX]".
fn parse_index(chunks: &[&str]) -> Option<usize> {
    if let Some(p1) = chunks.first() {
        if p1.starts_with('[') && p1.ends_with(']') {
            let idx = p1.replacen('[', "", 1).replacen(']', "", 1);
            if let Ok(idx) = idx.parse::<usize>() {
                return Some(idx);
            }
        } else if p1.starts_with("a") {
            // accumulator index
            let idx = p1.replace("a", "");
            if let Ok(idx) = idx.parse::<usize>() {
                return Some(idx);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        base::{Accumulator, Comparison, MemoryCell, Operation},
        instructions::{Instruction, TargetType, Value},
        runtime::{builder::RuntimeBuilder, error_handling::RuntimeBuildError, parse_index},
    };

    use super::{MemoryConfig, RuntimeArgs};

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

    #[test]
    fn test_parse_index() {
        assert_eq!(parse_index(&vec!["[10]"]), Some(10));
    }

    /// Constructs a memory config for testing purposes
    fn test_memory_config(
        accumulators: Option<Vec<(usize, Option<i32>)>>,
        gamma_accumulator: Option<Option<i32>>,
        memory_cells: Option<Vec<(String, Option<i32>)>>,
        index_memory_cells: Option<Vec<(usize, Option<i32>)>>,
    ) -> MemoryConfig {
        let accumulators = match accumulators {
            Some(values) => {
                let mut accumulators = HashMap::new();
                for value in values {
                    accumulators.insert(
                        value.0,
                        Accumulator {
                            id: value.0,
                            data: value.1,
                        },
                    );
                }
                accumulators
            }
            None => HashMap::new(),
        };
        let memory_cells = match memory_cells {
            Some(values) => {
                let mut memory_cells = HashMap::new();
                for value in values {
                    memory_cells.insert(
                        value.0.clone(),
                        MemoryCell {
                            label: value.0,
                            data: value.1,
                        },
                    );
                }
                memory_cells
            }
            None => HashMap::new(),
        };
        let index_memory_cells = match index_memory_cells {
            Some(values) => {
                let mut index_memory_cells = HashMap::new();
                for value in values {
                    index_memory_cells.insert(value.0, value.1);
                }
                index_memory_cells
            }
            None => HashMap::new(),
        };
        MemoryConfig {
            accumulators,
            gamma_accumulator,
            memory_cells,
            index_memory_cells,
        }
    }

    #[test]
    fn test_memory_config_from_file_contents_accumulator() {
        let content = vec!["a0=5".to_string(), "a20=30".to_string()];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(Some(vec![(0, Some(5)), (20, Some(30))]), None, None, None);
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_accumulator_none() {
        let content = vec!["a0".to_string(), "a5".to_string()];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(Some(vec![(0, None), (5, None)]), None, None, None);
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_gamma_accumulator() {
        let content = vec!["y=5".to_string()];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(None, Some(Some(5)), None, None);
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_gamma_accumulator_last_value() {
        let content = vec!["y=5".to_string(), "y=8".to_string()];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(None, Some(Some(8)), None, None);
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_gamma_accumulator_no_value() {
        let content = vec!["y".to_string()];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(None, Some(None), None, None);
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_gamma_accumulator_not_enabled() {
        let content = Vec::new();
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(None, None, None, None);
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_memory_cell() {
        let content = vec!["h1=10".to_string(), "hello=121".to_string()];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(
            None,
            None,
            Some(vec![
                ("h1".to_string(), Some(10)),
                ("hello".to_string(), Some(121)),
            ]),
            None,
        );
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_memory_cell_none() {
        let content = vec!["h1".to_string()];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(None, None, Some(vec![("h1".to_string(), None)]), None);
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_index_memory_cell() {
        let content = vec!["[0]=10".to_string(), "[30]=214".to_string()];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should =
            test_memory_config(None, None, None, Some(vec![(0, Some(10)), (30, Some(214))]));
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_index_memory_cell_none() {
        let content = vec!["[0]".to_string(), "[30]".to_string()];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(None, None, None, Some(vec![(0, None), (30, None)]));
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_comments() {
        let content = vec![
            "# this is a comment".to_string(),
            "// this is also a comment".to_string(),
        ];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(None, None, None, None);
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents_empty_line() {
        let content = vec!["".to_string(), "".to_string()];
        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(None, None, None, None);
        assert_eq!(should, res);
    }

    #[test]
    fn test_memory_config_from_file_contents() {
        let content = r#"
            # left value is the memory type that should be assigned
            # if no value is assigned, the memory type is just enabled
        
            # accumulators
            a0=1
            a1
        
            # gamma accumulator
            # use this to set a value
            y=5
            # use this to just enable the accumulator
            y
        
            # memory cells
            h1=5
            h2=10
            h3
            h4
        
            # index memory cells
            [0]=1
            [1]=2
            [7] 
        "#
        .lines()
        .map(|f| f.trim().to_string())
        .collect::<Vec<String>>();

        let res = MemoryConfig::from_file_contents(&content, "test").unwrap();
        let should = test_memory_config(
            Some(vec![(0, Some(1)), (1, None)]),
            Some(None),
            Some(vec![
                ("h1".to_string(), Some(5)),
                ("h2".to_string(), Some(10)),
                ("h3".to_string(), None),
                ("h4".to_string(), None),
            ]),
            Some(vec![(0, Some(1)), (1, Some(2)), (7, None)]),
        );
        assert_eq!(should, res);
    }
}
