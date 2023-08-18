use std::{io, thread, time::Duration};

use tui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders}};

use crate::{instructions::Instruction, runtime::{Runner, RuntimeArgs, RuntimeBuilder}, base::{Operation, Comparison}};

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
    
    //let stdout = io::stdout();
    //let backend = CrosstermBackend::new(stdout);
    //let mut terminal = Terminal::new(backend).unwrap();

    //terminal.draw(|f| {
    //    let size = f.size();
    //    let block = Block::default()
    //        .title("Block")
    //        .borders(Borders::ALL);
    //    f.render_widget(block, size);
    //}).unwrap();

    //thread::sleep(Duration::from_millis(5000));

    let instructions = vec![
        Instruction::AssignAccumulatorValue(0, 5),
        Instruction::AssignAccumulatorValue(1, 10),
        Instruction::AssignAccumulatorValue(2, 7),
        Instruction::Push(),
        Instruction::Pop(),
        Instruction::AssignMemoryCellValueFromAccumulator("c".to_string(),1),
        Instruction::AssignAccumulatorValueFromMemoryCell(3, "c".to_string()),
        Instruction::AssignAccumulatorValueFromAccumulator(0, 2),
        Instruction::AssignMemoryCellValue("f".to_string(), 17),
        Instruction::PrintAccumulators(),
        Instruction::PrintMemoryCells(),
        Instruction::PrintStack(),
    ];
    let mut runner = Runner::new(instructions);
    runner.run().unwrap();

    let mut builder = RuntimeBuilder::new_default();
    let mut instructions = Vec::new();
    instructions.push("a0 := df");
    let res = builder.build_instructions(&instructions);
    if res.is_err() {
        println!("{}", res.unwrap_err());
    }
    
}
