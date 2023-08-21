/// Errors that can occur when a runtime is constructed from a RuntimeBuilder.
#[derive(Debug, PartialEq)]
pub enum RuntimeBuildError {
    RuntimeArgsMissing,
    InstructionsMissing,
    InstructionsEmpty,
    /// Indicates that a label is used in an instruction that does not exist in the control flow.
    /// This would lead to a runtime error.
    LabelMissing(String),
    MemoryCellMissing(String),
    AccumulatorMissing(String),
}

#[derive(Debug)]
pub enum AddLabelError {
    InstructionsNotSet,
    IndexOutOfBounds,
}