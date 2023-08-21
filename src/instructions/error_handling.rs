use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, PartialEq, Diagnostic, Error)]
pub enum InstructionParseError {
    /// Indicates that the specified operation does not exist.
    /// Argument specifies the character index at which the error occurred.
    #[error("unknown operation '{1}'")]
    #[diagnostic(
        code("parse_instruction::unknown_operation"),
        help("Did you mean one of these?: + - * /")
    )]
    UnknownOperation((usize, usize), String),
    /// Indicates that the specified comparison does not exist.
    /// Argument specifies the character index at which the error occurred.
    /// and the string that caused it.
    #[error("unknown comparison '{1}'")]
    #[diagnostic(
        code("parse_instruction::unknown_comparison"),
        help("Did you mean one of these?: < <= == != >= >")
    )]
    UnknownComparison((usize, usize), String),
    /// Indicates that a value that was expected to be a number is not a number.
    /// Argument specifies the character index at which the error occurred.
    /// and the string that caused it.
    #[error("'{1}' is not a number")]
    #[diagnostic(code("parse_instruction::not_a_number"))]
    NotANumber((usize, usize), String),
    /// Indicates that the market expression is not valid.
    /// The reason might be a syntax error.
    #[error("invalid expression '{1}'")]
    #[diagnostic(
        code("parse_instruction::invalid_expression"),
        url("https://github.com/LMH01/alpha_tui/blob/master/instructions.md"),
        help("Make sure that you use a supported instruction.")
    )]
    InvalidExpression((usize, usize), String),
    /// Indicates that no instruction was found that matches the input.
    #[error("unknown instruction '{1}'")]
    #[diagnostic(
        code("parse_instruction::unknown_instruction"),
        url("https://github.com/LMH01/alpha_tui/blob/master/instructions.md"),
        help("Make sure that you use a supported instruction.")
    )]
    UnknownInstruction((usize, usize), String),
    /// Indicates that no instruction was found but gives a suggestion on what instruction might be meant.
    #[error("unknown instruction '{src}'")]
    #[diagnostic(
        code("parse_instruction::unknown_instruction_suggestion"),
        url("https://github.com/LMH01/alpha_tui/blob/master/instructions.md"),
    )]
    UnknownInstructionSuggestion {
        range: (usize, usize),
        #[help]
        help: String,
        src: String,
    },
    #[error("missing expression")]
    #[diagnostic(
        code("parse_instruction::missing_expression"),
        url("https://github.com/LMH01/alpha_tui/blob/master/instructions.md")
    )]
    MissingExpression {
        range: (usize, usize),
        #[help]
        help: String,
    },
}

impl InstructionParseError {
    pub fn range(&self) -> (usize, usize) {
        match self {
            InstructionParseError::UnknownOperation(c, _) => *c,
            InstructionParseError::UnknownComparison(c, _) => *c,
            InstructionParseError::NotANumber(c, _) => *c,
            InstructionParseError::InvalidExpression(c, _) => *c,
            InstructionParseError::UnknownInstruction(c, _) => *c,
            InstructionParseError::UnknownInstructionSuggestion { range: c, help: _ , src: _} => *c,
            InstructionParseError::MissingExpression { range: c, help: _ } => *c,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum BuildProgramErrorTypes {
    
    #[error("when parsing instruction")]
    #[diagnostic(code(build_program::parse_error))]
    ParseError {
        #[source_code]
        src: NamedSource,
        #[label("here")]
        bad_bit: SourceSpan,
        #[source]
        #[diagnostic_source]
        reason: InstructionParseError,
    },

    #[error("label '{0}' is defined multiple times")]
    #[diagnostic(code("build_program::label_definition_error"), help("Make sure that you define the label only once"))]
    LabelDefinedMultipleTimes(String)

}

#[derive(Debug, Diagnostic, Error)]
#[error("when building program")]
#[diagnostic(code("build_program_error"))]
pub struct BuildProgramError {
    #[diagnostic_source]
    pub reason: BuildProgramErrorTypes,
}