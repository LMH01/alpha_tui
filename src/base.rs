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
        Self {
            id,
            data: None,
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

/// Different ways of paring two values
#[derive(Debug, PartialEq, Clone)]
pub enum Comparison {
    Less,
    LessOrEqual,
    Equal,
    MoreOrEqual,
    More,    
}

impl Comparison {
    /// Compares two values with the selected method of comparison.
    pub fn cmp(&self, x: i32, y: i32) -> bool {
        match self {
            Self::Less => {
                x < y
            },
            Self::LessOrEqual => {
                x <= y
            },
            Self::Equal => {
                x == y
            },
            Self::MoreOrEqual => {
                x >= y
            },
            Self::More => {
                x > y
            }
        }
    }
}

impl TryFrom<&str> for Comparison {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "<" => Ok(Self::Less),
            "<=" => Ok(Self::LessOrEqual),
            "=<" => Ok(Self::LessOrEqual),
            "=" => Ok(Self::Equal),
            "==" => Ok(Self::Equal),
            ">=" => Ok(Self::MoreOrEqual),
            "=>" => Ok(Self::MoreOrEqual),
            ">" => Ok(Self::More),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operation {
    Plus,
    Minus,
    Multiplication,
    Division,
}

impl Operation {
    
    pub fn calc(&self, x: i32, y: i32) -> i32 {
        match self {
            Self::Plus => x+y,
            Self::Minus => x-y,
            Self::Multiplication => x*y,
            Self::Division => x/y,
        }
    }

}

impl TryFrom<&str> for Operation {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "+" => Ok(Operation::Plus),
            "-" => Ok(Operation::Minus),
            "*" => Ok(Operation::Multiplication),
            "/" => Ok(Operation::Division),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::base::{Comparison, Operation};

    #[test]
    fn test_comparison() {
        assert!(Comparison::Less.cmp(5, 10));
        assert!(Comparison::LessOrEqual.cmp(5, 10));
        assert!(Comparison::LessOrEqual.cmp(5, 5));
        assert!(Comparison::Equal.cmp(5, 5));
        assert!(Comparison::MoreOrEqual.cmp(5, 5));
        assert!(Comparison::MoreOrEqual.cmp(10, 5));
        assert!(Comparison::More.cmp(10, 5));
    }

    #[test]
    fn test_comparison_try_from_str() {
        assert_eq!(Comparison::try_from("<"), Ok(Comparison::Less));
        assert_eq!(Comparison::try_from("<="), Ok(Comparison::LessOrEqual));
        assert_eq!(Comparison::try_from("=<"), Ok(Comparison::LessOrEqual));
        assert_eq!(Comparison::try_from("="), Ok(Comparison::Equal));
        assert_eq!(Comparison::try_from("=="), Ok(Comparison::Equal));
        assert_eq!(Comparison::try_from(">="), Ok(Comparison::MoreOrEqual));
        assert_eq!(Comparison::try_from("=>"), Ok(Comparison::MoreOrEqual));
        assert_eq!(Comparison::try_from(">"), Ok(Comparison::More));
    }

    #[test]
    fn test_operation() {
        assert_eq!(Operation::Plus.calc(20, 5), 25);
        assert_eq!(Operation::Minus.calc(20, 5), 15);
        assert_eq!(Operation::Multiplication.calc(20, 5), 100);
        assert_eq!(Operation::Division.calc(20, 5), 4);
    }

    #[test]
    fn test_operation_try_from_str() {
        assert_eq!(Operation::try_from("+"), Ok(Operation::Plus));
        assert_eq!(Operation::try_from("-"), Ok(Operation::Minus));
        assert_eq!(Operation::try_from("*"), Ok(Operation::Multiplication));
        assert_eq!(Operation::try_from("/"), Ok(Operation::Division));
        assert_eq!(Operation::try_from("P"), Err(()));
    }

}