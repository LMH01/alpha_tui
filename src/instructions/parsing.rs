use crate::{instructions::error_handling::InstructionParseError, base::{Operation, Comparison}};

use super::{Instruction, TargetType, Value};

impl TryFrom<&Vec<&str>> for Instruction {
    type Error = InstructionParseError;

    fn try_from(parts: &Vec<&str>) -> Result<Self, Self::Error> {
        // very basic implementation more checks, more parsing, better structure and safeguards need to be added when this is used

        // Check if instruction is comparison
        if parts[0] == "if" {
            check_expression_missing(parts, 1, Some("an accumulator"))?;
            let value_a = Value::try_from((parts[1], part_range(parts, 1)))?;
            check_expression_missing(parts, 2, Some("a comparison"))?;
            let cmp = parse_comparison(parts[2], part_range(parts, 2))?;
            check_expression_missing(parts, 3, None)?;
            check_expression_missing(parts, 4, Some("then"))?;
            if parts[4] != "then" {
                return Err(InstructionParseError::InvalidExpression(
                    part_range(parts, 4),
                    parts[5].to_string(),
                ));
            }
            check_expression_missing(parts, 5, Some("goto"))?;
            if parts[5] != "goto" {
                return Err(InstructionParseError::InvalidExpression(
                    part_range(parts, 5),
                    parts[5].to_string(),
                ));
            }
            check_expression_missing(parts, 6, Some("a label"))?;
            let value_b = Value::try_from((parts[3], part_range(parts, 3)))?;
            return Ok(Instruction::JumpIf(value_a, cmp, value_b, parts[6].to_string()));
        }

        // Check if instruction is goto
        if parts[0] == "goto" {
            check_expression_missing(parts, 1, Some("a label"))?;
            return Ok(Instruction::Goto(parts[1].to_string()));
        }
        
        // Check if instruction is push
        if parts[0] == "push" {
            return Ok(Instruction::Push);
        }

        // Check if instruction is pop
        if parts[0] == "pop" {
            return Ok(Instruction::Pop);
        }

        // At this point only instructions follow that require := at second position
        if parts.len() < 2 {
            return Err(InstructionParseError::MissingExpression { range: (parts[0].len(), parts[0].len()), help: "You might be missing ':='".to_string() });
        }

        if parts[1] != ":=" {
            return Err(InstructionParseError::UnknownInstruction(
                whole_range(parts),
                parts.join(" "),
            ));
        }

        //TODO Add expression missing checks here
        let target = TargetType::try_from((parts[0], part_range(parts, 0)))?;
        if parts.len() == 2 {
            return Err(InstructionParseError::MissingExpression { range: (part_range(parts, 1).1+1, part_range(parts, 1).1+1), help: "Try inserting an accumulator or a memory cell".to_string() });
        }
        let source_a = Value::try_from((parts[2], part_range(parts, 2)))?;
        if parts.len() == 3 {
            // instruction is of type a := b
            return Ok(Instruction::Assign(target, source_a));
        } else if parts.len() == 4 {
            return Err(InstructionParseError::MissingExpression { range:(part_range(parts, 3).1+1, part_range(parts, 3).1+1), help: "Try inserting an accumulator or a memory cell".to_string() });
        } else if parts.len() == 5 {
            // instruction is of type a := b op c
            let op = parse_operation(parts[3], part_range(parts, 3))?;
            let source_b = Value::try_from((parts[4], part_range(parts, 4)))?;
            return Ok(Instruction::Calc(target, source_a, op, source_b));
        }
        Err(InstructionParseError::UnknownInstruction(
            whole_range(parts),
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
    if !s.starts_with('a') && !s.is_empty() {
        return Err(InstructionParseError::InvalidExpression(
            part_range,
            s.to_string(),
        ));
    }
    let input = s.replace('a', "");
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
    match Operation::try_from(s) {
        Ok(s) => Ok(s),
        Err(_) => Err(InstructionParseError::UnknownOperation(
            part_range,
            s.to_string(),
        )),
    }
}

/// Tries to parse the comparison.
///
/// `part_range` indicates the area that is affected.
pub fn parse_comparison(
    s: &str,
    part_range: (usize, usize),
) -> Result<Comparison, InstructionParseError> {
    match Comparison::try_from(s) {
        Ok(s) => Ok(s),
        Err(_) => Err(InstructionParseError::UnknownComparison(
            part_range,
            s.to_string(),
        )),
    }
}

/// Tries to parse a number.
///
/// `part_range` indicates the area that is affected.
fn parse_number(s: &str, part_range: (usize, usize)) -> Result<i32, InstructionParseError> {
    match s.parse::<i32>() {
        Ok(x) => Ok(x),
        Err(_) => Err(InstructionParseError::NotANumber(part_range, s.to_string())),
    }
}

/// Parses the name of a memory cell.
/// For that the content inside p() is taken.
///
/// `part_range` indicates the area that is affected.
pub fn parse_memory_cell(s: &str, part_range: (usize, usize)) -> Result<String, InstructionParseError> {
    if !s.starts_with("p(") {
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
    let name = s.replace("p(", "").replace(')', "");
    if name.is_empty() {
        return Err(InstructionParseError::InvalidExpression(
            part_range,
            s.to_string(),
        ));
    }
    Ok(name)
}

/// Calculates the character index range of a part.
///
/// `part_idx` specifies in what part the error occurs.
#[allow(clippy::needless_range_loop)]
pub fn part_range(parts: &[&str], part_idx: usize) -> (usize, usize) {
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
pub fn whole_range(parts: &[&str]) -> (usize, usize) {
    (0, parts.join(" ").len() - 1)
}

/// Returns error when the input vector does only contain `number` of elements.
fn check_expression_missing(
    parts: &[&str],
    number: usize,
    suggestion: Option<&str>,
) -> Result<(), InstructionParseError> {
    if parts.len() <= number {
        let pos = parts.join(" ").len();
        let base_help = "Make sure that you use a supported instruction.".to_string();
        let help = match suggestion {
            Some(s) => format!("{}\nMaybe you are missing: {}", base_help, s),
            None => base_help,
        };
        return Err(InstructionParseError::MissingExpression {
            range: (pos, pos),
            help,
        })?;
    }
    Ok(())
}