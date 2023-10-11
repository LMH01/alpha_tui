use clap::{ArgGroup, Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    author = "LMH01",
    version,
    about,
    long_about = "debugger and runtime environment for the alpha notation used in my Systemnahme Informatik lecture"
)]
#[clap(group = ArgGroup::new("memory").args(["memory_cells", "memory_cell_file"]))]
pub struct Cli {
    #[arg(
        short,
        long,
        help = "Number of available accumulators",
        long_help = "Number of available accumulators.\nIf the value is too large it can happen that accumulators are not displayed in the tui.",
        required_if_eq("disable_memory_detection", "true"),
        global = true
    )]
    pub accumulators: Option<u8>,
    #[arg(
        short,
        long,
        help = "List of available memory cells",
        long_help = "List of available memory cells.\nIf a large number of memory cells is specified, it can happen that some are not displayed in the tui.\nExample: -a a,b,c,d",
        value_delimiter = ',',
        global = true
    )]
    pub memory_cells: Option<Vec<String>>,

    #[arg(
        long,
        help = "List of available index memory cells",
        long_help = "List of available index memory cells.\nExample: 0,1,2,3\n\nCan be used to visualize how index memory cells are filled with values or to explicitly enable index memory cells when automatic detection has been disabled by the \"--disable-memory-detection\" flag.",
        value_delimiter = ',',
        global = true
    )]
    pub index_memory_cells: Option<Vec<usize>>,
    #[arg(
        long,
        help = "Load memory cells from a file",
        long_help = "Load memory cell values from a file.\nEach line contains a single memory cell in the following formatting: NAME=VALUE\nExample: h1=5\nEmpty cells can be set with: NAME\nExample: h2\n\nIndex memory cells can be set by using the following formatting: [INDEX]=VALUE\nExample: [0]=5",
        conflicts_with_all = [ "memory_cells", "index_memory_cells" ],
        global = true,
    )]
    pub memory_cell_file: Option<String>,
    #[arg(
        long,
        help = "Disable accumulator, gamma accumulator and memory_cell detection",
        long_help = "Set to disable accumulator, gamma accumulator and memory_cell detection.\nIf disabled, accumulators, gamma accumulator and memory cells won't be read from program,\ninstead they have to be specified using \"--accumulators\", \"--enable-gamma-accumulator\" and \"--memory-cells\" or \"--memory-cell-file\" and \"--index-memory-cells\" or \"--memory-cell-file\"",
        requires_all = [ "memory" ],
        global = true,
    )]
    pub disable_memory_detection: bool,

    #[arg(
        long,
        help = "Enable the gamma accumulator",
        long_help = "Enable the gamma accumulator, can be used to enable gamma accumulator when automatic detection is disabled by \"--disable-memory-detection\".",
        global = true
    )]
    pub enable_gamma_accumulator: bool,

    #[arg(
        long = "allowed-instructions",
        help = "Load allowed instructions from file",
        long_help = "Load allowed instructions from file.\nIf set, only these instructions are allowed, if the program\ncontains any instructions not contained in the file, it will fail to build.\n\nFor more help see https://github.com/LMH01/alpha_tui/blob/master/docs/cli.md",
        global = true
    )]
    pub allowed_instructions_file: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args, Clone, Debug)]
#[clap(group = ArgGroup::new("memory").args(["memory_cells", "memory_cell_file"]))]
pub struct LoadArgs {
    #[arg(
        long_help = "Specify the input file that contains the program",
        required = true
    )]
    pub file: String,

    #[arg(
        long,
        short,
        help = "Enable predetermined breakpoints",
        long_help = "Enable predetermined breakpoints.\nThe supplied element specifies the line in which the breakpoint should be set.\nExample: -b 1,7,8",
        value_delimiter = ',',
        global = true
    )]
    pub breakpoints: Option<Vec<usize>>,
    #[arg(
        short,
        long,
        help = "Disable alignment of labels, instructions and comments",
        long_help = "Per default labels, instructions and comments are aligned in columns to make reading easier, this can be disabled by setting this flag.",
        global = true
    )]
    pub disable_alignment: bool,
    #[arg(
        short,
        long,
        help = "Write the changed program file alignment to file",
        long_help = "Write the changed program file alignment for better readability to the source file.",
        conflicts_with = "disable_alignment",
        global = true
    )]
    pub write_alignment: bool,
}

#[derive(Args, Clone, Debug)]
#[clap(group = ArgGroup::new("memory").args(["memory_cells", "memory_cell_file"]))]
pub struct CheckArgs {
    #[arg(
        long_help = "Specify the input file that contains the program",
        required = true
    )]
    pub file: String,

    #[command(subcommand)]
    command: CheckCommands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
    #[command(about = "Load an alpha notation program")]
    Load(LoadArgs),
    #[command(
        about = "Perform different checks on the program",
        long_about = "Perform different checks on the program.\nReturn values:\n\n 0 - Check successful\n 1 - Compilation error\n 2 - Runtime error\n10 - IO error"
    )]
    Check(CheckArgs),
}

#[derive(Subcommand, Clone, Debug)]
pub enum CheckCommands {
    #[command(about = "Check if the program compiles")]
    #[clap(group = ArgGroup::new("memory").args(["memory_cells", "memory_cell_file"]))]
    Compile,
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;

    #[test]
    fn test_cmd_check_compile_with_allowed_instructions() {
        let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
        let assert = cmd
            .arg("check")
            .arg("tests/test_cmd_check_compile_with_allowed_instructions/program.alpha")
            .arg("compile")
            .arg("--allowed-instructions")
            .arg("tests/test_cmd_check_compile_with_allowed_instructions/instructions.txt")
            .assert();
        assert.success();
    }

    #[test]
    fn test_cmd_check_compile_with_allowed_instructions_2() {
        let mut cmd = Command::cargo_bin("alpha_tui").unwrap();
        let assert = cmd
            .arg("check")
            .arg("tests/test_cmd_check_compile_with_allowed_instructions_2/program.alpha")
            .arg("compile")
            .arg("--allowed-instructions")
            .arg("tests/test_cmd_check_compile_with_allowed_instructions_2/instructions.txt")
            .assert();
        assert.success();
    }
}
