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

#[allow(clippy::match_same_arms)]
impl InstructionParseError {
    pub fn range(&self) -> (usize, usize) {
        match self {
            InstructionParseError::UnknownOperation(c, _) => *c,
            InstructionParseError::UnknownComparison(c, _) => *c,
            InstructionParseError::NotANumber(c, _) => *c,
            InstructionParseError::InvalidExpression(c, _) => *c,
            InstructionParseError::UnknownInstruction(c, _) => *c,
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
    #[diagnostic(
        code("build_program::label_definition_error"),
        help("Make sure that you define the label only once")
    )]
    LabelDefinedMultipleTimes(String),

    #[error("you have defined at least two main labels 'main' and 'MAIN'")]
    #[diagnostic(
        code("build_program::main_definition_error"),
        help("Make sure that you define at most one main label, either 'main' or 'MAIN'")
    )]
    MainLabelDefinedMultipleTimes,

    /// Indicates that this instruction is not allowed because it is not contained in the whitelist
    #[error("instruction '{1}' in line '{0}' is not allowed")]
    #[diagnostic(
        code("build_program::instruction_not_allowed_error"),
        help("Make sure that you include this type ('{2}') of instruction in the whitelist or use a different instruction.\nThese types of instructions are allowed:\n\n{3}")
    )]
    InstructionNotAllowed(usize, String, String, String),

    #[error("comparison '{1}' in line '{0}' is not allowed")]
    #[diagnostic(
        code("build_program::comparison_not_allowed_error"),
        help("Make sure that you include this comparison ('{1}') in the allowed comparisons or use a different instruction.\nTo mark this comparison as allowed you can use: '--allowed-comparisons \"{1}\"'"),
    )]
    ComparisonNotAllowed(usize, String),//TODO add test

    #[error("operation '{1}' in line '{0}' is not allowed")]
    #[diagnostic(
        code("build_program::operation_not_allowed_error"),
        help("Make sure that you include this operation ('{1}') in the allowed operations or use a different instruction.\nTo mark this operation as allowed you can use: '--allowed-operations \"{1}\"'"),
    )]
    OperationNotAllowed(usize, String),// TODO add test
}

#[allow(clippy::match_same_arms)]
impl PartialEq for BuildProgramErrorTypes {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::ParseError {
                    src: l_src,
                    bad_bit: l_bad_bit,
                    reason: l_reason,
                },
                Self::ParseError {
                    src: r_src,
                    bad_bit: r_bad_bit,
                    reason: r_reason,
                },
            ) => l_src.name() == r_src.name() && l_bad_bit == r_bad_bit && l_reason == r_reason,
            (Self::LabelDefinedMultipleTimes(l0), Self::LabelDefinedMultipleTimes(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(Debug, Diagnostic, Error, PartialEq)]
#[error("when building program")]
#[diagnostic(code("build_program_error"))]
pub struct BuildProgramError {
    #[diagnostic_source]
    pub reason: BuildProgramErrorTypes,
}

#[derive(Debug, Diagnostic, Error)]
#[error("when building allowed instructions")]
#[diagnostic(
    code("build_allowed_instructions_error"),
    help("Maybe you wanted to use a token, make sure to use one of these: A, M, C, Y, OP, CMP\nFor more help take a look at the documentation: https://github.com/LMH01/alpha_tui/blob/master/docs/cli.md")
)]
pub struct BuildAllowedInstructionsError {
    #[source_code]
    pub src: NamedSource,
    #[label("here")]
    pub bad_bit: SourceSpan,
    #[source]
    #[diagnostic_source]
    pub reason: InstructionParseError,
}

#[cfg(test)]
mod tests {
    use crate::{
        instructions::{
            error_handling::{BuildProgramError, BuildProgramErrorTypes, InstructionParseError},
            Instruction,
        },
        runtime::builder::RuntimeBuilder,
    };

    #[test]
    fn test_ipe_unknown_operation() {
        assert_eq!(
            Instruction::try_from("a0 := p(h1) x p(h1)"),
            Err(InstructionParseError::UnknownOperation(
                (12, 12),
                "x".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("a0 := p(h1) ö p(h1)"),
            Err(InstructionParseError::UnknownOperation(
                (12, 14),
                "ö".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("a0 := a0 xxx p(h1)"),
            Err(InstructionParseError::UnknownOperation(
                (9, 11),
                "xxx".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("stackxxx"),
            Err(InstructionParseError::UnknownOperation(
                (5, 7),
                "xxx".to_string()
            ))
        );
    }

    #[test]
    fn test_ipe_unknown_comparison() {
        assert_eq!(
            Instruction::try_from("if a0 x a0 then goto loop"),
            Err(InstructionParseError::UnknownComparison(
                (6, 6),
                "x".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("if a0 ö a0 then goto loop"),
            Err(InstructionParseError::UnknownComparison(
                (6, 8),
                "ö".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("if p(h1) xxx 5 then goto loop"),
            Err(InstructionParseError::UnknownComparison(
                (9, 11),
                "xxx".to_string()
            ))
        );
    }

    #[test]
    fn test_ipe_not_a_number() {
        assert_eq!(
            Instruction::try_from("if axx != a0 then goto loop"),
            Err(InstructionParseError::NotANumber((4, 5), "xx".to_string()))
        );
        assert_eq!(
            Instruction::try_from("if a0 != axx then goto loop"),
            Err(InstructionParseError::NotANumber(
                (10, 11),
                "xx".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("axx := p(a)"),
            Err(InstructionParseError::NotANumber((1, 2), "xx".to_string()))
        );
    }

    #[test]
    fn test_ipe_invalid_expression() {
        assert_eq!(
            Instruction::try_from("xxx := xxx"),
            Err(InstructionParseError::InvalidExpression(
                (0, 2),
                "xxx".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("p(h1) := xxx"),
            Err(InstructionParseError::InvalidExpression(
                (9, 11),
                "xxx".to_string()
            ))
        );
    }

    #[test]
    fn test_ipe_unknown_instruction() {
        assert_eq!(
            Instruction::try_from("stackxxx + p(h1)"),
            Err(InstructionParseError::UnknownInstruction(
                (0, 15),
                "stackxxx + p(h1)".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("a0 := p(h1) + p(h2) +"),
            Err(InstructionParseError::UnknownInstruction(
                (0, 20),
                "a0 := p(h1) + p(h2) +".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("a0 := p(h1) + p(h2) + p(h3)"),
            Err(InstructionParseError::UnknownInstruction(
                (0, 26),
                "a0 := p(h1) + p(h2) + p(h3)".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("return xyz"),
            Err(InstructionParseError::UnknownInstruction(
                (0, 9),
                "return xyz".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("call xxx yz"),
            Err(InstructionParseError::UnknownInstruction(
                (0, 10),
                "call xxx yz".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("push xxx"),
            Err(InstructionParseError::UnknownInstruction(
                (0, 7),
                "push xxx".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("pop xxx"),
            Err(InstructionParseError::UnknownInstruction(
                (0, 6),
                "pop xxx".to_string()
            ))
        );
    }

    #[test]
    fn test_ipe_missing_expression() {
        assert_eq!(
            Instruction::try_from("xxx"),
            Err(InstructionParseError::MissingExpression {
                range: (3, 3),
                help: "You might be missing ':='".to_string()
            })
        );
        assert_eq!(
            Instruction::try_from("a0"),
            Err(InstructionParseError::MissingExpression {
                range: (2, 2),
                help: "You might be missing ':='".to_string()
            })
        );
        assert_eq!(
            Instruction::try_from("a0 :="),
            Err(InstructionParseError::MissingExpression {
                range: (5, 5),
                help: "Try inserting an accumulator or a memory cell".to_string()
            })
        );
        assert_eq!(
            Instruction::try_from("a0 := p(h1) +"),
            Err(InstructionParseError::MissingExpression {
                range: (13, 13),
                help: "Try inserting an accumulator or a memory cell".to_string()
            })
        );
    }

    #[test]
    fn test_bpe_label_defined_multiple_times() {
        let mut rb = RuntimeBuilder::new_debug(&["a", "b"]);
        let instructions_input = vec!["loop:", "", "loop:"];
        assert_eq!(
            rb.build_instructions(&instructions_input, "test"),
            Err(BuildProgramError {
                reason: BuildProgramErrorTypes::LabelDefinedMultipleTimes("loop".to_string())
            })
        )
    }

    #[test]
    fn test_bpe_main_label_defined_multiple_times() {
        let mut rb = RuntimeBuilder::new_debug(&["a", "b"]);
        let instructions_input = vec!["main:", "", "MAIN:"];
        assert_eq!(
            rb.build_instructions(&instructions_input, "test"),
            Err(BuildProgramError {
                reason: BuildProgramErrorTypes::MainLabelDefinedMultipleTimes
            })
        );
        let instructions_input = vec!["main:", "", "main:"];
        assert_eq!(
            rb.build_instructions(&instructions_input, "test"),
            Err(BuildProgramError {
                reason: BuildProgramErrorTypes::MainLabelDefinedMultipleTimes
            })
        )
    }
}
