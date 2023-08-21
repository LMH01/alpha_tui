use miette::Diagnostic;
use thiserror::Error;

/// Errors that can occur when a runtime is constructed from a RuntimeBuilder.
#[derive(Debug, PartialEq, Error, Diagnostic)]
pub enum RuntimeBuildError {
    #[error("runtime arguments missing")]
    #[diagnostic(code("runtime_build_error::runtime_args_missing"))]
    RuntimeArgsMissing,
    #[error("instructions missing")]
    #[diagnostic(code("runtime_build_error::instructions_missing"))]
    InstructionsMissing,
    /// Indicates that a label is used in an instruction that does not exist in the control flow.
    /// This would lead to a runtime error.
    #[error("label '{0}' undefined")]
    #[diagnostic(code("runtime_build_error::label_undefined"), help("Make sure that you include the label somewhere before an instruction.\nExample: '{0}: a0 := 5'"))]
    LabelUndefined(String),
    #[error("memory cell '{0}' is missing")]
    #[diagnostic(code("runtime_build_error::memory_cell_missing"), help("Make sure to include the memory cell '{0}' in the available memory cells.\nExample: alpha_tui -i FILE -m {0}"))]
    MemoryCellMissing(String),
    #[error("accumulator with id '{0}' is missing")]
    #[diagnostic(
        code("runtime_build_error::accumulator_missing"),
        help("Make sure to have the number of available accumulators set to at least {0}+1")
    )]
    AccumulatorMissing(String),
}

#[derive(Debug)]
pub enum AddLabelError {
    InstructionsNotSet,
    IndexOutOfBounds,
}
