use std::collections::HashMap;

use crate::{
    base::{Accumulator, Comparison, MemoryCell, Operation},
    instructions::{Instruction, TargetType, Value},
    runtime::{ControlFlow, RuntimeArgs, builder::RuntimeBuilder},
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
    Instruction::Push
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::Assign(TargetType::Accumulator(0), Value::Constant(10))
        .run(&mut args, &mut control_flow)
        .unwrap();
    Instruction::Push
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.stack, vec![5, 10]);
    Instruction::Pop
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 10);
    Instruction::Pop
        .run(&mut args, &mut control_flow)
        .unwrap();
    assert_eq!(args.accumulators[0].data.unwrap(), 5);
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
    assert_eq!(Instruction::try_from("stack+"), Ok(Instruction::StackOp(Operation::Add)));
    assert_eq!(Instruction::try_from("stack-"), Ok(Instruction::StackOp(Operation::Sub)));
    assert_eq!(Instruction::try_from("stack*"), Ok(Instruction::StackOp(Operation::Mul)));
    assert_eq!(Instruction::try_from("stack/"), Ok(Instruction::StackOp(Operation::Div)));
    assert_eq!(Instruction::try_from("stack%"), Ok(Instruction::StackOp(Operation::Mod)));
}

fn run_stack_op(op: Operation, result: i32) {
    let mut args = setup_runtime_args();
    let mut control_flow = ControlFlow::new();
    args.accumulators[0].data = Some(10);
    Instruction::Push.run(&mut args, &mut control_flow).unwrap();
    args.accumulators[0].data = Some(5);
    Instruction::Push.run(&mut args, &mut control_flow).unwrap();
    Instruction::StackOp(op).run(&mut args, &mut control_flow).unwrap();
    assert_eq!(args.stack.pop(), Some(result));
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
        Instruction::Assign(TargetType::MemoryCell("a".to_string()), Value::Constant(5)),
        Instruction::Assign(TargetType::MemoryCell("b".to_string()), Value::Constant(2)),
        Instruction::Assign(TargetType::MemoryCell("c".to_string()), Value::Constant(3)),
        Instruction::Assign(TargetType::MemoryCell("d".to_string()), Value::Constant(9)),
        Instruction::Assign(TargetType::MemoryCell("w".to_string()), Value::Constant(4)),
        Instruction::Assign(TargetType::MemoryCell("x".to_string()), Value::Constant(8)),
        Instruction::Assign(TargetType::MemoryCell("y".to_string()), Value::Constant(3)),
        Instruction::Assign(TargetType::MemoryCell("z".to_string()), Value::Constant(2)),
        Instruction::Calc(TargetType::MemoryCell("h1".to_string()), Value::MemoryCell("a".to_string()), Operation::Mul, Value::MemoryCell("w".to_string())),
        Instruction::Calc(TargetType::MemoryCell("h2".to_string()), Value::MemoryCell("b".to_string()), Operation::Mul, Value::MemoryCell("y".to_string())),
        Instruction::Calc(TargetType::MemoryCell("h3".to_string()), Value::MemoryCell("a".to_string()), Operation::Mul, Value::MemoryCell("x".to_string())),
        Instruction::Calc(TargetType::MemoryCell("h4".to_string()), Value::MemoryCell("b".to_string()), Operation::Mul, Value::MemoryCell("z".to_string())),
        Instruction::Calc(TargetType::MemoryCell("a".to_string()), Value::MemoryCell("h1".to_string()), Operation::Add, Value::MemoryCell("h2".to_string())),
        Instruction::Calc(TargetType::MemoryCell("b".to_string()), Value::MemoryCell("h3".to_string()), Operation::Add, Value::MemoryCell("h4".to_string())),
        Instruction::Calc(TargetType::MemoryCell("h1".to_string()), Value::MemoryCell("c".to_string()), Operation::Mul, Value::MemoryCell("w".to_string())),
        Instruction::Calc(TargetType::MemoryCell("h2".to_string()), Value::MemoryCell("d".to_string()), Operation::Mul, Value::MemoryCell("y".to_string())),
        Instruction::Calc(TargetType::MemoryCell("h3".to_string()), Value::MemoryCell("c".to_string()), Operation::Mul, Value::MemoryCell("x".to_string())),
        Instruction::Calc(TargetType::MemoryCell("h4".to_string()), Value::MemoryCell("d".to_string()), Operation::Mul, Value::MemoryCell("z".to_string())),
        Instruction::Calc(TargetType::MemoryCell("c".to_string()), Value::MemoryCell("h1".to_string()), Operation::Add, Value::MemoryCell("h2".to_string())),
        Instruction::Calc(TargetType::MemoryCell("d".to_string()), Value::MemoryCell("h3".to_string()), Operation::Add, Value::MemoryCell("h4".to_string())),
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
        Instruction::Assign(TargetType::Accumulator(0), Value::Constant(1)),
        Instruction::Assign(TargetType::MemoryCell("a".to_string()), Value::Constant(8)),
        Instruction::Calc(TargetType::Accumulator(0), Value::Accumulator(0), Operation::Mul, Value::Constant(2)),
        Instruction::Calc(TargetType::MemoryCell("a".to_string()), Value::MemoryCell("a".to_string()), Operation::Sub, Value::Constant(1)),
        Instruction::Assign(TargetType::Accumulator(1), Value::MemoryCell("a".to_string())),
        Instruction::JumpIf(Value::Accumulator(1), Comparison::Gt, Value::Constant(0), "loop".to_string())
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

/// Sets up runtime args where no memory cells or accumulators are set.
fn setup_empty_runtime_args() -> RuntimeArgs {
    let mut args = RuntimeArgs::new_debug(TEST_MEMORY_CELL_LABELS);
    args.accumulators = Vec::new();
    args.memory_cells = HashMap::new();
    args
}
