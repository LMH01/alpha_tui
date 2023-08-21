use miette::Result;

use crate::{
    base::{Comparison, Operation},
    runtime::{error_handling::RuntimeErrorType, ControlFlow, RuntimeArgs},
};

/// Functions and structs related to error handling
pub mod error_handling;
/// Functions related to instruction parsing
mod parsing;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    /// push
    ///
    /// See [push](fn.push.html)
    Push(),
    /// pop
    ///
    /// See [pop](fn.pop.html)
    Pop(),
    /// a := x
    ///
    /// See [assign_accumulator_value](fn.assign_accumulator_value.html)
    AssignAccumulatorValue(usize, i32),
    /// a := b
    ///
    /// See [assign_accumulator_value_from_accumulator](fn.assign_accumulator_value_from_accumulator.html)
    AssignAccumulatorValueFromAccumulator(usize, usize),
    /// a := p(i)
    ///
    /// See [assign_accumulator_value_from_memory_cell](fn.assign_accumulator_value_from_memory_cell.html)
    AssignAccumulatorValueFromMemoryCell(usize, String),
    /// p(i) := x
    ///
    /// See [assign_memory_cell_value](fn.assign_memory_cell_value.html)
    AssignMemoryCellValue(String, i32),
    /// p(i) := a
    ///
    /// See [assign_memory_cell_value_from_accumulator](fn.assign_memory_cell_value_from_accumulator.html)
    AssignMemoryCellValueFromAccumulator(String, usize),
    /// p(i) := p(j)
    ///
    /// See [assign_memory_cell_value_from_memory_cell](fn.assign_memory_cell_value_from_memory_cell.html)
    AssignMemoryCellValueFromMemoryCell(String, String),
    /// a := a op x
    ///
    /// See [calc_accumulator_with_constant](fn.calc_accumulator_with_constant.html)
    CalcAccumulatorWithConstant(Operation, usize, i32),
    /// a := a op b
    ///
    /// See [calc_accumulator_with_constant](fn.calc_accumulator_with_constant.html)
    CalcAccumulatorWithAccumulator(Operation, usize, usize),
    /// a := b op c
    ///
    /// See [calc_accumulator_with_accumulators](fn.calc_accumulator_with_accumulators.html)
    CalcAccumulatorWithAccumulators(Operation, usize, usize, usize),
    /// a := a op p(i)
    ///
    /// See [calc_accumulator_with_memory_cell](fn.calc_accumulator_with_memory_cell.html)
    CalcAccumulatorWithMemoryCell(Operation, usize, String),
    /// a := p(i) op p(j)
    ///
    /// See [calc_accumulator_with_memory_cells](fn.calc_accumulator_with_memory_cells.html)
    CalcAccumulatorWithMemoryCells(Operation, usize, String, String),
    /// p(i) := p(j) op x
    ///
    /// See [calc_memory_cell_with_memory_cell_constant](fn.calc_memory_cell_with_memory_cell_constant.html)
    CalcMemoryCellWithMemoryCellConstant(Operation, String, String, i32),
    /// p(i) := p(j) op a
    ///
    /// See [calc_memory_cell_with_memory_cell_accumulator](fn.calc_memory_cell_with_memory_cell_accumulator.html)
    CalcMemoryCellWithMemoryCellAccumulator(Operation, String, String, usize),
    /// p(i) := p(j) op p(k)
    ///
    /// See [calc_memory_cell_with_memory_cells](fn.calc_memory_cell_with_memory_cells.html)
    CalcMemoryCellWithMemoryCells(Operation, String, String, String),
    /// goto label
    ///
    /// See [ControlFlow](../runtime/struct.ControlFlow.html) and [goto](fn.goto.html) for further information.
    Goto(String),
    /// if a cmp b then goto label
    ///
    /// See [goto_if_accumulator](fn.goto_if_accumulator.html)
    GotoIfAccumulator(Comparison, String, usize, usize),
    /// if a cmp x then goto label
    ///
    /// See [goto_if_constant](fn.goto_if_constant.html)
    GotoIfConstant(Comparison, String, usize, i32),
    /// if a cmp p(i) then goto label
    ///
    /// See [goto_if_memory_cell](fn.goto_if_memory_cell.html)
    GotoIfMemoryCell(Comparison, String, usize, String),
    /// This instruction does nothing.
    Sleep(),
}

impl Instruction {
    /// Runs the instruction, retuns Err(String) when instruction could not be ran.
    /// Err contains the reason why running the instruction failed.
    pub fn run(
        &self,
        runtime_args: &mut RuntimeArgs,
        control_flow: &mut ControlFlow,
    ) -> Result<(), RuntimeErrorType> {
        match self {
            Self::Push() => push(runtime_args)?,
            Self::Pop() => pop(runtime_args)?,
            Self::AssignAccumulatorValue(a_idx, value) => {
                assign_accumulator_value(runtime_args, a_idx, value)?
            }
            Self::AssignAccumulatorValueFromAccumulator(a_idx_a, a_idx_b) => {
                assign_accumulator_value_from_accumulator(runtime_args, a_idx_a, a_idx_b)?
            }
            Self::AssignAccumulatorValueFromMemoryCell(a_idx, label) => {
                assign_accumulator_value_from_memory_cell(runtime_args, a_idx, label)?
            }
            Self::AssignMemoryCellValue(label, value) => {
                assign_memory_cell_value(runtime_args, label, value)?
            }
            Self::AssignMemoryCellValueFromAccumulator(label, a_idx) => {
                assign_memory_cell_value_from_accumulator(runtime_args, label, a_idx)?
            }
            Self::AssignMemoryCellValueFromMemoryCell(label_a, label_b) => {
                assign_memory_cell_value_from_memory_cell(runtime_args, label_a, label_b)?
            }
            Self::CalcAccumulatorWithConstant(operation, a_idx, value) => {
                calc_accumulator_with_constant(runtime_args, operation, a_idx, value)?
            }
            Self::CalcAccumulatorWithAccumulator(operation, a_idx_a, a_idx_b) => {
                calc_accumulator_with_accumulator(runtime_args, operation, a_idx_a, a_idx_b)?
            }
            Self::CalcAccumulatorWithAccumulators(operation, a_idx_a, a_idx_b, a_idx_c) => {
                calc_accumulator_with_accumulators(
                    runtime_args,
                    operation,
                    a_idx_a,
                    a_idx_b,
                    a_idx_c,
                )?
            }
            Self::CalcAccumulatorWithMemoryCell(operation, a_idx, label) => {
                calc_accumulator_with_memory_cell(runtime_args, operation, a_idx, label)?
            }
            Self::CalcAccumulatorWithMemoryCells(operation, a_idx, label_a, label_b) => {
                calc_accumulator_with_memory_cells(
                    runtime_args,
                    operation,
                    a_idx,
                    label_a,
                    label_b,
                )?
            }
            Self::CalcMemoryCellWithMemoryCellAccumulator(operation, label_a, label_b, a_idx) => {
                calc_memory_cell_with_memory_cell_accumulator(
                    runtime_args,
                    operation,
                    label_a,
                    label_b,
                    a_idx,
                )?
            }
            Self::CalcMemoryCellWithMemoryCellConstant(operation, label_a, label_b, value) => {
                calc_memory_cell_with_memory_cell_constant(
                    runtime_args,
                    operation,
                    label_a,
                    label_b,
                    value,
                )?
            }
            Self::CalcMemoryCellWithMemoryCells(operation, label_a, label_b, label_c) => {
                calc_memory_cell_with_memory_cells(
                    runtime_args,
                    operation,
                    label_a,
                    label_b,
                    label_c,
                )?
            }
            Self::Goto(label) => goto(control_flow, label)?,
            Self::GotoIfAccumulator(comparison, label, a_idx_a, a_idx_b) => goto_if_accumulator(
                runtime_args,
                control_flow,
                comparison,
                label,
                a_idx_a,
                a_idx_b,
            )?,
            Self::GotoIfConstant(comparison, label, a_idx, c) => {
                goto_if_constant(runtime_args, control_flow, comparison, label, a_idx, c)?
            }
            Self::GotoIfMemoryCell(comparison, label, a_idx, mcl) => {
                goto_if_memory_cell(runtime_args, control_flow, comparison, label, a_idx, mcl)?
            }
            Self::Sleep() => (),
        }
        Ok(())
    }
}

/// Runs code equal to **push**
///
/// Causes runtime error if accumulator does not contain data.
fn push(runtime_args: &mut RuntimeArgs) -> Result<(), RuntimeErrorType> {
    assert_accumulator_exists(runtime_args, &0)?;
    match runtime_args.accumulators[0].data {
        Some(d) => runtime_args.stack.push(d),
        None => return Err(RuntimeErrorType::PushFail),
    }
    Ok(())
}

/// Runs code equal to **pop**
///
/// Causes runtime error if stack does not contain data.
fn pop(runtime_args: &mut RuntimeArgs) -> Result<(), RuntimeErrorType> {
    assert_accumulator_exists(runtime_args, &0)?;
    match runtime_args.stack.pop() {
        Some(d) => runtime_args.accumulators[0].data = Some(d),
        None => return Err(RuntimeErrorType::PopFail),
    }
    Ok(())
}

/// Runs code equal to **a := x**
///
/// - a = value of accumulator with index **a_idx**
/// - x = constant with value **value**
fn assign_accumulator_value(
    runtime_args: &mut RuntimeArgs,
    a_idx: &usize,
    value: &i32,
) -> Result<(), RuntimeErrorType> {
    assert_accumulator_exists(runtime_args, a_idx)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(*value);
    Ok(())
}

/// Runs code equal to **a := b**
///
/// - a = value of accumulator with index **a_idx_a**
/// - b = value of accumulator with index **a_idx_b**
fn assign_accumulator_value_from_accumulator(
    runtime_args: &mut RuntimeArgs,
    a_idx_a: &usize,
    a_idx_b: &usize,
) -> Result<(), RuntimeErrorType> {
    let src = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    assert_accumulator_exists(runtime_args, a_idx_a)?;
    runtime_args.accumulators.get_mut(*a_idx_a).unwrap().data = Some(src);
    Ok(())
}

/// Runs code equal to **a := p(i)**
///
/// - a = value of accumulator with index **a_idx**
/// - p(i) = value of memory cell with label **label**
fn assign_accumulator_value_from_memory_cell(
    runtime_args: &mut RuntimeArgs,
    a_idx: &usize,
    label: &str,
) -> Result<(), RuntimeErrorType> {
    assert_accumulator_exists(runtime_args, a_idx)?;
    let value = assert_memory_cell_contains_value(runtime_args, label)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(value);
    Ok(())
}

/// Runs code equal to **p(i) := x**
///
/// - p(i) = value of memory cell with label **label**
/// - x = constant with value **value**
fn assign_memory_cell_value(
    runtime_args: &mut RuntimeArgs,
    label: &str,
    value: &i32,
) -> Result<(), RuntimeErrorType> {
    assert_memory_cell_exists(runtime_args, label)?;
    runtime_args.memory_cells.get_mut(label).unwrap().data = Some(*value);
    Ok(())
}

/// Runs code equal to **p(i) := a**
///
/// - p(i) = value of memory cell with label **label**
/// - a = value of accumulator with index **a_idx**
fn assign_memory_cell_value_from_accumulator(
    runtime_args: &mut RuntimeArgs,
    label: &str,
    a_idx: &usize,
) -> Result<(), RuntimeErrorType> {
    assert_memory_cell_exists(runtime_args, label)?;
    let value = assert_accumulator_contains_value(runtime_args, a_idx)?;
    runtime_args.memory_cells.get_mut(label).unwrap().data = Some(value);
    Ok(())
}

/// Runs code equal to **p(i) := p(j)**
///
/// - p(i) = value of memory cell with label **label_a**
/// - p(j) = value of memory cell with label **label_b**
fn assign_memory_cell_value_from_memory_cell(
    runtime_args: &mut RuntimeArgs,
    label_a: &str,
    label_b: &str,
) -> Result<(), RuntimeErrorType> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let value = assert_memory_cell_contains_value(runtime_args, label_b)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(value);
    Ok(())
}

/// Runs code equal to **a := a op x**
///
/// - a = value of accumulator with index **a_idx**
/// - x = constant with value **value**
/// - op = the operation to perform
fn calc_accumulator_with_constant(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    a_idx: &usize,
    value: &i32,
) -> Result<(), RuntimeErrorType> {
    let v = assert_accumulator_contains_value(runtime_args, a_idx)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(operation.calc(v, *value));
    Ok(())
}

/// Runs code equal to **a := a op b**
///
/// - a = accumulator with index **a_idx_a**
/// - b = accumulator with index **a_idx_b**
/// - op = the operation to perform
fn calc_accumulator_with_accumulator(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    a_idx_a: &usize,
    a_idx_b: &usize,
) -> Result<(), RuntimeErrorType> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx_a)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    runtime_args.accumulators.get_mut(*a_idx_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **a := b op c**
///
/// - a = value of accumulator with index **a_idx_a**
/// - b = value of accumulator with index **a_idx_b**
/// - c = value of accumulator with index **a_idx_c**
/// - op = the operation to perform
fn calc_accumulator_with_accumulators(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    a_idx_a: &usize,
    a_idx_b: &usize,
    a_idx_c: &usize,
) -> Result<(), RuntimeErrorType> {
    assert_accumulator_exists(runtime_args, a_idx_a)?;
    let a = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx_c)?;
    runtime_args.accumulators.get_mut(*a_idx_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **a := a op p(i)**
///
/// - a = value of accumulator with index **a_idx**
/// - p(i) = value of memory cell with label **label**
/// - op = the operation to perform
fn calc_accumulator_with_memory_cell(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    a_idx: &usize,
    label: &str,
) -> Result<(), RuntimeErrorType> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    let b = assert_memory_cell_contains_value(runtime_args, label)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **a := p(i) op p(j)**
///
/// - a = value of accumulator with index **a_idx**
/// - p(i) = value of memory cell with label **label_a**
/// - p(j) = value of memory cell with label **label_b**
/// - op = the operation to perform
fn calc_accumulator_with_memory_cells(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    a_idx: &usize,
    label_a: &str,
    label_b: &str,
) -> Result<(), RuntimeErrorType> {
    assert_accumulator_exists(runtime_args, a_idx)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_a)?;
    let b = assert_memory_cell_contains_value(runtime_args, label_b)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **p(i) := p(j) op x**
///
/// - p(i) = value of memory cell with label **label_a**
/// - p(j) = value of memory cell with label **label_b**
/// - x = constant with value **value**
/// - op = the operation to perform
fn calc_memory_cell_with_memory_cell_constant(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    label_a: &str,
    label_b: &str,
    value: &i32,
) -> Result<(), RuntimeErrorType> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_b)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(operation.calc(a, *value));
    Ok(())
}

/// Runs code equal to **p(i) := p(j) op a**
///
/// - p(i) = value of memory cell with label **label_a**
/// - p(j) = value of memory cell with label **label_b**
/// - a = value of accumulator with index **a_idx**
/// - op = the operation to perform
fn calc_memory_cell_with_memory_cell_accumulator(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    label_a: &str,
    label_b: &str,
    a_idx: &usize,
) -> Result<(), RuntimeErrorType> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_b)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **p(i) := p(j) op p(k)**
///
/// - p(i) = value of memory cell with label **label_a**
/// - p(j) = value of memory cell with label **label_b**
/// - p(k) = value of memory cell with label **label_c**
/// - op = the operation to perform
fn calc_memory_cell_with_memory_cells(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    label_a: &str,
    label_b: &str,
    label_c: &str,
) -> Result<(), RuntimeErrorType> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_b)?;
    let b = assert_memory_cell_contains_value(runtime_args, label_c)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **goto label**
///
/// - label = label to which to jump
///
/// Sets the next instruction index to index contained behind **label** in [instruction_labels](../runtime/struct.ControlFlow.html#structfield.instruction_labels) map.
fn goto(control_flow: &mut ControlFlow, label: &str) -> Result<(), RuntimeErrorType> {
    control_flow.next_instruction_index(label)?;
    Ok(())
}

/// Runs code equal to **if a cmp b then goto label**
/// - a = value of accumulator with index **a_idx_a**
/// - b = value of accumulator with index **a_idx_b**
/// - label = label to which to jump
/// - cmp = the way how **a** and **b** should be compared
fn goto_if_accumulator(
    runtime_args: &mut RuntimeArgs,
    control_flow: &mut ControlFlow,
    comparison: &Comparison,
    label: &str,
    a_idx_a: &usize,
    a_idx_b: &usize,
) -> Result<(), RuntimeErrorType> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx_a)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    if comparison.cmp(a, b) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Runs code equal to **if a cmp x then goto label**
/// - a = value of accumulator with index **a_idx**
/// - x = constant with value **value**
/// - label = label to which to jump
/// - cmp = the way how **a** and **x** should be compared
fn goto_if_constant(
    runtime_args: &mut RuntimeArgs,
    control_flow: &mut ControlFlow,
    comparison: &Comparison,
    label: &str,
    a_idx: &usize,
    c: &i32,
) -> Result<(), RuntimeErrorType> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    if comparison.cmp(a, *c) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Runs code equal to **if a cmp p(i) then goto label**
/// - a = value of accumulator with index **a_idx**
/// - p(i) = value of memory cell with label **mcl**
/// - label = label to which to jump
/// - cmp = the way how **a** and **x** should be compared
fn goto_if_memory_cell(
    runtime_args: &mut RuntimeArgs,
    control_flow: &mut ControlFlow,
    comparison: &Comparison,
    label: &str,
    a_idx: &usize,
    mcl: &str,
) -> Result<(), RuntimeErrorType> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    let b = assert_memory_cell_contains_value(runtime_args, mcl)?;
    if comparison.cmp(a, b) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Tests if the accumulator with **index** exists.
fn assert_accumulator_exists(
    runtime_args: &mut RuntimeArgs,
    index: &usize,
) -> Result<(), RuntimeErrorType> {
    if let Some(_value) = runtime_args.accumulators.get(*index) {
        Ok(())
    } else {
        Err(RuntimeErrorType::AccumulatorDoesNotExist(*index))
    }
}

/// Tests if the accumulator with **index** exists and contains a value.
///
/// Ok(i32) contains the accumulator value.
///
/// Err(String) contains error message.
fn assert_accumulator_contains_value(
    runtime_args: &mut RuntimeArgs,
    index: &usize,
) -> Result<i32, RuntimeErrorType> {
    if let Some(value) = runtime_args.accumulators.get(*index) {
        if value.data.is_some() {
            Ok(runtime_args.accumulators.get(*index).unwrap().data.unwrap())
        } else {
            Err(RuntimeErrorType::AccumulatorUninitialized(*index))
        }
    } else {
        Err(RuntimeErrorType::AccumulatorDoesNotExist(*index))
    }
}

/// Tests if the memory cell with **label** exists.
fn assert_memory_cell_exists(
    runtime_args: &mut RuntimeArgs,
    label: &str,
) -> Result<(), RuntimeErrorType> {
    if let Some(_value) = runtime_args.memory_cells.get(label) {
        Ok(())
    } else {
        Err(RuntimeErrorType::MemoryCellDoesNotExist(label.to_string()))
    }
}

/// Tests if the memory cell with **label** exists and contains a value.
///
/// Ok(i32) contains the memory cell value.
///
/// Err(String) contains error message.
fn assert_memory_cell_contains_value(
    runtime_args: &mut RuntimeArgs,
    label: &str,
) -> Result<i32, RuntimeErrorType> {
    if let Some(value) = runtime_args.memory_cells.get(label) {
        if value.data.is_some() {
            Ok(runtime_args.memory_cells.get(label).unwrap().data.unwrap())
        } else {
            Err(RuntimeErrorType::MemoryCellUninitialized(label.to_string()))
        }
    } else {
        Err(RuntimeErrorType::MemoryCellDoesNotExist(label.to_string()))
    }
}
