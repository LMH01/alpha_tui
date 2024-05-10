use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::{
    app::{COMMENT, CYAN, FOREGROUND, GREEN, PINK, PURPLE},
    base::Operation,
    instructions::{IndexMemoryCellIndexType, Instruction, TargetType, Value},
    utils::{self, remove_comment},
};

/// How many spaces should be between labels, instructions and comments when alignment is enabled
const SPACING: usize = 2;

/// This trait is used be able to transform specific data into spans.
///
/// In used to make syntax highlighting possible.
pub trait ToSpans {
    /// Creates a span from this element,
    fn to_spans(&self) -> Vec<Span<'static>>;
}

/// Creates a span containing ' := '.
fn assignment_span() -> Span<'static> {
    Span::from(" := ").style(Style::default().fg(PINK))
}

/// Creates a span containing the operation.
fn op_span(op: &Operation) -> Span<'static> {
    Span::from(format!("{op}")).style(Style::default().fg(PINK))
}

/// Create a span containing a label.
fn label_span(label: &str) -> Span<'static> {
    Span::from(format!(" {label}")).style(Style::default().fg(GREEN))
}

/// Span to use for build in functions.
fn build_in_span(text: &str) -> Span<'_> {
    Span::from(text).style(Style::default().fg(PINK))
}

impl ToSpans for Instruction {
    fn to_spans(&self) -> Vec<Span<'static>> {
        match self {
            Self::Assign(t, v) => {
                let mut spans = t.to_spans();
                spans.push(assignment_span());
                spans.append(&mut v.to_spans());
                spans
            }
            Self::Calc(t, v, op, v2) => {
                let mut spans = t.to_spans();
                spans.push(assignment_span());
                spans.append(&mut v.to_spans());
                spans.push(Span::from(" "));
                spans.push(op_span(op));
                spans.push(Span::from(" "));
                spans.append(&mut v2.to_spans());
                spans
            }
            Self::Call(label) => {
                vec![build_in_span("call"), label_span(label)]
            }
            Self::Goto(label) => {
                vec![build_in_span("goto"), label_span(label)]
            }
            Self::JumpIf(v, cmp, v2, label) => {
                let mut spans = vec![Span::from("if ").style(Style::default().fg(PINK))];
                spans.append(&mut v.to_spans());
                spans.push(Span::from(" "));
                spans.push(Span::from(format!("{cmp}")).style(Style::default().fg(PINK)));
                spans.push(Span::from(" "));
                spans.append(&mut v2.to_spans());
                spans.push(Span::from(" then goto").style(Style::default().fg(PINK)));
                spans.push(label_span(label));
                spans
            }
            Self::Noop => vec![Span::from("")],
            Self::Pop => vec![build_in_span("pop")],
            Self::Push => vec![build_in_span("push")],
            Self::Return => vec![build_in_span("return")],
            Self::StackOp(op) => vec![build_in_span("stack"), op_span(op)],
        }
    }
}

/// Creates a span formatted for an accumulator with index `idx`.
fn accumulator_span(idx: &usize) -> Span<'static> {
    Span::from(format!("\u{03b1}{idx}")).style(Style::default().fg(FOREGROUND))
}

/// Creates a span formatted for gamma.
fn gamma_span() -> Span<'static> {
    Span::from("\u{03b3}").style(Style::default().fg(PURPLE))
}

/// Creates formatted spans for a memory cell with label `label`.
fn memory_cell_spans(label: &str) -> Vec<Span<'static>> {
    vec![
        Span::from("\u{03c1}(".to_string()).style(Style::default().fg(CYAN)),
        Span::from(label.to_string()).style(Style::default().fg(FOREGROUND)),
        Span::from(")".to_string()).style(Style::default().fg(CYAN)),
    ]
}

/// Creates formatted spans for a index memory cell with type `imcit`.
fn index_memory_cell_spanns(imcit: &IndexMemoryCellIndexType) -> Vec<Span<'static>> {
    let mut spans = vec![Span::from("\u{03c1}(".to_string()).style(Style::default().fg(CYAN))];
    spans.append(&mut imcit.to_spans());
    spans.push(Span::from(")".to_string()).style(Style::default().fg(CYAN)));
    spans
}

/// Span to be used when the value is constant.
fn constant_span(value: &i32) -> Span<'static> {
    Span::from(format!("{value}")).style(Style::default().fg(PURPLE))
}

impl ToSpans for TargetType {
    /// Creates a span from this target type, with specific coloring.
    fn to_spans(&self) -> Vec<Span<'static>> {
        match self {
            Self::Accumulator(idx) => vec![accumulator_span(idx)],
            Self::Gamma => vec![gamma_span()],
            Self::MemoryCell(label) => memory_cell_spans(label),
            Self::IndexMemoryCell(imcit) => index_memory_cell_spanns(imcit),
        }
    }
}

impl ToSpans for IndexMemoryCellIndexType {
    /// Creates a span from this target type, with specific coloring.
    fn to_spans(&self) -> Vec<Span<'static>> {
        match self {
            Self::Accumulator(idx) => vec![accumulator_span(idx)],
            Self::Direct(idx) => vec![constant_span(&usize::try_into(*idx).unwrap_or_default())],
            Self::Gamma => vec![gamma_span()],
            Self::MemoryCell(label) => memory_cell_spans(label),
            Self::Index(idx) => {
                vec![
                    Span::from("\u{03c1}(".to_string()).style(Style::default().fg(GREEN)),
                    Span::from(format!("{idx}")).style(Style::default().fg(PURPLE)),
                    Span::from(")".to_string()).style(Style::default().fg(GREEN)),
                ]
            }
        }
    }
}

impl ToSpans for Value {
    fn to_spans(&self) -> Vec<Span<'static>> {
        match self {
            Self::Accumulator(idx) => vec![accumulator_span(idx)],
            Self::Constant(value) => vec![constant_span(value)],
            Self::Gamma => vec![gamma_span()],
            Self::MemoryCell(label) => memory_cell_spans(label),
            Self::IndexMemoryCell(imcit) => index_memory_cell_spanns(imcit),
        }
    }
}

/// This function turns the input strings into a
/// vector of lines, ready to be printed in the tui.
///
/// An input line can contain a label, instruction and comment.
///
/// If an input line does not contain anything an empty line is inserted.
///
/// Lines that start with `#` are not included in the resulting vector.
///
/// If `enable_alignment` is true, all labels, instructions and comments are aligned below each other
/// (excluding full line comments that start with //, they always start at the beginning of the line).
///
/// If `enable_syntax_highlighting` is true, all labels, instructions and comments will be colored.
pub fn input_to_lines(
    input: &[String],
    enable_alignment: bool,
    enable_syntax_highlighting: bool,
) -> miette::Result<Vec<Line<'static>>> {
    // determine max width of each block
    let (max_label_width, max_instruction_width) = if enable_alignment {
        determine_alignment(input)
    } else {
        (0, 0)
    };

    let mut lines = Vec::new();
    for line in input {
        let parts = match input_parts(line.clone()) {
            Some(parts) => parts,
            None => {
                lines.push(Line::default());
                continue;
            }
        };

        let mut spans = Vec::new();

        // if only comment is set, write comment at beginning of line
        if parts.label.is_none() && parts.instruction.is_none() && parts.comment.is_some() {
            let comment = parts.comment.unwrap();
            // if comment starts with '#' it will not be printed
            if comment.starts_with('#') {
                continue;
            }
            spans.push(string_into_span(
                comment,
                enable_syntax_highlighting,
                COMMENT,
            ));
            lines.push(Line::from(spans));
            continue;
        }

        // handle label
        if let Some(label) = parts.label {
            let len = label.chars().count() + 1; // add plus one because `:` is not included in label
            spans.push(string_into_span(label, enable_syntax_highlighting, GREEN));
            spans.push(string_into_span(
                ":".to_string(),
                enable_syntax_highlighting,
                PINK,
            ));
            // fill spaces if enabled until next part is reached
            if enable_alignment {
                spans.push(fill_span(max_label_width - len + SPACING))
            } else {
                spans.push(fill_span(1));
            }
        } else if (parts.instruction.is_some() || parts.comment.is_some()) && enable_alignment {
            spans.push(fill_span(max_label_width + SPACING))
        }

        // handle instruction
        if let Some(instruction) = parts.instruction {
            let len = instruction.chars().count();
            if enable_syntax_highlighting {
                let instruction = Instruction::try_from(instruction.as_str())?;
                spans.append(&mut instruction.to_spans());
            } else {
                spans.push(Span::from(instruction));
            }
            // fill spaces if enabled until next part is reached
            if enable_alignment {
                spans.push(fill_span(max_instruction_width - len + SPACING));
            } else {
                spans.push(fill_span(1));
            }
        } else if parts.comment.is_some() && enable_alignment {
            spans.push(fill_span(max_instruction_width + SPACING));
        }

        // handle comment
        if let Some(comment) = parts.comment {
            spans.push(string_into_span(
                comment,
                enable_syntax_highlighting,
                COMMENT,
            ));
        }
        lines.push(Line::from(remove_trailing_whitespaces(spans)));
    }

    Ok(lines)
}

/// Reads in the instructions vector and determines what the maximum width for
/// labels and instructions is.
///
/// Returns max width of labels in first variant and max width of instructions
/// in second variant. Label width includes the `:`.
fn determine_alignment(instructions: &[String]) -> (usize, usize) {
    let mut max_label_width = 0;
    let mut max_instruction_width = 0;
    for instruction in instructions {
        // Remove comments
        let instruction = remove_comment(instruction);

        let mut parts = instruction.split_whitespace().collect::<Vec<&str>>();
        if parts.is_empty() {
            continue;
        }
        if parts[0].ends_with(':') {
            // label detected
            let len = parts[0].chars().count();
            if max_label_width < len {
                max_label_width = len;
            }
            parts.remove(0);
        }
        // check if line contained only label and skip because parts is now empty
        if parts.is_empty() {
            continue;
        }

        // count width of instruction
        let mut instruction_width = parts.len() - 1; // used to add in the spaces between the parts
        for part in parts {
            instruction_width += part.chars().count();
        }
        if max_instruction_width < instruction_width {
            max_instruction_width = instruction_width;
        }
    }
    (max_label_width, max_instruction_width)
}

#[derive(Debug, PartialEq)]
struct InputParts {
    /// Label of the input, does not contain ':'
    label: Option<String>,
    instruction: Option<String>,
    comment: Option<String>,
}

/// Splits up the input into its sub components.
///
/// Returns all parts of the input as strings, wrapped in `InputParts`.
/// If an element does not exist, the corresponding entry in the struct is set to null.
///
/// `label` in `InputParts` does not contain the `:`.
fn input_parts(mut input: String) -> Option<InputParts> {
    if input.is_empty() {
        return None;
    }

    // get comment
    let comment = utils::get_comment(&input);
    if let Some(comment) = &comment {
        // remove comment from input
        input = input.replace(comment, "").trim().to_string();
    }

    // check if line only contained comment and is now empty
    if input.is_empty() {
        if let Some(comment) = comment {
            return Some(InputParts {
                label: None,
                instruction: None,
                comment: Some(comment),
            });
        } else {
            return None;
        }
    }

    // check for label
    let mut parts = input.split_whitespace().collect::<Vec<&str>>();
    let label = if parts[0].ends_with(':') {
        Some(parts.remove(0).to_string().replace(':', ""))
    } else {
        None
    };

    // check for instruction
    // at this point only instructions are left in parts
    let instruction = if parts.is_empty() {
        None
    } else {
        Some(parts.join(" ").to_string())
    };

    if comment.is_none() && label.is_none() && instruction.is_none() {
        None
    } else {
        Some(InputParts {
            label,
            instruction,
            comment,
        })
    }
}

/// Creates a new span with `str` as content and `color` as foreground color if
/// `enable_syntax_highlighting` is true. If false, the style is set to default.
fn string_into_span(
    string: String,
    enable_syntax_highlighting: bool,
    color: Color,
) -> Span<'static> {
    let span = Span::from(string);
    if enable_syntax_highlighting {
        span.style(Style::default().fg(color))
    } else {
        span
    }
}

/// Creates a span that contains `amount` number of spaces.
fn fill_span(amount: usize) -> Span<'static> {
    Span::from(" ".repeat(amount))
}

/// Removes all trailing whitespaces from the spans from the right.
fn remove_trailing_whitespaces(mut spans: Vec<Span<'static>>) -> Vec<Span<'static>> {
    // remove trailing whitespaces
    spans.reverse();
    let mut spans_done = Vec::new();
    let mut all_whitespaces_found = false;
    for span in spans {
        if span.content.chars().all(char::is_whitespace) && !all_whitespaces_found {
            continue;
        }
        all_whitespaces_found = true;
        spans_done.push(span);
    }
    spans_done.reverse();
    spans_done
}

#[cfg(test)]
mod tests {

    use crate::app::ui::syntax_highlighting::{determine_alignment, input_parts, InputParts};

    use super::input_to_lines;

    #[test]
    fn test_input_to_lines_alignment_enabled_syntax_highlighting_enabled() {
        let input = vec![
            "main: a := 20".to_string(),
            "// full line comment".to_string(),
            "# invisible".to_string(),
            "long_label: p(h1) := 20 * 30 // comment hey".to_string(),
            "hello: a := p(1)".to_string(),
            "goto main // repeat".to_string(),
            "".to_string(),
            "a := p(1)".to_string(),
            "label:".to_string(),
            "label2: // comment".to_string(),
            "if p(h1) == p(h2) then goto hello".to_string(),
        ];
        let res = input_to_lines(&input, true, true).unwrap();
        assert_eq!(
            res.iter().map(|f| f.to_string()).collect::<Vec<String>>(),
            vec![
                "main:        \u{03b1}0 := 20".to_string(),
                "// full line comment".to_string(),
                "long_label:  \u{03c1}(h1) := 20 * 30                   // comment hey".to_string(),
                "hello:       \u{03b1}0 := \u{03c1}(1)".to_string(),
                "             goto main                          // repeat".to_string(),
                "".to_string(),
                "             \u{03b1}0 := \u{03c1}(1)".to_string(),
                "label:".to_string(),
                "label2:                                         // comment".to_string(),
                "             if \u{03c1}(h1) == \u{03c1}(h2) then goto hello".to_string()
            ]
        );
    }

    #[test]
    fn test_input_to_lines_alignment_disabled_syntax_highlighting_enabled() {
        let input = vec![
            "main: a := 20".to_string(),
            "// full line comment".to_string(),
            "# invisible".to_string(),
            "long_label: p(h1) := 20 * 30 // comment hey".to_string(),
            "hello: a := p(1)".to_string(),
            "goto main // repeat".to_string(),
            "".to_string(),
            "a := p(1)".to_string(),
            "label:".to_string(),
            "label2: // comment".to_string(),
        ];
        let res = input_to_lines(&input, false, true).unwrap();
        assert_eq!(
            res.iter().map(|f| f.to_string()).collect::<Vec<String>>(),
            vec![
                "main: \u{03b1}0 := 20".to_string(),
                "// full line comment".to_string(),
                "long_label: \u{03c1}(h1) := 20 * 30 // comment hey".to_string(),
                "hello: \u{03b1}0 := \u{03c1}(1)".to_string(),
                "goto main // repeat".to_string(),
                "".to_string(),
                "\u{03b1}0 := \u{03c1}(1)".to_string(),
                "label:".to_string(),
                "label2: // comment".to_string(),
            ]
        );
    }

    #[test]
    fn test_determine_alignment() {
        assert_eq!(
            determine_alignment(&vec!["test_label: a := 20 // comment".to_string()]),
            (11, 7)
        );
        assert_eq!(
            determine_alignment(&vec![
                "test_label: a := 20 // comment".to_string(),
                "main: if a == p(h2) then goto test_label // comment".to_string()
            ]),
            (11, 34)
        );
    }

    #[test]
    fn test_input_parts() {
        assert_eq!(
            input_parts("main: a := 20 // comment".to_string()),
            Some(InputParts {
                label: Some("main".to_string()),
                instruction: Some("a := 20".to_string()),
                comment: Some("// comment".to_string())
            })
        );
        assert_eq!(
            input_parts("    a := 20 // comment".to_string()),
            Some(InputParts {
                label: None,
                instruction: Some("a := 20".to_string()),
                comment: Some("// comment".to_string())
            })
        );
        assert_eq!(
            input_parts(" // comment".to_string()),
            Some(InputParts {
                label: None,
                instruction: None,
                comment: Some("// comment".to_string())
            })
        );
        assert_eq!(
            input_parts("main: // comment".to_string()),
            Some(InputParts {
                label: Some("main".to_string()),
                instruction: None,
                comment: Some("// comment".to_string())
            })
        );
        assert_eq!(
            input_parts("main:".to_string()),
            Some(InputParts {
                label: Some("main".to_string()),
                instruction: None,
                comment: None
            })
        );
        assert_eq!(
            input_parts(" a := 20 ".to_string()),
            Some(InputParts {
                label: None,
                instruction: Some("a := 20".to_string()),
                comment: None
            })
        );
        assert_eq!(
            input_parts("main: a := 20 ".to_string()),
            Some(InputParts {
                label: Some("main".to_string()),
                instruction: Some("a := 20".to_string()),
                comment: None
            })
        );
        assert_eq!(input_parts("".to_string()), None);
    }
}
