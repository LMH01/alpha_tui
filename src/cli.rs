use std::collections::HashMap;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author = "LMH01", version, about, long_about = None)]
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
        default_value = "4"
    )]
    pub accumulators: u8,
    #[arg(
        long,
        help = "Number of available memory cells",
        long_help = "Number of available memory cells.\nIf a large number of memory cells is specified, it can happen that some are not displayed in the gui.",
        value_delimiter = ',',
        default_value = "a,b,c,d,w,x,y,z,h1,h2,h3,h4"
    )]
    pub memory_cells: Vec<String>,
    #[arg(
        short,
        long,
        help = "Load memory cells from a file.",
        long_help = "Load memory cell values from a file.\nEach line contains a single memory cell in the following formatting: NAME=VALUE\nExample: h1=5\nEmpty cells can be set with: NAME\nExample: h2",
        conflicts_with = "memory_cells",
    )]
    pub memory_cell_file: Option<String>,
}
