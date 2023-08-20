use std::collections::HashMap;

use clap::{Parser, ArgGroup};

#[derive(Parser, Debug)]
#[command(author = "LMH01", version, about, long_about = "debugger and runtime environment for the alpha notation used in my Systemnahme Informatik lecture")]
#[clap(group = ArgGroup::new("memory").args(["memory_cells", "memory_cell_file"]))]
pub struct Args {
    #[arg(
        short,
        long,
        long_help = "Specify the input file that contains the program",
        required = true
    )]
    pub input: String,
    #[arg(
        short,
        long,
        help = "Number of available accumulators",
        long_help = "Number of available accumulators.\nIf the value is too large it can happen that accumulators are not displayed in the gui.",
        required_if_eq("disable_memory_detection", "true"),
    )]
    pub accumulators: Option<u8>,
    #[arg(
        short,
        long,
        help = "Number of available memory cells",
        long_help = "Number of available memory cells.\nIf a large number of memory cells is specified, it can happen that some are not displayed in the gui.\nExample: -a a,b,c,d",
        value_delimiter = ',',
    )]
    pub memory_cells: Option<Vec<String>>,
    #[arg(
        long,
        help = "Load memory cells from a file",
        long_help = "Load memory cell values from a file.\nEach line contains a single memory cell in the following formatting: NAME=VALUE\nExample: h1=5\nEmpty cells can be set with: NAME\nExample: h2",
        conflicts_with = "memory_cells",
    )]
    pub memory_cell_file: Option<String>,
    #[arg(
        short,
        long,
        help = "Set to disable accumulator and memory_cell detection",
        long_help = "Set to disable accumulator and memory_cell detection.\nIf disabled, accumulators and memory cells won't be read from program,\ninstead they have to be specified using \"--accumulators\" and \"--memory-cells\" or \"--memory-cell-file\"",
        requires = "memory"
    )]
    pub disable_memory_detection: bool,
}
