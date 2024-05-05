use clap::{Args, Parser, Subcommand};
use miette::{Diagnostic, Result};
use thiserror::Error;

use crate::base::{Comparison, Operation};

#[derive(Parser, Debug)]
#[command(
    author = "LMH01",
    version,
    about,
    long_about = "debugger and runtime environment for the alpha notation used in my Systemnahe Informatik lecture"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub global_args: GlobalArgs,
}

#[derive(Args, Debug, Clone)]
pub struct GlobalArgs {
    #[arg(
        short,
        long,
        help = "Number of available accumulators",
        long_help = "Number of available accumulators.\nIf the value is too large it can happen that accumulators are not displayed in the tui.\n\nCan be used to visualize how accumulators are filled with values or to explicitly enable accumulators when automatic detection has been disabled by the \"--disable-memory-detection\" flag.",
        global = true,
        display_order = 20
    )]
    pub accumulators: Option<u8>,

    #[arg(
        long,
        help = "List of available index memory cells",
        long_help = "List of available index memory cells.\nExample: 0,1,2,3\n\nCan be used to visualize how index memory cells are filled with values or to explicitly enable index memory cells when automatic detection has been disabled by the \"--disable-memory-detection\" flag.",
        value_delimiter = ',',
        global = true,
        display_order = 23
    )]
    pub index_memory_cells: Option<Vec<usize>>,

    #[arg(
        short,
        long,
        help = "List of available memory cells",
        long_help = "List of available memory cells.\nIf a large number of memory cells is specified, it can happen that some are not displayed in the tui.\nExample: -a a,b,c,d\n\nCan be used to visualize how memory cells are filled with values or to explicitly enable memory cells when automatic detection has been disabled by the \"--disable-memory-detection\" flag.\n\nNote that memory cells named with numbers only are not allowed, as those would conflict with index memory cells.",
        value_delimiter = ',',
        global = true,
        display_order = 22
    )]
    pub memory_cells: Option<Vec<String>>,

    #[arg(
        long,
        help = "Load memory config from a json file",
        long_help = "Load accumulators, gamma accumulator, memory cells and index memory cells from a json file.\nThe memory config file might may include initial values alongside definitions.\n\nFurther help can be found here: https://github.com/LMH01/alpha_tui/blob/master/docs/cli.md.",
        conflicts_with_all = [ "memory_cells", "index_memory_cells", "accumulators" ],
        global = true,
        display_order = 24
    )]
    pub memory_config_file: Option<String>,

    #[arg(
        long, 
        hide = true, 
        global = true
    )]
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
        short,
        long,
        help = "Enable predetermined breakpoints",
        long_help = "Enable predetermined breakpoints.\nThe supplied element specifies the line in which the breakpoint should be set.\nExample: -b 1,7,8",
        value_delimiter = ',',
        global = true,
        display_order = 30
    )]
    pub breakpoints: Option<Vec<usize>>,

    #[arg(
        short,
        long,
        help = "Disable alignment of labels, instructions and comments",
        long_help = "Per default labels, instructions and comments are aligned in columns to make reading easier.\nThis can be disabled by setting this flag.",
        global = true,
        display_order = 32
    )]
    pub disable_alignment: bool,

    #[arg(
        short,
        long,
        help = "Write the changed program file alignment to file",
        long_help = "Write the changed program file alignment for better readability to the source file.",
        conflicts_with = "disable_alignment",
        global = true,
        display_order = 33
    )]
    pub write_alignment: bool,

    #[arg(
        short,
        long,
        help = "File to load and save the custom instruction history to/from",
        long_help = "File to load and save the custom instruction history to/from.\nIf set, the instructions in this list will be analyzed and be made available in the custom instruction window. Instructions not yet contained in this file will be added to it.\nIf the file does not exist, it is created.",
        display_order = 31
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
    command: CheckCommand,
}

#[derive(Args, Clone, Debug)]
pub struct PlaygroundArgs {
    #[arg(
        short,
        long,
        help = "File to load and save the custom instruction history to/from",
        long_help = "File to load and save the custom instruction history to/from.\nIf set, the instructions in this list will be analyzed and be made available in the custom instruction window. Instructions not yet contained in this file will be added to it.",
        display_order = 31
    )]
    pub custom_instruction_history_file: Option<String>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    #[command(about = "Load an alpha notation program")]
    Load(LoadArgs),

    #[command(
        about = "Perform different checks on the program",
        long_about = "Perform different checks on the program.\nReturn values:\n\n 0 - Check successful\n 1 - Compilation error\n 2 - Runtime error\n10 - IO error"
    )]
    Check(CheckArgs),

    #[command(
        about = "Start the tool in playground mode",
        long_about = "Start the tool in playground mode. This allows for custom commands to be run."
    )]
    Playground(PlaygroundArgs),
}

#[derive(Args, Debug, Clone, Default)]
pub struct InstructionLimitingArgs {
    #[arg(
        long,
        help = "Set allowed comparisons",
        long_help = "Set allowed comparisons. If set, comparisons not listed here will not be allowed.\nIf they are used anyway, they will lead to a build_program_error.",
        value_delimiter = ',',
        global = true,
        display_order = 10
    )]
    pub allowed_comparisons: Option<Vec<Comparison>>,

    #[arg(
        long,
        help = "Load allowed instructions from file",
        long_help = "Load allowed instructions from file.\nIf set, only these instructions are allowed, if the program\ncontains any instructions not contained in the file, it will fail to build.\n\nFor more help see https://github.com/LMH01/alpha_tui/blob/master/docs/cli.md",
        global = true,
        display_order = 12
    )]
    pub allowed_instructions_file: Option<String>,

    #[arg(
        long,
        help = "Set allowed operations",
        long_help = "Set allowed operations. If set, operations not listed here will be allowed.\nIf they are used anyway, they will lead to a build_program_error.",
        value_delimiter = ',',
        global = true,
        display_order = 11
    )]
    pub allowed_operations: Option<Vec<Operation>>,

    #[arg(
        long,
        help = "Disable accumulator, gamma accumulator, memory_cell and index_memory_cell detection",
        long_help = "Set to disable accumulator, gamma accumulator, memory_cell and index_memory_cell detection.\nIf disabled, accumulators, gamma accumulator, memory cells and index memory cells won't be read from program and cannot be added by using them at runtime.\nInstead they have to be specified using \"--accumulators\", \"--enable-gamma-accumulator\", \"--memory-cells\" and \"--index-memory-cells\" or \"--memory-config-file\"",
        global = true,
        display_order = 25
    )]
    pub disable_memory_detection: bool,

    #[arg(
        long,
        help = "Enable the gamma accumulator",
        long_help = "Enable the gamma accumulator, can be used to enable gamma accumulator when automatic detection is disabled by \"--disable-memory-detection\".",
        conflicts_with = "memory_config_file",
        global = true,
        display_order = 21
    )]
    pub enable_gamma_accumulator: Option<bool>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum CheckCommand {
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
