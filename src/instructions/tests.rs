use std::collections::HashMap;

use crate::{
    base::{Accumulator, Comparison, MemoryCell, Operation},
    instructions::{error_handling::InstructionParseError, Instruction},
    runtime::{ControlFlow, RuntimeArgs, RuntimeBuilder},
};

/// Used to set the available memory cells during testing.
const TEST_MEMORY_CELL_LABELS: &'static [&'static str] = &[
    "a", "b", "c", "d", "e", "f", "w", "x", "y", "z", "h1", "h2", "h3", "h4",
];

#[test]
fn test_stack() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::AssignAccumulatorValue(0, 5)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::Push()
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignAccumulatorValue(0, 10)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::Push()
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.stack, vec![5, 10]);
    Instruction::Pop()
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 10);
    Instruction::Pop()
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 5);
    assert_eq!(args.stack.len(), 0);
}

#[test]
fn test_parse_push() {
    assert_eq!(Instruction::try_from("push"), Ok(Instruction::Push()));
}

#[test]
fn test_parse_pop() {
    assert_eq!(Instruction::try_from("pop"), Ok(Instruction::Pop()));
}

#[test]
fn test_parse_assign_accumulator_value() {
    assert_eq!(
        Instruction::try_from("a0 := 20"),
        Ok(Instruction::AssignAccumulatorValue(0, 20))
    );
    assert_eq!(
        Instruction::try_from("a0 := x"),
        Err(InstructionParseError::InvalidExpression((6, 6)))
    );
}

#[test]
fn test_assign_accumulator_value_from_accumulator() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::AssignAccumulatorValue(0, 5)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignAccumulatorValue(1, 20)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignAccumulatorValue(2, 12)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignAccumulatorValueFromAccumulator(1, 2)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignAccumulatorValueFromAccumulator(0, 1)
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.accumulators[1].data.unwrap(), 12);
    assert_eq!(args.accumulators[2].data.unwrap(), 12);
}

#[test]
fn test_parse_assign_accumulator_value_from_accumulator() {
    assert_eq!(
        Instruction::try_from("a0 := a1"),
        Ok(Instruction::AssignAccumulatorValueFromAccumulator(0, 1))
    );
    assert_eq!(
        Instruction::try_from("a3 := a15"),
        Ok(Instruction::AssignAccumulatorValueFromAccumulator(3, 15))
    );
    assert_eq!(
        Instruction::try_from("a3 := a1x"),
        Err(InstructionParseError::NotANumber((7, 8)))
    );
    assert_eq!(
        Instruction::try_from("ab := a1x"),
        Err(InstructionParseError::NotANumber((1, 1)))
    );
}

#[test]
fn test_assign_accumulator_value_from_accumulator_error() {
    let mut args = setup_empty_runtime_args();
    let mut control_flow = ControlFlow::new();
    assert!(Instruction::AssignAccumulatorValueFromAccumulator(0, 1)
        .run(&mut args, &mut control_flow)
        .is_err());
    assert!(Instruction::AssignAccumulatorValueFromAccumulator(1, 0)
        .run(&mut args, &mut control_flow)
        .is_err());
}

#[test]
fn test_assign_accumulator_value_from_memory_cell() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::AssignMemoryCellValue("a".to_string(), 10)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string())
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 10);
}

#[test]
fn test_parse_assign_accumulator_value_from_memory_cell() {
    assert_eq!(
        Instruction::try_from("a0 := p(h1)"),
        Ok(Instruction::AssignAccumulatorValueFromMemoryCell(
            0,
            "h1".to_string()
        ))
    );
    assert_eq!(
        Instruction::try_from("a4 := p(x2)"),
        Ok(Instruction::AssignAccumulatorValueFromMemoryCell(
            4,
            "x2".to_string()
        ))
    );
    assert_eq!(
        Instruction::try_from("a4 := p()"),
        Err(InstructionParseError::InvalidExpression((6, 8)))
    );
}

#[test]
fn test_assign_accumulator_value_from_memory_cell_error() {
    let mut args = setup_empty_runtime_args();
    let mut control_flow = ControlFlow::new();
    args.accumulators = vec![Accumulator::new(0)];
    let err = Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string())
        .run(&mut args, &mut control_flow);
    assert!(err.is_err());
    assert!(err.err().unwrap().contains("Memory cell"));
    args.memory_cells
        .insert("a".to_string(), MemoryCell::new("a"));
    let err = Instruction::AssignAccumulatorValueFromMemoryCell(1, "a".to_string())
        .run(&mut args, &mut control_flow);
    assert!(err.is_err());
    assert!(err.err().unwrap().contains("Accumulator"));
}

#[test]
fn test_assign_memory_cell_value() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::AssignMemoryCellValue("a".to_string(), 2)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignMemoryCellValue("b".to_string(), 20)
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 2);
    assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
}

#[test]
fn test_parse_assign_memory_cell_value() {
    assert_eq!(
        Instruction::try_from("p(h1) := 10"),
        Ok(Instruction::AssignMemoryCellValue("h1".to_string(), 10))
    );
    assert_eq!(
        Instruction::try_from("p(h1) := x"),
        Err(InstructionParseError::InvalidExpression((9, 9)))
    );
}

#[test]
fn test_assign_memory_cell_value_error() {
    let mut args = setup_empty_runtime_args();
    let mut control_flow = ControlFlow::new();
    assert!(Instruction::AssignMemoryCellValue("c".to_string(), 10)
        .run(&mut args, &mut control_flow)
        .is_err());
}

#[test]
fn test_assign_memory_cell_value_from_accumulator() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::AssignAccumulatorValue(0, 20)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 0)
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 20);
}

#[test]
fn test_parse_assign_memory_cell_value_from_accumulator() {
    assert_eq!(
        Instruction::try_from("p(h1) := a0"),
        Ok(Instruction::AssignMemoryCellValueFromAccumulator(
            "h1".to_string(),
            0
        ))
    );
    assert_eq!(
        Instruction::try_from("p(h1) := a0x"),
        Err(InstructionParseError::InvalidExpression((9, 11)))
    );
}

#[test]
fn test_assign_memory_cell_value_from_accumulator_error() {
    let mut args = setup_empty_runtime_args();
    let mut control_flow = ControlFlow::new();
    args.accumulators = vec![Accumulator::new(0)];
    let err = Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 0)
        .run(&mut args, &mut control_flow);
    assert!(err.is_err());
    assert!(err.err().unwrap().contains("Memory cell"));
    args.memory_cells
        .insert("a".to_string(), MemoryCell::new("a"));
    let err = Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 1)
        .run(&mut args, &mut control_flow);
    assert!(err.is_err());
    assert!(err.err().unwrap().contains("Accumulator"));
}

#[test]
fn test_assign_memory_cell_value_from_memory_cell() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::AssignMemoryCellValue("a".to_string(), 20)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignMemoryCellValueFromMemoryCell("b".to_string(), "a".to_string())
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
}

#[test]
fn test_parse_assign_memory_cell_value_from_memory_cell() {
    assert_eq!(
        Instruction::try_from("p(h1) := p(h2)"),
        Ok(Instruction::AssignMemoryCellValueFromMemoryCell(
            "h1".to_string(),
            "h2".to_string()
        ))
    );
    assert_eq!(
        Instruction::try_from("p(h1) := p()"),
        Err(InstructionParseError::InvalidExpression((9, 11)))
    );
}

#[test]
fn test_assign_memory_cell_value_from_memory_cell_error() {
    let mut args = setup_empty_runtime_args();
    let mut control_flow = ControlFlow::new();
    assert!(
        Instruction::AssignMemoryCellValueFromMemoryCell("a".to_string(), "b".to_string())
            .run(&mut args, &mut control_flow)
            .is_err()
    );
    assert!(
        Instruction::AssignMemoryCellValueFromMemoryCell("b".to_string(), "a".to_string())
            .run(&mut args, &mut control_flow)
            .is_err()
    );
    args.memory_cells
        .insert("a".to_string(), MemoryCell::new("a"));
    args.memory_cells
        .insert("b".to_string(), MemoryCell::new("b"));
    args.memory_cells.get_mut("b").unwrap().data = Some(10);
    assert!(
        Instruction::AssignMemoryCellValueFromMemoryCell("b".to_string(), "a".to_string())
            .run(&mut args, &mut control_flow)
            .is_err()
    );
    assert!(
        Instruction::AssignMemoryCellValueFromMemoryCell("a".to_string(), "b".to_string())
            .run(&mut args, &mut control_flow)
            .is_ok()
    );
}

#[test]
fn test_calc_accumulator_with_constant() {
    let mut args = setup_runtime_args();
    let control_flow = &mut ControlFlow::new();
    Instruction::AssignAccumulatorValue(0, 20)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::CalcAccumulatorWithConstant(Operation::Plus, 0, 20)
        .run(&mut args, control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 40);
}

#[test]
fn test_parse_calc_accumulator_with_constant() {
    assert_eq!(
        Instruction::try_from("a1 := a1 + 20"),
        Ok(Instruction::CalcAccumulatorWithConstant(
            Operation::Plus,
            1,
            20
        ))
    );
    assert_eq!(
        Instruction::try_from("a1 := ab2 + a29"),
        Err(InstructionParseError::NotANumber((7, 8)))
    );
    assert_eq!(
        Instruction::try_from("a1 := a2 + 20"),
        Err(InstructionParseError::NoMatchSuggestion {
            range: (0, 13),
            help: "Did you mean: \"a1 := a1 + 20\" ?".to_string()
        })
    );
}

#[test]
fn test_calc_accumulator_with_accumulator() {
    let mut args = setup_runtime_args();
    let control_flow = &mut ControlFlow::new();
    Instruction::AssignAccumulatorValue(0, 20)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::AssignAccumulatorValue(1, 20)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::CalcAccumulatorWithAccumulator(Operation::Plus, 0, 1)
        .run(&mut args, control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 40);
}

#[test]
fn test_parse_calc_accumulator_with_accumulator() {
    assert_eq!(
        Instruction::try_from("a1 := a1 + a2"),
        Ok(Instruction::CalcAccumulatorWithAccumulator(
            Operation::Plus,
            1,
            2
        ))
    );
    assert_eq!(
        Instruction::try_from("a1 := a1 / a5"),
        Ok(Instruction::CalcAccumulatorWithAccumulator(
            Operation::Division,
            1,
            5
        ))
    );
}

#[test]
fn test_calc_accumulator_with_accumulators() {
    let mut args = setup_runtime_args();
    let control_flow = &mut ControlFlow::new();
    Instruction::AssignAccumulatorValue(1, 20)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::AssignAccumulatorValue(2, 20)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::CalcAccumulatorWithAccumulators(Operation::Plus, 0, 1, 2)
        .run(&mut args, control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 40);
}

#[test]
fn test_parse_calc_accumulator_with_accumulators() {
    assert_eq!(
        Instruction::try_from("a1 := a2 + a3"),
        Ok(Instruction::CalcAccumulatorWithAccumulators(
            Operation::Plus,
            1,
            2,
            3
        ))
    );
    assert_eq!(
        Instruction::try_from("a1 := a3 / a5"),
        Ok(Instruction::CalcAccumulatorWithAccumulators(
            Operation::Division,
            1,
            3,
            5
        ))
    );
}

#[test]
fn test_calc_accumulator_with_memory_cell() {
    let mut args = setup_runtime_args();
    let control_flow = &mut ControlFlow::new();
    Instruction::AssignAccumulatorValue(0, 20)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::AssignMemoryCellValue("a".to_string(), 20)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::CalcAccumulatorWithMemoryCell(Operation::Plus, 0, "a".to_string())
        .run(&mut args, control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 40);
}

#[test]
fn test_parse_calc_accumulator_with_memory_cell() {
    assert_eq!(
        Instruction::try_from("a1 := a1 * p(h1)"),
        Ok(Instruction::CalcAccumulatorWithMemoryCell(
            Operation::Multiplication,
            1,
            "h1".to_string()
        ))
    );
    assert_eq!(
        Instruction::try_from("a1 := a2 * p(h1)"),
        Err(InstructionParseError::NoMatchSuggestion {
            range: (0, 16),
            help: "Did you mean: \"a1 := a1 * p(h1)\" ?".to_string()
        })
    );
    assert_eq!(
        Instruction::try_from("a1 := a1 * p()"),
        Err(InstructionParseError::InvalidExpression((11, 13)))
    );
}

#[test]
fn test_calc_accumulator_with_memory_cells() {
    let mut args = setup_runtime_args();
    let control_flow = &mut ControlFlow::new();
    Instruction::AssignMemoryCellValue("a".to_string(), 10)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::AssignMemoryCellValue("b".to_string(), 10)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::CalcAccumulatorWithMemoryCells(
        Operation::Plus,
        0,
        "a".to_string(),
        "b".to_string(),
    )
    .run(&mut args, control_flow)
    .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 20);
}

#[test]
fn test_parse_calc_accumulator_with_memory_cells() {
    assert_eq!(
        Instruction::try_from("a0 := p(h1) / p(h2)"),
        Ok(Instruction::CalcAccumulatorWithMemoryCells(
            Operation::Division,
            0,
            "h1".to_string(),
            "h2".to_string()
        ))
    );
    assert_eq!(
        Instruction::try_from("a0 := p(h1) x p(h2)"),
        Err(InstructionParseError::UnknownOperation((12, 12)))
    );
    assert_eq!(
        Instruction::try_from("a0 := p(h1) / p()"),
        Err(InstructionParseError::InvalidExpression((14, 16)))
    );
}

#[test]
fn test_calc_memory_cell_with_memory_cell_constant() {
    let mut args = setup_runtime_args();
    let control_flow = &mut ControlFlow::new();
    Instruction::AssignMemoryCellValue("b".to_string(), 10)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::CalcMemoryCellWithMemoryCellConstant(
        Operation::Plus,
        "a".to_string(),
        "b".to_string(),
        10,
    )
    .run(&mut args, control_flow)
    .unwrap();
    assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 20);
}

#[test]
fn test_parse_calc_memory_cell_with_memory_cell_constant() {
    assert_eq!(
        Instruction::try_from("p(h1) := p(h2) * 10"),
        Ok(Instruction::CalcMemoryCellWithMemoryCellConstant(
            Operation::Multiplication,
            "h1".to_string(),
            "h2".to_string(),
            10
        ))
    );
    assert_eq!(
        Instruction::try_from("p(h1) := p(h2) o 10"),
        Err(InstructionParseError::UnknownOperation((15, 15)))
    );
}

#[test]
fn test_calc_memory_cell_with_memory_cell_accumulator() {
    let mut args = setup_runtime_args();
    let control_flow = &mut ControlFlow::new();
    Instruction::AssignAccumulatorValue(0, 20)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::AssignMemoryCellValue("b".to_string(), 10)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::CalcMemoryCellWithMemoryCellAccumulator(
        Operation::Plus,
        "a".to_string(),
        "b".to_string(),
        0,
    )
    .run(&mut args, control_flow)
    .unwrap();
    assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 30);
}

#[test]
fn test_parse_calc_memory_cell_with_memory_cell_accumulator() {
    assert_eq!(
        Instruction::try_from("p(h1) := p(h2) * a0"),
        Ok(Instruction::CalcMemoryCellWithMemoryCellAccumulator(
            Operation::Multiplication,
            "h1".to_string(),
            "h2".to_string(),
            0
        ))
    );
}

#[test]
fn test_calc_memory_cell_with_memory_cells() {
    let mut args = setup_runtime_args();
    let control_flow = &mut ControlFlow::new();
    Instruction::AssignMemoryCellValue("b".to_string(), 10)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::AssignMemoryCellValue("c".to_string(), 10)
        .run(&mut args, control_flow)
        .unwrap();
    Instruction::CalcMemoryCellWithMemoryCells(
        Operation::Plus,
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
    )
    .run(&mut args, control_flow)
    .unwrap();
    assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 20);
}

#[test]
fn test_parse_calc_memory_cell_with_memory_cells() {
    assert_eq!(
        Instruction::try_from("p(h1) := p(h2) * p(h3)"),
        Ok(Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Multiplication,
            "h1".to_string(),
            "h2".to_string(),
            "h3".to_string()
        ))
    );
}

#[test]
fn test_goto() {
    let mut args = setup_empty_runtime_args();
    let mut control_flow = ControlFlow::new();
    control_flow
        .instruction_labels
        .insert("loop".to_string(), 5);
    Instruction::Goto("loop".to_string())
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(control_flow.next_instruction_index, 5);
}

#[test]
fn test_parse_goto() {
    assert_eq!(
        Instruction::try_from("goto loop"),
        Ok(Instruction::Goto("loop".to_string()))
    );
}

#[test]
fn test_goto_error() {
    let mut args = setup_empty_runtime_args();
    let mut control_flow = ControlFlow::new();
    assert!(Instruction::Goto("loop".to_string())
        .run(&mut args, &mut control_flow)
        .is_err());
}

#[test]
fn test_goto_if_accumulator() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    control_flow
        .instruction_labels
        .insert("loop".to_string(), 20);
    Instruction::AssignAccumulatorValue(0, 20)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignAccumulatorValue(1, 30)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::GotoIfAccumulator(Comparison::Less, "loop".to_string(), 0, 1)
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(control_flow.next_instruction_index, 20);
    control_flow.next_instruction_index = 0;
    Instruction::GotoIfAccumulator(Comparison::Equal, "loop".to_string(), 0, 1)
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(control_flow.next_instruction_index, 0);
    assert!(
        Instruction::GotoIfAccumulator(Comparison::Less, "none".to_string(), 0, 1)
            .run(&mut args, &mut control_flow)
            .is_err()
    );
    assert!(
        Instruction::GotoIfAccumulator(Comparison::Equal, "none".to_string(), 0, 1)
            .run(&mut args, &mut control_flow)
            .is_ok()
    );
}

#[test]
fn test_parse_goto_if_accumulator() {
    assert_eq!(
        Instruction::try_from("if a0 <= a1 then goto loop"),
        Ok(Instruction::GotoIfAccumulator(
            Comparison::LessOrEqual,
            "loop".to_string(),
            0,
            1
        ))
    );
    assert_eq!(
        Instruction::try_from("if x <= a1 then goto loop"),
        Err(InstructionParseError::InvalidExpression((3, 3)))
    );
}

#[test]
fn test_goto_if_constant() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    control_flow
        .instruction_labels
        .insert("loop".to_string(), 20);
    Instruction::AssignAccumulatorValue(0, 20)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::GotoIfConstant(Comparison::Less, "loop".to_string(), 0, 40)
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(control_flow.next_instruction_index, 20);
    control_flow.next_instruction_index = 0;
    Instruction::GotoIfConstant(Comparison::Equal, "loop".to_string(), 0, 40)
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(control_flow.next_instruction_index, 0);
    assert!(
        Instruction::GotoIfConstant(Comparison::Less, "none".to_string(), 0, 40)
            .run(&mut args, &mut control_flow)
            .is_err()
    );
    assert!(
        Instruction::GotoIfConstant(Comparison::Equal, "none".to_string(), 0, 40)
            .run(&mut args, &mut control_flow)
            .is_ok()
    );
}

#[test]
fn test_parse_goto_if_constant() {
    assert_eq!(
        Instruction::try_from("if a0 == 20 then goto loop"),
        Ok(Instruction::GotoIfConstant(
            Comparison::Equal,
            "loop".to_string(),
            0,
            20
        ))
    );
}

#[test]
fn test_goto_if_memory_cell() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    control_flow
        .instruction_labels
        .insert("loop".to_string(), 20);
    Instruction::AssignAccumulatorValue(0, 20)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::AssignMemoryCellValue("a".to_string(), 50)
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::GotoIfMemoryCell(Comparison::Less, "loop".to_string(), 0, "a".to_string())
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(control_flow.next_instruction_index, 20);
    control_flow.next_instruction_index = 0;
    Instruction::GotoIfMemoryCell(Comparison::Equal, "loop".to_string(), 0, "a".to_string())
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(control_flow.next_instruction_index, 0);
    assert!(Instruction::GotoIfMemoryCell(
        Comparison::Less,
        "none".to_string(),
        0,
        "a".to_string()
    )
    .run(&mut args, &mut control_flow)
    .is_err());
    assert!(Instruction::GotoIfMemoryCell(
        Comparison::Equal,
        "none".to_string(),
        0,
        "a".to_string()
    )
    .run(&mut args, &mut control_flow)
    .is_ok());
}

#[test]
fn test_parse_goto_if_memory_cell() {
    assert_eq!(
        Instruction::try_from("if a0 == p(h1) then goto loop"),
        Ok(Instruction::GotoIfMemoryCell(
            Comparison::Equal,
            "loop".to_string(),
            0,
            "h1".to_string()
        ))
    );
    assert_eq!(
        Instruction::try_from("if a0 == p then goto loop"),
        Err(InstructionParseError::InvalidExpression((9, 9)))
    );
}

#[test]
fn test_example_program_1() {
    let mut runtime_args = RuntimeArgs::new_debug(TEST_MEMORY_CELL_LABELS);
    for _i in 1..=4 {
        runtime_args.add_accumulator();
    }
    runtime_args.add_storage_cell("a");
    runtime_args.add_storage_cell("b");
    runtime_args.add_storage_cell("c");
    runtime_args.add_storage_cell("d");
    runtime_args.add_storage_cell("w");
    runtime_args.add_storage_cell("x");
    runtime_args.add_storage_cell("y");
    runtime_args.add_storage_cell("z");
    runtime_args.add_storage_cell("h1");
    runtime_args.add_storage_cell("h2");
    runtime_args.add_storage_cell("h3");
    runtime_args.add_storage_cell("h4");
    let instructions = vec![
        Instruction::AssignMemoryCellValue("a".to_string(), 5),
        Instruction::AssignMemoryCellValue("b".to_string(), 2),
        Instruction::AssignMemoryCellValue("c".to_string(), 3),
        Instruction::AssignMemoryCellValue("d".to_string(), 9),
        Instruction::AssignMemoryCellValue("w".to_string(), 4),
        Instruction::AssignMemoryCellValue("x".to_string(), 8),
        Instruction::AssignMemoryCellValue("y".to_string(), 3),
        Instruction::AssignMemoryCellValue("z".to_string(), 2),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Multiplication,
            "h1".to_string(),
            "a".to_string(),
            "w".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Multiplication,
            "h2".to_string(),
            "b".to_string(),
            "y".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Multiplication,
            "h3".to_string(),
            "a".to_string(),
            "x".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Multiplication,
            "h4".to_string(),
            "b".to_string(),
            "z".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Plus,
            "a".to_string(),
            "h1".to_string(),
            "h2".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Plus,
            "b".to_string(),
            "h3".to_string(),
            "h4".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Multiplication,
            "h1".to_string(),
            "c".to_string(),
            "w".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Multiplication,
            "h2".to_string(),
            "d".to_string(),
            "y".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Multiplication,
            "h3".to_string(),
            "c".to_string(),
            "x".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Multiplication,
            "h4".to_string(),
            "d".to_string(),
            "z".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Plus,
            "c".to_string(),
            "h1".to_string(),
            "h2".to_string(),
        ),
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Plus,
            "d".to_string(),
            "h3".to_string(),
            "h4".to_string(),
        ),
    ];
    let mut runtime_builder = RuntimeBuilder::new();
    runtime_builder.set_instructions(instructions);
    runtime_builder.set_runtime_args(runtime_args);
    let mut runtime = runtime_builder.build().expect("Unable to build runtime!");
    runtime.run().unwrap();
    assert_eq!(
        runtime
            .runtime_args()
            .memory_cells
            .get("a")
            .unwrap()
            .data
            .unwrap(),
        26
    );
    assert_eq!(
        runtime
            .runtime_args()
            .memory_cells
            .get("b")
            .unwrap()
            .data
            .unwrap(),
        44
    );
    assert_eq!(
        runtime
            .runtime_args()
            .memory_cells
            .get("c")
            .unwrap()
            .data
            .unwrap(),
        39
    );
    assert_eq!(
        runtime
            .runtime_args()
            .memory_cells
            .get("d")
            .unwrap()
            .data
            .unwrap(),
        42
    );
}

#[test]
fn test_example_program_1_text_parsing() {
    let mut runtime_args = RuntimeArgs::new_debug(TEST_MEMORY_CELL_LABELS);
    for _i in 1..=4 {
        runtime_args.add_accumulator();
    }
    runtime_args.add_storage_cell("a");
    runtime_args.add_storage_cell("b");
    runtime_args.add_storage_cell("c");
    runtime_args.add_storage_cell("d");
    runtime_args.add_storage_cell("w");
    runtime_args.add_storage_cell("x");
    runtime_args.add_storage_cell("y");
    runtime_args.add_storage_cell("z");
    runtime_args.add_storage_cell("h1");
    runtime_args.add_storage_cell("h2");
    runtime_args.add_storage_cell("h3");
    runtime_args.add_storage_cell("h4");
    let mut instructions = Vec::new();
    instructions.push("p(a) := 5\n");
    instructions.push("p(b) := 2\n");
    instructions.push("p(c) := 3\n");
    instructions.push("p(d) := 9\n");
    instructions.push("p(w) := 4\n");
    instructions.push("p(x) := 8\n");
    instructions.push("p(y) := 3\n");
    instructions.push("p(z) := 2\n");
    instructions.push("p(h1) := p(a) * p(w)\n");
    instructions.push("p(h2) := p(b) * p(y)\n");
    instructions.push("p(h3) := p(a) * p(x)\n");
    instructions.push("p(h4) := p(b) * p(z)\n");
    instructions.push("p(a) := p(h1) + p(h2)\n");
    instructions.push("p(b) := p(h3) + p(h4)\n");
    instructions.push("p(h1) := p(c) * p(w)\n");
    instructions.push("p(h2) := p(d) * p(y)\n");
    instructions.push("p(h3) := p(c) * p(x)\n");
    instructions.push("p(h4) := p(d) * p(z)\n");
    instructions.push("p(c) := p(h1) + p(h2)\n");
    instructions.push("p(d) := p(h3) + p(h4)\n");
    let mut rb = RuntimeBuilder::new();
    rb.set_runtime_args(runtime_args);
    assert!(rb.build_instructions(&instructions, "test").is_ok());
    let rt = rb.build();
    assert!(rt.is_ok());
    let mut rt = rt.unwrap();
    assert!(rt.run().is_ok());
    assert_eq!(
        rt.runtime_args()
            .memory_cells
            .get("a")
            .unwrap()
            .data
            .unwrap(),
        26
    );
    assert_eq!(
        rt.runtime_args()
            .memory_cells
            .get("b")
            .unwrap()
            .data
            .unwrap(),
        44
    );
    assert_eq!(
        rt.runtime_args()
            .memory_cells
            .get("c")
            .unwrap()
            .data
            .unwrap(),
        39
    );
    assert_eq!(
        rt.runtime_args()
            .memory_cells
            .get("d")
            .unwrap()
            .data
            .unwrap(),
        42
    );
}

#[test]
fn test_example_program_2() {
    let instructions = vec![
        Instruction::AssignAccumulatorValue(0, 1),
        Instruction::AssignMemoryCellValue("a".to_string(), 8),
        Instruction::CalcAccumulatorWithConstant(Operation::Multiplication, 0, 2),
        Instruction::CalcMemoryCellWithMemoryCellConstant(
            Operation::Minus,
            "a".to_string(),
            "a".to_string(),
            1,
        ),
        Instruction::AssignAccumulatorValueFromMemoryCell(1, "a".to_string()),
        Instruction::GotoIfConstant(Comparison::More, "loop".to_string(), 1, 0),
    ];
    let mut runtime_builder = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
    runtime_builder.set_instructions(instructions);
    runtime_builder.add_label("loop".to_string(), 2).unwrap();
    let mut runtime = runtime_builder.build().expect("Unable to build runtime!");
    runtime.run().unwrap();
    assert_eq!(runtime.runtime_args().accumulators[0].data.unwrap(), 256);
}

#[test]
fn test_example_program_2_text_parsing() {
    let mut instructions = Vec::new();
    instructions.push("a0 := 1");
    instructions.push("p(a) := 8");
    instructions.push("loop: a0 := a0 * 2");
    instructions.push("p(a) := p(a) - 1");
    instructions.push("a1 := p(a)");
    instructions.push("if a1 > 0 then goto loop");
    let mut runtime_builder = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
    let res = runtime_builder.build_instructions(&instructions, "test");
    println!("{:?}", res);
    assert!(res.is_ok());
    let mut runtime = runtime_builder.build().expect("Unable to build runtime!");
    runtime.run().unwrap();
    assert_eq!(runtime.runtime_args().accumulators[0].data.unwrap(), 256);
}

/// Sets up runtime args in a conistent way because the default implementation for memory cells and accumulators is configgurable.
fn setup_runtime_args() -> RuntimeArgs {
    let mut args = RuntimeArgs::new_debug(TEST_MEMORY_CELL_LABELS);
    args.memory_cells = HashMap::new();
    args.memory_cells
        .insert("a".to_string(), MemoryCell::new("a"));
    args.memory_cells
        .insert("b".to_string(), MemoryCell::new("b"));
    args.memory_cells
        .insert("c".to_string(), MemoryCell::new("c"));
    args.accumulators = vec![
        Accumulator::new(0),
        Accumulator::new(1),
        Accumulator::new(2),
    ];
    args
}

/// Sets up runtime args where no memory cells or accumulators are set.
fn setup_empty_runtime_args() -> RuntimeArgs {
    let mut args = RuntimeArgs::new_debug(TEST_MEMORY_CELL_LABELS);
    args.accumulators = Vec::new();
    args.memory_cells = HashMap::new();
    args
}
