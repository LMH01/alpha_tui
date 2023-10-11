use std::collections::{HashMap, HashSet};

use assert_cmd::Command;
use miette::{NamedSource, SourceOffset, SourceSpan};

use crate::{
    base::{Accumulator, Comparison, MemoryCell, Operation},
    instructions::{
        assign_index_memory_cell, assign_index_memory_cell_from_value,
        error_handling::{BuildAllowedInstructionsError, BuildProgramError, InstructionParseError},
        IndexMemoryCellIndexType, Instruction, TargetType, Value, InstructionWhitelist, ACCUMULATOR_IDENTIFIER, CONSTANT_IDENTIFIER, GAMMA_IDENTIFIER, MEMORY_CELL_IDENTIFIER, COMPARISON_IDENTIFIER, OPERATOR_IDENTIFIER,
    },
    runtime::{
        builder::RuntimeBuilder, error_handling::RuntimeErrorType, ControlFlow, RuntimeArgs,
    },
    utils::build_instructions_with_whitelist,
};

/// Used to set the available memory cells during testing.
const TEST_MEMORY_CELL_LABELS: &[&str] = &[
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
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::Assign(TargetType::Accumulator(0), Value::Constant(10))
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 10);
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
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    args.accumulators.get_mut(&1).unwrap().data = Some(10);
    Instruction::Assign(TargetType::Accumulator(0), Value::Accumulator(1))
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 10);
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
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 10);
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
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    args.gamma = Some(None);
    Instruction::Assign(TargetType::Gamma, Value::Constant(5))
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.gamma, Some(Some(5)));
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
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    args.gamma = Some(None);
    Instruction::Calc(
        TargetType::Gamma,
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.gamma, Some(Some(10)));
    Instruction::Calc(
        TargetType::Gamma,
        Value::Gamma,
        Operation::Add,
        Value::Gamma,
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.gamma, Some(Some(20)));
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
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::Assign(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Direct(5)),
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.index_memory_cells.get(&5), Some(&Some(5)));

    args.index_memory_cells.insert(1, Some(1));
    Instruction::Assign(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Index(1)),
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.index_memory_cells.get(&1), Some(&Some(5)));

    args.index_memory_cells.insert(2, Some(1));
    args.memory_cells.get_mut("h1").unwrap().data = Some(2);
    Instruction::Assign(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::MemoryCell("h1".to_string())),
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.index_memory_cells.get(&2), Some(&Some(5)));

    args.index_memory_cells.insert(3, Some(4));
    args.accumulators.get_mut(&0).unwrap().data = Some(3);
    Instruction::Assign(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0)),
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.index_memory_cells.get(&3), Some(&Some(5)));

    args.index_memory_cells.insert(4, Some(0));
    args.gamma = Some(Some(4));
    Instruction::Assign(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Gamma),
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.index_memory_cells.get(&4), Some(&Some(5)));
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
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::Calc(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Direct(5)),
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.index_memory_cells.get(&5), Some(&Some(10)));

    args.index_memory_cells.insert(1, Some(1));
    Instruction::Calc(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Index(1)),
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.index_memory_cells.get(&1), Some(&Some(10)));

    args.index_memory_cells.insert(2, Some(1));
    args.memory_cells.get_mut("h1").unwrap().data = Some(2);
    Instruction::Calc(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::MemoryCell("h1".to_string())),
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.index_memory_cells.get(&2), Some(&Some(10)));

    args.index_memory_cells.insert(3, Some(1));
    args.accumulators.get_mut(&0).unwrap().data = Some(3);
    Instruction::Calc(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0)),
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.index_memory_cells.get(&3), Some(&Some(10)));

    args.index_memory_cells.insert(4, Some(1));
    args.gamma = Some(Some(4));
    Instruction::Calc(
        TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Gamma),
        Value::Constant(5),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.index_memory_cells.get(&4), Some(&Some(10)));
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
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 100);
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
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 100);
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
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    args.memory_cells.get_mut("h1").unwrap().data = Some(10);
    args.accumulators.get_mut(&1).unwrap().data = Some(10);
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::MemoryCell("h1".to_string()),
        Operation::Sub,
        Value::Accumulator(1),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 0);
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
    args.accumulators.get_mut(&1).unwrap().data = Some(10);
    args.accumulators.get_mut(&2).unwrap().data = Some(5);
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::Accumulator(1),
        Operation::Div,
        Value::Accumulator(2),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 2);
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
    args.accumulators.get_mut(&1).unwrap().data = Some(10);
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::Accumulator(1),
        Operation::Add,
        Value::Constant(5),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 15);
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
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    args.accumulators.get_mut(&1).unwrap().data = Some(10);
    args.memory_cells.get_mut("h1").unwrap().data = Some(5);
    Instruction::Calc(
        TargetType::Accumulator(0),
        Value::Accumulator(1),
        Operation::Sub,
        Value::MemoryCell("h1".to_string()),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 5);
}

#[test]
fn test_run_cmp() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    control_flow
        .instruction_labels
        .insert("loop".to_string(), 20);
    Instruction::Assign(TargetType::Accumulator(0), Value::Constant(20))
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::JumpIf(
        Value::Accumulator(0),
        Comparison::Lt,
        Value::Constant(40),
        "loop".to_string(),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(control_flow.next_instruction_index, 20);
    control_flow.next_instruction_index = 0;
    Instruction::JumpIf(
        Value::Accumulator(0),
        Comparison::Eq,
        Value::Constant(40),
        "loop".to_string(),
    )
    .run(&mut args, &mut control_flow)
    .unwrap();
    assert_eq!(control_flow.next_instruction_index, 0);
    assert!(Instruction::JumpIf(
        Value::Accumulator(0),
        Comparison::Lt,
        Value::Constant(40),
        "none".to_string()
    )
    .run(&mut args, &mut control_flow)
    .is_err());
    assert!(Instruction::JumpIf(
        Value::Accumulator(0),
        Comparison::Eq,
        Value::Constant(40),
        "none".to_string()
    )
    .run(&mut args, &mut control_flow)
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
fn test_stack() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    Instruction::Assign(TargetType::Accumulator(0), Value::Constant(5))
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::Push.run(&mut args, &mut control_flow).unwrap();
    Instruction::Assign(TargetType::Accumulator(0), Value::Constant(10))
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::Push.run(&mut args, &mut control_flow).unwrap();
    assert_eq!(args.stack, vec![5, 10]);
    Instruction::Pop.run(&mut args, &mut control_flow).unwrap();
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 10);
    Instruction::Pop.run(&mut args, &mut control_flow).unwrap();
    assert_eq!(args.accumulators.get(&0).unwrap().data.unwrap(), 5);
    assert_eq!(args.stack.len(), 0);
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
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    args.accumulators.get_mut(&0).unwrap().data = Some(10);
    Instruction::Push.run(&mut args, &mut control_flow).unwrap();
    args.accumulators.get_mut(&0).unwrap().data = Some(5);
    Instruction::Push.run(&mut args, &mut control_flow).unwrap();
    Instruction::StackOp(op)
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.stack.pop(), Some(result));
}

#[test]
fn test_run_call() {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    control_flow
        .instruction_labels
        .insert("function".to_string(), 10);
    Instruction::Call("function".to_string())
        .run(&mut args, &mut control_flow)
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
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    control_flow
        .instruction_labels
        .insert("function".to_string(), 10);
    Instruction::Call("function".to_string())
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::Return
        .run(&mut args, &mut control_flow)
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
fn test_example_program_memory_cells() {
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
        Instruction::Assign(TargetType::MemoryCell("a".to_string()), Value::Constant(5)),
        Instruction::Assign(TargetType::MemoryCell("b".to_string()), Value::Constant(2)),
        Instruction::Assign(TargetType::MemoryCell("c".to_string()), Value::Constant(3)),
        Instruction::Assign(TargetType::MemoryCell("d".to_string()), Value::Constant(9)),
        Instruction::Assign(TargetType::MemoryCell("w".to_string()), Value::Constant(4)),
        Instruction::Assign(TargetType::MemoryCell("x".to_string()), Value::Constant(8)),
        Instruction::Assign(TargetType::MemoryCell("y".to_string()), Value::Constant(3)),
        Instruction::Assign(TargetType::MemoryCell("z".to_string()), Value::Constant(2)),
        Instruction::Calc(
            TargetType::MemoryCell("h1".to_string()),
            Value::MemoryCell("a".to_string()),
            Operation::Mul,
            Value::MemoryCell("w".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("h2".to_string()),
            Value::MemoryCell("b".to_string()),
            Operation::Mul,
            Value::MemoryCell("y".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("h3".to_string()),
            Value::MemoryCell("a".to_string()),
            Operation::Mul,
            Value::MemoryCell("x".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("h4".to_string()),
            Value::MemoryCell("b".to_string()),
            Operation::Mul,
            Value::MemoryCell("z".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("a".to_string()),
            Value::MemoryCell("h1".to_string()),
            Operation::Add,
            Value::MemoryCell("h2".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("b".to_string()),
            Value::MemoryCell("h3".to_string()),
            Operation::Add,
            Value::MemoryCell("h4".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("h1".to_string()),
            Value::MemoryCell("c".to_string()),
            Operation::Mul,
            Value::MemoryCell("w".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("h2".to_string()),
            Value::MemoryCell("d".to_string()),
            Operation::Mul,
            Value::MemoryCell("y".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("h3".to_string()),
            Value::MemoryCell("c".to_string()),
            Operation::Mul,
            Value::MemoryCell("x".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("h4".to_string()),
            Value::MemoryCell("d".to_string()),
            Operation::Mul,
            Value::MemoryCell("z".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("c".to_string()),
            Value::MemoryCell("h1".to_string()),
            Operation::Add,
            Value::MemoryCell("h2".to_string()),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("d".to_string()),
            Value::MemoryCell("h3".to_string()),
            Operation::Add,
            Value::MemoryCell("h4".to_string()),
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
fn test_example_program_memory_cells_text_parsing() {
    let mut runtime_args = RuntimeArgs::new_debug(TEST_MEMORY_CELL_LABELS);
    for _i in 1..=4 {
        runtime_args.add_accumulator();
    }
    runtime_args.add_storage_cell("aa");
    runtime_args.add_storage_cell("b");
    runtime_args.add_storage_cell("c");
    runtime_args.add_storage_cell("d");
    runtime_args.add_storage_cell("w");
    runtime_args.add_storage_cell("x");
    runtime_args.add_storage_cell("yy");
    runtime_args.add_storage_cell("z");
    runtime_args.add_storage_cell("h1");
    runtime_args.add_storage_cell("h2");
    runtime_args.add_storage_cell("h3");
    runtime_args.add_storage_cell("h4");
    let mut instructions = Vec::new();
    instructions.push("p(aa) := 5\n");
    instructions.push("p(b) := 2\n");
    instructions.push("p(c) := 3\n");
    instructions.push("p(d) := 9\n");
    instructions.push("p(w) := 4\n");
    instructions.push("p(x) := 8\n");
    instructions.push("p(yy) := 3\n");
    instructions.push("p(z) := 2\n");
    instructions.push("p(h1) := p(aa) * p(w)\n");
    instructions.push("p(h2) := p(b) * p(yy)\n");
    instructions.push("p(h3) := p(aa) * p(x)\n");
    instructions.push("p(h4) := p(b) * p(z)\n");
    instructions.push("p(aa) := p(h1) + p(h2)\n");
    instructions.push("p(b) := p(h3) + p(h4)\n");
    instructions.push("p(h1) := p(c) * p(w)\n");
    instructions.push("p(h2) := p(d) * p(yy)\n");
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
            .get("aa")
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
fn test_example_program_loop() {
    let instructions = vec![
        Instruction::Assign(TargetType::Accumulator(0), Value::Constant(1)),
        Instruction::Assign(TargetType::MemoryCell("a".to_string()), Value::Constant(8)),
        Instruction::Calc(
            TargetType::Accumulator(0),
            Value::Accumulator(0),
            Operation::Mul,
            Value::Constant(2),
        ),
        Instruction::Calc(
            TargetType::MemoryCell("a".to_string()),
            Value::MemoryCell("a".to_string()),
            Operation::Sub,
            Value::Constant(1),
        ),
        Instruction::Assign(
            TargetType::Accumulator(1),
            Value::MemoryCell("a".to_string()),
        ),
        Instruction::JumpIf(
            Value::Accumulator(1),
            Comparison::Gt,
            Value::Constant(0),
            "loop".to_string(),
        ),
    ];
    let mut runtime_builder = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
    runtime_builder.set_instructions(instructions);
    runtime_builder.add_label("loop".to_string(), 2).unwrap();
    let mut runtime = runtime_builder.build().expect("Unable to build runtime!");
    runtime.run().unwrap();
    assert_eq!(
        runtime
            .runtime_args()
            .accumulators
            .get(&0)
            .unwrap()
            .data
            .unwrap(),
        256
    );
}

#[test]
fn test_example_program_loop_text_parsing() {
    let mut instructions = Vec::new();
    instructions.push("a0 := 1");
    instructions.push("p(h1) := 8");
    instructions.push("loop: a0 := a0 * 2");
    instructions.push("p(h1) := p(h1) - 1");
    instructions.push("a1 := p(h1)");
    instructions.push("if a1 > 0 then goto loop");
    let mut runtime_builder = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
    let res = runtime_builder.build_instructions(&instructions, "test");
    println!("{:?}", res);
    assert!(res.is_ok());
    let mut runtime = runtime_builder.build().expect("Unable to build runtime!");
    runtime.run().unwrap();
    assert_eq!(
        runtime
            .runtime_args()
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
    let mut instructions = Vec::new();
    instructions.push("func:");
    instructions.push("p(h1) := 5");
    instructions.push("p(h2) := 10");
    instructions.push("p(h3) := p(h1) * p(h2)");
    instructions.push("return");
    instructions.push("");
    instructions.push("main:");
    instructions.push("call func");
    instructions.push("a := p(h3)");
    instructions.push("return");
    let mut runtime_builder = RuntimeBuilder::new_debug(TEST_MEMORY_CELL_LABELS);
    let res = runtime_builder.build_instructions(&instructions, "test");
    println!("{:?}", res);
    assert!(res.is_ok());
    let mut runtime = runtime_builder.build().expect("Unable to build runtime!");
    runtime.run().unwrap();
    assert_eq!(
        runtime
            .runtime_args()
            .accumulators
            .get(&0)
            .unwrap()
            .data
            .unwrap(),
        50
    );
}

/// Sets up runtime args in a consistent way because the default implementation for memory cells and accumulators is configgurable.
fn setup_runtime_args() -> RuntimeArgs {
    let mut args = RuntimeArgs::new_debug(TEST_MEMORY_CELL_LABELS);
    args.memory_cells = HashMap::new();
    args.memory_cells
        .insert("h1".to_string(), MemoryCell::new("h1"));
    args.memory_cells
        .insert("h2".to_string(), MemoryCell::new("h2"));
    args.accumulators = HashMap::new();
    args.accumulators.insert(0, Accumulator::new(0));
    args.accumulators.insert(1, Accumulator::new(1));
    args.accumulators.insert(2, Accumulator::new(2));
    args
}

/// Sets up runtime args where no memory cells or accumulators are set.
fn setup_empty_runtime_args() -> RuntimeArgs {
    let mut args = RuntimeArgs::new_debug(TEST_MEMORY_CELL_LABELS);
    args.accumulators = HashMap::new();
    args.memory_cells = HashMap::new();
    args
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
    let mut runtime_args = RuntimeArgs::new_debug(&[&""]);
    runtime_args.settings.enable_imc_auto_creation = true;
    assert_eq!(assign_index_memory_cell(&mut runtime_args, 0, 5), Ok(()));
    runtime_args.settings.enable_imc_auto_creation = false;
    assert_eq!(
        assign_index_memory_cell(&mut runtime_args, 1, 5),
        Err(RuntimeErrorType::IndexMemoryCellDoesNotExist(1))
    );
}

#[test]
fn test_assign_index_memory_cell_from_value() {
    let mut runtime_args = RuntimeArgs::new_debug(&[&""]);
    runtime_args.settings.enable_imc_auto_creation = true;
    assert_eq!(
        assign_index_memory_cell_from_value(&mut runtime_args, 0, &Value::Constant(5)),
        Ok(())
    );
    runtime_args.settings.enable_imc_auto_creation = false;
    assert_eq!(
        assign_index_memory_cell_from_value(&mut runtime_args, 1, &Value::Constant(5)),
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
    assert_eq!(Value::Accumulator(0).identifier(), ACCUMULATOR_IDENTIFIER.to_string());
    assert_eq!(Value::Constant(0).identifier(), CONSTANT_IDENTIFIER.to_string());
    assert_eq!(Value::Gamma.identifier(), GAMMA_IDENTIFIER.to_string());
    assert_eq!(Value::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0)).identifier(), MEMORY_CELL_IDENTIFIER.to_string());
    assert_eq!(Value::MemoryCell("h1".to_string()).identifier(), MEMORY_CELL_IDENTIFIER.to_string());
}

#[test]
fn test_target_type_identifier() {
    assert_eq!(TargetType::Accumulator(0).identifier(), ACCUMULATOR_IDENTIFIER.to_string());
    assert_eq!(TargetType::Gamma.identifier(), GAMMA_IDENTIFIER.to_string());
    assert_eq!(TargetType::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0)).identifier(), MEMORY_CELL_IDENTIFIER.to_string());
    assert_eq!(TargetType::MemoryCell("h1".to_string()).identifier(), MEMORY_CELL_IDENTIFIER.to_string());
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
    assert_eq!(Instruction::Assign(TargetType::Accumulator(0), Value::Accumulator(0)).identifier(), "A := A".to_string());
    assert_eq!(Instruction::Calc(TargetType::MemoryCell("h1".to_string()), Value::Constant(5), Operation::Add, Value::IndexMemoryCell(IndexMemoryCellIndexType::Accumulator(0))).identifier(), "M := C OP M".to_string());
    assert_eq!(Instruction::Call("label".to_string()).identifier(), "call".to_string());
    assert_eq!(Instruction::Goto("loop".to_string()).identifier(), "goto".to_string());
    assert_eq!(Instruction::JumpIf(Value::Gamma, Comparison::Gt, Value::MemoryCell("h1".to_string()), "label".to_string()).identifier(), "if Y CMP M then goto".to_string());
    assert_eq!(Instruction::Noop.identifier(), "NOOP".to_string());
    assert_eq!(Instruction::Pop.identifier(), "pop".to_string());
    assert_eq!(Instruction::Push.identifier(), "push".to_string());
    assert_eq!(Instruction::Return.identifier(), "return".to_string());
    assert_eq!(Instruction::StackOp(Operation::Add).identifier(), "stackOP".to_string());
}

#[test]
fn test_bai_error() {
    let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
    let assert = cmd
        .arg("load")
        .arg("tests/test_bai_error/program.alpha")
        .arg("--allowed-instructions").arg("tests/test_bai_error/allowed_instructions_a.txt")
        .assert();
    assert.stderr(r#"Error: build_program_error

  × when building program
  ╰─▶ build_program::instruction_not_allowed_error
      
        × instruction 'pop' in line '1' is not allowed
        help: Make sure that you include this type of instruction in the
      whitelist
              or use a different instruction.
              These types of instructions are allowed:
      
              push
      

"#);
}
