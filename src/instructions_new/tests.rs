use std::collections::HashMap;

use crate::{
    base::{Accumulator, Comparison, MemoryCell, Operation},
    instructions_new::{Instruction, TargetType, Value},
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
        Ok(Instruction::Assign(
            TargetType::Accumulator(0),
            Value::Constant(5)
        ))
    );
}

#[test]
fn test_run_assign_accumulator_from_constant() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::Assign(TargetType::Accumulator(0), Value::Constant(10))
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 10);
}

#[test]
fn test_parse_assign_accumulator_from_accumulator() {
    assert_eq!(
        Instruction::try_from("a0 := a1"),
        Ok(Instruction::Assign(
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
    Instruction::Assign(TargetType::Accumulator(0), Value::Accumulator(1))
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 10);
}

#[test]
fn test_parse_assign_accumulator_from_memory_cell() {
    assert_eq!(
        Instruction::try_from("a0 := p(h1)"),
        Ok(Instruction::Assign(
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
    Instruction::Assign(
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
        Ok(Instruction::Calc(
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
    Instruction::Calc(
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
        Ok(Instruction::Calc(
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
    Instruction::Calc(
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
        Ok(Instruction::Calc(
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
    Instruction::Calc(
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
        Ok(Instruction::Calc(
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
    Instruction::Calc(
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
        Ok(Instruction::Calc(
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
    Instruction::Calc(
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
        Ok(Instruction::Calc(
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
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::Accumulator(1),
        Operation::Sub,
        Value::MemoryCell("h1".to_string()),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 5);
}

//TODO Add tests for all other calc instruction combinations

#[test]
fn test_parse_cmp() {
    assert_eq!(
        Instruction::try_from("if a0 != a1 then goto loop"),
        Ok(Instruction::Cmp(
            Value::Accumulator(0),
            Comparison::NotEqual,
            Value::Accumulator(1),
            "loop".to_string()
        ))
    );
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
