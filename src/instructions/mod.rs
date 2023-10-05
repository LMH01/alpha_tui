use miette::Result;

use crate::{
    base::{Comparison, Operation},
    instructions::error_handling::InstructionParseError,
    runtime::{error_handling::RuntimeErrorType, ControlFlow, RuntimeArgs},
};

use self::parsing::{parse_alpha, parse_gamma, parse_index_memory_cell, parse_memory_cell};

pub mod error_handling;

/// Functions related to instruction parsing
mod parsing;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Assign(TargetType, Value),
    Calc(TargetType, Value, Operation, Value),
    JumpIf(Value, Comparison, Value, String),
    Goto(String),
    Push,
    Pop,
    StackOp(Operation),
    Call(String),
    Return,

    /// Dummy instruction that does nothing, is inserted in empty lines
    Noop,
}

impl Instruction {
    pub fn run(
        &self,
        runtime_args: &mut RuntimeArgs,
        control_flow: &mut ControlFlow,
    ) -> Result<(), RuntimeErrorType> {
        match self {
            Self::Assign(target, source) => run_assign(runtime_args, target, source)?,
            Self::Calc(target, source_a, op, source_b) => {
                run_calc(runtime_args, target, source_a, *op, source_b)?;
            }
            Self::JumpIf(value_a, cmp, value_b, label) => {
                run_jump_if(runtime_args, control_flow, value_a, cmp, value_b, label)?;
            }
            Self::Goto(label) => run_goto(control_flow, label)?,
            Self::Push => run_push(runtime_args)?,
            Self::Pop => run_pop(runtime_args)?,
            Self::StackOp(op) => run_stack_op(runtime_args, *op)?,
            Self::Call(label) => run_call(control_flow, label)?,
            Self::Return => run_return(control_flow)?,
            Self::Noop => (),
        }
        Ok(())
    }

    ///// Checks if this instruction is legal by comparing if it matches one instruction in the instruction set
    ///// This is a workaround until I know if all instructions in the format are valid in alpha notation or if only specific instructions are allowed (= the instructions that I already made in the old version)
    //fn is_legal(&self) -> bool {//TODO Change return type to Result<InstructionParseError> and create error variant specific for this error
    //    //TODO Add in all other instructions that are allowed but that are not yet added as instructions to the instruction enum
    //    match self {
    //        Instruction::AssignInstruction(target, source) => {
    //            // All assign instructions are valid
    //            // These are: a := x, a := b, a := p(i), p(i) := x, p(i) := a, p(i) := p(j)
    //            return true;
    //        }
    //        Instruction::CalcInstruction(target, source_a, op, source_b) => {
    //            if let TargetType::Accumulator(idx_a) = target {
    //                if let Value::Accumulator(idx_b) = source_a {
    //                    if idx_a == idx_b  {
    //                        if let Value::Constant(_) = source_b {
    //                            // a := a op x
    //                            return true;
    //                        } else if let Value::Accumulator(_)  = source_b {
    //                            // a := a op b
    //                            return true;
    //                        }
    //                        return false;
    //                    }
    //                }
    //            }
    //        }
    //    }
    //    false
    //}
}

fn run_assign(
    runtime_args: &mut RuntimeArgs,
    target: &TargetType,
    source: &Value,
) -> Result<(), RuntimeErrorType> {
    match target {
        TargetType::Accumulator(a) => {
            assert_accumulator_exists(runtime_args, *a)?;
            runtime_args.accumulators.get_mut(a).unwrap().data = Some(source.value(runtime_args)?);
        }
        TargetType::Gamma => {
            runtime_args.gamma = Some(Some(source.value(runtime_args)?));
        }
        TargetType::MemoryCell(a) => {
            assert_memory_cell_exists(runtime_args, a)?;
            runtime_args.memory_cells.get_mut(a).unwrap().data = Some(source.value(runtime_args)?);
        }
        TargetType::IndexMemoryCell(t) => match t {
            IndexMemoryCellIndexType::Accumulator(idx) => {
                let idx = index_from_accumulator(runtime_args, idx)?;
                runtime_args
                    .index_memory_cells
                    .insert(idx, Some(source.value(runtime_args)?));
            }
            IndexMemoryCellIndexType::Direct(idx) => {
                runtime_args
                    .index_memory_cells
                    .insert(*idx, Some(source.value(runtime_args)?));
            }
            IndexMemoryCellIndexType::Gamma => {
                let idx = index_from_gamma(runtime_args)?;
                runtime_args
                    .index_memory_cells
                    .insert(idx, Some(source.value(runtime_args)?));
            }
            IndexMemoryCellIndexType::MemoryCell(name) => {
                let idx = index_from_memory_cell(runtime_args, &name)?;
                runtime_args
                    .index_memory_cells
                    .insert(idx as usize, Some(source.value(runtime_args)?));
            }
            IndexMemoryCellIndexType::Index(idx) => {
                let idx = index_from_index_memory_cell(runtime_args, idx)?;
                runtime_args
                    .index_memory_cells
                    .insert(idx as usize, Some(source.value(runtime_args)?));
            }
        },
    }
    Ok(())
}

fn run_calc(
    runtime_args: &mut RuntimeArgs,
    target: &TargetType,
    source_a: &Value,
    op: Operation,
    source_b: &Value,
) -> Result<(), RuntimeErrorType> {
    match target {
        TargetType::Accumulator(a) => {
            runtime_args.accumulators.get_mut(a).unwrap().data =
                Some(op.calc(source_a.value(runtime_args)?, source_b.value(runtime_args)?)?);
        }
        TargetType::Gamma => {
            assert_gamma_exists(runtime_args)?;
            runtime_args.gamma = Some(Some(
                op.calc(source_a.value(runtime_args)?, source_b.value(runtime_args)?)?,
            ));
        }
        TargetType::MemoryCell(a) => {
            runtime_args.memory_cells.get_mut(a).unwrap().data =
                Some(op.calc(source_a.value(runtime_args)?, source_b.value(runtime_args)?)?);
        }
        TargetType::IndexMemoryCell(t) => {
            let res = op.calc(source_a.value(runtime_args)?, source_b.value(runtime_args)?)?;
            match t {
                IndexMemoryCellIndexType::Accumulator(idx) => {
                    let idx = index_from_accumulator(runtime_args, idx)?;
                    runtime_args.index_memory_cells.insert(idx, Some(res));
                }
                IndexMemoryCellIndexType::Direct(idx) => {
                    runtime_args.index_memory_cells.insert(*idx, Some(res));
                }
                IndexMemoryCellIndexType::Gamma => {
                    let idx = index_from_gamma(runtime_args)?;
                    runtime_args.index_memory_cells.insert(idx, Some(res));
                }
                IndexMemoryCellIndexType::MemoryCell(name) => {
                    let idx = index_from_memory_cell(runtime_args, &name)?;
                    runtime_args
                        .index_memory_cells
                        .insert(idx as usize, Some(res));
                }
                IndexMemoryCellIndexType::Index(idx) => {
                    let idx = index_from_index_memory_cell(runtime_args, idx)?;
                    runtime_args
                        .index_memory_cells
                        .insert(idx as usize, Some(res));
                }
            }
        }
    }
    Ok(())
}

fn run_jump_if(
    runtime_args: &mut RuntimeArgs,
    control_flow: &mut ControlFlow,
    value_a: &Value,
    cmp: &Comparison,
    value_b: &Value,
    label: &str,
) -> Result<(), RuntimeErrorType> {
    if cmp.cmp(value_a.value(runtime_args)?, value_b.value(runtime_args)?) {
        control_flow.next_instruction_index(label)?;
    }
    Ok(())
}

fn run_goto(control_flow: &mut ControlFlow, label: &str) -> Result<(), RuntimeErrorType> {
    control_flow.next_instruction_index(label)?;
    Ok(())
}

/// Causes runtime error if accumulator does not contain data.
fn run_push(runtime_args: &mut RuntimeArgs) -> Result<(), RuntimeErrorType> {
    assert_accumulator_exists(runtime_args, 0)?;
    match runtime_args.accumulators[&0].data {
        Some(d) => runtime_args.stack.push(d),
        None => return Err(RuntimeErrorType::PushFail),
    }
    Ok(())
}

/// Causes runtime error if stack does not contain data.
fn run_pop(runtime_args: &mut RuntimeArgs) -> Result<(), RuntimeErrorType> {
    assert_accumulator_exists(runtime_args, 0)?;
    match runtime_args.stack.pop() {
        Some(d) => runtime_args.accumulators.get_mut(&0).unwrap().data = Some(d),
        None => return Err(RuntimeErrorType::PopFail),
    }
    Ok(())
}

/// Causes runtime error if stack does not contain two values.
fn run_stack_op(runtime_args: &mut RuntimeArgs, op: Operation) -> Result<(), RuntimeErrorType> {
    match runtime_args.stack.pop() {
        Some(a) => match runtime_args.stack.pop() {
            Some(b) => {
                runtime_args.stack.push(op.calc(b, a)?);
                Ok(())
            }
            None => Err(RuntimeErrorType::StackOpFail(op)),
        },
        None => Err(RuntimeErrorType::StackOpFail(op)),
    }
}

fn run_call(control_flow: &mut ControlFlow, label: &str) -> Result<(), RuntimeErrorType> {
    control_flow.call_function(label)
}

fn run_return(control_flow: &mut ControlFlow) -> Result<(), RuntimeErrorType> {
    match control_flow.call_stack.pop() {
        Some(i) => control_flow.next_instruction_index = i,
        None => run_goto(control_flow, "END")?,
    }
    Ok(())
}

/// Tests if the accumulator with **index** exists.
fn assert_accumulator_exists(
    runtime_args: &mut RuntimeArgs,
    index: usize,
) -> Result<(), RuntimeErrorType> {
    if let Some(_value) = runtime_args.accumulators.get(&index) {
        Ok(())
    } else {
        Err(RuntimeErrorType::AccumulatorDoesNotExist(index))
    }
}

/// Tests if the accumulator with **index** exists and contains a value.
///
/// Ok(i32) contains the accumulator value.
///
/// Err(String) contains error message.
fn assert_accumulator_contains_value(
    runtime_args: &RuntimeArgs,
    index: usize,
) -> Result<i32, RuntimeErrorType> {
    if let Some(value) = runtime_args.accumulators.get(&index) {
        if value.data.is_some() {
            Ok(runtime_args.accumulators.get(&index).unwrap().data.unwrap())
        } else {
            Err(RuntimeErrorType::AccumulatorUninitialized(index))
        }
    } else {
        Err(RuntimeErrorType::AccumulatorDoesNotExist(index))
    }
}

/// Tests if gamma exists
fn assert_gamma_exists(runtime_args: &RuntimeArgs) -> Result<(), RuntimeErrorType> {
    if let Some(value) = runtime_args.gamma {
        return Ok(());
    }
    Err(RuntimeErrorType::GammaDoesNotExist)
}

/// Tests if gamma contains a value.
fn assert_gamma_contains_value(runtime_args: &RuntimeArgs) -> Result<i32, RuntimeErrorType> {
    if let Some(value) = runtime_args.gamma {
        if let Some(value) = value {
            return Ok(value);
        } else {
            return Err(RuntimeErrorType::GammaUninitialized);
        }
    }
    Err(RuntimeErrorType::GammaDoesNotExist)
}

/// Tests if the memory cell with **label** exists.
fn assert_memory_cell_exists(
    runtime_args: &RuntimeArgs,
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
    runtime_args: &RuntimeArgs,
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

fn assert_index_memory_cell_contains_value(
    runtime_args: &RuntimeArgs,
    index: usize,
) -> Result<i32, RuntimeErrorType> {
    if let Some(value) = runtime_args.index_memory_cells.get(&index) {
        if let Some(value) = value {
            Ok(*value)
        } else {
            Err(RuntimeErrorType::IndexMemoryCellUninitialized(index))
        }
    } else {
        Err(RuntimeErrorType::IndexMemoryCellUninitialized(index)) //TODO create new error type index memory cell does not exist
    }
}

/// Specifies the location where the index memory cell should look for the value of the index of the index memory cell
#[derive(Debug, PartialEq, Clone)]
pub enum IndexMemoryCellIndexType {
    /// Indicates that this index memory cell uses the value of an accumulator as index where the data is accessed.
    Accumulator(usize),
    /// Indicates that this index memory cell uses a direct index to access data.
    ///
    /// E.g. p(1)
    Direct(usize),
    /// Indicates that this index memory cell uses the value of the gamma accumulator as index where the data is accessed.
    Gamma,
    /// Indicates that this index memory cell searches for the index in the location of memory cell with name String.
    ///
    /// E.g. p(p(h1)), String would be h1.
    MemoryCell(String),
    /// Indicates that this index memory cell searches for the index in the location of the index memory cell with usize.
    ///
    /// E.g. p(p(1)), usize would be 1.
    Index(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TargetType {
    Accumulator(usize),
    Gamma,
    MemoryCell(String),
    IndexMemoryCell(IndexMemoryCellIndexType),
}

impl TryFrom<(&String, (usize, usize))> for TargetType {
    type Error = InstructionParseError;

    fn try_from(value: (&String, (usize, usize))) -> Result<Self, Self::Error> {
        if let Ok(v) = parse_index_memory_cell(value.0, value.1) {
            return Ok(Self::IndexMemoryCell(v));
        }
        if let Ok(v) = parse_memory_cell(value.0, value.1) {
            return Ok(Self::MemoryCell(v));
        }
        if let Ok(_) = parse_gamma(value.0, value.1) {
            return Ok(Self::Gamma);
        }
        Ok(Self::Accumulator(parse_alpha(value.0, value.1, true)?))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Accumulator(usize),
    Gamma,
    MemoryCell(String),
    Constant(i32),
    IndexMemoryCell(IndexMemoryCellIndexType),
}

impl Value {
    fn value(&self, runtime_args: &RuntimeArgs) -> Result<i32, RuntimeErrorType> {
        match self {
            Self::Accumulator(a) => {
                assert_accumulator_contains_value(runtime_args, *a)?;
                Ok(runtime_args.accumulators.get(a).unwrap().data.unwrap())
            }
            Self::Gamma => {
                return assert_gamma_contains_value(runtime_args);
            }
            Self::Constant(a) => Ok(*a),
            Self::MemoryCell(a) => {
                assert_memory_cell_contains_value(runtime_args, a)?;
                Ok(runtime_args.memory_cells.get(a).unwrap().data.unwrap())
            }
            Self::IndexMemoryCell(t) => match t {
                IndexMemoryCellIndexType::Accumulator(idx) => {
                    let idx = index_from_accumulator(runtime_args, idx)?;
                    Ok(assert_index_memory_cell_contains_value(
                        runtime_args,
                        idx as usize,
                    )?)
                }
                IndexMemoryCellIndexType::Direct(idx) => {
                    Ok(assert_index_memory_cell_contains_value(runtime_args, *idx)?)
                }
                IndexMemoryCellIndexType::Gamma => {
                    let idx = index_from_gamma(runtime_args)?;
                    Ok(assert_index_memory_cell_contains_value(runtime_args, idx)?)
                }
                IndexMemoryCellIndexType::Index(idx) => {
                    let idx = index_from_index_memory_cell(runtime_args, idx)?;
                    Ok(assert_index_memory_cell_contains_value(runtime_args, idx)?)
                }
                IndexMemoryCellIndexType::MemoryCell(name) => {
                    let idx = index_from_memory_cell(runtime_args, name)?;
                    Ok(assert_index_memory_cell_contains_value(runtime_args, idx)?)
                }
            },
        }
    }
}

impl TryFrom<(&String, (usize, usize))> for Value {
    type Error = InstructionParseError;

    fn try_from(value: (&String, (usize, usize))) -> Result<Self, Self::Error> {
        if let Ok(t) = parse_index_memory_cell(value.0, value.1) {
            return Ok(Self::IndexMemoryCell(t));
        }
        if let Ok(v) = parse_memory_cell(value.0, value.1) {
            return Ok(Self::MemoryCell(v));
        }
        if let Ok(v) = value.0.parse::<i32>() {
            return Ok(Self::Constant(v));
        }
        if let Ok(_) = parse_gamma(value.0, value.1) {
            return Ok(Self::Gamma);
        }
        Ok(Self::Accumulator(parse_alpha(value.0, value.1, true)?))
    }
}

impl TryFrom<(String, (usize, usize))> for Value {
    type Error = InstructionParseError;

    fn try_from(value: (String, (usize, usize))) -> Result<Self, Self::Error> {
        Self::try_from((&value.0, value.1))
    }
}

/// Gets the content from the accumulator with the index `idx` and checks if this value is positive,
/// return the value if it is.
fn index_from_accumulator(
    runtime_args: &RuntimeArgs,
    idx: &usize,
) -> Result<usize, RuntimeErrorType> {
    let idx = assert_accumulator_contains_value(runtime_args, *idx)?;
    if idx.is_negative() {
        return Err(RuntimeErrorType::IndexMemoryCellNegativeIndex(idx));
    }
    Ok(idx as usize)
}

/// Gets the content from the gamma accumulator and checks if the value is positive,
/// return the value if it is.
fn index_from_gamma(runtime_args: &RuntimeArgs) -> Result<usize, RuntimeErrorType> {
    let idx = assert_gamma_contains_value(runtime_args)?;
    if idx.is_negative() {
        return Err(RuntimeErrorType::IndexMemoryCellNegativeIndex(idx));
    }
    Ok(idx as usize)
}

/// Gets the content of the memory cell with name `name` and check if this value is positive,
/// returns the value if it is.
fn index_from_memory_cell(
    runtime_args: &RuntimeArgs,
    name: &str,
) -> Result<usize, RuntimeErrorType> {
    let idx = assert_memory_cell_contains_value(runtime_args, name)?;
    if idx.is_negative() {
        return Err(RuntimeErrorType::IndexMemoryCellNegativeIndex(idx));
    }
    Ok(idx as usize)
}

/// Gets the content of the index memory cell with index `idx` and checks if this value is positive,
/// returns the value if it is.
fn index_from_index_memory_cell(
    runtime_args: &RuntimeArgs,
    idx: &usize,
) -> Result<usize, RuntimeErrorType> {
    let idx = assert_index_memory_cell_contains_value(runtime_args, *idx)?;
    if idx.is_negative() {
        return Err(RuntimeErrorType::IndexMemoryCellNegativeIndex(idx));
    }
    Ok(idx as usize)
}
