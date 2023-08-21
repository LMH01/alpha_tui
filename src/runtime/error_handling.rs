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

#[cfg(test)]
mod tests {
    use crate::{runtime::{builder::RuntimeBuilder, error_handling::RuntimeBuildError, RuntimeArgs}, instructions::Instruction};


    #[test]
    fn test_runtime_args_missing_error() {
        let mut rt = RuntimeBuilder::new();
        rt.set_instructions(vec![Instruction::Push()]);
        assert_eq!(rt.build(), Err(RuntimeBuildError::RuntimeArgsMissing));
    }

    #[test]
    fn test_instructions_missing_error() {
        let mut rt = RuntimeBuilder::new();
        rt.set_runtime_args(RuntimeArgs::new_debug(&vec![""]));
        assert_eq!(rt.build(), Err(RuntimeBuildError::InstructionsMissing));
    }

    #[test]
    fn test_label_undefined_error() {
        let mut rt = RuntimeBuilder::new_debug(&vec![]);
        rt.set_instructions(vec![Instruction::Goto("loop".to_string())]);
        assert_eq!(rt.build(), Err(RuntimeBuildError::LabelUndefined("loop".to_string())));
    }

    #[test]
    fn test_memory_cell_missing() {
        let mut rt = RuntimeBuilder::new_debug(&vec![]);
        rt.set_instructions(vec![Instruction::AssignMemoryCellValue("h1".to_string(), 10)]);
        assert_eq!(rt.build(), Err(RuntimeBuildError::MemoryCellMissing("h1".to_string())));
    }

    #[test]
    fn test_accumulator_missing() {
        let mut rt = RuntimeBuilder::new();
        rt.set_runtime_args(RuntimeArgs::new_empty());
        rt.set_instructions(vec![Instruction::AssignAccumulatorValue(0, 10)]);
        assert_eq!(rt.build(), Err(RuntimeBuildError::AccumulatorMissing("0".to_string())));
    }

}
