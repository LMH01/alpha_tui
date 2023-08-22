use crate::{
    base::Operation,
    instructions::parsing::{parse_operation, part_range},
    runtime::{error_handling::RuntimeErrorType, ControlFlow, RuntimeArgs},
};

use super::{
    error_handling::InstructionParseError,
    parsing::{parse_alpha, parse_memory_cell, whole_range},
};

#[derive(Debug, PartialEq)]
enum Instruction {
    AssignInstruction(TargetType, Value),
    CalcInstruction(TargetType, Value, Operation, Value),
    // more instructions to follow
}

impl Instruction {
    fn run(
        &self,
        runtime_args: &mut RuntimeArgs,
        control_flow: &mut ControlFlow,
    ) -> Result<(), RuntimeErrorType> {
        match self {
            // Performed action can be moved into own function when I use this
            Self::AssignInstruction(target, source) => {
                match target {
                    // TODO When I use this add in checks if accumulator / memory_cell exists before assigning value
                    TargetType::Accumulator(a) => {
                        runtime_args.accumulators[*a].data = Some(source.value(runtime_args));
                    }
                    TargetType::MemoryCell(a) => {
                        runtime_args.memory_cells.get_mut(a).unwrap().data =
                            Some(source.value(runtime_args));
                    }
                }
            }
            Self::CalcInstruction(target, source_a, op, source_b) => match target {
                TargetType::Accumulator(a) => {
                    runtime_args.accumulators[*a].data =
                        Some(op.calc(source_a.value(runtime_args), source_b.value(runtime_args))?)
                }
                TargetType::MemoryCell(a) => {
                    runtime_args.memory_cells.get_mut(a).unwrap().data =
                        Some(op.calc(source_a.value(runtime_args), source_b.value(runtime_args))?)
                }
            },
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

impl TryFrom<&Vec<&str>> for Instruction {
    type Error = InstructionParseError;

    fn try_from(parts: &Vec<&str>) -> Result<Self, Self::Error> {
        // very basic implementation more checks, more parsing, better structure and safeguards need to be added when this is used
        let target = TargetType::try_from((parts[0], part_range(parts, 0)))?;
        let source_a = Value::try_from((parts[2], part_range(parts, 1)))?;
        if parts.len() == 3 {
            // instruction is of type a := b
            return Ok(Instruction::AssignInstruction(target, source_a));
        } else if parts.len() == 5 {
            // instruction is of type a := b op c
            let op = parse_operation(parts[3], part_range(parts, 3))?;
            let source_b = Value::try_from((parts[4], part_range(parts, 4)))?;
            return Ok(Instruction::CalcInstruction(target, source_a, op, source_b));
        }
        Err(InstructionParseError::UnknownInstruction(
            whole_range(parts),
            parts.join(" "),
        ))
    }
}

impl TryFrom<&str> for Instruction {
    type Error = InstructionParseError;

    /// Tries to parse an instruction from the input string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(&value.split_whitespace().collect::<Vec<&str>>())
    }
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        base::{Accumulator, MemoryCell, Operation},
        instructions::new_backend::{Instruction, TargetType, Value},
        runtime::{ControlFlow, RuntimeArgs},
    };

    /// Used to set the available memory cells during testing.
    const TEST_MEMORY_CELL_LABELS: &'static [&'static str] = &[
        "a", "b", "c", "d", "e", "f", "w", "x", "y", "z", "h1", "h2", "h3", "h4",
    ];

    #[test]
    fn test_parse_assign_accumulator_from_constant() {
        assert_eq!(
            Instruction::try_from("a0 := 5"),
            Ok(Instruction::AssignInstruction(
                TargetType::Accumulator(0),
                Value::Constant(5)
            ))
        );
    }

    #[test]
    fn test_run_assign_accumulator_from_constant() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignInstruction(TargetType::Accumulator(0), Value::Constant(10))
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
    }

    #[test]
    fn test_parse_assign_accumulator_from_accumulator() {
        assert_eq!(
            Instruction::try_from("a0 := a1"),
            Ok(Instruction::AssignInstruction(
                TargetType::Accumulator(0),
                Value::Accumulator(1)
            ))
        );
    }

    #[test]
    fn test_run_assign_accumulator_from_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.accumulators[1].data = Some(10);
        Instruction::AssignInstruction(TargetType::Accumulator(0), Value::Accumulator(1))
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
    }

    #[test]
    fn test_parse_assign_accumulator_from_memory_cell() {
        assert_eq!(
            Instruction::try_from("a0 := p(h1)"),
            Ok(Instruction::AssignInstruction(
                TargetType::Accumulator(0),
                Value::MemoryCell("h1".to_string())
            ))
        );
    }

    #[test]
    fn test_run_assign_accumulator_from_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.memory_cells.get_mut("h1").unwrap().data = Some(10);
        Instruction::AssignInstruction(
            TargetType::Accumulator(0),
            Value::MemoryCell("h1".to_string()),
        )
        .run(&mut args, &mut control_flow)
        .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
    }

    #[test]
    fn test_parse_calc_accumulator_with_memory_cells() {
        assert_eq!(
            Instruction::try_from("a0 := p(h1) / p(h2)"),
            Ok(Instruction::CalcInstruction(
                TargetType::Accumulator(0),
                Value::MemoryCell("h1".to_string()),
                Operation::Div,
                Value::MemoryCell("h2".to_string())
            ))
        );
    }

    #[test]
    fn test_run_calc_accumulator_with_memory_cells() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.memory_cells.get_mut("h1").unwrap().data = Some(10);
        args.memory_cells.get_mut("h2").unwrap().data = Some(10);
        Instruction::CalcInstruction(
            TargetType::Accumulator(0),
            Value::MemoryCell("h1".to_string()),
            Operation::Mul,
            Value::MemoryCell("h2".to_string()),
        )
        .run(&mut args, &mut control_flow)
        .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 100);
    }

    #[test]
    fn test_parse_calc_accumulator_with_memory_cell_constant() {
        assert_eq!(
            Instruction::try_from("a0 := p(h1) + 5"),
            Ok(Instruction::CalcInstruction(
                TargetType::Accumulator(0),
                Value::MemoryCell("h1".to_string()),
                Operation::Add,
                Value::Constant(5)
            ))
        );
    }

    #[test]
    fn test_run_calc_accumulator_with_memory_cell_constant() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.memory_cells.get_mut("h1").unwrap().data = Some(10);
        Instruction::CalcInstruction(
            TargetType::Accumulator(0),
            Value::MemoryCell("h1".to_string()),
            Operation::Mul,
            Value::Constant(10),
        )
        .run(&mut args, &mut control_flow)
        .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 100);
    }

    #[test]
    fn test_parse_calc_accumulator_with_memory_cell_accumulator() {
        assert_eq!(
            Instruction::try_from("a0 := p(h1) - a0"),
            Ok(Instruction::CalcInstruction(
                TargetType::Accumulator(0),
                Value::MemoryCell("h1".to_string()),
                Operation::Sub,
                Value::Accumulator(0)
            ))
        );
    }

    #[test]
    fn test_run_calc_accumulator_with_memory_cell_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.memory_cells.get_mut("h1").unwrap().data = Some(10);
        args.accumulators[1].data = Some(10);
        Instruction::CalcInstruction(
            TargetType::Accumulator(0),
            Value::MemoryCell("h1".to_string()),
            Operation::Sub,
            Value::Accumulator(1),
        )
        .run(&mut args, &mut control_flow)
        .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 0);
    }

    #[test]
    fn test_parse_calc_accumulator_with_accumulators() {
        assert_eq!(
            Instruction::try_from("a0 := a1 * a2"),
            Ok(Instruction::CalcInstruction(
                TargetType::Accumulator(0),
                Value::Accumulator(1),
                Operation::Mul,
                Value::Accumulator(2)
            ))
        );
    }

    #[test]
    fn test_run_calc_accumulator_with_accumulators() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.accumulators[1].data = Some(10);
        args.accumulators[2].data = Some(5);
        Instruction::CalcInstruction(
            TargetType::Accumulator(0),
            Value::Accumulator(1),
            Operation::Div,
            Value::Accumulator(2),
        )
        .run(&mut args, &mut control_flow)
        .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 2);
    }

    #[test]
    fn test_parse_calc_accumulator_with_accumulator_constant() {
        assert_eq!(
            Instruction::try_from("a0 := a1 * 5"),
            Ok(Instruction::CalcInstruction(
                TargetType::Accumulator(0),
                Value::Accumulator(1),
                Operation::Mul,
                Value::Constant(5)
            ))
        );
    }

    #[test]
    fn test_run_calc_accumulator_with_accumulator_constant() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.accumulators[1].data = Some(10);
        Instruction::CalcInstruction(
            TargetType::Accumulator(0),
            Value::Accumulator(1),
            Operation::Add,
            Value::Constant(5),
        )
        .run(&mut args, &mut control_flow)
        .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 15);
    }

    #[test]
    fn test_parse_calc_accumulator_with_accumulator_memory_cell() {
        assert_eq!(
            Instruction::try_from("a0 := a1 * p(a)"),
            Ok(Instruction::CalcInstruction(
                TargetType::Accumulator(0),
                Value::Accumulator(1),
                Operation::Mul,
                Value::MemoryCell("a".to_string())
            ))
        );
    }

    #[test]
    fn test_run_calc_accumulator_with_accumulator_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.accumulators[1].data = Some(10);
        args.memory_cells.get_mut("h1").unwrap().data = Some(5);
        Instruction::CalcInstruction(
            TargetType::Accumulator(0),
            Value::Accumulator(1),
            Operation::Sub,
            Value::MemoryCell("h1".to_string()),
        )
        .run(&mut args, &mut control_flow)
        .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 5);
    }

    /// Sets up runtime args in a consistent way because the default implementation for memory cells and accumulators is configgurable.
    fn setup_runtime_args() -> RuntimeArgs {
        let mut args = RuntimeArgs::new_debug(TEST_MEMORY_CELL_LABELS);
        args.memory_cells = HashMap::new();
        args.memory_cells
            .insert("h1".to_string(), MemoryCell::new("a"));
        args.memory_cells
            .insert("h2".to_string(), MemoryCell::new("b"));
        args.accumulators = vec![
            Accumulator::new(0),
            Accumulator::new(1),
            Accumulator::new(2),
        ];
        args
    }
}
