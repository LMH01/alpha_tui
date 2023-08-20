use crate::{
    base::{Comparison, Operation},
    runtime::{ControlFlow, RuntimeArgs},
    suggestion,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    /// push
    ///
    /// See [push](fn.push.html)
    Push(),
    /// pop
    ///
    /// See [pop](fn.pop.html)
    Pop(),
    /// a := x
    ///
    /// See [assign_accumulator_value](fn.assign_accumulator_value.html)
    AssignAccumulatorValue(usize, i32),
    /// a := b
    ///
    /// See [assign_accumulator_value_from_accumulator](fn.assign_accumulator_value_from_accumulator.html)
    AssignAccumulatorValueFromAccumulator(usize, usize),
    /// a := p(i)
    ///
    /// See [assign_accumulator_value_from_memory_cell](fn.assign_accumulator_value_from_memory_cell.html)
    AssignAccumulatorValueFromMemoryCell(usize, String),
    /// p(i) := x
    ///
    /// See [assign_memory_cell_value](fn.assign_memory_cell_value.html)
    AssignMemoryCellValue(String, i32),
    /// p(i) := a
    ///
    /// See [assign_memory_cell_value_from_accumulator](fn.assign_memory_cell_value_from_accumulator.html)
    AssignMemoryCellValueFromAccumulator(String, usize),
    /// p(i) := p(j)
    ///
    /// See [assign_memory_cell_value_from_memory_cell](fn.assign_memory_cell_value_from_memory_cell.html)
    AssignMemoryCellValueFromMemoryCell(String, String),
    /// a := a op x
    ///
    /// See [calc_accumulator_with_constant](fn.calc_accumulator_with_constant.html)
    CalcAccumulatorWithConstant(Operation, usize, i32),
    /// a := a op b
    ///
    /// See [calc_accumulator_with_constant](fn.calc_accumulator_with_constant.html)
    CalcAccumulatorWithAccumulator(Operation, usize, usize),
    /// a := b op c
    ///
    /// See [calc_accumulator_with_accumulators](fn.calc_accumulator_with_accumulators.html)
    CalcAccumulatorWithAccumulators(Operation, usize, usize, usize),
    /// a := a op p(i)
    ///
    /// See [calc_accumulator_with_memory_cell](fn.calc_accumulator_with_memory_cell.html)
    CalcAccumulatorWithMemoryCell(Operation, usize, String),
    /// a := p(i) op p(j)
    ///
    /// See [calc_accumulator_with_memory_cells](fn.calc_accumulator_with_memory_cells.html)
    CalcAccumulatorWithMemoryCells(Operation, usize, String, String),
    /// p(i) := p(j) op x
    ///
    /// See [calc_memory_cell_with_memory_cell_constant](fn.calc_memory_cell_with_memory_cell_constant.html)
    CalcMemoryCellWithMemoryCellConstant(Operation, String, String, i32),
    /// p(i) := p(j) op a
    ///
    /// See [calc_memory_cell_with_memory_cell_accumulator](fn.calc_memory_cell_with_memory_cell_accumulator.html)
    CalcMemoryCellWithMemoryCellAccumulator(Operation, String, String, usize),
    /// p(i) := p(j) op p(k)
    ///
    /// See [calc_memory_cell_with_memory_cells](fn.calc_memory_cell_with_memory_cells.html)
    CalcMemoryCellWithMemoryCells(Operation, String, String, String),
    /// goto label
    ///
    /// See [ControlFlow](../runtime/struct.ControlFlow.html) and [goto](fn.goto.html) for further information.
    Goto(String),
    /// if a cmp b then goto label
    ///
    /// See [goto_if_accumulator](fn.goto_if_accumulator.html)
    GotoIfAccumulator(Comparison, String, usize, usize),
    /// if a cmp x then goto label
    ///
    /// See [goto_if_constant](fn.goto_if_constant.html)
    GotoIfConstant(Comparison, String, usize, i32),
    /// if a cmp p(i) then goto label
    ///
    /// See [goto_if_memory_cell](fn.goto_if_memory_cell.html)
    GotoIfMemoryCell(Comparison, String, usize, String),
}

impl Instruction {
    /// Runs the instruction, retuns Err(String) when instruction could not be ran.
    /// Err contains the reason why running the instruction failed.
    pub fn run(
        &self,
        runtime_args: &mut RuntimeArgs,
        control_flow: &mut ControlFlow,
    ) -> Result<(), String> {
        match self {
            Self::Push() => push(runtime_args)?,
            Self::Pop() => pop(runtime_args)?,
            Self::AssignAccumulatorValue(a_idx, value) => {
                assign_accumulator_value(runtime_args, a_idx, value)?
            }
            Self::AssignAccumulatorValueFromAccumulator(a_idx_a, a_idx_b) => {
                assign_accumulator_value_from_accumulator(runtime_args, a_idx_a, a_idx_b)?
            }
            Self::AssignAccumulatorValueFromMemoryCell(a_idx, label) => {
                assign_accumulator_value_from_memory_cell(runtime_args, a_idx, label)?
            }
            Self::AssignMemoryCellValue(label, value) => {
                assign_memory_cell_value(runtime_args, label, value)?
            }
            Self::AssignMemoryCellValueFromAccumulator(label, a_idx) => {
                assign_memory_cell_value_from_accumulator(runtime_args, label, a_idx)?
            }
            Self::AssignMemoryCellValueFromMemoryCell(label_a, label_b) => {
                assign_memory_cell_value_from_memory_cell(runtime_args, label_a, label_b)?
            }
            Self::CalcAccumulatorWithConstant(operation, a_idx, value) => {
                calc_accumulator_with_constant(runtime_args, operation, a_idx, value)?
            }
            Self::CalcAccumulatorWithAccumulator(operation, a_idx_a, a_idx_b) => {
                calc_accumulator_with_accumulator(runtime_args, operation, a_idx_a, a_idx_b)?
            }
            Self::CalcAccumulatorWithAccumulators(operation, a_idx_a, a_idx_b, a_idx_c) => {
                calc_accumulator_with_accumulators(
                    runtime_args,
                    operation,
                    a_idx_a,
                    a_idx_b,
                    a_idx_c,
                )?
            }
            Self::CalcAccumulatorWithMemoryCell(operation, a_idx, label) => {
                calc_accumulator_with_memory_cell(runtime_args, operation, a_idx, label)?
            }
            Self::CalcAccumulatorWithMemoryCells(operation, a_idx, label_a, label_b) => {
                calc_accumulator_with_memory_cells(
                    runtime_args,
                    operation,
                    a_idx,
                    label_a,
                    label_b,
                )?
            }
            Self::CalcMemoryCellWithMemoryCellAccumulator(operation, label_a, label_b, a_idx) => {
                calc_memory_cell_with_memory_cell_accumulator(
                    runtime_args,
                    operation,
                    label_a,
                    label_b,
                    a_idx,
                )?
            }
            Self::CalcMemoryCellWithMemoryCellConstant(operation, label_a, label_b, value) => {
                calc_memory_cell_with_memory_cell_constant(
                    runtime_args,
                    operation,
                    label_a,
                    label_b,
                    value,
                )?
            }
            Self::CalcMemoryCellWithMemoryCells(operation, label_a, label_b, label_c) => {
                calc_memory_cell_with_memory_cells(
                    runtime_args,
                    operation,
                    label_a,
                    label_b,
                    label_c,
                )?
            }
            Self::Goto(label) => goto(runtime_args, control_flow, label)?,
            Self::GotoIfAccumulator(comparison, label, a_idx_a, a_idx_b) => goto_if_accumulator(
                runtime_args,
                control_flow,
                comparison,
                label,
                a_idx_a,
                a_idx_b,
            )?,
            Self::GotoIfConstant(comparison, label, a_idx, c) => {
                goto_if_constant(runtime_args, control_flow, comparison, label, a_idx, c)?
            }
            Self::GotoIfMemoryCell(comparison, label, a_idx, mcl) => {
                goto_if_memory_cell(runtime_args, control_flow, comparison, label, a_idx, mcl)?
            }
        }
        Ok(())
    }
}

impl TryFrom<&Vec<&str>> for Instruction {
    type Error = InstructionParseError;

    /// Tries to parse an instruction from the input vector.
    /// Each element in the vector is one part of the instruction.
    fn try_from(value: &Vec<&str>) -> Result<Self, Self::Error> {
        let mut parts = value;
        
        // Instructions that compare values
        if parts[0] == "if" {
            if !parts[1].starts_with('a') {
                return Err(InstructionParseError::InvalidExpression(
                    err_idx(&parts, 1),
                    parts[1].to_string(),
                ));
            }
            if parts[4] != "then" {
                return Err(InstructionParseError::InvalidExpression(
                    err_idx(&parts, 4),
                    parts[4].to_string(),
                ));
            }
            if parts[5] != "goto" {
                return Err(InstructionParseError::InvalidExpression(
                    err_idx(&parts, 5),
                    parts[5].to_string(),
                ));
            }
            let a_idx = parse_alpha(parts[1], err_idx(&parts, 1))?;
            let cmp = parse_comparison(parts[2], err_idx(&parts, 2))?;
            let a_idx_b = parse_alpha(parts[3], err_idx(&parts, 3));
            let no = parse_number(parts[3], err_idx(&parts, 3));
            let m_cell = parse_memory_cell(parts[3], err_idx(&parts, 3));
            // Check if instruction is goto_if_accumulator
            if a_idx_b.is_ok() {
                return Ok(Instruction::GotoIfAccumulator(
                    cmp,
                    parts[6].to_string(),
                    a_idx,
                    a_idx_b.unwrap(),
                ));
            } else if no.is_ok() {
                // Check if instruction is goto_if_constant
                return Ok(Instruction::GotoIfConstant(
                    cmp,
                    parts[6].to_string(),
                    a_idx,
                    no.unwrap(),
                ));
            } else if m_cell.is_ok() {
                // Check if instruction is goto_if_memory_cell
                return Ok(Instruction::GotoIfMemoryCell(
                    cmp,
                    parts[6].to_string(),
                    a_idx,
                    m_cell.unwrap(),
                ));
            } else {
                return Err(InstructionParseError::InvalidExpression(
                    err_idx(&parts, 3),
                    parts[3].to_string(),
                ));
            }
        }

        // Check if instruction is goto
        if parts[0] == "goto" {
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
        // Check if := is present
        if parts[1] != ":=" {
            return Err(InstructionParseError::NoMatch);
        }

        // Instructions where the first part is an accumulator
        if parts[0].starts_with('a') {
            let a_idx = parse_alpha(parts[0], 0)?;
            // Instructions that use a second accumulator to assign the value
            if parts[2].starts_with('a') {
                let a_idx_b = parse_alpha(parts[2], err_idx(&parts, 2))?;
                // Check if instruction is assign_accumulator_value_from_accumulator
                if parts.len() == 3 {
                    return Ok(Instruction::AssignAccumulatorValueFromAccumulator(
                        a_idx, a_idx_b,
                    ));
                }
                // Parse operation
                let op = parse_operation(parts[3], err_idx(&parts, 3))?;

                // Instructions that use a third accumulator
                if parts[4].starts_with('a') {
                    let a_idx_c = parse_alpha(parts[4], err_idx(&parts, 4))?;
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
                    let no = parse_number(parts[4], err_idx(&parts, 4));
                    let m_cell = parse_memory_cell(parts[4], err_idx(&parts, 4));

                    // Check if instruction is calc_accumulator_value_with_constant
                    if no.is_ok() {
                        return Ok(Instruction::CalcAccumulatorWithConstant(
                            op,
                            a_idx,
                            no.unwrap(),
                        ));
                    } else if m_cell.is_ok() {
                        // Check if instruction is calc_accumulator_value_with_memory_cell
                        return Ok(Instruction::CalcAccumulatorWithMemoryCell(
                            op,
                            a_idx,
                            m_cell.unwrap(),
                        ));
                    } else {
                        return Err(m_cell.unwrap_err());
                    }
                }

                return Err(InstructionParseError::NoMatchSuggestion(suggestion!(
                    parts[0], ":=", parts[0], parts[3], parts[4]
                )));
            }

            let m_cell_a = parse_memory_cell(parts[2], err_idx(&parts, 3));
            let no = parse_number(parts[2], err_idx(&parts, 2));

            // Instructions where the third part is a memory cell
            if m_cell_a.is_ok() {
                // Check if instruction is assign_accumulator_value_from_memory_cell
                if parts.len() == 3 {
                    return Ok(Instruction::AssignAccumulatorValueFromMemoryCell(
                        a_idx,
                        m_cell_a.unwrap(),
                    ));
                }
                // Longer, instruction is calc_accumulator_with_memory_cells
                let op = parse_operation(parts[3], err_idx(&parts, 3))?;
                let m_cell_b = parse_memory_cell(parts[4], err_idx(&parts, 4))?;
                return Ok(Instruction::CalcAccumulatorWithMemoryCells(
                    op,
                    a_idx,
                    m_cell_a.unwrap(),
                    m_cell_b,
                )); //TODO write test
            }

            // Instruction is assign_accumulator__value
            if no.is_ok() {
                return Ok(Instruction::AssignAccumulatorValue(a_idx, no.unwrap()));
                //TODO write test for this
            }
            return Err(InstructionParseError::InvalidExpression(
                err_idx(&parts, 2),
                parts[2].to_string(),
            ));
        }

        // Instructions where the first part is a memory  cell
        if let Ok(m_cell) = parse_memory_cell(parts[0], 0) {
            // Instructions that use use second memory cell in part 2
            if let Ok(m_cell_b) = parse_memory_cell(parts[2], err_idx(&parts, 2)) {
                // Check if instruction is assign_memory_cell_value_from_memory_cell
                if parts.len() == 3 {
                    return Ok(Instruction::AssignMemoryCellValueFromMemoryCell(
                        m_cell, m_cell_b,
                    ));
                }
                let op = parse_operation(parts[3], err_idx(&parts, 3))?;
                let a_idx = parse_alpha(parts[4], err_idx(&parts, 4));
                let no = parse_number(parts[4], err_idx(&parts, 4)); //TODO Fix index out of bounds when parts is of length 4 or of length 1
                let m_cell_c = parse_memory_cell(parts[4], err_idx(&parts, 4));
                // Check if instruction is calc_memory_cell_with_memory_cell_accumulator
                if a_idx.is_ok() {
                    return Ok(Instruction::CalcMemoryCellWithMemoryCellAccumulator(
                        op,
                        m_cell,
                        m_cell_b,
                        a_idx.unwrap(),
                    ));
                } else if no.is_ok() {
                    // Check if instruction is calc_memory_cell_with_memory_cell_constant
                    return Ok(Instruction::CalcMemoryCellWithMemoryCellConstant(
                        op,
                        m_cell,
                        m_cell_b,
                        no.unwrap(),
                    ));
                } else if m_cell_c.is_ok() {
                    // Check if instruction is calc_memory_cell_with_memory_cells
                    return Ok(Instruction::CalcMemoryCellWithMemoryCells(
                        op,
                        m_cell,
                        m_cell_b,
                        m_cell_c.unwrap(),
                    ));
                } else {
                    return Err(InstructionParseError::InvalidExpression(
                        err_idx(&parts, 4),
                        parts[4].to_string(),
                    ));
                }
            }

            let a_idx = parse_alpha(parts[2], err_idx(&parts, 2));
            let no = parse_number(parts[2], err_idx(&parts, 2));
            // Check if instruction is assign_memory_cell_value_from_accumulator
            if a_idx.is_ok() {
                return Ok(Instruction::AssignMemoryCellValueFromAccumulator(
                    m_cell,
                    a_idx.unwrap(),
                ));
            } else if no.is_ok() {
                // Check if instruction is assign_memory_cell_value
                return Ok(Instruction::AssignMemoryCellValue(m_cell, no.unwrap()));
            } else {
                return Err(InstructionParseError::InvalidExpression(
                    err_idx(&parts, 2),
                    parts[2].to_string(),
                ));
            }
        }

        Err(InstructionParseError::NoMatch)
    }
}

impl TryFrom<&str> for Instruction {
    type Error = InstructionParseError;

    /// Tries to parse an instruction from the input string.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(&value.split_whitespace().collect::<Vec<&str>>())
    }
}

#[derive(Debug, PartialEq)]
pub enum InstructionParseError {
    /// Indicates that the specified operation does not exist.
    /// Argument specifies the character index at which the error occurred
    /// and the string that caused it.
    UnknownOperation(usize, String),
    /// Indicates that the specified comparison does not exist.
    /// Argument specifies the character index at which the error occurred.
    /// and the string that caused it.
    UnknownComparison(usize, String),
    /// Indicates that a value that was expected to be a number is not a number.
    /// Argument specifies the character index at which the error occurred.
    /// and the string that caused it.
    NotANumber(usize, String),
    /// Indicates that the market expression is not valid.
    /// The reason might be a syntax error.
    InvalidExpression(usize, String),
    /// Indicates that no instruction was found that matches the input.
    NoMatch,
    /// Indicates that no instruction was found but gives a suggestion on what instruction might be meant.
    NoMatchSuggestion(String),
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
            let mut result = String::new();
            $(
                result.push_str(&format!("{} ", $val));
            )*
            result.trim().to_string()
        }
    };() => {

    };
}

/// Tries to parse the index of the accumulator.
///
/// `err_idx` indicates at what index the parse error originates.
///
/// # Example
/// ```
/// assert_eq!(parse_alpha("a10", 1), Ok(10));
/// assert_eq!(parse_alpha("ab3", 1), Err(1));
/// ```
fn parse_alpha(s: &str, err_idx: usize) -> Result<usize, InstructionParseError> {
    if !s.starts_with("a") && !s.is_empty() {
        return Err(InstructionParseError::InvalidExpression(
            err_idx,
            String::from(s.chars().nth(0).unwrap()),
        ));
    }
    let input = s.replace("a", "");
    match input.parse::<usize>() {
        Ok(x) => Ok(x),
        Err(_) => Err(InstructionParseError::NotANumber(err_idx + 1, input)),
    }
}

/// Tries to parse the operation.
///
/// `err_idx` indicates at what index the parse error originates.
fn parse_operation(s: &str, err_idx: usize) -> Result<Operation, InstructionParseError> {
    match Operation::try_from(s) {
        Ok(s) => Ok(s),
        Err(_) => Err(InstructionParseError::UnknownOperation(
            err_idx,
            s.to_string(),
        )),
    }
}

/// Tries to parse the comparison.
///
/// `err_idx` indicates at what index the parse error originates.
fn parse_comparison(s: &str, err_idx: usize) -> Result<Comparison, InstructionParseError> {
    match Comparison::try_from(s) {
        Ok(s) => Ok(s),
        Err(_) => Err(InstructionParseError::UnknownComparison(
            err_idx,
            s.to_string(),
        )),
    }
}

/// Tries to parse a number.
///
/// `err_idx` indicates at what index the parse error originates.
fn parse_number(s: &str, err_idx: usize) -> Result<i32, InstructionParseError> {
    match s.parse::<i32>() {
        Ok(x) => Ok(x),
        Err(_) => Err(InstructionParseError::NotANumber(err_idx, s.to_string())),
    }
}

/// Parses the name of a memory cell.
/// For that the content inside p() is taken.
///
/// `err_base_idx` indicates at what index the input string starts.
fn parse_memory_cell(s: &str, err_base_idx: usize) -> Result<String, InstructionParseError> {
    if !s.starts_with("p(") {
        return Err(InstructionParseError::InvalidExpression(
            err_base_idx,
            String::from(s.chars().nth(0).unwrap()),
        ));
    }
    if !s.ends_with(")") {
        return Err(InstructionParseError::InvalidExpression(
            err_base_idx + s.len() - 1,
            String::from(s.chars().nth_back(0).unwrap()),
        ));
    }
    let name = s.replace("p(", "").replace(")", "");
    if name.is_empty() {
        return Err(InstructionParseError::InvalidExpression(
            err_base_idx,
            s.to_string(),
        ));
    }
    return Ok(name);
}

/// Calculates the error index depending on the part in which the error occurs.
///
/// `part_idx` specifies in what part the error occurs.
fn err_idx(parts: &Vec<&str>, part_idx: usize) -> usize {
    let mut idx = 0;
    for i in 0..=part_idx - 1 {
        idx += parts[i].len() + 1;
    }
    idx
}

/// Runs code equal to **push**
fn push(runtime_args: &mut RuntimeArgs) -> Result<(), String> {
    assert_accumulator_exists(runtime_args, &0)?;
    runtime_args
        .stack
        .push(runtime_args.accumulators[0].data.unwrap_or(0));
    Ok(())
}

/// Runs code equal to **pop**
fn pop(runtime_args: &mut RuntimeArgs) -> Result<(), String> {
    assert_accumulator_contains_value(runtime_args, &0)?;
    runtime_args.accumulators[0].data = Some(runtime_args.stack.pop().unwrap_or(0));
    Ok(())
}

/// Runs code equal to **a := x**
///
/// - a = value of accumulator with index **a_idx**
/// - x = constant with value **value**
fn assign_accumulator_value(
    runtime_args: &mut RuntimeArgs,
    a_idx: &usize,
    value: &i32,
) -> Result<(), String> {
    assert_accumulator_exists(runtime_args, a_idx)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(*value);
    Ok(())
}

/// Runs code equal to **a := b**
///
/// - a = value of accumulator with index **a_idx_a**
/// - b = value of accumulator with index **a_idx_b**
fn assign_accumulator_value_from_accumulator(
    runtime_args: &mut RuntimeArgs,
    a_idx_a: &usize,
    a_idx_b: &usize,
) -> Result<(), String> {
    let src = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    assert_accumulator_exists(runtime_args, a_idx_a)?;
    runtime_args.accumulators.get_mut(*a_idx_a).unwrap().data = Some(src);
    Ok(())
}

/// Runs code equal to **a := p(i)**
///
/// - a = value of accumulator with index **a_idx**
/// - p(i) = value of memory cell with label **label**
fn assign_accumulator_value_from_memory_cell(
    runtime_args: &mut RuntimeArgs,
    a_idx: &usize,
    label: &str,
) -> Result<(), String> {
    assert_accumulator_exists(runtime_args, a_idx)?;
    let value = assert_memory_cell_contains_value(runtime_args, label)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(value);
    Ok(())
}

/// Runs code equal to **p(i) := x**
///
/// - p(i) = value of memory cell with label **label**
/// - x = constant with value **value**
fn assign_memory_cell_value(
    runtime_args: &mut RuntimeArgs,
    label: &str,
    value: &i32,
) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label)?;
    runtime_args.memory_cells.get_mut(label).unwrap().data = Some(*value);
    Ok(())
}

/// Runs code equal to **p(i) := a**
///
/// - p(i) = value of memory cell with label **label**
/// - a = value of accumulator with index **a_idx**
fn assign_memory_cell_value_from_accumulator(
    runtime_args: &mut RuntimeArgs,
    label: &str,
    a_idx: &usize,
) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label)?;
    let value = assert_accumulator_contains_value(runtime_args, a_idx)?;
    runtime_args.memory_cells.get_mut(label).unwrap().data = Some(value);
    Ok(())
}

/// Runs code equal to **p(i) := p(j)**
///
/// - p(i) = value of memory cell with label **label_a**
/// - p(j) = value of memory cell with label **label_b**
fn assign_memory_cell_value_from_memory_cell(
    runtime_args: &mut RuntimeArgs,
    label_a: &str,
    label_b: &str,
) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let value = assert_memory_cell_contains_value(runtime_args, label_b)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(value);
    Ok(())
}

/// Runs code equal to **a := a op x**
///
/// - a = value of accumulator with index **a_idx**
/// - x = constant with value **value**
/// - op = the operation to perform
fn calc_accumulator_with_constant(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    a_idx: &usize,
    value: &i32,
) -> Result<(), String> {
    let v = assert_accumulator_contains_value(runtime_args, a_idx)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(operation.calc(v, *value));
    Ok(())
}

/// Runs code equal to **a := a op b**
///
/// - a = accumulator with index **a_idx_a**
/// - b = accumulator with index **a_idx_b**
/// - op = the operation to perform
fn calc_accumulator_with_accumulator(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    a_idx_a: &usize,
    a_idx_b: &usize,
) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx_a)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    runtime_args.accumulators.get_mut(*a_idx_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **a := b op c**
///
/// - a = value of accumulator with index **a_idx_a**
/// - b = value of accumulator with index **a_idx_b**
/// - c = value of accumulator with index **a_idx_c**
/// - op = the operation to perform
fn calc_accumulator_with_accumulators(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    a_idx_a: &usize,
    a_idx_b: &usize,
    a_idx_c: &usize,
) -> Result<(), String> {
    assert_accumulator_exists(runtime_args, a_idx_a)?;
    let a = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx_c)?;
    runtime_args.accumulators.get_mut(*a_idx_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **a := a op p(i)**
///
/// - a = value of accumulator with index **a_idx**
/// - p(i) = value of memory cell with label **label**
/// - op = the operation to perform
fn calc_accumulator_with_memory_cell(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    a_idx: &usize,
    label: &str,
) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    let b = assert_memory_cell_contains_value(runtime_args, label)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **a := p(i) op p(j)**
///
/// - a = value of accumulator with index **a_idx**
/// - p(i) = value of memory cell with label **label_a**
/// - p(j) = value of memory cell with label **label_b**
/// - op = the operation to perform
fn calc_accumulator_with_memory_cells(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    a_idx: &usize,
    label_a: &str,
    label_b: &str,
) -> Result<(), String> {
    assert_accumulator_exists(runtime_args, a_idx)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_a)?;
    let b = assert_memory_cell_contains_value(runtime_args, label_b)?;
    runtime_args.accumulators.get_mut(*a_idx).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **p(i) := p(j) op x**
///
/// - p(i) = value of memory cell with label **label_a**
/// - p(j) = value of memory cell with label **label_b**
/// - x = constant with value **value**
/// - op = the operation to perform
fn calc_memory_cell_with_memory_cell_constant(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    label_a: &str,
    label_b: &str,
    value: &i32,
) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_b)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(operation.calc(a, *value));
    Ok(())
}

/// Runs code equal to **p(i) := p(j) op a**
///
/// - p(i) = value of memory cell with label **label_a**
/// - p(j) = value of memory cell with label **label_b**
/// - a = value of accumulator with index **a_idx**
/// - op = the operation to perform
fn calc_memory_cell_with_memory_cell_accumulator(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    label_a: &str,
    label_b: &str,
    a_idx: &usize,
) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_b)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **p(i) := p(j) op p(k)**
///
/// - p(i) = value of memory cell with label **label_a**
/// - p(j) = value of memory cell with label **label_b**
/// - p(k) = value of memory cell with label **label_c**
/// - op = the operation to perform
fn calc_memory_cell_with_memory_cells(
    runtime_args: &mut RuntimeArgs,
    operation: &Operation,
    label_a: &str,
    label_b: &str,
    label_c: &str,
) -> Result<(), String> {
    assert_memory_cell_exists(runtime_args, label_a)?;
    let a = assert_memory_cell_contains_value(runtime_args, label_b)?;
    let b = assert_memory_cell_contains_value(runtime_args, label_c)?;
    runtime_args.memory_cells.get_mut(label_a).unwrap().data = Some(operation.calc(a, b));
    Ok(())
}

/// Runs code equal to **goto label**
///
/// - label = label to which to jump
///
/// Sets the next instruction index to index contained behind **label** in [instruction_labels](../runtime/struct.ControlFlow.html#structfield.instruction_labels) map.
fn goto(
    runtime_args: &mut RuntimeArgs,
    control_flow: &mut ControlFlow,
    label: &str,
) -> Result<(), String> {
    control_flow.next_instruction_index(label)?;
    Ok(())
}

/// Runs code equal to **if a cmp b then goto label**
/// - a = value of accumulator with index **a_idx_a**
/// - b = value of accumulator with index **a_idx_b**
/// - label = label to which to jump
/// - cmp = the way how **a** and **b** should be compared
fn goto_if_accumulator(
    runtime_args: &mut RuntimeArgs,
    control_flow: &mut ControlFlow,
    comparison: &Comparison,
    label: &str,
    a_idx_a: &usize,
    a_idx_b: &usize,
) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx_a)?;
    let b = assert_accumulator_contains_value(runtime_args, a_idx_b)?;
    if comparison.cmp(a, b) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Runs code equal to **if a cmp x then goto label**
/// - a = value of accumulator with index **a_idx**
/// - x = constant with value **value**
/// - label = label to which to jump
/// - cmp = the way how **a** and **x** should be compared
fn goto_if_constant(
    runtime_args: &mut RuntimeArgs,
    control_flow: &mut ControlFlow,
    comparison: &Comparison,
    label: &str,
    a_idx: &usize,
    c: &i32,
) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    if comparison.cmp(a, *c) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Runs code equal to **if a cmp p(i) then goto label**
/// - a = value of accumulator with index **a_idx**
/// - p(i) = value of memory cell with label **mcl**
/// - label = label to which to jump
/// - cmp = the way how **a** and **x** should be compared
fn goto_if_memory_cell(
    runtime_args: &mut RuntimeArgs,
    control_flow: &mut ControlFlow,
    comparison: &Comparison,
    label: &str,
    a_idx: &usize,
    mcl: &str,
) -> Result<(), String> {
    let a = assert_accumulator_contains_value(runtime_args, a_idx)?;
    let b = assert_memory_cell_contains_value(runtime_args, mcl)?;
    if comparison.cmp(a, b) {
        control_flow.next_instruction_index(label)?
    }
    Ok(())
}

/// Tests if the accumulator with **index** exists.
fn assert_accumulator_exists(runtime_args: &mut RuntimeArgs, index: &usize) -> Result<(), String> {
    if let Some(_value) = runtime_args.accumulators.get(*index) {
        Ok(())
    } else {
        Err(format!("Accumulator with index {} does not exist!", index))
    }
}

/// Tests if the accumulator with **index** exists and contains a value.
///
/// Ok(i32) contains the accumulator value.
///
/// Err(String) contains error message.
fn assert_accumulator_contains_value(
    runtime_args: &mut RuntimeArgs,
    index: &usize,
) -> Result<i32, String> {
    if let Some(value) = runtime_args.accumulators.get(*index) {
        if value.data.is_some() {
            Ok(runtime_args.accumulators.get(*index).unwrap().data.unwrap())
        } else {
            Err(format!(
                "Accumulator with index {} does not contain data!",
                index
            ))
        }
    } else {
        Err(format!("Accumulator with index {} does not exist!", index))
    }
}

/// Tests if the memory cell with **label** exists.
fn assert_memory_cell_exists(runtime_args: &mut RuntimeArgs, label: &str) -> Result<(), String> {
    if let Some(_value) = runtime_args.memory_cells.get(label) {
        Ok(())
    } else {
        Err(format!("Memory cell with label {} does not exist!", label))
    }
}

/// Tests if the memory cell with **label** exists and contains a value.
///
/// Ok(i32) contains the memory cell value.
///
/// Err(String) contains error message.
fn assert_memory_cell_contains_value(
    runtime_args: &mut RuntimeArgs,
    label: &str,
) -> Result<i32, String> {
    if let Some(value) = runtime_args.memory_cells.get(label) {
        if value.data.is_some() {
            Ok(runtime_args.memory_cells.get(label).unwrap().data.unwrap())
        } else {
            Err(format!(
                "Memory cell with label {} does not contain data!",
                label
            ))
        }
    } else {
        Err(format!("Memory cell with label {} does not exist!", label))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        base::{Accumulator, Comparison, MemoryCell, Operation},
        instructions::{
            err_idx, parse_alpha, parse_memory_cell, parse_operation, Instruction,
            InstructionParseError,
        },
        runtime::{self, ControlFlow, Runtime, RuntimeArgs, RuntimeBuilder},
    };

    #[test]
    fn test_parse_alpha() {
        assert_eq!(parse_alpha("a3", 1), Ok(3));
        assert_eq!(parse_alpha("a10", 1), Ok(10));
        assert_eq!(
            parse_alpha("a10x", 0),
            Err(InstructionParseError::NotANumber(1, String::from("10x")))
        );
        assert_eq!(
            parse_alpha("ab3", 0),
            Err(InstructionParseError::NotANumber(1, String::from("b3")))
        );
        assert_eq!(
            parse_alpha("ab3i", 0),
            Err(InstructionParseError::NotANumber(1, String::from("b3i")))
        );
    }

    #[test]
    fn test_parse_operation() {
        assert_eq!(parse_operation("+", 0), Ok(Operation::Plus));
        assert_eq!(parse_operation("-", 0), Ok(Operation::Minus));
        assert_eq!(parse_operation("*", 0), Ok(Operation::Multiplication));
        assert_eq!(parse_operation("/", 0), Ok(Operation::Division));
        assert_eq!(
            parse_operation("x", 0),
            Err(InstructionParseError::UnknownOperation(
                0,
                String::from("x")
            ))
        );
    }

    #[test]
    fn test_parse_memory_cell() {
        assert_eq!(parse_memory_cell("p(a)", 0), Ok("a".to_string()));
        assert_eq!(parse_memory_cell("p(xyz)", 0), Ok("xyz".to_string()));
        assert_eq!(
            parse_memory_cell("p(xyzX", 0),
            Err(InstructionParseError::InvalidExpression(5, "X".to_string()))
        );
        assert_eq!(
            parse_memory_cell("pxyz)", 0),
            Err(InstructionParseError::InvalidExpression(0, "p".to_string()))
        );
        assert_eq!(
            parse_memory_cell("p(p()", 0),
            Err(InstructionParseError::InvalidExpression(
                0,
                "p(p()".to_string()
            ))
        );
    }

    #[test]
    fn test_err_idx() {
        let s = String::from("a1 := a2 + a4");
        let parts: Vec<&str> = s.split_whitespace().collect();
        assert_eq!(err_idx(&parts, 2), 6);
    }

    #[test]
    fn test_stack() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 5)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::Push()
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignAccumulatorValue(0, 10)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::Push()
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(args.stack, vec![5, 10]);
        Instruction::Pop()
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
        Instruction::Pop()
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 5);
        assert_eq!(args.stack.len(), 0);
    }

    #[test]
    fn test_parse_push() {
        assert_eq!(Instruction::try_from("push"), Ok(Instruction::Push()));
    }

    #[test]
    fn test_parse_pop() {
        assert_eq!(Instruction::try_from("pop"), Ok(Instruction::Pop()));
    }

    #[test]
    fn test_parse_assign_accumulator_value() {
        assert_eq!(
            Instruction::try_from("a0 := 20"),
            Ok(Instruction::AssignAccumulatorValue(0, 20))
        );
        assert_eq!(
            Instruction::try_from("a0 := x"),
            Err(InstructionParseError::InvalidExpression(6, "x".to_string()))
        );
    }

    #[test]
    fn test_assign_accumulator_value_from_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 5)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignAccumulatorValue(1, 20)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignAccumulatorValue(2, 12)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignAccumulatorValueFromAccumulator(1, 2)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignAccumulatorValueFromAccumulator(0, 1)
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(args.accumulators[1].data.unwrap(), 12);
        assert_eq!(args.accumulators[2].data.unwrap(), 12);
    }

    #[test]
    fn test_parse_assign_accumulator_value_from_accumulator() {
        assert_eq!(
            Instruction::try_from("a0 := a1"),
            Ok(Instruction::AssignAccumulatorValueFromAccumulator(0, 1))
        );
        assert_eq!(
            Instruction::try_from("a3 := a15"),
            Ok(Instruction::AssignAccumulatorValueFromAccumulator(3, 15))
        );
        assert_eq!(
            Instruction::try_from("a3 := a1x"),
            Err(InstructionParseError::NotANumber(7, String::from("1x")))
        );
        assert_eq!(
            Instruction::try_from("ab := a1x"),
            Err(InstructionParseError::NotANumber(1, String::from("b")))
        );
    }

    #[test]
    fn test_assign_accumulator_value_from_accumulator_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::AssignAccumulatorValueFromAccumulator(0, 1)
            .run(&mut args, &mut control_flow)
            .is_err());
        assert!(Instruction::AssignAccumulatorValueFromAccumulator(1, 0)
            .run(&mut args, &mut control_flow)
            .is_err());
    }

    #[test]
    fn test_assign_accumulator_value_from_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignMemoryCellValue("a".to_string(), 10)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string())
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 10);
    }

    #[test]
    fn test_parse_assign_accumulator_value_from_memory_cell() {
        assert_eq!(
            Instruction::try_from("a0 := p(h1)"),
            Ok(Instruction::AssignAccumulatorValueFromMemoryCell(
                0,
                "h1".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("a4 := p(x2)"),
            Ok(Instruction::AssignAccumulatorValueFromMemoryCell(
                4,
                "x2".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("a4 := p()"),
            Err(InstructionParseError::InvalidExpression(
                6,
                "p()".to_string()
            ))
        );
    }

    #[test]
    fn test_assign_accumulator_value_from_memory_cell_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.accumulators = vec![Accumulator::new(0)];
        let err = Instruction::AssignAccumulatorValueFromMemoryCell(0, "a".to_string())
            .run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Memory cell"));
        args.memory_cells.insert("a", MemoryCell::new("a"));
        let err = Instruction::AssignAccumulatorValueFromMemoryCell(1, "a".to_string())
            .run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Accumulator"));
    }

    #[test]
    fn test_assign_memory_cell_value() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignMemoryCellValue("a".to_string(), 2)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignMemoryCellValue("b".to_string(), 20)
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 2);
        assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_parse_assign_memory_cell_value() {
        assert_eq!(
            Instruction::try_from("p(h1) := 10"),
            Ok(Instruction::AssignMemoryCellValue("h1".to_string(), 10))
        );
        assert_eq!(
            Instruction::try_from("p(h1) := x"),
            Err(InstructionParseError::InvalidExpression(9, "x".to_string()))
        );
    }

    #[test]
    fn test_assign_memory_cell_value_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::AssignMemoryCellValue("c".to_string(), 10)
            .run(&mut args, &mut control_flow)
            .is_err());
    }

    #[test]
    fn test_assign_memory_cell_value_from_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 0)
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_parse_assign_memory_cell_value_from_accumulator() {
        assert_eq!(
            Instruction::try_from("p(h1) := a0"),
            Ok(Instruction::AssignMemoryCellValueFromAccumulator(
                "h1".to_string(),
                0
            ))
        );
        assert_eq!(
            Instruction::try_from("p(h1) := a0x"),
            Err(InstructionParseError::InvalidExpression(
                9,
                "a0x".to_string()
            ))
        );
    }

    #[test]
    fn test_assign_memory_cell_value_from_accumulator_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        args.accumulators = vec![Accumulator::new(0)];
        let err = Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 0)
            .run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Memory cell"));
        args.memory_cells.insert("a", MemoryCell::new("a"));
        let err = Instruction::AssignMemoryCellValueFromAccumulator("a".to_string(), 1)
            .run(&mut args, &mut control_flow);
        assert!(err.is_err());
        assert!(err.err().unwrap().contains("Accumulator"));
    }

    #[test]
    fn test_assign_memory_cell_value_from_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        Instruction::AssignMemoryCellValue("a".to_string(), 20)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignMemoryCellValueFromMemoryCell("b".to_string(), "a".to_string())
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(args.memory_cells.get("b").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_parse_assign_memory_cell_value_from_memory_cell() {
        assert_eq!(
            Instruction::try_from("p(h1) := p(h2)"),
            Ok(Instruction::AssignMemoryCellValueFromMemoryCell(
                "h1".to_string(),
                "h2".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("p(h1) := p()"),
            Err(InstructionParseError::InvalidExpression(
                9,
                "p()".to_string()
            ))
        );
    }

    #[test]
    fn test_assign_memory_cell_value_from_memory_cell_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(
            Instruction::AssignMemoryCellValueFromMemoryCell("a".to_string(), "b".to_string())
                .run(&mut args, &mut control_flow)
                .is_err()
        );
        assert!(
            Instruction::AssignMemoryCellValueFromMemoryCell("b".to_string(), "a".to_string())
                .run(&mut args, &mut control_flow)
                .is_err()
        );
        args.memory_cells.insert("a", MemoryCell::new("a"));
        args.memory_cells.insert("b", MemoryCell::new("b"));
        args.memory_cells.get_mut("b").unwrap().data = Some(10);
        assert!(
            Instruction::AssignMemoryCellValueFromMemoryCell("b".to_string(), "a".to_string())
                .run(&mut args, &mut control_flow)
                .is_err()
        );
        assert!(
            Instruction::AssignMemoryCellValueFromMemoryCell("a".to_string(), "b".to_string())
                .run(&mut args, &mut control_flow)
                .is_ok()
        );
    }

    #[test]
    fn test_calc_accumulator_with_constant() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::CalcAccumulatorWithConstant(Operation::Plus, 0, 20)
            .run(&mut args, control_flow)
            .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 40);
    }

    #[test]
    fn test_parse_calc_accumulator_with_constant() {
        assert_eq!(
            Instruction::try_from("a1 := a1 + 20"),
            Ok(Instruction::CalcAccumulatorWithConstant(
                Operation::Plus,
                1,
                20
            ))
        );
        assert_eq!(
            Instruction::try_from("a1 := ab2 + a29"),
            Err(InstructionParseError::NotANumber(7, "b2".to_string()))
        );
        assert_eq!(
            Instruction::try_from("a1 := a2 + 20"),
            Err(InstructionParseError::NoMatchSuggestion(
                "a1 := a1 + 20".to_string()
            ))
        );
    }

    #[test]
    fn test_calc_accumulator_with_accumulator() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::AssignAccumulatorValue(1, 20)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::CalcAccumulatorWithAccumulator(Operation::Plus, 0, 1)
            .run(&mut args, control_flow)
            .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 40);
    }

    #[test]
    fn test_parse_calc_accumulator_with_accumulator() {
        assert_eq!(
            Instruction::try_from("a1 := a1 + a2"),
            Ok(Instruction::CalcAccumulatorWithAccumulator(
                Operation::Plus,
                1,
                2
            ))
        );
        assert_eq!(
            Instruction::try_from("a1 := a1 / a5"),
            Ok(Instruction::CalcAccumulatorWithAccumulator(
                Operation::Division,
                1,
                5
            ))
        );
    }

    #[test]
    fn test_calc_accumulator_with_accumulators() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignAccumulatorValue(1, 20)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::AssignAccumulatorValue(2, 20)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::CalcAccumulatorWithAccumulators(Operation::Plus, 0, 1, 2)
            .run(&mut args, control_flow)
            .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 40);
    }

    #[test]
    fn test_parse_calc_accumulator_with_accumulators() {
        assert_eq!(
            Instruction::try_from("a1 := a2 + a3"),
            Ok(Instruction::CalcAccumulatorWithAccumulators(
                Operation::Plus,
                1,
                2,
                3
            ))
        );
        assert_eq!(
            Instruction::try_from("a1 := a3 / a5"),
            Ok(Instruction::CalcAccumulatorWithAccumulators(
                Operation::Division,
                1,
                3,
                5
            ))
        );
    }

    #[test]
    fn test_calc_accumulator_with_memory_cell() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::AssignMemoryCellValue("a".to_string(), 20)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::CalcAccumulatorWithMemoryCell(Operation::Plus, 0, "a".to_string())
            .run(&mut args, control_flow)
            .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 40);
    }

    #[test]
    fn test_parse_calc_accumulator_with_memory_cell() {
        assert_eq!(
            Instruction::try_from("a1 := a1 * p(h1)"),
            Ok(Instruction::CalcAccumulatorWithMemoryCell(
                Operation::Multiplication,
                1,
                "h1".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("a1 := a2 * p(h1)"),
            Err(InstructionParseError::NoMatchSuggestion(
                "a1 := a1 * p(h1)".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("a1 := a1 * p()"),
            Err(InstructionParseError::InvalidExpression(
                11,
                "p()".to_string()
            ))
        );
    }

    #[test]
    fn test_calc_accumulator_with_memory_cells() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignMemoryCellValue("a".to_string(), 10)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::AssignMemoryCellValue("b".to_string(), 10)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::CalcAccumulatorWithMemoryCells(
            Operation::Plus,
            0,
            "a".to_string(),
            "b".to_string(),
        )
        .run(&mut args, control_flow)
        .unwrap();
        assert_eq!(args.accumulators[0].data.unwrap(), 20);
    }

    #[test]
    fn test_parse_calc_accumulator_with_memory_cells() {
        assert_eq!(
            Instruction::try_from("a0 := p(h1) / p(h2)"),
            Ok(Instruction::CalcAccumulatorWithMemoryCells(
                Operation::Division,
                0,
                "h1".to_string(),
                "h2".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("a0 := p(h1) x p(h2)"),
            Err(InstructionParseError::UnknownOperation(12, "x".to_string()))
        );
        assert_eq!(
            Instruction::try_from("a0 := p(h1) / p()"),
            Err(InstructionParseError::InvalidExpression(
                14,
                "p()".to_string()
            ))
        );
    }

    #[test]
    fn test_calc_memory_cell_with_memory_cell_constant() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignMemoryCellValue("b".to_string(), 10)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::CalcMemoryCellWithMemoryCellConstant(
            Operation::Plus,
            "a".to_string(),
            "b".to_string(),
            10,
        )
        .run(&mut args, control_flow)
        .unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_parse_calc_memory_cell_with_memory_cell_constant() {
        assert_eq!(
            Instruction::try_from("p(h1) := p(h2) * 10"),
            Ok(Instruction::CalcMemoryCellWithMemoryCellConstant(
                Operation::Multiplication,
                "h1".to_string(),
                "h2".to_string(),
                10
            ))
        );
        assert_eq!(
            Instruction::try_from("p(h1) := p(h2) o 10"),
            Err(InstructionParseError::UnknownOperation(15, "o".to_string()))
        );
    }

    #[test]
    fn test_calc_memory_cell_with_memory_cell_accumulator() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignAccumulatorValue(0, 20)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::AssignMemoryCellValue("b".to_string(), 10)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::CalcMemoryCellWithMemoryCellAccumulator(
            Operation::Plus,
            "a".to_string(),
            "b".to_string(),
            0,
        )
        .run(&mut args, control_flow)
        .unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 30);
    }

    #[test]
    fn test_parse_calc_memory_cell_with_memory_cell_accumulator() {
        assert_eq!(
            Instruction::try_from("p(h1) := p(h2) * a0"),
            Ok(Instruction::CalcMemoryCellWithMemoryCellAccumulator(
                Operation::Multiplication,
                "h1".to_string(),
                "h2".to_string(),
                0
            ))
        );
    }

    #[test]
    fn test_calc_memory_cell_with_memory_cells() {
        let mut args = setup_runtime_args();
        let control_flow = &mut ControlFlow::new();
        Instruction::AssignMemoryCellValue("b".to_string(), 10)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::AssignMemoryCellValue("c".to_string(), 10)
            .run(&mut args, control_flow)
            .unwrap();
        Instruction::CalcMemoryCellWithMemoryCells(
            Operation::Plus,
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        )
        .run(&mut args, control_flow)
        .unwrap();
        assert_eq!(args.memory_cells.get("a").unwrap().data.unwrap(), 20);
    }

    #[test]
    fn test_parse_calc_memory_cell_with_memory_cells() {
        assert_eq!(
            Instruction::try_from("p(h1) := p(h2) * p(h3)"),
            Ok(Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Multiplication,
                "h1".to_string(),
                "h2".to_string(),
                "h3".to_string()
            ))
        );
    }

    #[test]
    fn test_goto() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow
            .instruction_labels
            .insert("loop".to_string(), 5);
        Instruction::Goto("loop".to_string())
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(control_flow.next_instruction_index, 5);
    }

    #[test]
    fn test_parse_goto() {
        assert_eq!(
            Instruction::try_from("goto loop"),
            Ok(Instruction::Goto("loop".to_string()))
        );
    }

    #[test]
    fn test_goto_error() {
        let mut args = setup_empty_runtime_args();
        let mut control_flow = ControlFlow::new();
        assert!(Instruction::Goto("loop".to_string())
            .run(&mut args, &mut control_flow)
            .is_err());
    }

    #[test]
    fn test_goto_if_accumulator() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow
            .instruction_labels
            .insert("loop".to_string(), 20);
        Instruction::AssignAccumulatorValue(0, 20)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignAccumulatorValue(1, 30)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::GotoIfAccumulator(Comparison::Less, "loop".to_string(), 0, 1)
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(control_flow.next_instruction_index, 20);
        control_flow.next_instruction_index = 0;
        Instruction::GotoIfAccumulator(Comparison::Equal, "loop".to_string(), 0, 1)
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(control_flow.next_instruction_index, 0);
        assert!(
            Instruction::GotoIfAccumulator(Comparison::Less, "none".to_string(), 0, 1)
                .run(&mut args, &mut control_flow)
                .is_err()
        );
        assert!(
            Instruction::GotoIfAccumulator(Comparison::Equal, "none".to_string(), 0, 1)
                .run(&mut args, &mut control_flow)
                .is_ok()
        );
    }

    #[test]
    fn test_parse_goto_if_accumulator() {
        assert_eq!(
            Instruction::try_from("if a0 <= a1 then goto loop"),
            Ok(Instruction::GotoIfAccumulator(
                Comparison::LessOrEqual,
                "loop".to_string(),
                0,
                1
            ))
        );
        assert_eq!(
            Instruction::try_from("if x <= a1 then goto loop"),
            Err(InstructionParseError::InvalidExpression(3, "x".to_string()))
        );
    }

    #[test]
    fn test_goto_if_constant() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow
            .instruction_labels
            .insert("loop".to_string(), 20);
        Instruction::AssignAccumulatorValue(0, 20)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::GotoIfConstant(Comparison::Less, "loop".to_string(), 0, 40)
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(control_flow.next_instruction_index, 20);
        control_flow.next_instruction_index = 0;
        Instruction::GotoIfConstant(Comparison::Equal, "loop".to_string(), 0, 40)
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(control_flow.next_instruction_index, 0);
        assert!(
            Instruction::GotoIfConstant(Comparison::Less, "none".to_string(), 0, 40)
                .run(&mut args, &mut control_flow)
                .is_err()
        );
        assert!(
            Instruction::GotoIfConstant(Comparison::Equal, "none".to_string(), 0, 40)
                .run(&mut args, &mut control_flow)
                .is_ok()
        );
    }

    #[test]
    fn test_parse_goto_if_constant() {
        assert_eq!(
            Instruction::try_from("if a0 == 20 then goto loop"),
            Ok(Instruction::GotoIfConstant(
                Comparison::Equal,
                "loop".to_string(),
                0,
                20
            ))
        );
    }

    #[test]
    fn test_goto_if_memory_cell() {
        let mut args = setup_runtime_args();
        let mut control_flow = ControlFlow::new();
        control_flow
            .instruction_labels
            .insert("loop".to_string(), 20);
        Instruction::AssignAccumulatorValue(0, 20)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::AssignMemoryCellValue("a".to_string(), 50)
            .run(&mut args, &mut control_flow)
            .unwrap();
        Instruction::GotoIfMemoryCell(Comparison::Less, "loop".to_string(), 0, "a".to_string())
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(control_flow.next_instruction_index, 20);
        control_flow.next_instruction_index = 0;
        Instruction::GotoIfMemoryCell(Comparison::Equal, "loop".to_string(), 0, "a".to_string())
            .run(&mut args, &mut control_flow)
            .unwrap();
        assert_eq!(control_flow.next_instruction_index, 0);
        assert!(Instruction::GotoIfMemoryCell(
            Comparison::Less,
            "none".to_string(),
            0,
            "a".to_string()
        )
        .run(&mut args, &mut control_flow)
        .is_err());
        assert!(Instruction::GotoIfMemoryCell(
            Comparison::Equal,
            "none".to_string(),
            0,
            "a".to_string()
        )
        .run(&mut args, &mut control_flow)
        .is_ok());
    }

    #[test]
    fn test_parse_goto_if_memory_cell() {
        assert_eq!(
            Instruction::try_from("if a0 == p(h1) then goto loop"),
            Ok(Instruction::GotoIfMemoryCell(
                Comparison::Equal,
                "loop".to_string(),
                0,
                "h1".to_string()
            ))
        );
        assert_eq!(
            Instruction::try_from("if a0 == p then goto loop"),
            Err(InstructionParseError::InvalidExpression(9, "p".to_string()))
        );
    }

    #[test]
    fn test_example_program_1() {
        let mut runtime_args = RuntimeArgs::new();
        for _i in 1..=4 {
            runtime_args.add_accumulator();
        }
        runtime_args.add_storage_cell("a");
        runtime_args.add_storage_cell("b");
        runtime_args.add_storage_cell("c");
        runtime_args.add_storage_cell("d");
        runtime_args.add_storage_cell("w");
        runtime_args.add_storage_cell("x");
        runtime_args.add_storage_cell("y");
        runtime_args.add_storage_cell("z");
        runtime_args.add_storage_cell("h1");
        runtime_args.add_storage_cell("h2");
        runtime_args.add_storage_cell("h3");
        runtime_args.add_storage_cell("h4");
        let instructions = vec![
            Instruction::AssignMemoryCellValue("a".to_string(), 5),
            Instruction::AssignMemoryCellValue("b".to_string(), 2),
            Instruction::AssignMemoryCellValue("c".to_string(), 3),
            Instruction::AssignMemoryCellValue("d".to_string(), 9),
            Instruction::AssignMemoryCellValue("w".to_string(), 4),
            Instruction::AssignMemoryCellValue("x".to_string(), 8),
            Instruction::AssignMemoryCellValue("y".to_string(), 3),
            Instruction::AssignMemoryCellValue("z".to_string(), 2),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Multiplication,
                "h1".to_string(),
                "a".to_string(),
                "w".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Multiplication,
                "h2".to_string(),
                "b".to_string(),
                "y".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Multiplication,
                "h3".to_string(),
                "a".to_string(),
                "x".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Multiplication,
                "h4".to_string(),
                "b".to_string(),
                "z".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Plus,
                "a".to_string(),
                "h1".to_string(),
                "h2".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Plus,
                "b".to_string(),
                "h3".to_string(),
                "h4".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Multiplication,
                "h1".to_string(),
                "c".to_string(),
                "w".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Multiplication,
                "h2".to_string(),
                "d".to_string(),
                "y".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Multiplication,
                "h3".to_string(),
                "c".to_string(),
                "x".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Multiplication,
                "h4".to_string(),
                "d".to_string(),
                "z".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Plus,
                "c".to_string(),
                "h1".to_string(),
                "h2".to_string(),
            ),
            Instruction::CalcMemoryCellWithMemoryCells(
                Operation::Plus,
                "d".to_string(),
                "h3".to_string(),
                "h4".to_string(),
            ),
        ];
        let mut runtime_builder = RuntimeBuilder::new();
        runtime_builder.set_instructions(instructions);
        runtime_builder.set_runtime_args(runtime_args);
        let mut runtime = runtime_builder.build().expect("Unable to build runtime!");
        runtime.run().unwrap();
        assert_eq!(
            runtime
                .runtime_args()
                .memory_cells
                .get("a")
                .unwrap()
                .data
                .unwrap(),
            26
        );
        assert_eq!(
            runtime
                .runtime_args()
                .memory_cells
                .get("b")
                .unwrap()
                .data
                .unwrap(),
            44
        );
        assert_eq!(
            runtime
                .runtime_args()
                .memory_cells
                .get("c")
                .unwrap()
                .data
                .unwrap(),
            39
        );
        assert_eq!(
            runtime
                .runtime_args()
                .memory_cells
                .get("d")
                .unwrap()
                .data
                .unwrap(),
            42
        );
    }

    #[test]
    fn test_example_program_1_text_parsing() {
        let mut runtime_args = RuntimeArgs::new();
        for _i in 1..=4 {
            runtime_args.add_accumulator();
        }
        runtime_args.add_storage_cell("a");
        runtime_args.add_storage_cell("b");
        runtime_args.add_storage_cell("c");
        runtime_args.add_storage_cell("d");
        runtime_args.add_storage_cell("w");
        runtime_args.add_storage_cell("x");
        runtime_args.add_storage_cell("y");
        runtime_args.add_storage_cell("z");
        runtime_args.add_storage_cell("h1");
        runtime_args.add_storage_cell("h2");
        runtime_args.add_storage_cell("h3");
        runtime_args.add_storage_cell("h4");
        let mut instructions = Vec::new();
        instructions.push("p(a) := 5\n");
        instructions.push("p(b) := 2\n");
        instructions.push("p(c) := 3\n");
        instructions.push("p(d) := 9\n");
        instructions.push("p(w) := 4\n");
        instructions.push("p(x) := 8\n");
        instructions.push("p(y) := 3\n");
        instructions.push("p(z) := 2\n");
        instructions.push("p(h1) := p(a) * p(w)\n");
        instructions.push("p(h2) := p(b) * p(y)\n");
        instructions.push("p(h3) := p(a) * p(x)\n");
        instructions.push("p(h4) := p(b) * p(z)\n");
        instructions.push("p(a) := p(h1) + p(h2)\n");
        instructions.push("p(b) := p(h3) + p(h4)\n");
        instructions.push("p(h1) := p(c) * p(w)\n");
        instructions.push("p(h2) := p(d) * p(y)\n");
        instructions.push("p(h3) := p(c) * p(x)\n");
        instructions.push("p(h4) := p(d) * p(z)\n");
        instructions.push("p(c) := p(h1) + p(h2)\n");
        instructions.push("p(d) := p(h3) + p(h4)\n");
        let mut rb = RuntimeBuilder::new();
        rb.set_runtime_args(runtime_args);
        assert!(rb.build_instructions(&instructions).is_ok());
        let rt = rb.build();
        assert!(rt.is_ok());
        let mut rt = rt.unwrap();
        assert!(rt.run().is_ok());
        assert_eq!(
            rt.runtime_args()
                .memory_cells
                .get("a")
                .unwrap()
                .data
                .unwrap(),
            26
        );
        assert_eq!(
            rt.runtime_args()
                .memory_cells
                .get("b")
                .unwrap()
                .data
                .unwrap(),
            44
        );
        assert_eq!(
            rt.runtime_args()
                .memory_cells
                .get("c")
                .unwrap()
                .data
                .unwrap(),
            39
        );
        assert_eq!(
            rt.runtime_args()
                .memory_cells
                .get("d")
                .unwrap()
                .data
                .unwrap(),
            42
        );
    }

    #[test]
    fn test_example_program_2() {
        let instructions = vec![
            Instruction::AssignAccumulatorValue(0, 1),
            Instruction::AssignMemoryCellValue("a".to_string(), 8),
            Instruction::CalcAccumulatorWithConstant(Operation::Multiplication, 0, 2),
            Instruction::CalcMemoryCellWithMemoryCellConstant(
                Operation::Minus,
                "a".to_string(),
                "a".to_string(),
                1,
            ),
            Instruction::AssignAccumulatorValueFromMemoryCell(1, "a".to_string()),
            Instruction::GotoIfConstant(Comparison::More, "loop".to_string(), 1, 0),
        ];
        let mut runtime_builder = RuntimeBuilder::new_default();
        runtime_builder.set_instructions(instructions);
        runtime_builder.add_label("loop".to_string(), 2).unwrap();
        let mut runtime = runtime_builder.build().expect("Unable to build runtime!");
        runtime.run().unwrap();
        assert_eq!(runtime.runtime_args().accumulators[0].data.unwrap(), 256);
    }

    #[test]
    fn test_example_program_2_text_parsing() {
        let mut instructions = Vec::new();
        instructions.push("a0 := 1");
        instructions.push("p(a) := 8");
        instructions.push("loop: a0 := a0 * 2");
        instructions.push("p(a) := p(a) - 1");
        instructions.push("a1 := p(a)");
        instructions.push("if a1 > 0 then goto loop");
        let mut runtime_builder = RuntimeBuilder::new_default();
        let res = runtime_builder.build_instructions(&instructions);
        println!("{:?}", res);
        assert!(res.is_ok());
        let mut runtime = runtime_builder.build().expect("Unable to build runtime!");
        runtime.run().unwrap();
        assert_eq!(runtime.runtime_args().accumulators[0].data.unwrap(), 256);
    }

    /// Sets up runtime args in a conistent way because the default implementation for memory cells and accumulators is configgurable.
    fn setup_runtime_args() -> RuntimeArgs<'static> {
        let mut args = RuntimeArgs::new();
        args.memory_cells = HashMap::new();
        args.memory_cells.insert("a", MemoryCell::new("a"));
        args.memory_cells.insert("b", MemoryCell::new("b"));
        args.memory_cells.insert("c", MemoryCell::new("c"));
        args.accumulators = vec![
            Accumulator::new(0),
            Accumulator::new(1),
            Accumulator::new(2),
        ];
        args
    }

    /// Sets up runtime args where no memory cells or accumulators are set.
    fn setup_empty_runtime_args() -> RuntimeArgs<'static> {
        let mut args = RuntimeArgs::new();
        args.accumulators = Vec::new();
        args.memory_cells = HashMap::new();
        args
    }
}
