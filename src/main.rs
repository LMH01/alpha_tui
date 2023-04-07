use crate::{instructions::Instruction, runtime::Runner};

/// Contains all required data types used to run programs
mod base;
/// Program execution
mod runtime;
/// Supported instructions
mod instructions;

/// Used to set the maximum number of accumulators.
///
/// Should be at least 1.
const ACCUMULATORS: i32 = 4;
/// Used to set the available memory cells.
const MEMORY_CELL_LABELS: &'static [&'static str] = &["a", "b", "c", "d", "e", "f"];

fn main() {
    println!("Hello, world!");
    
    let instructions = vec![
        Instruction::AssignAccumulatorValue(0, 5),
        Instruction::AssignAccumulatorValue(1, 10),
        Instruction::AssignAccumulatorValue(2, 7),
        Instruction::Push(),
        Instruction::Pop(),
        Instruction::AssignMemoryCellValueFromAccumulator("c",1),
        Instruction::AssignAccumulatorValueFromMemoryCell(3, "c"),
        Instruction::AssignAccumulatorValueFromAccumulator(0, 2),
        Instruction::AssignMemoryCellValue("f", 17),
        Instruction::PrintAccumulators(),
        Instruction::PrintMemoryCells(),
        Instruction::PrintStack(),
    ];
    let mut runner = Runner::new(instructions);
    runner.run().unwrap();
}
