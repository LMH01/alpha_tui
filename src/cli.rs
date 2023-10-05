use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[command(
    author = "LMH01",
    version,
    about,
    long_about = "debugger and runtime environment for the alpha notation used in my Systemnahme Informatik lecture"
)]
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
        long_help = "Number of available accumulators.\nIf the value is too large it can happen that accumulators are not displayed in the tui.",
        required_if_eq("disable_memory_detection", "true")
    )]
    pub accumulators: Option<u8>,
    #[arg(
        short,
        long,
        help = "List of available memory cells",
        long_help = "List of available memory cells.\nIf a large number of memory cells is specified, it can happen that some are not displayed in the tui.\nExample: -a a,b,c,d",
        value_delimiter = ','
    )]
    pub memory_cells: Option<Vec<String>>,

    #[arg(
        long,
        help = "List of available index memory cells",
        long_help = "List of available index memory cells.\nExample: 0,1,2,3\n\nCan be used to visualize how index memory cells are filled with values as they are normally created on a need to use basis.",
        value_delimiter = ','
    )]
    pub index_memory_cells: Option<Vec<usize>>,
    #[arg(
        long,
        help = "Load memory cells from a file",
        long_help = "Load memory cell values from a file.\nEach line contains a single memory cell in the following formatting: NAME=VALUE\nExample: h1=5\nEmpty cells can be set with: NAME\nExample: h2\n\nIndex memory cells can be set by using the following formatting: [INDEX]=VALUE\nExample: [0]=5",
        conflicts_with_all = [ "memory_cells", "index_memory_cells" ]
    )]
    pub memory_cell_file: Option<String>,
    #[arg(
        long,
        help = "Disable accumulator, gamma accumulator and memory_cell detection",
        long_help = "Set to disable accumulator, gamma accumulator and memory_cell detection.\nIf disabled, accumulators, gamma accumulator and memory cells won't be read from program,\ninstead they have to be specified using \"--accumulators\", \"--enable-gamma-accumulator\" and \"--memory-cells\" or \"--memory-cell-file\"\n\nThis however does not apply to index_memory_cells, they are still created on a need to use basis.",
        requires_all = [ "memory" ]
    )]
    pub disable_memory_detection: bool,

    #[arg(
        long,
        help = "Enable the gamma accumulator",
        long_help = "Enable the gamma accumulator, can be used to enable gamma accumulator when automatic detection is disabled by \"--disable-memory-detection\"."
    )]
    pub enable_gamma_accumulator: bool,

    #[arg(
        long,
        short,
        help = "Enable predetermined breakpoints",
        long_help = "Enable predetermined breakpoints.\nThe supplied element specifies the line in which the breakpoint should be set.\nExample: -b 1,7,8",
        value_delimiter = ','
    )]
    pub breakpoints: Option<Vec<usize>>,
    #[arg(
        short,
        long,
        help = "Disable alignment of labels, instructions and comments",
        long_help = "Per default labels, instructions and comments are aligned in columns to make reading easier, this can be disabled by setting this flag."
    )]
    pub disable_alignment: bool,
    #[arg(
        short,
        long,
        help = "Write the changed program file alignment to file",
        long_help = "Write the changed program file alignment for better readability to the source file.",
        conflicts_with = "disable_alignment"
    )]
    pub write_alignment: bool,
}
