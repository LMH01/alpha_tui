use std::collections::HashMap;

use crate::{
    base::{Accumulator, Comparison, MemoryCell, Operation},
    instructions::{
        assign_index_memory_cell, assign_index_memory_cell_from_value, Identifier,
        IndexMemoryCellIndexType, Instruction, TargetType, Value, ACCUMULATOR_IDENTIFIER,
        COMPARISON_IDENTIFIER, CONSTANT_IDENTIFIER, GAMMA_IDENTIFIER, INDEX_MEMORY_CELL_IDENTIFIER,
        MEMORY_CELL_IDENTIFIER, OPERATOR_IDENTIFIER,
    },
    runtime::{error_handling::RuntimeErrorType, ControlFlow, RuntimeMemory, RuntimeSettings},
    utils::test_utils,
};

/// Used to set the available memory cells during testing.
const TEST_MEMORY_CELL_LABELS: &[&str] = &[
    "a", "b", "c", "d", "e", "f", "w", "x", "y", "z", "h1", "h2", "h3", "h4",
];

/// Returns runtime settings, configured for these test functions
fn setup_runtime_settings() -> RuntimeSettings {
    RuntimeSettings::default()
}

#[test]
fn test_instruction_comparison() {
    assert_eq!(
        Instruction::JumpIf(
            Value::Constant(0),
            Comparison::Eq,
            Value::Constant(0),
            "label".to_string()
        )
        .comparison(),
        Some(&Comparison::Eq)
    );
    assert_eq!(
        Instruction::Assign(TargetType::Gamma, Value::Constant(0)).comparison(),
        None
    );
}

#[test]
fn test_instruction_operation() {
    assert_eq!(
        Instruction::Calc(
            TargetType::Gamma,
            Value::Constant(0),
            Operation::Add,
            Value::Constant(0)
        )
        .operation(),
        Some(&Operation::Add)
    );
    assert_eq!(
        Instruction::StackOp(Operation::Add).operation(),
        Some(&Operation::Add)
    );
    assert_eq!(
        Instruction::Assign(TargetType::Gamma, Value::Constant(0)).operation(),
        None
    );
}

#[test]
fn test_parse_assign_accumulator_from_constant() {
    assert_eq!(
        Instruction::try_from("a0 := 5"),
        Ok(Instruction::Assign(
            TargetType::Accumulator(0),
            Value::Constant(5)
        ))
    );
    assert_eq!(
        Instruction::try_from("a := 5"),
        Ok(Instruction::Assign(
            TargetType::Accumulator(0),
            Value::Constant(5)
        ))
    );
    assert_eq!(
        Instruction::try_from("α0 := 5"),
        Ok(Instruction::Assign(
            TargetType::Accumulator(0),
            Value::Constant(5)
        ))
    );
}

#[test]
fn test_run_assign_accumulator_from_constant() {
    let mut runtime_memory = setup_runtime_memory();
    let runtime_settings = setup_runtime_settings();
    let mut control_flow = ControlFlow::new();
    Instruction::Assign(TargetType::Accumulator(0), Value::Constant(10))
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        10
    );
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
fn test_parse_assign_memory_cell_from_constant() {
    assert_eq!(
        Instruction::try_from("ρ(h1) := 5"),
        Ok(Instruction::Assign(
            TargetType::MemoryCell("h1".to_string()),
            Value::Constant(5)
        ))
    );
}

#[test]
fn test_run_assign_accumulator_from_accumulator() {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.accumulators.get_mut(&1).unwrap().data = Some(10);
    Instruction::Assign(TargetType::Accumulator(0), Value::Accumulator(1))
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        10
    );
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
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.memory_cells.get_mut("h1").unwrap().data = Some(10);
    Instruction::Assign(
        TargetType::Accumulator(0),
        Value::MemoryCell("h1".to_string()),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        10
    );
}

#[test]
fn test_parse_assign_gamma() {
    assert_eq!(
        Instruction::try_from("y := 5"),
        Ok(Instruction::Assign(TargetType::Gamma, Value::Constant(5)))
    );
    assert_eq!(
        Instruction::try_from("γ := 5"),
        Ok(Instruction::Assign(TargetType::Gamma, Value::Constant(5)))
    );
}

#[test]
fn test_run_assign_gamma() {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.gamma = Some(None);
    Instruction::Assign(TargetType::Gamma, Value::Constant(5))
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    assert_eq!(runtime_memory.gamma, Some(Some(5)));
}

#[test]
fn test_parse_calc_gamma() {
    assert_eq!(
        Instruction::try_from("y := y + y"),
        Ok(Instruction::Calc(
            TargetType::Gamma,
            Value::Gamma,
            Operation::Add,
            Value::Gamma
        ))
    );
    assert_eq!(
        Instruction::try_from("γ := γ + γ"),
        Ok(Instruction::Calc(
            TargetType::Gamma,
            Value::Gamma,
            Operation::Add,
            Value::Gamma
        ))
    );
}

#[test]
fn test_run_calc_gamma() {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.gamma = Some(None);
    Instruction::Calc(
        TargetType::Gamma,
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.gamma, Some(Some(10)));
    Instruction::Calc(
        TargetType::Gamma,
        Value::Gamma,
        Operation::Add,
        Value::Gamma,
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.gamma, Some(Some(20)));
}

#[test]
fn test_parse_assign_index_memory_cell() {
    assert_eq!(
        Instruction::try_from("p(5) := 5"),
        Ok(Instruction::Assign(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Direct(5)),
            Value::Constant(5)
        ))
    );
    assert_eq!(
        Instruction::try_from("p(p(5)) := 5"),
        Ok(Instruction::Assign(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Index(5)),
            Value::Constant(5)
        ))
    );
    assert_eq!(
        Instruction::try_from("p(y) := 5"),
        Ok(Instruction::Assign(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Gamma),
            Value::Constant(5)
        ))
    );
    assert_eq!(
        Instruction::try_from("p(p(h1)) := 5"),
        Ok(Instruction::Assign(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::MemoryCell("h1".to_string())),
            Value::Constant(5)
        ))
    );
    assert_eq!(
        Instruction::try_from("p(a0) := 5"),
        Ok(Instruction::Assign(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0)),
            Value::Constant(5)
        ))
    );
}

#[test]
fn test_run_assign_index_memory_cell() {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    Instruction::Assign(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Direct(5)),
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.index_memory_cells.get(&5), Some(&Some(5)));

    runtime_memory.index_memory_cells.insert(1, Some(1));
    Instruction::Assign(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Index(1)),
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.index_memory_cells.get(&1), Some(&Some(5)));

    runtime_memory.index_memory_cells.insert(2, Some(1));
    runtime_memory.memory_cells.get_mut("h1").unwrap().data = Some(2);
    Instruction::Assign(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::MemoryCell("h1".to_string())),
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.index_memory_cells.get(&2), Some(&Some(5)));

    runtime_memory.index_memory_cells.insert(3, Some(4));
    runtime_memory.accumulators.get_mut(&0).unwrap().data = Some(3);
    Instruction::Assign(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0)),
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.index_memory_cells.get(&3), Some(&Some(5)));

    runtime_memory.index_memory_cells.insert(4, Some(0));
    runtime_memory.gamma = Some(Some(4));
    Instruction::Assign(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Gamma),
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.index_memory_cells.get(&4), Some(&Some(5)));
}

#[test]
fn test_parse_calc_index_memory_cell() {
    assert_eq!(
        Instruction::try_from("p(5) := 1 + 3"),
        Ok(Instruction::Calc(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Direct(5)),
            Value::Constant(1),
            Operation::Add,
            Value::Constant(3)
        ))
    );
    assert_eq!(
        Instruction::try_from("p(p(5)) := 1 + 3"),
        Ok(Instruction::Calc(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Index(5)),
            Value::Constant(1),
            Operation::Add,
            Value::Constant(3)
        ))
    );
    assert_eq!(
        Instruction::try_from("p(y) := 5 + 5"),
        Ok(Instruction::Calc(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Gamma),
            Value::Constant(5),
            Operation::Add,
            Value::Constant(5)
        ))
    );
    assert_eq!(
        Instruction::try_from("p(p(h1)) := 1 + 3"),
        Ok(Instruction::Calc(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::MemoryCell("h1".to_string())),
            Value::Constant(1),
            Operation::Add,
            Value::Constant(3)
        ))
    );
    assert_eq!(
        Instruction::try_from("p(a0) := 5 + 5"),
        Ok(Instruction::Calc(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0)),
            Value::Constant(5),
            Operation::Add,
            Value::Constant(5)
        ))
    );
}

#[test]
fn test_run_calc_index_memory_cell() {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    Instruction::Calc(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Direct(5)),
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.index_memory_cells.get(&5), Some(&Some(10)));

    runtime_memory.index_memory_cells.insert(1, Some(1));
    Instruction::Calc(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Index(1)),
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.index_memory_cells.get(&1), Some(&Some(10)));

    runtime_memory.index_memory_cells.insert(2, Some(1));
    runtime_memory.memory_cells.get_mut("h1").unwrap().data = Some(2);
    Instruction::Calc(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::MemoryCell("h1".to_string())),
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.index_memory_cells.get(&2), Some(&Some(10)));

    runtime_memory.index_memory_cells.insert(3, Some(1));
    runtime_memory.accumulators.get_mut(&0).unwrap().data = Some(3);
    Instruction::Calc(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0)),
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.index_memory_cells.get(&3), Some(&Some(10)));

    runtime_memory.index_memory_cells.insert(4, Some(1));
    runtime_memory.gamma = Some(Some(4));
    Instruction::Calc(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Gamma),
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(runtime_memory.index_memory_cells.get(&4), Some(&Some(10)));
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
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.memory_cells.get_mut("h1").unwrap().data = Some(10);
    runtime_memory.memory_cells.get_mut("h2").unwrap().data = Some(10);
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::MemoryCell("h1".to_string()),
        Operation::Mul,
        Value::MemoryCell("h2".to_string()),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        100
    );
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
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.memory_cells.get_mut("h1").unwrap().data = Some(10);
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::MemoryCell("h1".to_string()),
        Operation::Mul,
        Value::Constant(10),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        100
    );
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

    assert_eq!(
        Instruction::try_from("a := p(h1) - a"),
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
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.memory_cells.get_mut("h1").unwrap().data = Some(10);
    runtime_memory.accumulators.get_mut(&1).unwrap().data = Some(10);
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::MemoryCell("h1".to_string()),
        Operation::Sub,
        Value::Accumulator(1),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        0
    );
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
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.accumulators.get_mut(&1).unwrap().data = Some(10);
    runtime_memory.accumulators.get_mut(&2).unwrap().data = Some(5);
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::Accumulator(1),
        Operation::Div,
        Value::Accumulator(2),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        2
    );
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
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.accumulators.get_mut(&1).unwrap().data = Some(10);
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::Accumulator(1),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        15
    );
}

#[test]
fn test_parse_calc_accumulator_with_accumulator_memory_cell() {
    assert_eq!(
        Instruction::try_from("a0 := a1 * p(h1)"),
        Ok(Instruction::Calc(
            TargetType::Accumulator(0),
            Value::Accumulator(1),
            Operation::Mul,
            Value::MemoryCell("h1".to_string())
        ))
    );
}

#[test]
fn test_run_calc_accumulator_with_accumulator_memory_cell() {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.accumulators.get_mut(&1).unwrap().data = Some(10);
    runtime_memory.memory_cells.get_mut("h1").unwrap().data = Some(5);
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::Accumulator(1),
        Operation::Sub,
        Value::MemoryCell("h1".to_string()),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        5
    );
}

#[test]
fn test_run_cmp() {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    control_flow
        .instruction_labels
        .insert("loop".to_string(), 20);
    Instruction::Assign(TargetType::Accumulator(0), Value::Constant(20))
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    Instruction::JumpIf(
        Value::Accumulator(0),
        Comparison::Lt,
        Value::Constant(40),
        "loop".to_string(),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(control_flow.next_instruction_index, 20);
    control_flow.next_instruction_index = 0;
    Instruction::JumpIf(
        Value::Accumulator(0),
        Comparison::Eq,
        Value::Constant(40),
        "loop".to_string(),
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .unwrap();
    assert_eq!(control_flow.next_instruction_index, 0);
    assert!(Instruction::JumpIf(
        Value::Accumulator(0),
        Comparison::Lt,
        Value::Constant(40),
        "none".to_string()
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .is_err());
    assert!(Instruction::JumpIf(
        Value::Accumulator(0),
        Comparison::Eq,
        Value::Constant(40),
        "none".to_string()
    )
    .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
    .is_ok());
}

#[test]
fn test_parse_cmp() {
    assert_eq!(
        Instruction::try_from("if a0 != a1 then goto loop"),
        Ok(Instruction::JumpIf(
            Value::Accumulator(0),
            Comparison::Neq,
            Value::Accumulator(1),
            "loop".to_string()
        ))
    );
}

//Add run test for cmp

#[test]
fn test_parse_goto() {
    assert_eq!(
        Instruction::try_from("goto loop"),
        Ok(Instruction::Goto("loop".to_string()))
    );
}

#[test]
fn test_run_goto() {
    let mut runtime_memory = setup_empty_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    control_flow
        .instruction_labels
        .insert("loop".to_string(), 5);
    Instruction::Goto("loop".to_string())
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    assert_eq!(control_flow.next_instruction_index, 5);
}

#[test]
fn test_stack() {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    Instruction::Assign(TargetType::Accumulator(0), Value::Constant(5))
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    Instruction::Push
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    Instruction::Assign(TargetType::Accumulator(0), Value::Constant(10))
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    Instruction::Push
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    assert_eq!(runtime_memory.stack, vec![5, 10]);
    Instruction::Pop
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        10
    );
    Instruction::Pop
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    assert_eq!(
        runtime_memory.accumulators.get(&0).unwrap().data.unwrap(),
        5
    );
    assert_eq!(runtime_memory.stack.len(), 0);
}

#[test]
fn test_parse_push() {
    assert_eq!(Instruction::try_from("push"), Ok(Instruction::Push));
}

#[test]
fn test_parse_pop() {
    assert_eq!(Instruction::try_from("pop"), Ok(Instruction::Pop));
}

#[test]
fn test_run_stack_op() {
    run_stack_op(Operation::Add, 15);
    run_stack_op(Operation::Sub, 5);
    run_stack_op(Operation::Mul, 50);
    run_stack_op(Operation::Div, 2);
    run_stack_op(Operation::Mod, 0);
}

#[test]
fn test_parse_stack_op() {
    assert_eq!(
        Instruction::try_from("stack+"),
        Ok(Instruction::StackOp(Operation::Add))
    );
    assert_eq!(
        Instruction::try_from("stack-"),
        Ok(Instruction::StackOp(Operation::Sub))
    );
    assert_eq!(
        Instruction::try_from("stack*"),
        Ok(Instruction::StackOp(Operation::Mul))
    );
    assert_eq!(
        Instruction::try_from("stack/"),
        Ok(Instruction::StackOp(Operation::Div))
    );
    assert_eq!(
        Instruction::try_from("stack%"),
        Ok(Instruction::StackOp(Operation::Mod))
    );
    assert_eq!(
        Instruction::try_from("stack +"),
        Ok(Instruction::StackOp(Operation::Add))
    );
    assert_eq!(
        Instruction::try_from("stack -"),
        Ok(Instruction::StackOp(Operation::Sub))
    );
    assert_eq!(
        Instruction::try_from("stack *"),
        Ok(Instruction::StackOp(Operation::Mul))
    );
    assert_eq!(
        Instruction::try_from("stack /"),
        Ok(Instruction::StackOp(Operation::Div))
    );
    assert_eq!(
        Instruction::try_from("stack %"),
        Ok(Instruction::StackOp(Operation::Mod))
    );
}

fn run_stack_op(op: Operation, result: i32) {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    runtime_memory.accumulators.get_mut(&0).unwrap().data = Some(10);
    Instruction::Push
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    runtime_memory.accumulators.get_mut(&0).unwrap().data = Some(5);
    Instruction::Push
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    Instruction::StackOp(op)
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    assert_eq!(runtime_memory.stack.pop(), Some(result));
}

#[test]
fn test_run_call() {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    control_flow
        .instruction_labels
        .insert("function".to_string(), 10);
    Instruction::Call("function".to_string())
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    assert_eq!(control_flow.next_instruction_index, 10);
    assert_eq!(control_flow.call_stack.pop(), Some(0));
}

#[test]
fn test_parse_call() {
    assert_eq!(
        Instruction::try_from("call function"),
        Ok(Instruction::Call("function".to_string()))
    );
}

#[test]
fn test_run_return() {
    let mut runtime_memory = setup_runtime_memory();
    let mut control_flow = ControlFlow::new();
    let runtime_settings = setup_runtime_settings();
    control_flow
        .instruction_labels
        .insert("function".to_string(), 10);
    Instruction::Call("function".to_string())
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    Instruction::Return
        .run(&mut runtime_memory, &mut control_flow, &runtime_settings)
        .unwrap();
    assert_eq!(control_flow.next_instruction_index, 0);
}

#[test]
fn test_parse_return() {
    assert_eq!(Instruction::try_from("return"), Ok(Instruction::Return));
}

#[test]
fn test_parsing_with_semicolon() {
    assert_eq!(Instruction::try_from("return;"), Ok(Instruction::Return));
    assert_eq!(
        Instruction::try_from("a := 5;"),
        Ok(Instruction::Assign(
            TargetType::Accumulator(0),
            Value::Constant(5)
        ))
    );
    assert_eq!(
        Instruction::try_from("a := 5 * 5;"),
        Ok(Instruction::Calc(
            TargetType::Accumulator(0),
            Value::Constant(5),
            Operation::Mul,
            Value::Constant(5)
        ))
    );
    assert_eq!(
        Instruction::try_from("call label;"),
        Ok(Instruction::Call("label".to_string()))
    );
    assert_eq!(
        Instruction::try_from("goto label;"),
        Ok(Instruction::Goto("label".to_string()))
    );
}

#[test]
fn test_alternative_assignment_parsing() {
    assert_eq!(
        Instruction::try_from("a = 5;"),
        Ok(Instruction::Assign(
            TargetType::Accumulator(0),
            Value::Constant(5)
        ))
    );
    assert_eq!(
        Instruction::try_from("a = 5 * 5;"),
        Ok(Instruction::Calc(
            TargetType::Accumulator(0),
            Value::Constant(5),
            Operation::Mul,
            Value::Constant(5)
        ))
    );
}

#[test]
fn test_example_program_memory_cells() {
    let instructions = r#"
p(a) := 5
p(b) := 2
p(c) := 3
p(d) := 9
p(w) := 4
p(x) := 8
p(yy) := 3
p(z) := 2
p(h1) := p(a) * p(w)
p(h2) := p(b) * p(yy)
p(h3) := p(a) * p(x)
p(h4) := p(b) * p(z)
p(a) := p(h1) + p(h2)
p(b) := p(h3) + p(h4)
p(h1) := p(c) * p(w)
p(h2) := p(d) * p(yy)
p(h3) := p(c) * p(x)
p(h4) := p(d) * p(z)
p(c) := p(h1) + p(h2)
p(d) := p(h3) + p(h4)
    "#;
    let mut runtime = test_utils::runtime_from_str(&instructions).unwrap();
    runtime.run().unwrap();
    assert_eq!(
        runtime
            .runtime_memory()
            .memory_cells
            .get("a")
            .unwrap()
            .data
            .unwrap(),
        26
    );
    assert_eq!(
        runtime
            .runtime_memory()
            .memory_cells
            .get("b")
            .unwrap()
            .data
            .unwrap(),
        44
    );
    assert_eq!(
        runtime
            .runtime_memory()
            .memory_cells
            .get("c")
            .unwrap()
            .data
            .unwrap(),
        39
    );
    assert_eq!(
        runtime
            .runtime_memory()
            .memory_cells
            .get("d")
            .unwrap()
            .data
            .unwrap(),
        42
    );
}

#[test]
fn test_example_program_loop() {
    let program = r#"
a0 := 1
p(a) := 8
loop: a0 := a0 * 2
p(a) := p(a) - 1
a1 := p(a)
if a1 > 0 then goto loop
    "#;
    let mut rt = test_utils::runtime_from_str(&program).unwrap();
    rt.run().unwrap();
    assert_eq!(
        rt.runtime_memory()
            .accumulators
            .get(&0)
            .unwrap()
            .data
            .unwrap(),
        256
    );
}

#[test]
fn test_example_program_functions() {
    let program = r#"
func:
p(h1) := 5
p(h2) := 10
p(h3) := p(h1) * p(h2)
return

main:
call func
a := p(h3)
return
    "#;
    let mut rt = test_utils::runtime_from_str(&program).unwrap();
    rt.run().unwrap();
    assert_eq!(
        rt.runtime_memory()
            .accumulators
            .get(&0)
            .unwrap()
            .data
            .unwrap(),
        50
    );
}

/// Sets up runtime runtime_memory in a consistent way because the default implementation for memory cells and accumulators is configgurable.
fn setup_runtime_memory() -> RuntimeMemory {
    let mut runtime_memory = RuntimeMemory::new_debug(TEST_MEMORY_CELL_LABELS);
    runtime_memory.memory_cells = HashMap::new();
    runtime_memory
        .memory_cells
        .insert("h1".to_string(), MemoryCell::new("h1"));
    runtime_memory
        .memory_cells
        .insert("h2".to_string(), MemoryCell::new("h2"));
    runtime_memory.accumulators = HashMap::new();
    runtime_memory.accumulators.insert(0, Accumulator::new(0));
    runtime_memory.accumulators.insert(1, Accumulator::new(1));
    runtime_memory.accumulators.insert(2, Accumulator::new(2));
    runtime_memory
}

/// Sets up runtime runtime_memory where no memory cells or accumulators are set.
fn setup_empty_runtime_memory() -> RuntimeMemory {
    let mut runtime_memory = RuntimeMemory::new_debug(TEST_MEMORY_CELL_LABELS);
    runtime_memory.accumulators = HashMap::new();
    runtime_memory.memory_cells = HashMap::new();
    runtime_memory
}

#[test]
fn test_try_target_type_from_string_usize_usize_tuple() {
    assert_eq!(
        TargetType::try_from((&"a5".to_string(), (0, 4))),
        Ok(TargetType::Accumulator(5))
    );
    assert_eq!(
        TargetType::try_from((&"p(h1)".to_string(), (0, 4))),
        Ok(TargetType::MemoryCell("h1".to_string()))
    );
    assert_eq!(
        TargetType::try_from((&"p(10)".to_string(), (0, 4))),
        Ok(TargetType::IndexMemoryCell(
            IndexMemoryCellIndexType::Direct(10)
        ))
    );
    assert_eq!(
        TargetType::try_from((&"p(p(10))".to_string(), (0, 7))),
        Ok(TargetType::IndexMemoryCell(
            IndexMemoryCellIndexType::Index(10)
        ))
    );
    assert_eq!(
        TargetType::try_from((&"p(p(h1))".to_string(), (0, 7))),
        Ok(TargetType::IndexMemoryCell(
            IndexMemoryCellIndexType::MemoryCell("h1".to_string())
        ))
    );
}

#[test]
fn test_try_value_from_string_usize_usize_tuple() {
    assert_eq!(
        Value::try_from((&"5".to_string(), (0, 4))),
        Ok(Value::Constant(5))
    );
    assert_eq!(
        Value::try_from((&"a5".to_string(), (0, 4))),
        Ok(Value::Accumulator(5))
    );
    assert_eq!(
        Value::try_from((&"p(h1)".to_string(), (0, 4))),
        Ok(Value::MemoryCell("h1".to_string()))
    );
    assert_eq!(
        Value::try_from((&"p(10)".to_string(), (0, 4))),
        Ok(Value::IndexMemoryCell(IndexMemoryCellIndexType::Direct(10)))
    );
    assert_eq!(
        Value::try_from((&"p(p(10))".to_string(), (0, 7))),
        Ok(Value::IndexMemoryCell(IndexMemoryCellIndexType::Index(10)))
    );
    assert_eq!(
        Value::try_from((&"p(p(h1))".to_string(), (0, 7))),
        Ok(Value::IndexMemoryCell(
            IndexMemoryCellIndexType::MemoryCell("h1".to_string())
        ))
    );
}

#[test]
fn test_assign_index_memory_cell() {
    let mut runtime_memory = RuntimeMemory::new_debug(&[&""]);
    let mut runtime_settings = setup_runtime_settings();
    runtime_settings.autodetect_index_memory_cells = true;
    assert_eq!(
        assign_index_memory_cell(&mut runtime_memory, &runtime_settings, 0, 5),
        Ok(())
    );
    runtime_settings.autodetect_index_memory_cells = false;
    assert_eq!(
        assign_index_memory_cell(&mut runtime_memory, &runtime_settings, 1, 5),
        Err(RuntimeErrorType::IndexMemoryCellDoesNotExist(1))
    );
}

#[test]
fn test_assign_index_memory_cell_from_value() {
    let mut runtime_memory = RuntimeMemory::new_debug(&[&""]);
    let mut runtime_settings = setup_runtime_settings();
    runtime_settings.autodetect_index_memory_cells = true;
    assert_eq!(
        assign_index_memory_cell_from_value(
            &mut runtime_memory,
            &runtime_settings,
            0,
            &Value::Constant(5)
        ),
        Ok(())
    );
    runtime_settings.autodetect_index_memory_cells = false;
    assert_eq!(
        assign_index_memory_cell_from_value(
            &mut runtime_memory,
            &runtime_settings,
            1,
            &Value::Constant(5)
        ),
        Err(RuntimeErrorType::IndexMemoryCellDoesNotExist(1))
    );
}

#[test]
fn test_instruction_display() {
    assert_eq!(
        format!(
            "{}",
            Instruction::Assign(TargetType::Accumulator(0), Value::Constant(5))
        ),
        "a0 := 5".to_string()
    );
    assert_eq!(
        format!(
            "{}",
            Instruction::Calc(
                TargetType::Accumulator(0),
                Value::Constant(5),
                Operation::Add,
                Value::MemoryCell("h1".to_string())
            )
        ),
        "a0 := 5 + p(h1)".to_string()
    );
    assert_eq!(
        format!("{}", Instruction::Call("fun".to_string())),
        "call fun".to_string()
    );
    assert_eq!(
        format!("{}", Instruction::Goto("loop".to_string())),
        "goto loop".to_string()
    );
    assert_eq!(
        format!(
            "{}",
            Instruction::JumpIf(
                Value::Accumulator(0),
                Comparison::Eq,
                Value::IndexMemoryCell(IndexMemoryCellIndexType::Direct(0)),
                "loop".to_string()
            )
        ),
        "if a0 == p(0) then goto loop".to_string()
    );
    assert_eq!(format!("{}", Instruction::Noop), "".to_string());
    assert_eq!(format!("{}", Instruction::Pop), "pop".to_string());
    assert_eq!(format!("{}", Instruction::Push), "push".to_string());
    assert_eq!(format!("{}", Instruction::Return), "return".to_string());
    assert_eq!(
        format!("{}", Instruction::StackOp(Operation::Mul)),
        "stack*".to_string()
    );
}

#[test]
fn test_value_display() {
    assert_eq!(format!("{}", Value::Accumulator(0)), "a0".to_string());
    assert_eq!(format!("{}", Value::Constant(5)), "5".to_string());
    assert_eq!(format!("{}", Value::Gamma), "y".to_string());
    assert_eq!(
        format!(
            "{}",
            Value::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0))
        ),
        "p(a0)".to_string()
    );
    assert_eq!(
        format!(
            "{}",
            Value::IndexMemoryCell(IndexMemoryCellIndexType::Direct(0))
        ),
        "p(0)".to_string()
    );
    assert_eq!(
        format!(
            "{}",
            Value::IndexMemoryCell(IndexMemoryCellIndexType::Gamma)
        ),
        "p(y)".to_string()
    );
    assert_eq!(
        format!(
            "{}",
            Value::IndexMemoryCell(IndexMemoryCellIndexType::Index(0))
        ),
        "p(p(0))".to_string()
    );
    assert_eq!(
        format!(
            "{}",
            Value::IndexMemoryCell(IndexMemoryCellIndexType::MemoryCell("h1".to_string()))
        ),
        "p(p(h1))".to_string()
    );
    assert_eq!(
        format!("{}", Value::MemoryCell("h1".to_string())),
        "p(h1)".to_string()
    );
}

#[test]
fn test_target_type_display() {
    assert_eq!(format!("{}", TargetType::Accumulator(0)), "a0".to_string());
    assert_eq!(format!("{}", TargetType::Gamma), "y".to_string());
    assert_eq!(
        format!(
            "{}",
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0))
        ),
        "p(a0)".to_string()
    );
    assert_eq!(
        format!(
            "{}",
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Direct(0))
        ),
        "p(0)".to_string()
    );
    assert_eq!(
        format!(
            "{}",
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Gamma)
        ),
        "p(y)".to_string()
    );
    assert_eq!(
        format!(
            "{}",
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Index(0))
        ),
        "p(p(0))".to_string()
    );
    assert_eq!(
        format!(
            "{}",
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::MemoryCell("h1".to_string()))
        ),
        "p(p(h1))".to_string()
    );
    assert_eq!(
        format!("{}", TargetType::MemoryCell("h1".to_string())),
        "p(h1)".to_string()
    );
}

#[test]
fn test_index_memory_cell_index_type_display() {
    assert_eq!(
        format!("{}", IndexMemoryCellIndexType::Accumulator(0)),
        "a0".to_string()
    );
    assert_eq!(
        format!("{}", IndexMemoryCellIndexType::Direct(0)),
        "0".to_string()
    );
    assert_eq!(
        format!("{}", IndexMemoryCellIndexType::Gamma),
        "y".to_string()
    );
    assert_eq!(
        format!("{}", IndexMemoryCellIndexType::Index(0)),
        "p(0)".to_string()
    );
    assert_eq!(
        format!("{}", IndexMemoryCellIndexType::MemoryCell("h1".to_string())),
        "p(h1)".to_string()
    );
}

#[test]
fn test_value_instruction_identifier() {
    assert_eq!(
        Value::Accumulator(0).identifier(),
        ACCUMULATOR_IDENTIFIER.to_string()
    );
    assert_eq!(
        Value::Constant(0).identifier(),
        CONSTANT_IDENTIFIER.to_string()
    );
    assert_eq!(Value::Gamma.identifier(), GAMMA_IDENTIFIER.to_string());
    assert_eq!(
        Value::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0)).identifier(),
        MEMORY_CELL_IDENTIFIER.to_string()
    );
    assert_eq!(
        Value::MemoryCell("h1".to_string()).identifier(),
        MEMORY_CELL_IDENTIFIER.to_string()
    );
}

#[test]
fn test_target_type_identifier() {
    assert_eq!(
        TargetType::Accumulator(0).identifier(),
        ACCUMULATOR_IDENTIFIER.to_string()
    );
    assert_eq!(TargetType::Gamma.identifier(), GAMMA_IDENTIFIER.to_string());
    assert_eq!(
        TargetType::MemoryCell("h1".to_string()).identifier(),
        MEMORY_CELL_IDENTIFIER.to_string()
    );
}

#[test]
fn test_target_type_identifier_imc() {
    assert_eq!(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(1)).identifier(),
        format!(
            "{}({})",
            INDEX_MEMORY_CELL_IDENTIFIER, ACCUMULATOR_IDENTIFIER
        )
    );
    assert_eq!(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Direct(1)).identifier(),
        format!("{}({})", INDEX_MEMORY_CELL_IDENTIFIER, CONSTANT_IDENTIFIER)
    );
    assert_eq!(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Gamma).identifier(),
        format!("{}({})", INDEX_MEMORY_CELL_IDENTIFIER, GAMMA_IDENTIFIER)
    );
    assert_eq!(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Index(1)).identifier(),
        format!(
            "{}({}({}))",
            INDEX_MEMORY_CELL_IDENTIFIER, INDEX_MEMORY_CELL_IDENTIFIER, CONSTANT_IDENTIFIER
        )
    );
    assert_eq!(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::MemoryCell("h1".to_string()))
            .identifier(),
        format!(
            "{}({})",
            INDEX_MEMORY_CELL_IDENTIFIER, MEMORY_CELL_IDENTIFIER
        )
    );
}

#[test]
fn test_comparison_identifier() {
    assert_eq!(Comparison::Eq.identifier(), COMPARISON_IDENTIFIER);
}

#[test]
fn test_operation_identifier() {
    assert_eq!(Operation::Add.identifier(), OPERATOR_IDENTIFIER);
}

#[test]
fn test_instruction_identifier() {
    assert_eq!(
        Instruction::Assign(TargetType::Accumulator(0), Value::Accumulator(0)).identifier(),
        "A := A".to_string()
    );
    assert_eq!(
        Instruction::Calc(
            TargetType::MemoryCell("h1".to_string()),
            Value::Constant(5),
            Operation::Add,
            Value::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0))
        )
        .identifier(),
        "M := C OP M".to_string()
    );
    assert_eq!(
        Instruction::Call("label".to_string()).identifier(),
        "call".to_string()
    );
    assert_eq!(
        Instruction::Goto("loop".to_string()).identifier(),
        "goto".to_string()
    );
    assert_eq!(
        Instruction::JumpIf(
            Value::Gamma,
            Comparison::Gt,
            Value::MemoryCell("h1".to_string()),
            "label".to_string()
        )
        .identifier(),
        "if Y CMP M then goto".to_string()
    );
    assert_eq!(Instruction::Noop.identifier(), "NOOP".to_string());
    assert_eq!(Instruction::Pop.identifier(), "pop".to_string());
    assert_eq!(Instruction::Push.identifier(), "push".to_string());
    assert_eq!(Instruction::Return.identifier(), "return".to_string());
    assert_eq!(
        Instruction::StackOp(Operation::Add).identifier(),
        "stackOP".to_string()
    );
}

#[test]
fn test_instruction_identifier_imc() {
    assert_eq!(
        Instruction::Assign(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0)),
            Value::Accumulator(0)
        )
        .identifier(),
        "M(A) := A".to_string()
    );
    assert_eq!(
        Instruction::Assign(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Direct(0)),
            Value::Accumulator(0)
        )
        .identifier(),
        "M(C) := A".to_string()
    );
    assert_eq!(
        Instruction::Assign(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Gamma),
            Value::Accumulator(0)
        )
        .identifier(),
        "M(Y) := A".to_string()
    );
    assert_eq!(
        Instruction::Assign(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Index(1)),
            Value::Accumulator(0)
        )
        .identifier(),
        "M(M(C)) := A".to_string()
    );
    assert_eq!(
        Instruction::Assign(
            TargetType::IndexMemoryCell(IndexMemoryCellIndexType::MemoryCell("h1".to_string())),
            Value::Accumulator(0)
        )
        .identifier(),
        "M(M) := A".to_string()
    );
}
