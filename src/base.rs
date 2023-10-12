use std::fmt::Display;

use clap::{builder::PossibleValue, ValueEnum};

use crate::{
    cli::CliHint,
    instructions::{Identifier, COMPARISON_IDENTIFIER, OPERATOR_IDENTIFIER},
    runtime::error_handling::{CalcError, RuntimeErrorType},
};

/// A single accumulator, represents "Akkumulator/Alpha" from SysInf lecture.
#[allow(clippy::doc_markdown)]
#[derive(Debug, Clone, PartialEq)]
pub struct Accumulator {
    /// Used to identify accumulator
    pub id: usize,
    /// The data stored in the Accumulator
    pub data: Option<i32>,
}

impl Accumulator {
    /// Creates a new accumulator
    pub fn new(id: usize) -> Self {
        Self { id, data: None }
    }
}

impl Display for Accumulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.data {
            Some(d) => write!(f, "{:2}: {}", self.id, d),
            None => write!(f, "{:2}: None", self.id),
        }
    }
}

/// Representation of a single memory cell.
/// The term memory cell is equal to "Speicherzelle" in the SysInf lecture.
#[allow(clippy::doc_markdown)]
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCell {
    pub label: String,
    pub data: Option<i32>,
}

impl MemoryCell {
    /// Creates a new register
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            data: None,
        }
    }
}

impl Display for MemoryCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.data {
            Some(d) => write!(f, "{:2}: {}", self.label, d),
            None => write!(f, "{:2}: None", self.label),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexMemoryCell {
    pub index: usize,
    pub data: i32,
}

//impl PartialEq for IndexMemoryCell {
//    fn eq(&self, other: &Self) -> bool {
//        self.index == other.index
//    }
//}
//
//impl PartialOrd for IndexMemoryCell {
//    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//        match self.index.partial_cmp(&other.index) {
//            Some(core::cmp::Ordering::Equal) => return Some(Ordering::Equal),
//            ord => return ord,
//        }
//    }
//}

/// Different ways of paring two values
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Comparison {
    Lt,
    Le,
    Eq,
    Neq,
    Ge,
    Gt,
}

impl Comparison {
    /// Compares two values with the selected method of comparison.
    pub fn cmp(&self, x: i32, y: i32) -> bool {
        match self {
            Self::Lt => x < y,
            Self::Le => x <= y,
            Self::Eq => x == y,
            Self::Neq => x != y,
            Self::Ge => x >= y,
            Self::Gt => x > y,
        }
    }
}

impl TryFrom<&str> for Comparison {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<" => Ok(Self::Lt),
            "<=" | "=<" | "≤" => Ok(Self::Le),
            "=" | "==" => Ok(Self::Eq),
            "!=" | "≠" => Ok(Self::Neq),
            ">=" | "=>" | "≥" => Ok(Self::Ge),
            ">" => Ok(Self::Gt),
            _ => Err(()),
        }
    }
}

impl Display for Comparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Eq => write!(f, "=="),
            Self::Neq => write!(f, "!="),
            Self::Ge => write!(f, ">="),
            Self::Gt => write!(f, ">"),
        }
    }
}

impl Identifier for Comparison {
    fn identifier(&self) -> String {
        COMPARISON_IDENTIFIER.to_string()
    }
}

impl ValueEnum for Comparison {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Comparison::Lt,
            Comparison::Le,
            Comparison::Eq,
            Comparison::Neq,
            Comparison::Ge,
            Comparison::Gt,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::Lt => Some(PossibleValue::new("lt")),
            Self::Le => Some(PossibleValue::new("le")),
            Self::Eq => Some(PossibleValue::new("eq")),
            Self::Neq => Some(PossibleValue::new("neq")),
            Self::Ge => Some(PossibleValue::new("ge")),
            Self::Gt => Some(PossibleValue::new("gt")),
        }
    }
}

impl CliHint for Comparison {
    fn cli_hint(&self) -> String {
        match self {
            Self::Lt => String::from("lt"),
            Self::Le => String::from("le"),
            Self::Eq => String::from("eq"),
            Self::Neq => String::from("neq"),
            Self::Ge => String::from("ge"),
            Self::Gt => String::from("gt"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Operation {
    pub fn calc(self, x: i32, y: i32) -> Result<i32, RuntimeErrorType> {
        match self {
            Self::Add => match x.checked_add(y) {
                Some(v) => Ok(v),
                None => Err(RuntimeErrorType::IllegalCalculation {
                    cause: CalcError::AttemptToOverflow("add".to_string(), "Addition".to_string()),
                }),
            },
            Self::Sub => match x.checked_sub(y) {
                Some(v) => Ok(v),
                None => Err(RuntimeErrorType::IllegalCalculation {
                    cause: CalcError::AttemptToOverflow(
                        "subtract".to_string(),
                        "Subtraction".to_string(),
                    ),
                }),
            },
            Self::Mul => match x.checked_mul(y) {
                Some(v) => Ok(v),
                None => Err(RuntimeErrorType::IllegalCalculation {
                    cause: CalcError::AttemptToOverflow(
                        "multiply".to_string(),
                        "Multiplication".to_string(),
                    ),
                }),
            },
            Self::Div => {
                if y == 0 {
                    Err(RuntimeErrorType::IllegalCalculation {
                        cause: CalcError::AttemptToDivideByZero(),
                    })
                } else {
                    match x.checked_div(y) {
                        Some(v) => Ok(v),
                        None => Err(RuntimeErrorType::IllegalCalculation {
                            cause: CalcError::AttemptToOverflow(
                                "divide".to_string(),
                                "Division".to_string(),
                            ),
                        }),
                    }
                }
            }
            Self::Mod => {
                if y == 0 {
                    Err(RuntimeErrorType::IllegalCalculation {
                        cause: CalcError::AttemptToDivideByZero(),
                    })
                } else {
                    match x.checked_rem_euclid(y) {
                        Some(v) => Ok(v),
                        None => Err(RuntimeErrorType::IllegalCalculation {
                            cause: CalcError::AttemptToOverflow(
                                "mod".to_string(),
                                "Modulo".to_string(),
                            ),
                        }),
                    }
                }
            }
        }
    }
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
        }
    }
}

impl Identifier for Operation {
    fn identifier(&self) -> String {
        OPERATOR_IDENTIFIER.to_string()
    }
}

impl TryFrom<&str> for Operation {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Operation::Add),
            "-" => Ok(Operation::Sub),
            "*" | "×" => Ok(Operation::Mul),
            "/" | "÷" => Ok(Operation::Div),
            "%" => Ok(Operation::Mod),
            _ => Err(()),
        }
    }
}

impl ValueEnum for Operation {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Operation::Add,
            Operation::Sub,
            Operation::Mul,
            Operation::Div,
            Operation::Mod,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Self::Add => Some(PossibleValue::new("add")),
            Self::Sub => Some(PossibleValue::new("sub")),
            Self::Mul => Some(PossibleValue::new("mul")),
            Self::Div => Some(PossibleValue::new("div")),
            Self::Mod => Some(PossibleValue::new("mod")),
        }
    }
}

impl CliHint for Operation {
    fn cli_hint(&self) -> String {
        match self {
            Self::Add => String::from("add"),
            Self::Sub => String::from("sub"),
            Self::Mul => String::from("mul"),
            Self::Div => String::from("div"),
            Self::Mod => String::from("mod"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{Comparison, MemoryCell, Operation},
        cli::CliHint,
    };

    use super::Accumulator;

    #[test]
    fn test_accumultor_display() {
        let mut acc = Accumulator::new(0);
        acc.data = Some(5);
        assert_eq!(format!("{}", acc), " 0: 5");
        acc.data = None;
        assert_eq!(format!("{}", acc), " 0: None");
    }

    #[test]
    fn test_memory_cell_display() {
        let mut acc = MemoryCell::new("a");
        acc.data = Some(5);
        assert_eq!(format!("{}", acc), "a : 5");
        acc.data = None;
        assert_eq!(format!("{}", acc), "a : None");
    }

    #[test]
    fn test_comparison() {
        assert!(Comparison::Lt.cmp(5, 10));
        assert!(Comparison::Le.cmp(5, 10));
        assert!(Comparison::Le.cmp(5, 5));
        assert!(Comparison::Eq.cmp(5, 5));
        assert!(Comparison::Neq.cmp(5, 6));
        assert!(!Comparison::Neq.cmp(6, 6));
        assert!(Comparison::Ge.cmp(5, 5));
        assert!(Comparison::Ge.cmp(10, 5));
        assert!(Comparison::Gt.cmp(10, 5));
    }

    #[test]
    fn test_comparison_try_from_str() {
        assert_eq!(Comparison::try_from("<"), Ok(Comparison::Lt));
        assert_eq!(Comparison::try_from("<="), Ok(Comparison::Le));
        assert_eq!(Comparison::try_from("=<"), Ok(Comparison::Le));
        assert_eq!(Comparison::try_from("≤"), Ok(Comparison::Le));
        assert_eq!(Comparison::try_from("="), Ok(Comparison::Eq));
        assert_eq!(Comparison::try_from("=="), Ok(Comparison::Eq));
        assert_eq!(Comparison::try_from("!="), Ok(Comparison::Neq));
        assert_eq!(Comparison::try_from("≠"), Ok(Comparison::Neq));
        assert_eq!(Comparison::try_from(">="), Ok(Comparison::Ge));
        assert_eq!(Comparison::try_from("=>"), Ok(Comparison::Ge));
        assert_eq!(Comparison::try_from("≥"), Ok(Comparison::Ge));
        assert_eq!(Comparison::try_from(">"), Ok(Comparison::Gt));
    }

    #[test]
    fn test_comparison_display() {
        assert_eq!(format!("{}", Comparison::Lt), "<".to_string());
        assert_eq!(format!("{}", Comparison::Le), "<=".to_string());
        assert_eq!(format!("{}", Comparison::Eq), "==".to_string());
        assert_eq!(format!("{}", Comparison::Neq), "!=".to_string());
        assert_eq!(format!("{}", Comparison::Ge), ">=".to_string());
        assert_eq!(format!("{}", Comparison::Gt), ">".to_string());
    }

    #[test]
    fn test_comparison_cli_hint() {
        assert_eq!(Comparison::Lt.cli_hint(), "lt".to_string());
        assert_eq!(Comparison::Le.cli_hint(), "le".to_string());
        assert_eq!(Comparison::Eq.cli_hint(), "eq".to_string());
        assert_eq!(Comparison::Neq.cli_hint(), "neq".to_string());
        assert_eq!(Comparison::Ge.cli_hint(), "ge".to_string());
        assert_eq!(Comparison::Gt.cli_hint(), "gt".to_string());
    }

    #[test]
    fn test_operation() {
        assert_eq!(Operation::Add.calc(20, 5).unwrap(), 25);
        assert_eq!(Operation::Sub.calc(20, 5).unwrap(), 15);
        assert_eq!(Operation::Mul.calc(20, 5).unwrap(), 100);
        assert_eq!(Operation::Div.calc(20, 5).unwrap(), 4);
        assert_eq!(Operation::Mod.calc(20, 5).unwrap(), 0)
    }

    #[test]
    fn test_operation_try_from_str() {
        assert_eq!(Operation::try_from("+"), Ok(Operation::Add));
        assert_eq!(Operation::try_from("-"), Ok(Operation::Sub));
        assert_eq!(Operation::try_from("*"), Ok(Operation::Mul));
        assert_eq!(Operation::try_from("×"), Ok(Operation::Mul));
        assert_eq!(Operation::try_from("/"), Ok(Operation::Div));
        assert_eq!(Operation::try_from("÷"), Ok(Operation::Div));
        assert_eq!(Operation::try_from("%"), Ok(Operation::Mod));
        assert_eq!(Operation::try_from("P"), Err(()));
    }

    #[test]
    fn test_operation_display() {
        assert_eq!(format!("{}", Operation::Add), "+".to_string());
        assert_eq!(format!("{}", Operation::Sub), "-".to_string());
        assert_eq!(format!("{}", Operation::Mul), "*".to_string());
        assert_eq!(format!("{}", Operation::Div), "/".to_string());
        assert_eq!(format!("{}", Operation::Mod), "%".to_string());
    }

    #[test]
    fn test_operation_cli_hint() {
        assert_eq!(Operation::Add.cli_hint(), "add".to_string());
        assert_eq!(Operation::Sub.cli_hint(), "sub".to_string());
        assert_eq!(Operation::Mul.cli_hint(), "mul".to_string());
        assert_eq!(Operation::Div.cli_hint(), "div".to_string());
        assert_eq!(Operation::Mod.cli_hint(), "mod".to_string());
    }
}
