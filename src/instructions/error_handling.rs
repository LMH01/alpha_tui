use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, PartialEq, Diagnostic, Error)]
pub enum InstructionParseError {
    /// Indicates that the specified operation does not exist.
    /// Argument specifies the character index at which the error occurred.
    #[error("unknown operation")]
    #[diagnostic(
        code("parse_instruction::unknown_operation"),
        help("Did you mean one of these?: + - * /")
    )]
    UnknownOperation((usize, usize)),
    /// Indicates that the specified comparison does not exist.
    /// Argument specifies the character index at which the error occurred.
    /// and the string that caused it.
    #[error("unknown comparison")]
    #[diagnostic(
        code("parse_instruction::unknown_comparison"),
        help("Did you mean one of these?: < <= == != >= >")
    )]
    UnknownComparison((usize, usize)),
    /// Indicates that a value that was expected to be a number is not a number.
    /// Argument specifies the character index at which the error occurred.
    /// and the string that caused it.
    #[error("not a number")]
    #[diagnostic(code("parse_instruction::not_a_number"))]
    NotANumber((usize, usize)),
    /// Indicates that the market expression is not valid.
    /// The reason might be a syntax error.
    #[error("invalid expression")]
    #[diagnostic(code("parse_instruction::invalid_expression"))]
    InvalidExpression((usize, usize)),
    /// Indicates that no instruction was found that matches the input.
    #[error("no match")]
    #[diagnostic(code("parse_instruction::no_match"))]
    NoMatch((usize, usize)),
    /// Indicates that no instruction was found but gives a suggestion on what instruction might be meant.
    #[error("no match")]
    #[diagnostic(code("parse_instruction::no_match_suggestion"))]
    NoMatchSuggestion{
        range: (usize, usize),
        #[help]
        help: String,
    },
}

impl InstructionParseError {
    pub fn range(&self) -> (usize, usize) {
        match self {
            InstructionParseError::UnknownOperation(c) => *c,
            InstructionParseError::UnknownComparison(c) => *c,
            InstructionParseError::NotANumber(c) => *c,
            InstructionParseError::InvalidExpression(c) => *c,
            InstructionParseError::NoMatch(c) => *c,
            InstructionParseError::NoMatchSuggestion { range: c, help: _ } => *c,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("when building program")]
#[diagnostic(code("build_program"))]
pub struct BuildProgramError {
    #[source_code]
    pub src: NamedSource,
    #[label("here")]
    pub bad_bit: SourceSpan,
    #[diagnostic_source]
    pub reason: InstructionParseError,
}