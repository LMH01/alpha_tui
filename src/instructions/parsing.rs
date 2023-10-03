use crate::{
    base::{Comparison, Operation},
    instructions::error_handling::InstructionParseError,
};

use super::{Instruction, TargetType, Value, IndexMemoryCellIndexType};

impl TryFrom<&Vec<&str>> for Instruction {
    type Error = InstructionParseError;

    fn try_from(parts: &Vec<&str>) -> Result<Self, Self::Error> {
        // Remove ; from end of line;
        let parts: Vec<String> = parts
            .iter()
            .map(|s| match s.chars().last() {
                Some(c) => {
                    if c == ';' {
                        s.split(';').collect::<String>()
                    } else {
                        (*s).to_string()
                    }
                }
                None => (*s).to_string(),
            })
            .collect();

        // Check if instruction is comparison
        if parts[0] == "if" {
            check_expression_missing(&parts, 1, Some("an accumulator"))?;
            let value_a = Value::try_from((&parts[1], part_range(&parts, 1)))?;
            check_expression_missing(&parts, 2, Some("a comparison"))?;
            let cmp = parse_comparison(&parts[2], part_range(&parts, 2))?;
            check_expression_missing(&parts, 3, None)?;
            check_expression_missing(&parts, 4, Some("then"))?;
            if parts[4] != "then" {
                return Err(InstructionParseError::InvalidExpression(
                    part_range(&parts, 4),
                    parts[5].to_string(),
                ));
            }
            check_expression_missing(&parts, 5, Some("goto"))?;
            if parts[5] != "goto" {
                return Err(InstructionParseError::InvalidExpression(
                    part_range(&parts, 5),
                    parts[5].to_string(),
                ));
            }
            check_expression_missing(&parts, 6, Some("a label"))?;
            let value_b = Value::try_from((&parts[3], part_range(&parts, 3)))?;
            return Ok(Instruction::JumpIf(
                value_a,
                cmp,
                value_b,
                parts[6].to_string(),
            ));
        }

        // Check if instruction is goto
        if parts[0] == "goto" {
            check_expression_missing(&parts, 1, Some("a label"))?;
            return Ok(Instruction::Goto(parts[1].to_string()));
        }

        // Check if instruction is push
        if parts[0] == "push" && parts.len() == 1 {
            return Ok(Instruction::Push);
        }

        // Check if instruction is pop
        if parts[0] == "pop" && parts.len() == 1 {
            return Ok(Instruction::Pop);
        }

        // Check if instruction is call
        if parts[0] == "call" && parts.len() == 2 {
            return Ok(Instruction::Call(parts[1].to_string()));
        }

        // Check if instruction is return
        if parts[0] == "return" && parts.len() == 1 {
            return Ok(Instruction::Return);
        }

        // Handle stack operations
        if parts[0].starts_with("stack") {
            match parts.len() {
                1 => {
                    let op_txt = parts[0].replace("stack", "");
                    return Ok(Instruction::StackOp(parse_operation(&op_txt, (5, 4 + op_txt.len()))?));
                },
                2 => {
                    return Ok(Instruction::StackOp(parse_operation(&parts[1], (6, 5 + parts[1].len()))?));
                },
                _ => return Err(InstructionParseError::UnknownInstruction(whole_range(&parts), parts.join(" "))),
            };
        }

        // At this point only instructions follow that require := at second position
        if parts.len() < 2 {
            return Err(InstructionParseError::MissingExpression {
                range: (parts[0].len(), parts[0].len()),
                help: "You might be missing ':='".to_string(),
            });
        }

        if parts[1] != ":=" {
            return Err(InstructionParseError::UnknownInstruction(
                whole_range(&parts),
                parts.join(" "),
            ));
        }

        //TODO Add expression missing checks here
        let target = TargetType::try_from((&parts[0], part_range(&parts, 0)))?;
        if parts.len() == 2 {
            return Err(InstructionParseError::MissingExpression {
                range: (part_range(&parts, 1).1 + 1, part_range(&parts, 1).1 + 1),
                help: "Try inserting an accumulator or a memory cell".to_string(),
            });
        }
        let source_a = Value::try_from((&parts[2], part_range(&parts, 2)))?;
        if parts.len() == 3 {
            // instruction is of type a := b
            return Ok(Instruction::Assign(target, source_a));
        } else if parts.len() == 4 {
            return Err(InstructionParseError::MissingExpression {
                range: (part_range(&parts, 3).1 + 1, part_range(&parts, 3).1 + 1),
                help: "Try inserting an accumulator or a memory cell".to_string(),
            });
        } else if parts.len() == 5 {
            // instruction is of type a := b op c
            let op = parse_operation(&parts[3], part_range(&parts, 3))?;
            let source_b = Value::try_from((&parts[4], part_range(&parts, 4)))?;
            return Ok(Instruction::Calc(target, source_a, op, source_b));
        }
        Err(InstructionParseError::UnknownInstruction(
            whole_range(&parts),
            parts.join(" "),
        ))
    }
}

impl TryFrom<&str> for Instruction {
    type Error = InstructionParseError;

    /// Tries to parse an instruction from the input string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(&value.split_whitespace().collect::<Vec<&str>>())
    }
}

/// Tries to parse the index of the accumulator.
///
/// `part_range` indicates the area that is affected.
pub fn parse_alpha(s: &str, part_range: (usize, usize)) -> Result<usize, InstructionParseError> {
    if !s.starts_with('a') && !s.starts_with('α') && !s.is_empty() {
        return Err(InstructionParseError::InvalidExpression(
            part_range,
            s.to_string(),
        ));
    }
    let input = s.replace(['a', 'α'], "");
    if input.is_empty() {
        // if no index is supplied default to accumulator 0
        return Ok(0);
    }
    match input.parse::<usize>() {
        Ok(x) => Ok(x),
        Err(_) => Err(InstructionParseError::NotANumber(
            (part_range.0 + 1, part_range.1),
            input,
        )),
    }
}

/// Tries to parse the operation.
///
/// `part_range` indicates the area that is affected.
pub fn parse_operation(
    s: &str,
    part_range: (usize, usize),
) -> Result<Operation, InstructionParseError> {
    if let Ok(s) = Operation::try_from(s) {
        Ok(s)
    } else {
        if !s.is_ascii() {
            return Err(InstructionParseError::UnknownOperation(
                (part_range.0, part_range.1 + 1),
                s.to_string(),
            ));
        }
        Err(InstructionParseError::UnknownOperation(
            part_range,
            s.to_string(),
        ))
    }
}

/// Tries to parse the comparison.
///
/// `part_range` indicates the area that is affected.
pub fn parse_comparison(
    s: &str,
    part_range: (usize, usize),
) -> Result<Comparison, InstructionParseError> {
    if let Ok(s) = Comparison::try_from(s) {
        Ok(s)
    } else {
        if !s.is_ascii() {
            return Err(InstructionParseError::UnknownComparison(
                (part_range.0, part_range.1 + 1),
                s.to_string(),
            ));
        }
        Err(InstructionParseError::UnknownComparison(
            part_range,
            s.to_string(),
        ))
    }
}

/// Parses the name of a memory cell.
/// For that the content inside p() is taken.
/// The name of the memory cell is only allowed to contain the letters A-Z and a-z.
/// The numbers 0-9 are also allowed, if at least one letter is included.
///
/// `part_range` indicates the area that is affected.
pub fn parse_memory_cell(
    s: &str,
    part_range: (usize, usize),
) -> Result<String, InstructionParseError> {
    if !s.starts_with("p(") && !s.starts_with('ρ') {
        return Err(InstructionParseError::InvalidExpression(
            part_range,
            s.to_string(),
        ));
    }
    if !s.ends_with(')') {
        return Err(InstructionParseError::InvalidExpression(
            (part_range.0, part_range.1),
            s.to_string(),
        ));
    }
    let name = s.replacen("p(", "", 1).replacen("ρ(", "", 1).replacen(')', "", 1);
    if name.is_empty() {
        return Err(InstructionParseError::InvalidExpression(
            part_range,
            s.to_string(),
        ));
    }
    if !name.chars().any(|c| matches!(c, 'a'..='z')) {// TODO fix that ONLY a-z, A-Z and 0-9 are allowed, names that contain ( or ; should fail!
        return Err(InstructionParseError::InvalidExpression((part_range.0+2, part_range.1-2), name));
    }
    Ok(name)
}

pub fn parse_index_memory_cell(s: &str, part_range: (usize, usize)) -> Result<IndexMemoryCellIndexType, InstructionParseError> {
    if !s.starts_with("p(") && !s.starts_with("ρ(") {
        return Err(InstructionParseError::InvalidExpression(
            part_range,
            s.to_string(),
        ));
    }
    if !s.ends_with(')') {
        return Err(InstructionParseError::InvalidExpression(
            (part_range.0, part_range.1),
            s.to_string(),
        ));
    }
    // At this point we know that the string starts with p( and ends with ), we can remove these indicators to get the inner value
    let location = s.chars().skip(2).take(s.chars().count()-1-2).collect::<String>();
    //let location = s.replacen("p(", "", 1).replacen("ρ(", "", 1).replacen(')', "", 1);
    if let Ok(idx) = location.parse::<usize>() {
        return Ok(IndexMemoryCellIndexType::Direct(idx));
    }
    if let Ok(name) = parse_memory_cell(&location, part_range) {
        return Ok(IndexMemoryCellIndexType::MemoryCell(name));
    }
    // Call this function again to determine if inner value is a number (= instance of Direct), if so the index type is an index.
    match parse_index_memory_cell(&location, (part_range.0+2, part_range.1-1)) {
        Ok(t) => {
            match t {
                IndexMemoryCellIndexType::Direct(idx) => return Ok(IndexMemoryCellIndexType::Index(idx)),
                _ => return Err(InstructionParseError::UnknownInstruction((0,0), location)),
            }
        }
        Err(e) => return Err(e),
    }
}

/// Calculates the character index range of a part.
///
/// `part_idx` specifies in what part the error occurs.
#[allow(clippy::needless_range_loop)]
pub fn part_range(parts: &[String], part_idx: usize) -> (usize, usize) {
    let mut start_idx = 0;
    for (idx, part) in parts.iter().enumerate() {
        if idx == part_idx {
            break;
        }
        start_idx += part.len() + 1; //Add one to add in the space
    }
    (start_idx, start_idx + parts[part_idx].len() - 1) // remove one because we start counting at 0
}

/// Calculates a range over all parts
pub fn whole_range(parts: &[String]) -> (usize, usize) {
    (0, parts.join(" ").len() - 1)
}

/// Returns error when the input vector does only contain `number` of elements.
fn check_expression_missing(
    parts: &Vec<String>,
    number: usize,
    suggestion: Option<&str>,
) -> Result<(), InstructionParseError> {
    if parts.len() <= number {
        let pos = parts.join(" ").len();
        let base_help = "Make sure that you use a supported instruction.".to_string();
        let help = match suggestion {
            Some(s) => format!("{base_help}\nMaybe you are missing: {s}"),
            None => base_help,
        };
        return Err(InstructionParseError::MissingExpression {
            range: (pos, pos),
            help,
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{instructions::{parsing::{parse_index_memory_cell, parse_memory_cell}, IndexMemoryCellIndexType, error_handling::InstructionParseError}, base::MemoryCell};

    #[test]
    fn test_parse_memory_cell() {
        assert_eq!(parse_memory_cell("p(h1)", (0, 4)), Ok("h1".to_string()));
        assert_eq!(parse_memory_cell("ρ(h1)", (0, 4)), Ok("h1".to_string()));
        assert_eq!(parse_memory_cell("p(1)", (0, 4)), Err(InstructionParseError::InvalidExpression((2, 2), "1".to_string())));
        assert_eq!(parse_memory_cell("ρ(10)", (0, 5)), Err(InstructionParseError::InvalidExpression((2, 3), "10".to_string())));
    }

    #[test]
    fn test_parse_index_memory_cell() {
        assert_eq!(parse_index_memory_cell("p(p(h1))", (0, 7)), Ok(IndexMemoryCellIndexType::MemoryCell("h1".to_string())));
        assert_eq!(parse_index_memory_cell("ρ(ρ(h1))", (0, 7)), Ok(IndexMemoryCellIndexType::MemoryCell("h1".to_string())));
        assert_eq!(parse_index_memory_cell("p(p(hello))", (0, 7)), Ok(IndexMemoryCellIndexType::MemoryCell("hello".to_string())));
        assert_eq!(parse_index_memory_cell("p(p(1))", (0, 7)), Ok(IndexMemoryCellIndexType::Index(1)));
        assert_eq!(parse_index_memory_cell("p(10)", (0, 7)), Ok(IndexMemoryCellIndexType::Direct(10)));
        assert_eq!(parse_index_memory_cell("p(p())", (0, 6)), Err(InstructionParseError::InvalidExpression((4, 4), "".to_string())));
        assert_eq!(parse_index_memory_cell("p(p()))", (0, 7)), Err(InstructionParseError::InvalidExpression((4, 5), ")".to_string())));
    }
}