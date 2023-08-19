use std::collections::HashMap;

use crate::{
    base::{Accumulator, MemoryCell},
    instructions::{
        print_accumulators, print_memory_cells, print_stack, Instruction, InstructionParseError,
    },
    ACCUMULATORS, MEMORY_CELL_LABELS,
};

/// Type that is used to build a new runtime environment.
///
/// This runtime can be configured to only allow a selected amount of accumulators and memory cells.
/// When a runtime is build from this builder compatibility checks are performed.
#[derive(Debug)]
pub struct RuntimeBuilder<'a> {
    runtime_args: Option<RuntimeArgs<'a>>,
    instructions: Option<Vec<Instruction>>,
    control_flow: ControlFlow,
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
            runtime_args: Some(RuntimeArgs::new_default()),
            instructions: None,
            control_flow: ControlFlow::new(),
        }
    }

    /// Constructs a new runtime.
    ///
    /// Performs some compatibility checks.
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
        if let Err(e) = self.check_labels() {
            return Err(RuntimeBuildError::LabelMissing(e));
        }
        if let Err(e) = self.check_accumulators() {
            return Err(RuntimeBuildError::AccumulatorMissing(e));
        }
        if let Err(e) = self.check_memory_cells() {
            return Err(RuntimeBuildError::MemoryCellMissing(e));
        }
        return Ok(Runtime {
            runtime_args: self.runtime_args.clone().unwrap(),
            instructions: self.instructions.clone().unwrap(),
            control_flow: self.control_flow.clone(),
        });
    }

    /// Resets the current values to none.
    pub fn reset(&mut self) {
        self.runtime_args = None;
        self.instructions = None;
        self.control_flow.reset();
    }

    pub fn set_runtime_args(&mut self, runtime_args: RuntimeArgs<'a>) {
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
                continue;
            }
            if splits[0].ends_with(":") {
                let label = splits.remove(0).replace(":", "");
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
    pub fn set_instructions(&mut self, instructions: Vec<Instruction>) {
        self.instructions = Some(instructions);
    }

    /// Adds label to instruction labels.
    ///
    /// Errors when **instruction_index** is out of bounds.
    ///
    /// Note: Make sure that you start counting at 0 and not 1!
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
    fn check_accumulators(&self) -> Result<(), String> {
        if self.instructions.is_none() {
            return Ok(());
        }
        for instruction in self.instructions.as_ref().unwrap() {
            match instruction {
                Instruction::AssignAccumulatorValue(id, _) => {
                    check_accumulators(&self.runtime_args.as_ref().unwrap(), id)?
                }
                Instruction::AssignAccumulatorValueFromAccumulator(id_a, id_b) => {
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id_a)?;
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id_b)?;
                }
                Instruction::AssignAccumulatorValueFromMemoryCell(id, _) => {
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id)?
                }
                Instruction::AssignMemoryCellValueFromAccumulator(_, id) => {
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id)?
                }
                Instruction::CalcAccumulatorWithAccumulator(_, id_a, id_b) => {
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id_a)?;
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id_b)?;
                }
                Instruction::CalcAccumulatorWithAccumulators(_, id_a, id_b, id_c) => {
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id_a)?;
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id_b)?;
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id_c)?;
                }
                Instruction::CalcAccumulatorWithConstant(_, id, _) => {
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id)?
                }
                Instruction::CalcAccumulatorWithMemoryCell(_, id, _) => {
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id)?
                }
                Instruction::CalcAccumulatorWithMemoryCells(_, id, _, _) => {
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id)?
                }
                Instruction::CalcMemoryCellWithMemoryCellAccumulator(_, _, _, id) => {
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id)?
                }
                Instruction::GotoIfAccumulator(_, _, id_a, id_b) => {
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id_a)?;
                    check_accumulators(self.runtime_args.as_ref().unwrap(), id_b)?;
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
    fn check_memory_cells(&self) -> Result<(), String> {
        if self.instructions.is_none() {
            return Ok(());
        }
        for instruction in self.instructions.as_ref().unwrap() {
            match instruction {
                Instruction::AssignAccumulatorValueFromMemoryCell(_, name) => {
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name)?
                }
                Instruction::AssignMemoryCellValue(name, _) => {
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name)?
                }
                Instruction::AssignMemoryCellValueFromAccumulator(name, _) => {
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name)?
                }
                Instruction::AssignMemoryCellValueFromMemoryCell(name_a, name_b) => {
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name_a)?;
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name_b)?;
                }
                Instruction::CalcAccumulatorWithMemoryCell(_, _, name) => {
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name)?
                }
                Instruction::CalcAccumulatorWithMemoryCells(_, _, name_a, name_b) => {
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name_a)?;
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name_b)?;
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
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name_a)?;
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name_b)?;
                }
                Instruction::CalcMemoryCellWithMemoryCells(_, name_a, name_b, name_c) => {
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name_a)?;
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name_b)?;
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name_c)?;
                }
                Instruction::GotoIfMemoryCell(_, _, _, name) => {
                    check_memory_cell(self.runtime_args.as_ref().unwrap(), name)?
                }
                _ => (),
            }
        }
        Ok(())
    }
}

fn error_handling(e: InstructionParseError, instruction: &str, line: u32) -> String {
    let mut message = String::from(format!(
        "Error while building instruction (line {}):\n",
        line
    ));
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

fn check_accumulators(runtime_args: &RuntimeArgs, id: &usize) -> Result<(), String> {
    if !runtime_args.exists_accumulator(id) {
        return Err(id.to_string());
    }
    Ok(())
}

fn check_memory_cell(runtime_args: &RuntimeArgs, name: &str) -> Result<(), String> {
    if !runtime_args.memory_cells.contains_key(&name) {
        return Err(name.to_string());
    }
    Ok(())
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
pub struct Runtime<'a> {
    runtime_args: RuntimeArgs<'a>,
    instructions: Vec<Instruction>,
    control_flow: ControlFlow,
}

impl<'a> Runtime<'a> {
    /// Runs the complete program.
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
            println!(
                "Unable to continue execution, an irrecoverable error occured: {}",
                e
            );
            return Err(format!("Execution terminated: {}", e));
        }
        Ok(())
    }

    /// Returns the index of the current instruction
    pub fn current_instruction_index(&self) -> usize {
        self.control_flow.next_instruction_index - 1
    }

    /// Prints information about current register values, current accumulator values and the current status of the stack.
    pub fn debug(&self) {
        print_accumulators(&self.runtime_args);
        print_memory_cells(&self.runtime_args);
        print_stack(&self.runtime_args);
    }

    /// Adds an instruction to the end of the instruction vector with a label mapping.
    pub fn add_instruction_with_label(&mut self, instruction: Instruction, label: String) {
        //TODO Move to runtime builder
        self.instructions.push(instruction);
        self.control_flow
            .instruction_labels
            .insert(label, self.instructions.len() - 1);
    }

    /// Returns reference to **runtime_args**.
    pub fn runtime_args(&self) -> &RuntimeArgs {
        &self.runtime_args
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeArgs<'a> {
    /// Current values stored in accumulators
    pub accumulators: Vec<Accumulator>,
    /// All registers that are used to store data
    pub memory_cells: HashMap<&'a str, MemoryCell>,
    /// The stack of the runner
    pub stack: Vec<i32>,
}

impl<'a> RuntimeArgs<'a> {
    /// Creates a new runtimes args struct with empty lists.
    pub fn new() -> Self {
        Self {
            accumulators: Vec::new(),
            memory_cells: HashMap::new(),
            stack: Vec::new(),
        }
    }

    pub fn new_default() -> Self {
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
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{Comparison, Operation},
        instructions::{self, Instruction},
        runtime::RuntimeBuildError,
    };

    use super::{RuntimeArgs, RuntimeBuilder};

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
        let mut rb = RuntimeBuilder::new_default();
        rb.set_instructions(instructions.clone());
        assert!(rb.add_label(label.to_string(), 0).is_ok());
        assert!(rb.build().is_ok());
        rb.reset();
        rb.set_instructions(instructions);
        rb.set_runtime_args(RuntimeArgs::new_default());
        assert_eq!(
            rb.build(),
            Err(RuntimeBuildError::LabelMissing(label.to_string()))
        );
    }

    #[test]
    fn test_accumulator_missing() {
        test_accumulator_instruction(
            Instruction::AssignAccumulatorValue(0, 1),
            vec![&(0 as usize)],
        );
        test_accumulator_instruction(
            Instruction::AssignAccumulatorValueFromAccumulator(0, 1),
            vec![&(0 as usize), &(1 as usize)],
        );
        test_accumulator_instruction(
            Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string()),
            vec![&(0 as usize)],
        );
        test_accumulator_instruction(
            Instruction::AssignAccumulatorValue(0, 1),
            vec![&(0 as usize)],
        );
        test_accumulator_instruction(
            Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 0),
            vec![&(0 as usize)],
        );
        test_accumulator_instruction(
            Instruction::CalcAccumulatorWithAccumulator(Operation::Plus, 0, 1),
            vec![&(0 as usize), &(1 as usize)],
        );
        test_accumulator_instruction(
            Instruction::CalcAccumulatorWithAccumulators(Operation::Plus, 0, 1, 2),
            vec![&(0 as usize), &(1 as usize), &(2 as usize)],
        );
        test_accumulator_instruction(
            Instruction::CalcAccumulatorWithConstant(Operation::Plus, 0, 0),
            vec![&(0 as usize)],
        );
        test_accumulator_instruction(
            Instruction::CalcAccumulatorWithMemoryCell(Operation::Plus, 0, "a".to_string()),
            vec![&(0 as usize)],
        );
        test_accumulator_instruction(
            Instruction::CalcAccumulatorWithMemoryCells(
                Operation::Plus,
                0,
                "a".to_string(),
                "b".to_string(),
            ),
            vec![&(0 as usize)],
        );
        test_accumulator_instruction(
            Instruction::CalcMemoryCellWithMemoryCellAccumulator(
                Operation::Plus,
                "a".to_string(),
                "b".to_string(),
                0,
            ),
            vec![&(0 as usize)],
        );
        test_accumulator_instruction(
            Instruction::GotoIfAccumulator(Comparison::Equal, "loop".to_string(), 0, 0),
            vec![&(0 as usize)],
        );
    }

    fn test_accumulator_instruction(instruction: Instruction, to_test: Vec<&usize>) {
        let mut rb = RuntimeBuilder::new();
        rb.set_instructions(vec![instruction]);
        _ = rb.add_label("loop".to_string(), 0);
        // Test if ok works
        let mut runtime_args = RuntimeArgs::new();
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
        for (i1, s) in to_test.iter().enumerate() {
            let mut runtime_args = RuntimeArgs::new();
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
        let mut runtime_args = RuntimeArgs::new();
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
            let mut runtime_args = RuntimeArgs::new();
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
        let mut rb = RuntimeBuilder::new_default();
        assert!(rb.build_instructions(&instructions).is_ok());
    }
}
