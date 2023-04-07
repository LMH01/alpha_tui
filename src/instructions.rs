use crate::{runtime::{RuntimeArgs, ControlFlow}, base::{Comparison, Operation}};

pub enum Instruction<'a> {
    /// See [push](fn.push.html)
    Push(),
    /// See [pop](fn.pop.html)
    Pop(),
    /// See [assign_accumulator_value](fn.assign_accumulator_value.html)
    AssignAccumulatorValue(usize, i32),
    /// See [assign_accumulator_value_from_accumulator](fn.assign_accumulator_value_from_accumulator.html)
    AssignAccumulatorValueFromAccumulator(usize, usize),
    /// See [assign_accumulator_value_from_memory_cell](fn.assign_accumulator_value_from_memory_cell.html)
    AssignAccumulatorValueFromMemoryCell(usize, &'a str),
    /// See [assign_memory_cell_value](fn.assign_memory_cell_value.html)
    AssignMemoryCellValue(&'a str, i32),
    /// See [assign_memory_cell_value_from_accumulator](fn.assign_memory_cell_value_from_accumulator.html)
    AssignMemoryCellValueFromAccumulator(&'a str, usize),
    /// See [assign_memory_cell_value_from_memory_cell](fn.assign_memory_cell_value_from_memory_cell.html)
    AssingMemoryCellValueFromMemoryCell(&'a str, &'a str),
    CalcAccumulatorWithConstant(Operation, usize, i32),
    CalcAccumulatorWithAccumulator(Operation, usize, usize),
    CalcAccumulatorWithAccumulators(Operation, usize, usize, usize),
    CalcAccumulatorWithMemoryCell(Operation, usize, &'a str),
    CalcAccumulatorWithMemoryCells(Operation, usize, &'a str, &'a str),
    CalcMemoryCellWithMemoryCellConstant(Operation, &'a str, &'a str, i32),
    CalcMemoryCellWithMemoryCellAccumulator(Operation, &'a str, &'a str, usize),
    /// See [ControlFlow](../runtime/struct.ControlFlow.html) and [goto](fn.goto.html) for further information.
    Goto(&'a str),
    /// See [goto_if_accumulator](fn.goto_if_accumulator.html)
    GotoIfAccumulator(Comparison, &'a str, usize, usize),
    /// See [goto_if_constant](fn.goto_if_constant.html)
    GotoIfConstant(Comparison, &'a str, usize, i32),
    /// See [goto_if_memory_cell](fn.goto_if_memory_cell.html)
    GotoIfMemoryCell(Comparison, &'a str, usize, &'a str),
    /// See [print_accumulators](fn.print_accumulators.html)
    PrintAccumulators(),
    /// See [print_memory_cells](fn.print_memory_cells.html)
    PrintMemoryCells(),
    /// See [print_stack](fn.print_stack.html)
    PrintStack(),
}

impl<'a> Instruction<'a> {
    /// Runs the instruction, retuns Err(String) when instruction could not be ran.
    /// Err contains the reason why running the instruction failed.
    pub fn run(&self, runtime_args: &mut RuntimeArgs<'a>, control_flow: &mut ControlFlow<'a>) -> Result<(), String> {
        match self {
            Self::Push() => push(runtime_args)?,
            Self::Pop() => pop(runtime_args)?,
            Self::AssignAccumulatorValue(a_idx, value) => assign_accumulator_value(runtime_args, a_idx, value)?,
            Self::AssignAccumulatorValueFromAccumulator(a_idx_a, a_idx_b) => assign_accumulator_value_from_accumulator(runtime_args, a_idx_a, a_idx_b)?,
            Self::AssignAccumulatorValueFromMemoryCell(a_idx, label) => assign_accumulator_value_from_memory_cell(runtime_args, a_idx, label)?,
            Self::AssignMemoryCellValue(label, value) => assign_memory_cell_value(runtime_args, label, value)?,
            Self::AssignMemoryCellValueFromAccumulator(label, a_idx) => assign_memory_cell_value_from_accumulator(runtime_args, label, a_idx)?,
            Self::AssingMemoryCellValueFromMemoryCell(label_a, label_b) => assign_memory_cell_value_from_memory_cell(runtime_args, label_a, label_b)?,
            Self::CalcAccumulatorWithConstant(operation, a_idx, value) => calc_accumulator_with_constant(runtime_args, operation, a_idx, value)?,
            Self::CalcAccumulatorWithAccumulator(operation, a_idx_a, a_idx_b) => calc_accumulator_with_accumulator(runtime_args, operation, a_idx_a, a_idx_b)?,
            Self::CalcAccumulatorWithAccumulators(operation, a_idx_a, a_idx_b, a_idx_c) => calc_accumulator_with_accumulators(runtime_args, operation, a_idx_a, a_idx_b, a_idx_c)?,
            Self::CalcAccumulatorWithMemoryCell(operation, a_idx, label) => calc_accumulator_with_memory_cell(runtime_args, operation, a_idx, label)?,
            Self::CalcAccumulatorWithMemoryCells(operation, a_idx, label_a, label_b) => calc_accumulator_with_memory_cells(runtime_args, operation, a_idx, label_a, label_b)?,
            Self::CalcMemoryCellWithMemoryCellAccumulator(operation, label_a, label_b, a_idx) => calc_memory_cell_with_memory_cell_accumulator(runtime_args, operation, label_a, label_b, a_idx)?,
            Self::CalcMemoryCellWithMemoryCellConstant(operation, label_a, label_b, value) => calc_memory_cell_with_memory_cell_constant(runtime_args, operation, label_a, label_b, value)?,
            Self::Goto(label) => goto(runtime_args, control_flow, label)?,
            Self::GotoIfAccumulator(comparison, label, a_idx_a, a_idx_b) => goto_if_accumulator(runtime_args, control_flow, comparison, label, a_idx_a, a_idx_b)?,
            Self::GotoIfConstant(comparison, label, a_idx, c) => goto_if_constant(runtime_args, control_flow, comparison, label, a_idx, c)?,
            Self::GotoIfMemoryCell(comparison, label, a_idx, mcl) => goto_if_memory_cell(runtime_args, control_flow, comparison, label, a_idx, mcl)?,
            Self::PrintAccumulators() => print_accumulators(runtime_args),
            Self::PrintMemoryCells() => print_memory_cells(runtime_args),
            Self::PrintStack() => print_stack(runtime_args),
        }
        Ok(())
    }
}

/// Pushes the value of alpha_0 onto the stack.
fn push(runtime_args: &mut RuntimeArgs) -> Result<(), String> {
    assert_accumulator_exists(runtime_args, &0)?;
    runtime_args.stack.push(runtime_args.accumulators[0].data.unwrap_or(0));
    Ok(())
}

/// Pops top value of the stack into alpha_0.
fn pop(runtime_args: &mut RuntimeArgs) -> Result<(), String> {
    assert_accumulator_contains_value(runtime_args, &0)?;
    runtime_args.accumulators[0].data = Some(runtime_args.stack.pop().unwrap_or(0));
    Ok(())
}

/// Assings **value** to accumulator with index **a_idx**.
/// 
/// Errors when accumulator does not exist.
fn assign_accumulator_value(runtime_args: &mut RuntimeArgs, a_idx: &usize, value: &i32) -> Result<(), String> {
    assert_accumulator_exists(runtime_args, a_idx)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(*value);
    Ok(())
}

/// Assigns value of accumulator with index **a_idx_b** to accumulator with index **a_idx_a**.
///
/// Errors when either accumulator does not exist or when **a_idx_b** does not contain a value.
fn assign_accumulator_value_from_accumulator(runtime_args: &mut RuntimeArgs, a_idx_a: &usize, a_idx_b: &usize) -> Result<(), String> {
    let src = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    assert_accumulator_exists(runtime_args, a_idx_a)?;
    runtime_args.accumulators.get_mut(*a_idx_a).unwrap().data = Some(src);
    Ok(())
}

/// Assigns value of memory cell with label **label** to accumulator with index **a_idx**.
///
/// Errors when accumulator does not exist or when memory cell does not contain a value.
fn assign_accumulator_value_from_memory_cell(runtime_args: &mut RuntimeArgs, a_idx: &usize, label: &str) -> Result<(), String> {
    assert_accumulator_exists(runtime_args, a_idx)?;
    let value = assert_memory_cell_contains_value(runtime_args, label)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(value);
    Ok(())
}

/// Assings **value** to memory cell with label **label**.
/// 
/// Errors when accumulator does not exist.
fn assign_memory_cell_value(runtime_args: &mut RuntimeArgs, label: &str, value: &i32) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label)?;
    runtime_args.memory_cells.get_mut(label).unwrap().data = Some(*value);
    Ok(())
}

/// Assings value of accumulator with index **a_idx** to memory cell with label **label**.
/// 
/// Errors when memory cell does not exist or when the accumulator does not contain a value.
fn assign_memory_cell_value_from_accumulator(runtime_args: &mut RuntimeArgs, label: &str, a_idx: &usize) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label)?;
    let value = assert_accumulator_contains_value(runtime_args, a_idx)?;
    runtime_args.memory_cells.get_mut(label).unwrap().data = Some(value);
    Ok(())
}
/// Assings value of accumulator with index **a_idx** to memory cell with label **label**.
/// 
/// Errors when memory cell does not exist or when the accumulator does not contain a value.
fn assign_memory_cell_value_from_memory_cell(runtime_args: &mut RuntimeArgs, label_a: &str, label_b: &str) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let value = assert_memory_cell_contains_value(runtime_args, label_b)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(value);
    Ok(())
}

fn calc_accumulator_with_constant(runtime_args: &mut RuntimeArgs, operation: &Operation, a_idx: &usize, value: &i32) -> Result<(), String> {
    let v = assert_accumulator_contains_value(runtime_args, a_idx)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(operation.calc(v, *value));
    Ok(())
}

fn calc_accumulator_with_accumulator(runtime_args: &mut RuntimeArgs, operation: &Operation, a_idx_a: &usize, a_idx_b: &usize) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx_a)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    runtime_args.accumulators.get_mut(*a_idx_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

fn calc_accumulator_with_accumulators(runtime_args: &mut RuntimeArgs, operation: &Operation, a_idx_a: &usize, a_idx_b: &usize, a_idx_c: &usize) -> Result<(), String> {
    assert_accumulator_exists(runtime_args, a_idx_a)?;
    let a = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx_c)?;
    runtime_args.accumulators.get_mut(*a_idx_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

fn calc_accumulator_with_memory_cell(runtime_args: &mut RuntimeArgs, operation: &Operation, a_idx: &usize, label: &str) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    let b = assert_memory_cell_contains_value(runtime_args, label)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

fn calc_accumulator_with_memory_cells(runtime_args: &mut RuntimeArgs, operation: &Operation, a_idx: &usize, label_a: &str, label_b: &str) -> Result<(), String> {
    assert_accumulator_exists(runtime_args, a_idx)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_a)?;
    let b = assert_memory_cell_contains_value(runtime_args, label_b)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

fn calc_memory_cell_with_memory_cell_constant(runtime_args: &mut RuntimeArgs, operation: &Operation, label_a: &str, label_b: &str, value: &i32) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_b)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(operation.calc(a, *value));
    Ok(())
}

fn calc_memory_cell_with_memory_cell_accumulator(runtime_args: &mut RuntimeArgs, operation: &Operation, label_a: &str, label_b: &str, a_idx: &usize) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_b)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Sets the next instruction index to index contained behind **label** in [instruction_labels](../runtime/struct.ControlFlow.html#structfield.instruction_labels) map.
fn goto(runtime_args: &mut RuntimeArgs, control_flow: &mut ControlFlow, label: &str) -> Result<(), String> {
    control_flow.next_instruction_index(label)?;
    Ok(())
}

/// Sets next instruction to instruction behind **label** when comparison between accumulator **a_idx_a** and accumulator **a_idx_b** succeeds.
/// ## Arguments
/// - 'comparison' - The way the two values should be compared
/// - 'label' - The label behind which the instruction is found that should be executed when comparison succeeds
/// - 'a_idx_a' - The index of accumulator a whichs value should be compared
/// - 'a_idx_b' - The index of accumulator b whichs value should be compared
fn goto_if_accumulator(runtime_args: &mut RuntimeArgs, control_flow: &mut ControlFlow, comparison: &Comparison, label: &str, a_idx_a: &usize, a_idx_b: &usize) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx_a)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    if comparison.cmp(a, b) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Sets next instruction to instruction behind **label** when comparison between accumulator **a_idx** and constant **c** succeeds.
/// ## Arguments
/// - 'comparison' - The way the two values should be compared
/// - 'label' - The label behind which the instruction is found that should be executed when comparison succeeds
/// - 'a_idx' - The index of accumulator a whichs value should be compared
/// - 'c' - Constant that should be compared with **a_idx_a**
fn goto_if_constant(runtime_args: &mut RuntimeArgs, control_flow: &mut ControlFlow, comparison: &Comparison, label: &str, a_idx: &usize, c: &i32) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    if comparison.cmp(a, *c) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Sets next instruction to instruction behind **label** when comparison between accumulator **a_idx** and memory cell with label **mcl** succeeds.
/// ## Arguments
/// - 'comparison' - The way the two values should be compared
/// - 'label' - The label behind which the instruction is found that should be executed when comparison succeeds
/// - 'a_idx' - The index of accumulator a whichs value should be compared
/// - 'mcl' - The label of the memory cell behind wich the value can be found that should be compared
fn goto_if_memory_cell(runtime_args: &mut RuntimeArgs, control_flow: &mut ControlFlow, comparison: &Comparison, label: &str, a_idx: &usize, mcl: &str) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    let b = assert_memory_cell_contains_value(runtime_args, mcl)?;
    if comparison.cmp(a, b) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Tests if the accumulator with **index** exists.
fn assert_accumulator_exists(runtime_args: &mut RuntimeArgs, index: &usize) -> Result<(), String> {
    if let Some(_value) = runtime_args.accumulators.get(*index) {
        Ok(())
    } else {
        Err(format!("Accumulator with index {} does not exist!", index))
    }
}

/// Tests if the accumulator with **index** exists and contains a value.
/// 
/// Ok(i32) contains the accumulator value.
/// 
/// Err(String) contains error message.
fn assert_accumulator_contains_value(runtime_args: &mut RuntimeArgs, index: &usize) -> Result<i32, String> {
    if let Some(value) = runtime_args.accumulators.get(*index) {
        if value.data.is_some() {
            Ok(runtime_args.accumulators.get(*index).unwrap().data.unwrap())
        } else {
            Err(format!("Accumulator with index {} does not contain data!", index))
        }
    } else {
        Err(format!("Accumulator with index {} does not exist!", index))
    }
}

/// Tests if the memory cell with **label** exists.
fn assert_memory_cell_exists(runtime_args: &mut RuntimeArgs, label: &str) -> Result<(), String> {
    if let Some(_value) = runtime_args.memory_cells.get(label) {
        Ok(())
    } else {
        Err(format!("Memory cell with label {} does not exist!", label))
    }
}

/// Tests if the memory cell with **label** exists and contains a value.
/// 
/// Ok(i32) contains the memory cell value.
/// 
/// Err(String) contains error message.
fn assert_memory_cell_contains_value(runtime_args: &mut RuntimeArgs, label: &str) -> Result<i32, String> {
    if let Some(value) = runtime_args.memory_cells.get(label) {
        if value.data.is_some() {
            Ok(runtime_args.memory_cells.get(label).unwrap().data.unwrap())
        } else {
            Err(format!("Memory cell with label {} does not contain data!", label))
        }
    } else {
        Err(format!("Memory cell with label {} does not exist!", label))
    }
}

/// Prints the current contents of the accumulators into the console
fn print_accumulators(runtime_args: &RuntimeArgs) {
    println!("--- Accumulators ---");
    for (index, i) in runtime_args.accumulators.iter().enumerate() {
        println!("{} - {:?}", index, i.data);
    }
    println!("--------------------");
}

/// Prints the current contents of the memory cells into the console
fn print_memory_cells(runtime_args: &RuntimeArgs) {
    // TODO Make print sorted (Alpabetically by label name)
    println!("--- Memory Cells ---");
    for (k, v) in &runtime_args.memory_cells {
        println!("{} - {:?}", k, v.data);
    }
    println!("--------------------");
}

/// Prints the current layout of the stack into the console
fn print_stack(runtime_args: &RuntimeArgs) {
    println!("------ Stack -------");
    for (index, i) in runtime_args.stack.iter().enumerate() {
        println!("{} - {:?}", index, i);
    }
    println!("--------------------");
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{runtime::{ControlFlow, RuntimeArgs}, instructions::Instruction, base::{Accumulator, MemoryCell, Comparison, Operation}};

    
    #[test]
    fn test_stack() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 5).run(&mut args, &mut control_flow).unwrap();
        Instruction::Push().run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValue(0, 10).run(&mut args, &mut control_flow).unwrap();
        Instruction::Push().run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.stack, vec![5, 10]);
        Instruction::Pop().run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
        Instruction::Pop().run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 5);
        assert_eq!(args.stack.len(), 0);
    }

    #[test]
    fn test_assign_accumulator_value_from_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 5).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValue(1, 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValue(2, 12).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValueFromAccumulator(1, 2).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValueFromAccumulator(0, 1).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.accumulators[1].data.unwrap(), 12);
        assert_eq!(args.accumulators[2].data.unwrap(), 12);
    }

    #[test]
    fn test_assign_accumulator_value_from_accumulator_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::AssignAccumulatorValueFromAccumulator(0, 1).run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::AssignAccumulatorValueFromAccumulator(1, 0).run(&mut args, &mut control_flow).is_err());
    }

    #[test]
    fn test_assign_accumulator_value_from_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignMemoryCellValue("a", 10).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValueFromMemoryCell(0, "a").run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
    }
    
    #[test]
    fn test_assign_accumulator_value_from_memory_cell_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.accumulators = vec![Accumulator::new(0)];
        let err = Instruction::AssignAccumulatorValueFromMemoryCell(0, "a").run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Memory cell"));
        args.memory_cells.insert("a", MemoryCell::new("a"));
        let err = Instruction::AssignAccumulatorValueFromMemoryCell(1, "a").run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Accumulator"));
    }
    
    #[test]
    fn test_assign_memory_cell_value() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignMemoryCellValue("a", 2).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignMemoryCellValue("b", 20).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 2);
        assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_assign_memory_cell_value_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::AssignMemoryCellValue("c", 10).run(&mut args, &mut control_flow).is_err());
    }

    #[test]
    fn test_assign_memory_cell_value_from_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignMemoryCellValueFromAccumulator("a", 0).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_assign_memory_cell_value_from_accumulator_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.accumulators = vec![Accumulator::new(0)];
        let err = Instruction::AssignMemoryCellValueFromAccumulator("a", 0).run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Memory cell"));
        args.memory_cells.insert("a", MemoryCell::new("a"));
        let err = Instruction::AssignMemoryCellValueFromAccumulator("a", 1).run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Accumulator"));
    }

    #[test]
    fn test_assign_memory_cell_value_from_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignMemoryCellValue("a", 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssingMemoryCellValueFromMemoryCell("b", "a").run(&mut args, &mut control_flow).unwrap();
        assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_assign_memory_cell_value_from_memory_cell_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("a", "b").run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("b", "a").run(&mut args, &mut control_flow).is_err());
        args.memory_cells.insert("a", MemoryCell::new("a"));
        args.memory_cells.insert("b", MemoryCell::new("b"));
        args.memory_cells.get_mut("b").unwrap().data = Some(10);
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("b", "a").run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::AssingMemoryCellValueFromMemoryCell("a", "b").run(&mut args, &mut control_flow).is_ok());
    }

    #[test]
    fn test_calc_accumulator_with_constant() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, control_flow).unwrap();
        Instruction::CalcAccumulatorWithConstant(Operation::Plus, 0, 20).run(&mut args, control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 40);
    }

    #[test]
    fn test_calc_accumulator_with_accumulator() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, control_flow).unwrap();
        Instruction::AssignAccumulatorValue(1, 20).run(&mut args, control_flow).unwrap();
        Instruction::CalcAccumulatorWithAccumulator(Operation::Plus, 0, 1).run(&mut args, control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 40);
    }

    #[test]
    fn test_calc_accumulator_with_accumulators() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignAccumulatorValue(1, 20).run(&mut args, control_flow).unwrap();
        Instruction::AssignAccumulatorValue(2, 20).run(&mut args, control_flow).unwrap();
        Instruction::CalcAccumulatorWithAccumulators(Operation::Plus, 0, 1, 2).run(&mut args, control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 40);
    }

    #[test]
    fn test_calc_accumulator_with_memory_cell() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, control_flow).unwrap();
        Instruction::AssignMemoryCellValue("a", 20).run(&mut args, control_flow).unwrap();
        Instruction::CalcAccumulatorWithMemoryCell(Operation::Plus, 0, "a").run(&mut args, control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 40);
    }

    #[test]
    fn test_calc_accumulator_with_memory_cells() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignMemoryCellValue("a", 10).run(&mut args, control_flow).unwrap();
        Instruction::AssignMemoryCellValue("b", 10).run(&mut args, control_flow).unwrap();
        Instruction::CalcAccumulatorWithMemoryCells(Operation::Plus, 0, "a", "b").run(&mut args, control_flow).unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 20);
    }

    #[test]
    fn test_calc_memory_cell_with_memory_cell_constant() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignMemoryCellValue("b", 10).run(&mut args, control_flow).unwrap();
        Instruction::CalcMemoryCellWithMemoryCellConstant(Operation::Plus, "a", "b", 10).run(&mut args, control_flow).unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_calc_memory_cell_with_memory_cell_accumulator() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, control_flow).unwrap();
        Instruction::AssignMemoryCellValue("b", 10).run(&mut args, control_flow).unwrap();
        Instruction::CalcMemoryCellWithMemoryCellAccumulator(Operation::Plus, "a", "b", 0).run(&mut args, control_flow).unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 30);
    }

    #[test]
    fn test_goto() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow.instruction_labels.insert("loop", 5);
        Instruction::Goto("loop").run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 5);
    }

    #[test]
    fn test_goto_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::Goto("loop").run(&mut args, &mut control_flow).is_err());
    }

    #[test]
    fn test_goto_if_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow.instruction_labels.insert("loop", 20);
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignAccumulatorValue(1, 30).run(&mut args, &mut control_flow).unwrap();
        Instruction::GotoIfAccumulator(Comparison::Less, "loop", 0, 1).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 20);
        control_flow.next_instruction_index = 0;
        Instruction::GotoIfAccumulator(Comparison::Equal, "loop", 0, 1).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 0);
        assert!(Instruction::GotoIfAccumulator(Comparison::Less, "none", 0, 1).run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::GotoIfAccumulator(Comparison::Equal, "none", 0, 1).run(&mut args, &mut control_flow).is_ok());
    }

    #[test]
    fn test_goto_if_constant() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow.instruction_labels.insert("loop", 20);
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::GotoIfConstant(Comparison::Less, "loop", 0, 40).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 20);
        control_flow.next_instruction_index = 0;
        Instruction::GotoIfConstant(Comparison::Equal, "loop", 0, 40).run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 0);
        assert!(Instruction::GotoIfConstant(Comparison::Less, "none", 0, 40).run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::GotoIfConstant(Comparison::Equal, "none", 0, 40).run(&mut args, &mut control_flow).is_ok());
    }

    #[test]
    fn test_goto_if_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow.instruction_labels.insert("loop", 20);
        Instruction::AssignAccumulatorValue(0, 20).run(&mut args, &mut control_flow).unwrap();
        Instruction::AssignMemoryCellValue("a", 50).run(&mut args, &mut control_flow).unwrap();
        Instruction::GotoIfMemoryCell(Comparison::Less, "loop", 0, "a").run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 20);
        control_flow.next_instruction_index = 0;
        Instruction::GotoIfMemoryCell(Comparison::Equal, "loop", 0, "a").run(&mut args, &mut control_flow).unwrap();
        assert_eq!(control_flow.next_instruction_index, 0);
        assert!(Instruction::GotoIfMemoryCell(Comparison::Less, "none", 0, "a").run(&mut args, &mut control_flow).is_err());
        assert!(Instruction::GotoIfMemoryCell(Comparison::Equal, "none", 0, "a").run(&mut args, &mut control_flow).is_ok());
    }

    /// Sets up runtime args in a conistent way because the default implementation for memory cells and accumulators is configgurable.
    fn setup_runtime_args() -> RuntimeArgs<'static> {
        let mut args = RuntimeArgs::new();
        args.memory_cells = HashMap::new();
        args.memory_cells.insert("a", MemoryCell::new("a"));
        args.memory_cells.insert("b", MemoryCell::new("b"));
        args.accumulators = vec![Accumulator::new(0), Accumulator::new(1), Accumulator::new(2)];
        args
    }

    /// Sets up runtime args where no memory cells or accumulators are set.
    fn setup_empty_runtime_args() -> RuntimeArgs<'static> {
        let mut args = RuntimeArgs::new();
        args.accumulators = Vec::new();
        args.memory_cells = HashMap::new();
        args
    }
}