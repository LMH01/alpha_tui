use crate::{base::{Operation, Comparison}, runtime::{RuntimeArgs, ControlFlow, error_handling::RuntimeErrorType}, instructions::error_handling::InstructionParseError};

use self::parsing::{parse_memory_cell, parse_alpha};

mod error_handling;

/// Functions related to instruction parsing
mod parsing;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq)]
enum Instruction {
    Assign(TargetType, Value),
    Calc(TargetType, Value, Operation, Value),
    Cmp(Value, Comparison, Value, String),
    // more instructions to follow
}

impl Instruction {
    fn run(
        &self,
        runtime_args: &mut RuntimeArgs,
        control_flow: &mut ControlFlow,
    ) -> Result<(), RuntimeErrorType> {
        match self {
            // TODO Add in checks if accumulator / memory_cell exists before assigning / using value
            // Performed action can be moved into own function when I use this
            Self::Assign(target, source) => {
                match target {
                    TargetType::Accumulator(a) => {
                        runtime_args.accumulators[*a].data = Some(source.value(runtime_args));
                    }
                    TargetType::MemoryCell(a) => {
                        runtime_args.memory_cells.get_mut(a).unwrap().data =
                            Some(source.value(runtime_args));
                    }
                }
            }
            Self::Calc(target, source_a, op, source_b) => match target {
                TargetType::Accumulator(a) => {
                    runtime_args.accumulators[*a].data =
                        Some(op.calc(source_a.value(runtime_args), source_b.value(runtime_args))?)
                }
                TargetType::MemoryCell(a) => {
                    runtime_args.memory_cells.get_mut(a).unwrap().data =
                        Some(op.calc(source_a.value(runtime_args), source_b.value(runtime_args))?)
                }
            },
            Self::Cmp(value_a, cmp, value_b, label) => {
                if cmp.cmp(value_a.value(runtime_args), value_b.value(runtime_args)) {
                    control_flow.next_instruction_index(label)?
                }
            }
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


#[derive(Debug, PartialEq)]
enum TargetType {
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

#[derive(Debug, PartialEq)]
enum Value {
    Accumulator(usize),
    MemoryCell(String),
    Constant(i32),
}

impl Value {
    fn value(&self, runtime_args: &RuntimeArgs) -> i32 {
        match self {
            //TODO When I use this add checks to test if accumulator / memory_cell exists / contains data before accessing it
            Self::Accumulator(a) => runtime_args.accumulators[*a].data.unwrap(),
            Self::Constant(a) => *a,
            Self::MemoryCell(a) => runtime_args.memory_cells.get(a).unwrap().data.unwrap(),
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
