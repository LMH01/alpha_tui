use std::{io, thread, time::Duration, process::exit};

use clap::Parser;
use cli::Args;
use tui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders}};
use utils::read_file;

use crate::{instructions::Instruction, runtime::{Runtime, RuntimeArgs, RuntimeBuilder}, base::{Operation, Comparison}};

/// Contains all required data types used to run programs
mod base;
/// Program execution
mod runtime;
/// Supported instructions
mod instructions;
/// Command line parsing
mod cli;
/// Utility functions
mod utils;

/// Used to set the maximum number of accumulators.
///
/// Should be at least 1.
const ACCUMULATORS: usize = 4;
/// Used to set the available memory cells.
const MEMORY_CELL_LABELS: &'static [&'static str] = &["a", "b", "c", "d", "e", "f", "w", "x", "y", "z", "h1", "h2", "h3", "h4"];

fn main() {
    let args = Args::parse();
    
    let instructions = match read_file(&args.input) {
        Ok(i) => i,
        Err(e) => {
            println!("Unable to read file: {}", e);
            exit(-1);
        },
    };
    println!("Building program");
    let mut rb = RuntimeBuilder::new_default();
    match rb.build_instructions(&instructions.iter().map(|s| s.as_str()).collect()) {
        Ok(_) => (),
        Err(e) => {
            println!("{e}");
            exit(-1);
        },
    };
    println!("Building runtime");
    let mut rt = match rb.build() {
        Ok(rt) => rt,
        Err(e) => {
            println!("Unable to build runtime: {:?}", e);
            exit(-1);
        },
    };
    println!("Running program");
    println!("----- Program start -----");
    match rt.run() {
        Ok(_) => {
            println!("----- Program end -----");
            rt.debug();
            println!("Program run successfully")
        },
        Err(e) => {
            println!("Runtime error: {}", e);
            exit(-1);
        }
    };
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

    //let instructions = vec![
    //    Instruction::AssignAccumulatorValue(0, 5),
    //    Instruction::AssignAccumulatorValue(1, 10),
    //    Instruction::AssignAccumulatorValue(2, 7),
    //    Instruction::Push(),
    //    Instruction::Pop(),
    //    Instruction::AssignMemoryCellValueFromAccumulator("c".to_string(),1),
    //    Instruction::AssignAccumulatorValueFromMemoryCell(3, "c".to_string()),
    //    Instruction::AssignAccumulatorValueFromAccumulator(0, 2),
    //    Instruction::AssignMemoryCellValue("f".to_string(), 17),
    //    Instruction::PrintAccumulators(),
    //    Instruction::PrintMemoryCells(),
    //    Instruction::PrintStack(),
    //];
    //let mut runtime_builder = RuntimeBuilder::new_default();
    //runtime_builder.set_instructions(instructions);
    //let mut runtime = runtime_builder.build().expect("Unable to build runtime!");
    //runtime.run().unwrap();

    //let mut builder = RuntimeBuilder::new_default();
    //let mut instructions = Vec::new();
    //instructions.push("a0 := af2 + 10");
    //let res = builder.build_instructions(&instructions);
    //if res.is_err() {
    //    println!("{}", res.unwrap_err());
    //}
    
}
