use std::collections::HashMap;

use crate::{
    base::{Accumulator, MemoryCell},
    instructions::{
        Instruction, InstructionParseError,
    }, cli::Args, utils::read_file,
};

/// Type that is used to build a new runtime environment.
///
/// This runtime can be configured to only allow a selected amount of accumulators and memory cells.
/// When a runtime is build from this builder compatibility checks are performed.
#[derive(Debug)]
pub struct RuntimeBuilder {
    runtime_args: Option<RuntimeArgs>,
    instructions: Option<Vec<Instruction>>,
    control_flow: ControlFlow,
    add_missing: bool,
}

impl RuntimeBuilder {

    /// Creates a new runtime builder with no values set.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            runtime_args: None,
            instructions: None,
            control_flow: ControlFlow::new(),
            add_missing: false,
        }
    }

    /// Creates a new runtime builder from the cli arguments.
    pub fn from_args(args: &Args) -> Result<Self, String> {
        Ok(Self {
            runtime_args: Some(RuntimeArgs::from_args(args)?),
            instructions: None,
            control_flow: ControlFlow::new(),
            add_missing: !args.disable_memory_detection,
        })
    }

    /// Creates a new runtime builder with default values.
    #[allow(dead_code)]
    pub fn new_debug(memory_cells: &[&'static str]) -> Self {
        Self {
            runtime_args: Some(RuntimeArgs::new_debug(memory_cells)),
            instructions: None,
            control_flow: ControlFlow::new(),
            add_missing: false,
        }
    }

    /// Constructs a new runtime.
    ///
    /// Performs some compatibility checks.
    /// Set `add_missing` to true to automatically add missing accumulators and memory cells.
    ///
    /// Returns `RuntimeBuildError` when the runtime could not be constructed due to missing information.
    pub fn build(&mut self) -> Result<Runtime, RuntimeBuildError> {
        if self.runtime_args.is_none() {
            return Err(RuntimeBuildError::RuntimeArgsMissing);
        }
        if self.instructions.is_none() {
            return Err(RuntimeBuildError::InstructionsMissing);
        }
        if self.instructions.as_ref().unwrap().is_empty() {
            return Err(RuntimeBuildError::InstructionsEmpty);
        }
        // Inject end labels to give option to end program by using goto END
        inject_end_labels(&mut self.control_flow, &self.instructions.as_ref().unwrap().len());
        if let Err(e) = self.check_labels() {
            return Err(RuntimeBuildError::LabelMissing(e));
        }
        if let Err(e) = self.check_accumulators(self.add_missing) {
            return Err(RuntimeBuildError::AccumulatorMissing(e));
        }
        if let Err(e) = self.check_memory_cells(self.add_missing) {
            return Err(RuntimeBuildError::MemoryCellMissing(e));
        }
        Ok(Runtime {
            runtime_args: self.runtime_args.clone().unwrap(),
            instructions: self.instructions.clone().unwrap(),
            control_flow: self.control_flow.clone(),
        })
    }

    /// Resets the current values to none.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.runtime_args = None;
        self.instructions = None;
        self.control_flow.reset();
    }

    #[allow(dead_code)]
    pub fn set_runtime_args(&mut self, runtime_args: RuntimeArgs) {
        self.runtime_args = Some(runtime_args);
    }

    /// Builds instructions from the vector.
    ///
    /// Each element is a single instruction.
    ///
    /// Control flow is reset and updated accordingly.
    ///
    /// If an instruction could not be parsed, an error is returned containing the reason.
    pub fn build_instructions(&mut self, instructions_input: &Vec<&str>) -> Result<(), String> {
        self.control_flow.reset();
        let mut instructions = Vec::new();
        for (index, instruction) in instructions_input.iter().enumerate() {
            // Remove comments
            let instruction = instruction.split(&['#', '/'][..]).collect::<Vec<&str>>()[0];
            // Check for labels
            let mut splits = instruction.split_whitespace().collect::<Vec<&str>>();
            if splits.is_empty() {
                // Line is empty / line contains comment, add dummy instruction
                instructions.push(Instruction::Sleep());
                continue;
            }
            if splits[0].ends_with(':') {
                let label = splits.remove(0).replace(':', "");
                self.control_flow.instruction_labels.insert(label, index);
            }
            match Instruction::try_from(&splits) {
                Ok(i) => instructions.push(i),
                Err(e) => return Err(error_handling(e, instruction, (index as u32) + 1)),
            }
        }
        self.instructions = Some(instructions);
        Ok(())
    }

    /// Sets the instructions to the provided instructions.
    ///
    /// If loops and labels are used, they have to be set manually by using [RuntimeBuilder::add_label](#add_label).
    #[allow(dead_code)]
    pub fn set_instructions(&mut self, instructions: Vec<Instruction>) {
        self.instructions = Some(instructions);
    }

    /// Adds label to instruction labels.
    ///
    /// Errors when **instruction_index** is out of bounds.
    ///
    /// Note: Make sure that you start counting at 0 and not 1!
    #[allow(dead_code)]
    pub fn add_label(
        &mut self,
        label: String,
        instruction_index: usize,
    ) -> Result<(), AddLabelError> {
        if self.instructions.is_none() {
            return Err(AddLabelError::InstructionsNotSet);
        }
        if self.instructions.as_ref().unwrap().len() <= instruction_index {
            Err(AddLabelError::IndexOutOfBounds)
        } else {
            self.control_flow
                .instruction_labels
                .insert(label, instruction_index);
            Ok(())
        }
    }

    /// Checks if all labels that are called in the instructions exist in the control flow.
    ///
    /// If label is missing the name of the label that is missing is returned.
    fn check_labels(&self) -> Result<(), String> {
        if self.instructions.is_none() {
            return Ok(());
        }
        for instruction in self.instructions.as_ref().unwrap() {
            match instruction {
                Instruction::Goto(label) => check_label(&self.control_flow, label)?,
                Instruction::GotoIfAccumulator(_, label, _, _) => {
                    check_label(&self.control_flow, label)?
                }
                Instruction::GotoIfConstant(_, label, _, _) => {
                    check_label(&self.control_flow, label)?
                }
                Instruction::GotoIfMemoryCell(_, label, _, _) => {
                    check_label(&self.control_flow, label)?
                }
                _ => (),
            };
        }
        Ok(())
    }

    /// Checks if all accumulators that are used in the instructions exist in the runtime args.
    ///
    /// If accumulator is missing, the id of the missing accumulator is returned.
    ///
    /// Panics if runtime_args is `None`.
    fn check_accumulators(&mut self, add_missing: bool) -> Result<(), String> {
        if self.instructions.is_none() {
            return Ok(());
        }
        for instruction in self.instructions.as_ref().unwrap() {
            match instruction {
                Instruction::AssignAccumulatorValue(id, _) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::AssignAccumulatorValueFromAccumulator(id_a, id_b) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_a, add_missing)?;
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_b, add_missing)?;
                }
                Instruction::AssignAccumulatorValueFromMemoryCell(id, _) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::AssignMemoryCellValueFromAccumulator(_, id) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::CalcAccumulatorWithAccumulator(_, id_a, id_b) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_a, add_missing)?;
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_b, add_missing)?;
                }
                Instruction::CalcAccumulatorWithAccumulators(_, id_a, id_b, id_c) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_a, add_missing)?;
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_b, add_missing)?;
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_c, add_missing)?;
                }
                Instruction::CalcAccumulatorWithConstant(_, id, _) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::CalcAccumulatorWithMemoryCell(_, id, _) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::CalcAccumulatorWithMemoryCells(_, id, _, _) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::CalcMemoryCellWithMemoryCellAccumulator(_, _, _, id) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id, add_missing)?
                }
                Instruction::GotoIfAccumulator(_, _, id_a, id_b) => {
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_a, add_missing)?;
                    check_accumulators(self.runtime_args.as_mut().unwrap(), id_b, add_missing)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    /// Checks if all memory cells that are used in the instructions exist in the runtime args.
    ///
    /// If memory cell is missing, the name of the missing memory cell is returned.
    ///
    /// Panics if runtime_args is `None`.
    fn check_memory_cells(&mut self, add_missing: bool) -> Result<(), String> {
        if self.instructions.is_none() {
            return Ok(());
        }
        for instruction in self.instructions.as_ref().unwrap() {
            match instruction {
                Instruction::AssignAccumulatorValueFromMemoryCell(_, name) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name, add_missing)?
                }
                Instruction::AssignMemoryCellValue(name, _) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name, add_missing)?
                }
                Instruction::AssignMemoryCellValueFromAccumulator(name, _) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name, add_missing)?
                }
                Instruction::AssignMemoryCellValueFromMemoryCell(name_a, name_b) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_a, add_missing)?;
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_b, add_missing)?;
                }
                Instruction::CalcAccumulatorWithMemoryCell(_, _, name) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name, add_missing)?
                }
                Instruction::CalcAccumulatorWithMemoryCells(_, _, name_a, name_b) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_a, add_missing)?;
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_b, add_missing)?;
                }
                Instruction::CalcMemoryCellWithMemoryCellConstant(_, name_a, name_b, _) => {
                    if !self
                        .runtime_args
                        .as_ref()
                        .unwrap()
                        .memory_cells
                        .contains_key(name_a.as_str())
                    {
                        return Err(name_a.clone());
                    }
                    if !self
                        .runtime_args
                        .as_ref()
                        .unwrap()
                        .memory_cells
                        .contains_key(name_b.as_str())
                    {
                        return Err(name_b.clone());
                    }
                }
                Instruction::CalcMemoryCellWithMemoryCellAccumulator(_, name_a, name_b, _) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_a, add_missing)?;
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_b, add_missing)?;
                }
                Instruction::CalcMemoryCellWithMemoryCells(_, name_a, name_b, name_c) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_a, add_missing)?;
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_b, add_missing)?;
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name_c, add_missing)?;
                }
                Instruction::GotoIfMemoryCell(_, _, _, name) => {
                    check_memory_cell(self.runtime_args.as_mut().unwrap(), name, add_missing)?
                }
                _ => (),
            }
        }
        Ok(())
    }
}

fn error_handling(e: InstructionParseError, instruction: &str, line: u32) -> String {
    let mut message = format!(
        "Error while building instruction (line {}):\n",
        line
    );
    message.push_str(instruction);
    message.push('\n');
    match e {
        InstructionParseError::UnknownOperation(idx, str) => {
            append_char_indicator(&mut message, idx);
            message.push_str(&format!("Unknown operation: {}", str));
        }
        InstructionParseError::UnknownComparison(idx, str) => {
            append_char_indicator(&mut message, idx);
            message.push_str(&format!("Unknown comparison: {}", str));
        }
        InstructionParseError::NotANumber(idx, str) => {
            append_char_indicator(&mut message, idx);
            message.push_str(&format!("Not a number: {}", str));
        }
        InstructionParseError::InvalidExpression(idx, str) => {
            append_char_indicator(&mut message, idx);
            message.push_str(&format!("Invalid expression: {}", str));
        }
        InstructionParseError::NoMatch => {
            message.push_str("^\n");
            message.push_str("No matching instruction found!");
        }
        InstructionParseError::NoMatchSuggestion(str) => {
            append_char_indicator(&mut message, 0);
            message.push_str(&format!(
                "No matching instruction found, did you mean: {} ?",
                str
            ));
        }
    }
    message
}

/// Prints a pointer at index.
fn append_char_indicator(str: &mut String, idx: usize) {
    for _i in 0..idx {
        str.push(' ');
    }
    str.push_str("^\n");
}

fn check_label(control_flow: &ControlFlow, label: &str) -> Result<(), String> {
    if !control_flow.instruction_labels.contains_key(label) {
        return Err(label.to_string());
    }
    Ok(())
}

/// Checks if accumulators with id exist.
/// 
/// If `add_missing` is set, the accumulator is added with empty value instead of returning an error.
fn check_accumulators(runtime_args: &mut RuntimeArgs, id: &usize, add_missing: bool) -> Result<(), String> {
    if !runtime_args.exists_accumulator(id) {
        if add_missing {
            runtime_args.accumulators.push(Accumulator::new(*id));
        } else {
            return Err(id.to_string());
        }
    }
    Ok(())
}

/// Checks if the memory cell with name exists.
/// 
/// If `add_missing` is set, the memory cell is added with empty value instead of returning an error.
fn check_memory_cell(runtime_args: &mut RuntimeArgs, name: &str, add_missing: bool) -> Result<(), String> {
    if !runtime_args.memory_cells.contains_key(name) {
        if add_missing {
            runtime_args.memory_cells.insert(name.to_string(), MemoryCell::new(name));
        } else {
            return Err(name.to_string());
        }
    }
    Ok(())
}

fn inject_end_labels(control_flow: &mut ControlFlow, last_instruction_index: &usize) {
    control_flow.instruction_labels.insert("END".to_string(), *last_instruction_index);
    control_flow.instruction_labels.insert("ENDE".to_string(), *last_instruction_index);
    control_flow.instruction_labels.insert("end".to_string(), *last_instruction_index);
    control_flow.instruction_labels.insert("ende".to_string(), *last_instruction_index);
}

/// Errors that can occur when a runtime is constructed from a RuntimeBuilder.
#[derive(Debug, PartialEq)]
pub enum RuntimeBuildError {
    RuntimeArgsMissing,
    InstructionsMissing,
    InstructionsEmpty,
    /// Indicates that a label is used in an instruction that does not exist in the control flow.
    /// This would lead to a runtime error.
    LabelMissing(String),
    MemoryCellMissing(String),
    AccumulatorMissing(String),
}

#[derive(Debug)]
pub enum AddLabelError {
    InstructionsNotSet,
    IndexOutOfBounds,
}

//TODO make fields private and add access functions, move into separate module
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

impl<'a> ControlFlow {
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
        runtime::RuntimeBuildError,
    };

    use super::{RuntimeArgs, RuntimeBuilder};

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

    #[test]
    fn test_instruction_building_with_comments() {
        let instructions = vec![
            "a0 := 4 // Set alpha to 4",
            "p(h1) := a0 # Set memory cell h1 to 4",
            "a0 := a1 # Just some stuff",
            "a1 := a2 // Just some more stuff",
        ];
        let mut rb = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
        assert!(rb.build_instructions(&instructions).is_ok());
    }
}
