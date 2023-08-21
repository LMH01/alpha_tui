use crate::{
    base::{Comparison, Operation},
    suggestion,
};

use super::{error_handling::InstructionParseError, Instruction};

impl TryFrom<&Vec<&str>> for Instruction {
    type Error = InstructionParseError;

    /// Tries to parse an instruction from the input vector.
    /// Each element in the vector is one part of the instruction.
    fn try_from(value: &Vec<&str>) -> Result<Self, InstructionParseError> {
        let parts = value;

        // Instructions that compare values
        if parts[0] == "if" {
            check_expression_missing(parts, 1, Some("an accumulator"))?;
            if !parts[1].starts_with('a') {
                return Err(InstructionParseError::InvalidExpression(
                    part_range(parts, 1),
                    parts[1].to_string(),
                ));
            }
            let a_idx = parse_alpha(parts[1], part_range(parts, 1))?;
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
            let a_idx_b = parse_alpha(parts[3], part_range(parts, 3));
            let no = parse_number(parts[3], part_range(parts, 3));
            let m_cell = parse_memory_cell(parts[3], part_range(parts, 3));
            // Check if instruction is goto_if_accumulator
            if let Ok(a_idx_b) = a_idx_b {
                return Ok(Instruction::GotoIfAccumulator(
                    cmp,
                    parts[6].to_string(),
                    a_idx,
                    a_idx_b,
                ));
            } else if let Ok(no) = no {
                // Check if instruction is goto_if_constant
                return Ok(Instruction::GotoIfConstant(
                    cmp,
                    parts[6].to_string(),
                    a_idx,
                    no,
                ));
            } else if let Ok(m_cell) = m_cell {
                // Check if instruction is goto_if_memory_cell
                return Ok(Instruction::GotoIfMemoryCell(
                    cmp,
                    parts[6].to_string(),
                    a_idx,
                    m_cell,
                ));
            } else {
                return Err(InstructionParseError::InvalidExpression(
                    part_range(parts, 3),
                    parts[3].to_string(),
                ));
            }
        }

        // Check if instruction is goto
        if parts[0] == "goto" {
            check_expression_missing(parts, 1, Some("a label"))?;
            return Ok(Instruction::Goto(parts[1].to_string()));
        }

        // Check if instruction is push
        if parts[0] == "push" {
            return Ok(Instruction::Push());
        }

        // Check if instruction is pop
        if parts[0] == "pop" {
            return Ok(Instruction::Pop());
        }

        // At this point only instructions follow that require a := in the second part
        check_expression_missing(parts, 1, Some(":="))?;

        // Check if := is present
        if parts[1] != ":=" {
            return Err(InstructionParseError::UnknownInstruction(
                whole_range(parts),
                parts.join(" "),
            ));
        }

        // Instructions where the first part is an accumulator
        if parts[0].starts_with('a') {
            let a_idx = parse_alpha(parts[0], part_range(parts, 0))?;
            // Check if instruction is missing expression
            check_expression_missing(parts, 2, None)?;
            // Instructions that use a second accumulator to assign the value
            if parts[2].starts_with('a') {
                let a_idx_b = parse_alpha(parts[2], part_range(parts, 2))?;
                // Check if instruction is assign_accumulator_value_from_accumulator
                if parts.len() == 3 {
                    return Ok(Instruction::AssignAccumulatorValueFromAccumulator(
                        a_idx, a_idx_b,
                    ));
                }
                // Parse operation
                let op = parse_operation(parts[3], part_range(parts, 3))?;

                check_expression_missing(parts, 4, None)?;
                // Instructions that use a third accumulator
                if parts[4].starts_with('a') {
                    let a_idx_c = parse_alpha(parts[4], part_range(parts, 4))?;
                    // Check if instruction is calc_accumulator_value_with_accumulator or calc_accumulator_value_with_accumulators
                    if a_idx == a_idx_b {
                        return Ok(Instruction::CalcAccumulatorWithAccumulator(
                            op, a_idx, a_idx_c,
                        ));
                    } else {
                        return Ok(Instruction::CalcAccumulatorWithAccumulators(
                            op, a_idx, a_idx_b, a_idx_c,
                        ));
                    }
                }

                // Check if booth accumulators are the same
                if a_idx == a_idx_b {
                    let no = parse_number(parts[4], part_range(parts, 4));
                    let m_cell = parse_memory_cell(parts[4], part_range(parts, 4));

                    // Check if instruction is calc_accumulator_value_with_constant
                    if let Ok(no) = no {
                        return Ok(Instruction::CalcAccumulatorWithConstant(op, a_idx, no));
                    } else {
                        // Check if instruction is calc_accumulator_value_with_memory_cell
                        match m_cell {
                            Ok(v) => {
                                return Ok(Instruction::CalcAccumulatorWithMemoryCell(
                                    op, a_idx, v,
                                ));
                            }
                            Err(e) => return Err(e),
                        }
                    }
                }

                return Err(InstructionParseError::UnknownInstructionSuggestion {
                    range: (0, parts.join(" ").len()),
                    help: suggestion!(parts[0], ":=", parts[0], parts[3], parts[4]),
                    src: parts.join(" "),
                });
            }

            let m_cell_a = parse_memory_cell(parts[2], part_range(parts, 2));
            let no = parse_number(parts[2], part_range(parts, 2));

            // Instructions where the third part is a memory cell
            if let Ok(m_cell_a) = m_cell_a {
                // Check if instruction is assign_accumulator_value_from_memory_cell
                if parts.len() == 3 {
                    return Ok(Instruction::AssignAccumulatorValueFromMemoryCell(
                        a_idx, m_cell_a,
                    ));
                }
                // Longer, instruction is calc_accumulator_with_memory_cells
                let op = parse_operation(parts[3], part_range(parts, 3))?;
                let m_cell_b = parse_memory_cell(parts[4], part_range(parts, 4))?;
                return Ok(Instruction::CalcAccumulatorWithMemoryCells(
                    op, a_idx, m_cell_a, m_cell_b,
                ));
            }

            // Instruction is assign_accumulator__value
            if let Ok(no) = no {
                if parts.len() == 3 {
                    return Ok(Instruction::AssignAccumulatorValue(a_idx, no));
                } else {
                    return Err(InstructionParseError::UnknownInstruction(
                        whole_range(parts),
                        parts.join(" "),
                    ));
                }
            }
            return Err(InstructionParseError::InvalidExpression(
                part_range(parts, 2),
                parts[2].to_string(),
            ));
        }

        // Instructions where the first part is a memory  cell
        if let Ok(m_cell) = parse_memory_cell(parts[0], part_range(parts, 0)) {
            // Check if instruction is missing expression
            check_expression_missing(parts, 2, None)?;
            // Instructions that use use second memory cell in part 2
            if let Ok(m_cell_b) = parse_memory_cell(parts[2], part_range(parts, 2)) {
                // Check if instruction is assign_memory_cell_value_from_memory_cell
                if parts.len() == 3 {
                    return Ok(Instruction::AssignMemoryCellValueFromMemoryCell(
                        m_cell, m_cell_b,
                    ));
                }
                let op = parse_operation(parts[3], part_range(parts, 3))?;
                check_expression_missing(parts, 4, None)?;
                let a_idx = parse_alpha(parts[4], part_range(parts, 4));
                let no = parse_number(parts[4], part_range(parts, 4));
                let m_cell_c = parse_memory_cell(parts[4], part_range(parts, 4));
                // Check if instruction is calc_memory_cell_with_memory_cell_accumulator
                if let Ok(a_idx) = a_idx {
                    return Ok(Instruction::CalcMemoryCellWithMemoryCellAccumulator(
                        op, m_cell, m_cell_b, a_idx,
                    ));
                } else if let Ok(no) = no {
                    // Check if instruction is calc_memory_cell_with_memory_cell_constant
                    return Ok(Instruction::CalcMemoryCellWithMemoryCellConstant(
                        op, m_cell, m_cell_b, no,
                    ));
                } else if let Ok(m_cell_c) = m_cell_c {
                    // Check if instruction is calc_memory_cell_with_memory_cells
                    return Ok(Instruction::CalcMemoryCellWithMemoryCells(
                        op, m_cell, m_cell_b, m_cell_c,
                    ));
                } else {
                    return Err(InstructionParseError::InvalidExpression(
                        part_range(parts, 4),
                        parts[4].to_string(),
                    ));
                }
            }

            let a_idx = parse_alpha(parts[2], part_range(parts, 2));
            let no = parse_number(parts[2], part_range(parts, 2));
            // Check if instruction is assign_memory_cell_value_from_accumulator
            if let Ok(idx) = a_idx {
                return Ok(Instruction::AssignMemoryCellValueFromAccumulator(
                    m_cell, idx,
                ));
            } else if let Ok(v) = no {
                // Check if instruction is assign_memory_cell_value
                return Ok(Instruction::AssignMemoryCellValue(m_cell, v));
            } else {
                return Err(InstructionParseError::InvalidExpression(
                    part_range(parts, 2),
                    parts[2].to_string(),
                ));
            }
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

/// Parses all parameters into string and returns the concatenated string.
/// Inserts a whitespace between each parameters string representation.
#[macro_export]
macro_rules! suggestion {
    () => {
        String::new()
    };
    ($($val:expr),*) => {
        {
            let mut result = String::from("Did you mean: \"");
            $(
                result.push_str(&format!("{} ", $val));
            )*
            result = result.trim().to_string();
            result.push_str("\" ?");
            result
        }
    };() => {

    };
}

/// Tries to parse the index of the accumulator.
///
/// `part_range` indicates the area that is affected.
fn parse_alpha(s: &str, part_range: (usize, usize)) -> Result<usize, InstructionParseError> {
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
fn parse_operation(
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
fn parse_comparison(
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
fn parse_memory_cell(s: &str, part_range: (usize, usize)) -> Result<String, InstructionParseError> {
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
fn part_range(parts: &[&str], part_idx: usize) -> (usize, usize) {
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
fn whole_range(parts: &[&str]) -> (usize, usize) {
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

#[cfg(test)]
mod tests {
    use crate::{
        base::{Comparison, Operation},
        instructions::{
            error_handling::InstructionParseError,
            parsing::{
                parse_alpha, parse_comparison, parse_memory_cell, parse_number, parse_operation,
                part_range, whole_range,
            },
            Instruction,
        },
    };

    #[test]
    fn test_parse_alpha() {
        assert_eq!(parse_alpha("a3", (1, 1)), Ok(3));
        assert_eq!(parse_alpha("a10", (1, 2)), Ok(10));
        assert_eq!(
            parse_alpha("a10x", (0, 3)),
            Err(InstructionParseError::NotANumber((1, 3), "10x".to_string()))
        );
        assert_eq!(
            parse_alpha("ab3", (0, 2)),
            Err(InstructionParseError::NotANumber((1, 2), "b3".to_string()))
        );
        assert_eq!(
            parse_alpha("ab3i", (0, 3)),
            Err(InstructionParseError::NotANumber((1, 3), "b3i".to_string()))
        );
    }

    #[test]
    fn test_parse_operation() {
        assert_eq!(parse_operation("+", (0, 0)), Ok(Operation::Add));
        assert_eq!(parse_operation("-", (0, 0)), Ok(Operation::Sub));
        assert_eq!(parse_operation("*", (0, 0)), Ok(Operation::Mul));
        assert_eq!(parse_operation("/", (0, 0)), Ok(Operation::Div));
        assert_eq!(
            parse_operation("x", (0, 0)),
            Err(InstructionParseError::UnknownOperation(
                (0, 0),
                "x".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_comparison() {
        assert_eq!(parse_comparison("<", (0, 0)), Ok(Comparison::Less));
        assert_eq!(parse_comparison("<=", (0, 1)), Ok(Comparison::LessOrEqual));
        assert_eq!(parse_comparison("==", (0, 1)), Ok(Comparison::Equal));
        assert_eq!(parse_comparison("!=", (0, 1)), Ok(Comparison::NotEqual));
        assert_eq!(parse_comparison(">=", (0, 1)), Ok(Comparison::MoreOrEqual));
        assert_eq!(parse_comparison(">", (0, 0)), Ok(Comparison::More));
        assert_eq!(
            parse_comparison("!x", (0, 1)),
            Err(InstructionParseError::UnknownComparison(
                (0, 1),
                "!x".to_string()
            ))
        );
        assert_eq!(
            parse_comparison("x", (0, 0)),
            Err(InstructionParseError::UnknownComparison(
                (0, 0),
                "x".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_memory_cell() {
        assert_eq!(parse_memory_cell("p(a)", (0, 3)), Ok("a".to_string()));
        assert_eq!(parse_memory_cell("p(xyz)", (0, 3)), Ok("xyz".to_string()));
        assert_eq!(
            parse_memory_cell("p(xyzX", (0, 6)),
            Err(InstructionParseError::InvalidExpression(
                (0, 6),
                "p(xyzX".to_string()
            ))
        );
        assert_eq!(
            parse_memory_cell("pxyz)", (0, 4)),
            Err(InstructionParseError::InvalidExpression(
                (0, 4),
                "pxyz)".to_string()
            ))
        );
        assert_eq!(
            parse_memory_cell("p(p()", (0, 4)),
            Err(InstructionParseError::InvalidExpression(
                (0, 4),
                "p(p()".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("20", (0, 0)), Ok(20));
        assert_eq!(
            parse_number("xxx", (0, 2)),
            Err(InstructionParseError::NotANumber((0, 2), "xxx".to_string()))
        );
    }

    #[test]
    fn test_parse_negative_number() {
        assert_eq!(parse_number("-20", (0, 0)), Ok(-20));
    }

    #[test]
    fn test_part_range() {
        let s = String::from("a1 := a2 + a4");
        let parts: Vec<&str> = s.split_whitespace().collect();
        assert_eq!(part_range(&parts, 2), (6, 7));
    }

    #[test]
    fn test_whole_range() {
        let s = String::from("a1 := a2 + a4");
        let parts: Vec<&str> = s.split_whitespace().collect();
        assert_eq!(whole_range(&parts), (0, 12))
    }

    #[test]
    fn test_missing_expression() {
        assert_instruction_parse_error_suggestion("a", (1, 1), Some(":="));
        assert_instruction_parse_error("a0 :=", (5, 5));
        assert_instruction_parse_error("a0 := a0 +", (10, 10));
        assert_instruction_parse_error_suggestion("p(h1)", (5, 5), Some(":="));
        assert_instruction_parse_error("p(h1) :=", (8, 8));
        assert_instruction_parse_error("p(h1) := p(h2) +", (16, 16));
        assert_instruction_parse_error_suggestion("if", (2, 2), Some("an accumulator"));
        assert_instruction_parse_error_suggestion("if a0", (5, 5), Some("a comparison"));
        assert_instruction_parse_error("if a0 ==", (8, 8));
        assert_instruction_parse_error_suggestion("if a0 == 5", (10, 10), Some("then"));
        assert_instruction_parse_error_suggestion("if a0 == 5 then", (15, 15), Some("goto"));
        assert_instruction_parse_error_suggestion(
            "if a0 == 5 then goto",
            (20, 20),
            Some("a label"),
        );
        assert_instruction_parse_error_suggestion("goto", (4, 4), Some("a label"));
    }

    /// Shortcut for instruction parse errors that don't specify a suggestion.
    fn assert_instruction_parse_error(src: &str, range: (usize, usize)) {
        assert_instruction_parse_error_suggestion(src, range, None);
    }

    /// Shortcut for instruction parse errors that specify a suggestion.
    fn assert_instruction_parse_error_suggestion(
        src: &str,
        range: (usize, usize),
        suggestion: Option<&str>,
    ) {
        let base_help = "Make sure that you use a supported instruction.".to_string();
        let help = match suggestion {
            Some(s) => format!("{}\nMaybe you are missing: {}", base_help, s),
            None => base_help,
        };
        println!("src: {src}");
        assert_eq!(
            Instruction::try_from(src),
            Err(InstructionParseError::MissingExpression { range, help })
        );
    }

    #[test]
    fn test_unknown_instruction() {
        assert_eq!(
            Instruction::try_from("a0 := 5 * 5"),
            Err(InstructionParseError::UnknownInstruction(
                (0, 10),
                "a0 := 5 * 5".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("xyz := 5"),
            Err(InstructionParseError::UnknownInstruction(
                (0, 7),
                "xyz := 5".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("xyz abf daf"),
            Err(InstructionParseError::UnknownInstruction(
                (0, 10),
                "xyz abf daf".to_string()
            ))
        );
    }
}
