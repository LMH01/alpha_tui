use std::fmt::Display;

use crate::runtime::error_handling::{CalcError, RuntimeErrorType};

/// A single accumulator, represents "Akkumulator/Alpha" from SysInf lecture.
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

/// Different ways of paring two values
#[derive(Debug, PartialEq, Clone)]
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
            "<=" => Ok(Self::Le),
            "≤" => Ok(Self::Le),
            "=<" => Ok(Self::Le),
            "=" => Ok(Self::Eq),
            "==" => Ok(Self::Eq),
            "!=" => Ok(Self::Neq),
            "≠" => Ok(Self::Neq),
            ">=" => Ok(Self::Ge),
            "=>" => Ok(Self::Ge),
            "≥" => Ok(Self::Ge),
            ">" => Ok(Self::Gt),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Operation {
    pub fn calc(&self, x: i32, y: i32) -> Result<i32, RuntimeErrorType> {
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
                if y != 0 {
                    match x.checked_div(y) {
                        Some(v) => Ok(v),
                        None => Err(RuntimeErrorType::IllegalCalculation {
                            cause: CalcError::AttemptToOverflow(
                                "divide".to_string(),
                                "Division".to_string(),
                            ),
                        }),
                    }
                } else {
                    Err(RuntimeErrorType::IllegalCalculation {
                        cause: CalcError::AttemptToDivideByZero(),
                    })
                }
            }
            Self::Mod => {
                if y != 0 {
                    match x.checked_rem_euclid(y) {
                        Some(v) => Ok(v),
                        None => Err(RuntimeErrorType::IllegalCalculation {
                            cause: CalcError::AttemptToOverflow(
                                "mod".to_string(),
                                "Modulo".to_string(),
                            ),
                        }),
                    }
                } else {
                    Err(RuntimeErrorType::IllegalCalculation {
                        cause: CalcError::AttemptToDivideByZero(),
                    })
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

impl TryFrom<&str> for Operation {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Operation::Add),
            "-" => Ok(Operation::Sub),
            "*" => Ok(Operation::Mul),
            "×" => Ok(Operation::Mul),
            "/" => Ok(Operation::Div),
            "÷" => Ok(Operation::Div),
            "%" => Ok(Operation::Mod),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::base::{Comparison, MemoryCell, Operation};

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
}
