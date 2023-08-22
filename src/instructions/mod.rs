use miette::Result;

use crate::{base::{Operation, Comparison}, runtime::{RuntimeArgs, ControlFlow, error_handling::{RuntimeErrorType, RuntimeBuildError}, builder::{check_accumulator, check_memory_cell}}, instructions::error_handling::InstructionParseError};

use self::parsing::{parse_memory_cell, parse_alpha};

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
    /// Dummy instruction that does nothing, is inserted in empty lines
    Noop,
    // more instructions to follow
}

impl Instruction {
    pub fn run(
        &self,
        runtime_args: &mut RuntimeArgs,
        control_flow: &mut ControlFlow,
    ) -> Result<(), RuntimeErrorType> {
        match self {
            Self::Assign(target, source) => run_assign(runtime_args, target, source)?,
            Self::Calc(target, source_a, op, source_b) => run_calc(runtime_args, target, source_a, op, source_b)?,
            Self::JumpIf(value_a, cmp, value_b, label) => run_jump_if(runtime_args, control_flow, value_a, cmp, value_b, label)?,
            Self::Goto(label) => run_goto(control_flow, label)?,
            Self::Push => run_push(runtime_args)?,
            Self::Pop => run_pop(runtime_args)?,
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


fn run_assign(runtime_args: &mut RuntimeArgs, target: &TargetType, source: &Value) -> Result<(), RuntimeErrorType> {
    match target {
        TargetType::Accumulator(a) => {
            assert_accumulator_exists(runtime_args, a)?;
            runtime_args.accumulators[*a].data = Some(source.value(runtime_args)?);
        }
        TargetType::MemoryCell(a) => {
            assert_memory_cell_exists(runtime_args, a)?;
            runtime_args.memory_cells.get_mut(a).unwrap().data =
                Some(source.value(runtime_args)?);
        }
    }
    Ok(())
}

fn run_calc(runtime_args: &mut RuntimeArgs, target: &TargetType, source_a: &Value, op: &Operation, source_b: &Value) -> Result<(), RuntimeErrorType> {
    match target {
        TargetType::Accumulator(a) => {
            runtime_args.accumulators[*a].data =
                Some(op.calc(source_a.value(runtime_args)?, source_b.value(runtime_args)?)?)
        }
        TargetType::MemoryCell(a) => {
            runtime_args.memory_cells.get_mut(a).unwrap().data =
                Some(op.calc(source_a.value(runtime_args)?, source_b.value(runtime_args)?)?)
        }
    }
    Ok(())
}

fn run_jump_if(runtime_args: &mut RuntimeArgs, control_flow: &mut ControlFlow, value_a: &Value, cmp: &Comparison, value_b: &Value, label: &str) -> Result<(), RuntimeErrorType> {
    if cmp.cmp(value_a.value(runtime_args)?, value_b.value(runtime_args)?) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

fn run_goto(control_flow: &mut ControlFlow, label: &str) -> Result<(), RuntimeErrorType> {
    control_flow.next_instruction_index(label)?;
    Ok(())
}

/// Causes runtime error if accumulator does not contain data.
fn run_push(runtime_args: &mut RuntimeArgs) -> Result<(), RuntimeErrorType> {
    assert_accumulator_exists(runtime_args, &0)?;
    match runtime_args.accumulators[0].data {
        Some(d) => runtime_args.stack.push(d),
        None => return Err(RuntimeErrorType::PushFail),
    }
    Ok(())
}

/// Causes runtime error if stack does not contain data.
fn run_pop(runtime_args: &mut RuntimeArgs) -> Result<(), RuntimeErrorType> {
    assert_accumulator_exists(runtime_args, &0)?;
    match runtime_args.stack.pop() {
        Some(d) => runtime_args.accumulators[0].data = Some(d),
        None => return Err(RuntimeErrorType::PopFail),
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
    runtime_args: &RuntimeArgs,
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

#[derive(Debug, PartialEq, Clone)]
pub enum TargetType {
    Accumulator(usize),
    MemoryCell(String),
}

impl TryFrom<(&str, (usize, usize))> for TargetType {
    type Error = InstructionParseError;

    fn try_from(value: (&str, (usize, usize))) -> Result<Self, Self::Error> {
        if let Ok(v) = parse_alpha(&value.0, value.1) {
            return Ok(Self::Accumulator(v));
        }
        if let Ok(v) = parse_memory_cell(&value.0, value.1) {
            return Ok(Self::MemoryCell(v));
        }
        Err(InstructionParseError::InvalidExpression(
            value.1,
            value.0.to_string(),
        ))
    }

}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Accumulator(usize),
    MemoryCell(String),
    Constant(i32),
}

impl Value {

    fn value(&self, runtime_args: &RuntimeArgs) -> Result<i32, RuntimeErrorType> {
        match self {
            //TODO When I use this add checks to test if accumulator / memory_cell exists / contains data before accessing it
            Self::Accumulator(a) => {
                assert_accumulator_contains_value(runtime_args, a)?;
                Ok(runtime_args.accumulators[*a].data.unwrap())
            },
            Self::Constant(a) => Ok(*a),
            Self::MemoryCell(a) => {
                assert_memory_cell_contains_value(runtime_args, a)?;
                Ok(runtime_args.memory_cells.get(a).unwrap().data.unwrap())
            },
        }
    }
    
}

impl TryFrom<(&str, (usize, usize))> for Value {
    type Error = InstructionParseError;

    fn try_from(value: (&str, (usize, usize))) -> Result<Self, Self::Error> {
        if let Ok(v) = parse_alpha(&value.0, value.1) {
            return Ok(Self::Accumulator(v));
        }
        if let Ok(v) = parse_memory_cell(&value.0, value.1) {
            return Ok(Self::MemoryCell(v));
        }
        if let Ok(v) = value.0.parse::<i32>() {
            return Ok(Self::Constant(v));
        }
        Err(InstructionParseError::InvalidExpression(
            value.1,
            value.0.to_string(),
        ))
    }
}

impl TryFrom<(String, (usize, usize))> for Value {
    type Error = InstructionParseError;

    fn try_from(value: (String, (usize, usize))) -> Result<Self, Self::Error> {
        Self::try_from((value.0.as_str(), value.1))
    }
}
