/// A single accumulator, represents "Akkumulator/Alpha" from SysInf lecture.
pub struct Accumulator {
    /// Used to identify accumulator
    pub id: i32,
    /// The data stored in the Accumulator
    pub data: Option<i32>,
}

impl Accumulator {
    /// Creates a new accumulator
    pub fn new(id: i32) -> Self {
        Self {
            id,
            data: None,
        }
    }
}

/// Representation of a single memory cell.
/// The term memory cell is equal to "Speicherzelle" in the SysInf lecture.
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

#[cfg(test)]
mod tests {
    use crate::base::Comparison;

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

}