use clap::{Args, Parser, Subcommand};
use miette::{Diagnostic, Result};
use thiserror::Error;

use crate::base::{Comparison, Operation};

#[derive(Parser, Debug)]
#[command(
    author = "LMH01",
    version,
    about,
    long_about = "debugger and runtime environment for the alpha notation used in my Systemnahme Informatik lecture"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub global_args: GlobalArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalArgs {
    #[arg(
        short,
        long,
        help = "Number of available accumulators",
        long_help = "Number of available accumulators.\nIf the value is too large it can happen that accumulators are not displayed in the tui.",
        global = true
    )]
    pub accumulators: Option<u8>,
    #[arg(
        short,
        long,
        help = "List of available memory cells",
        long_help = "List of available memory cells.\nIf a large number of memory cells is specified, it can happen that some are not displayed in the tui.\nExample: -a a,b,c,d\n\nNote that memory cells named with numbers only are not allowed, as those would conflict with index memory cells.",
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
        help = "Load memory config from a json file",
        long_help = "Load accumulators, gamma accumulator and memory cell values from a json file.\n\nFurther help can be found here: https://github.com/LMH01/alpha_tui/blob/master/docs/cli.md.",
        conflicts_with_all = [ "memory_cells", "index_memory_cells", "accumulators" ],
        global = true,
    )]
    pub memory_config_file: Option<String>,

    #[arg(long = "disable-instruction-limit", hide = true, global = true)]
    pub disable_instruction_limit: bool,
}

#[derive(Args, Clone, Debug)]
pub struct LoadArgs {
    #[command(flatten)]
    pub instruction_limiting_args: InstructionLimitingArgs,

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
    #[arg(
        short,
        long,
        help = "File to load and save the custom instruction history to/from",
        long_help = "File to load and save the custom instruction history to/from.\nIf set, the instructions in this list will be analyzed and be made available in the custom instruction window. Instructions not yet contained in this file will be added to it."
    )]
    pub custom_instruction_history_file: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct CheckArgs {
    #[command(flatten)]
    pub instruction_limiting_args: InstructionLimitingArgs,

    #[arg(
        long_help = "Specify the input file that contains the program",
        required = true
    )]
    pub file: String,

    #[command(subcommand)]
    command: CheckCommands,
}

#[derive(Args, Clone, Debug)]
pub struct SandboxArgs {
    #[arg(
        short,
        long,
        help = "File to load and save the custom instruction history to/from",
        long_help = "File to load and save the custom instruction history to/from.\nIf set, the instructions in this list will be analyzed and be made available in the custom instruction window. Instructions not yet contained in this file will be added to it."
    )]
    pub custom_instruction_history_file: Option<String>,
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
    #[command(
        about = "Start the tool in sandbox mode",
        long_about = "Start the tool in sandbox mode. This allows for custom commands to be run."
    )]
    Sandbox(SandboxArgs),
}

#[derive(Args, Debug, Clone, Default)]
pub struct InstructionLimitingArgs {
    #[arg(
        long,
        help = "Disable accumulator, gamma accumulator and memory_cell detection",
        long_help = "Set to disable accumulator, gamma accumulator and memory_cell detection.\nIf disabled, accumulators, gamma accumulator and memory cells won't be read from program,\ninstead they have to be specified using \"--accumulators\", \"--enable-gamma-accumulator\", \"--memory-cells\" and \"--index-memory-cells\" or \"--memory-config-file\"",
        global = true
    )]
    pub disable_memory_detection: bool,

    #[arg(
        long,
        help = "Set allowed comparisons",
        long_help = "Set allowed comparisons. If set, comparisons not listed here will not be allowed.\nIf they are used anyway, they will lead to a build_program_error.",
        value_delimiter = ',',
        global = true
    )]
    pub allowed_comparisons: Option<Vec<Comparison>>,

    #[arg(
        long,
        help = "Set allowed operations",
        long_help = "Set allowed operations. If set, operations not listed here will be allowed.\nIf they are used anyway, they will lead to a build_program_error.",
        value_delimiter = ',',
        global = true
    )]
    pub allowed_operations: Option<Vec<Operation>>,

    #[arg(
        long,
        help = "Enable the gamma accumulator",
        long_help = "Enable the gamma accumulator, can be used to enable gamma accumulator when automatic detection is disabled by \"--disable-memory-detection\".",
        conflicts_with = "memory_config_file",
        global = true
    )]
    pub enable_gamma_accumulator: Option<bool>,

    #[arg(
        long = "allowed-instructions",
        help = "Load allowed instructions from file",
        long_help = "Load allowed instructions from file.\nIf set, only these instructions are allowed, if the program\ncontains any instructions not contained in the file, it will fail to build.\n\nFor more help see https://github.com/LMH01/alpha_tui/blob/master/docs/cli.md",
        global = true
    )]
    pub allowed_instructions_file: Option<String>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum CheckCommands {
    #[command(about = "Check if the program compiles")]
    Compile,
}

#[allow(clippy::module_name_repetitions)]
pub trait CliHint {
    fn cli_hint(&self) -> String;
}

/// Validates if the provided arguments are allowed.
///
/// This function is used to test some additional requirements, that can't be programmed into clap.
pub fn validate_arguments(cli: &Cli) -> Result<()> {
    if let Some(memory_cells) = &cli.global_args.memory_cells {
        for cell in memory_cells {
            if cell.chars().any(|c| c.is_ascii_digit()) && !cell.chars().any(|c| c.is_alphabetic()) {
                return Err(CliError::new(CliErrorType::MemoryCellsInvalid(cell.clone())).into());
            }
        }
    }
    Ok(())
}

#[derive(Debug, Diagnostic, Error)]
#[error("while checking cli arguments")]
pub struct CliError {
    #[diagnostic_source]
    reason: CliErrorType,
}

impl CliError {
    fn new(reason: CliErrorType) -> Self {
        Self { reason }
    }
}

/// Provided cli arguments are not valid.
#[derive(Debug, Diagnostic, Error)]
pub enum CliErrorType {
    #[error("memory cell found that has a name consisting of only numbers: {0}")]
    #[diagnostic(code("cli::memory_cells_invalid"), help("Try adding a char: a{0}"))]
    MemoryCellsInvalid(String),
}
