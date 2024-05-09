use std::{
    collections::HashSet,
    fs::{remove_file, File},
    io::{BufRead, BufReader, LineWriter, Write},
};

use miette::{miette, IntoDiagnostic, NamedSource, Result, SourceOffset, SourceSpan};
use ratatui::{
    style::Style,
    text::{Line, Span},
};

use crate::{
    app::{ui::ToSpans, COMMENT, GREEN, PINK},
    instructions::{
        error_handling::{BuildAllowedInstructionsError, InstructionParseError},
        Identifier, Instruction,
    },
};

// TODO remove
/// How many spaces should be between labels, instructions and comments when pretty formatting them
const SPACING: usize = 2;

/// Reads a file into a string vector.
///
/// Each  line is a new entry.
pub fn read_file(path: &str) -> Result<Vec<String>> {
    let mut content = Vec::new();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(miette::miette!(e)),
    };
    let reader = BufReader::new(file);

    for line in reader.lines() {
        match line {
            Ok(l) => content.push(l),
            Err(e) => return Err(miette::miette!(e)),
        }
    }
    Ok(content)
}

pub fn write_file(contet: &Vec<String>, path: &str) -> Result<()> {
    remove_file(path).into_diagnostic()?;
    let file = File::create(path).into_diagnostic()?;

    let mut writer = LineWriter::new(file);
    for line in contet {
        writer.write_all(line.as_bytes()).into_diagnostic()?;
        writer.write_all("\n".as_bytes()).into_diagnostic()?;
    }
    writer.flush().into_diagnostic()?;
    Ok(())
}

/// Writes the specified line to the end of the file.
pub fn write_line_to_file(line: &str, path: &str) -> Result<()> {
    let mut content = match read_file(path) {
        Ok(content) => content,
        Err(e) => return Err(miette!("Unable to read file: {e}")),
    };
    content.push(line.to_string());
    write_file(&content, path)
}

// TODO rewrite to remove all options: This
// function should always apply spacing, syntax highlighting
// and remove hashtag lines -> Allowing these values to be
// configurable increases the complexity of this function
// and I don't think that these options are required that hard
// if an issue comes that these features should be added back in
// I will rewrite this function again to include these features
// (mention in changelog)
// Don't mention in changelog that syntax highlighting can be disabled, as this was never implemented
/// Takes the input instructions and applies optional
/// syntax highlighting and alignment (alignment means in this
/// case that all labels, instructions and comments are aligned).
///
/// If syntax highlighting is enabled, the lines are
/// parsed as instructions to be able to print them highlighted.
/// As this can fail the function returns a result.
///
/// If `remove_hashtag_lines` is set, all lines are removed that begin with `#`.
///
/// Returns a vector of formatted lines.
pub fn format_instructions(
    instructions: &[String],
    enable_alignment: bool,
    enable_syntax_highlighting: bool,
    remove_hashtag_lines: bool,
) -> Result<Vec<Line<'static>>> {
    // determine spacings
    let mut max_label_length = 0;
    let mut max_instruction_length = 0;
    if enable_alignment {
        for instruction in instructions {
            // remove comments
            let instruction = remove_comment(instruction);

            let mut parts = instruction.split_whitespace().collect::<Vec<&str>>();
            if parts.is_empty() {
                continue;
            }
            if parts[0].ends_with(':') {
                // label detected
                if max_label_length < parts[0].chars().count() {
                    max_label_length = parts[0].chars().count();
                }
                parts.remove(0);
            }
            // check if line contained only label and skip because parts is now empty
            if parts.is_empty() {
                continue;
            }

            // count length of instruction
            let mut instruction_length = parts.len() - 1; // used to add in the spaces between the parts
            for part in parts {
                instruction_length += part.len();
            }
            if max_instruction_length < instruction_length {
                max_instruction_length = instruction_length;
            }
        }
    }

    // apply spacing and formatting to each instruction (if enabled)
    let mut pretty_instructions = Vec::new();
    for instruction in instructions {
        let mut label: Option<Vec<Span<'static>>> = None;

        // Check if instruction is empty string
        if instruction.is_empty() {
            pretty_instructions.push(Line::default());
            continue;
        }

        // Check for labels
        let mut parts = instruction.split_whitespace().collect::<Vec<&str>>();
        if parts[0].ends_with(':') {
            // label detected
            let label_span = Span::from(parts.remove(0).replace(":", "").trim().to_string());
            let colon_span = Span::from(":");
            if enable_syntax_highlighting {
                label = Some(vec![
                    label_span.style(Style::default().fg(GREEN)),
                    colon_span.style(Style::default().fg(PINK)),
                ]);
            } else {
                label = Some(vec![label_span, colon_span]);
            }
        } else if parts[0].starts_with("#") && remove_hashtag_lines {
            continue;
        }

        // Detect comment
        let without_label = parts.join(" ");
        let mut comment = match get_comment(&without_label) {
            Some(comment) => {
                let span = Span::from(comment);
                if enable_syntax_highlighting {
                    Some(span.style(Style::default().fg(COMMENT)))
                } else {
                    Some(span)
                }
            }
            None => None,
        };

        // Detect instruction
        // remove comment from instruction line, if comment exists
        let instruction_txt = match comment {
            Some(ref c) => without_label
                .replace(&c.content.to_string(), "")
                .trim()
                .to_string(),
            None => without_label,
        };

        // Create pretty instruction from gathered parts and apply spacing if enabled
        let mut pretty_instruction = Vec::new();
        // label
        match label.take() {
            Some(mut l) => {
                let len = l.iter().map(|f| f.width()).sum::<usize>();
                pretty_instruction.append(&mut l);
                if enable_alignment {
                    pretty_instruction
                        .push(Span::from(" ".repeat(max_label_length - len + SPACING)));
                } else {
                    pretty_instruction.push(Span::from(" "));
                }
            }
            None => {
                if enable_alignment {
                    pretty_instruction.push(Span::from(" ".repeat(max_label_length + SPACING)))
                }
            }
        }
        // instruction
        let len = instruction_txt.chars().count();
        if enable_syntax_highlighting && !instruction_txt.is_empty() {
            let instruction =
                Instruction::try_from(cleanup_instruction_line(instruction_txt).as_str())?;
            pretty_instruction.append(&mut instruction.to_spans());
        } else {
            pretty_instruction.push(Span::from(instruction_txt));
        }
        // add whitespaces after instruction
        if comment.is_some() {
            if enable_alignment {
                pretty_instruction.push(Span::from(
                    " ".repeat(max_instruction_length - len + SPACING),
                ));
            } else {
                pretty_instruction.push(Span::from(" "));
            }
        }

        // comment
        match comment.take() {
            Some(c) => pretty_instruction.push(c),
            None => (),
        }
        // remove trailing whitespaces
        pretty_instruction.reverse();
        let mut pretty_instruction_done = Vec::new();
        let mut all_whitespaces_found = false;
        for span in pretty_instruction {
            if span.content.chars().all(char::is_whitespace) && !all_whitespaces_found {
                continue;
            }
            all_whitespaces_found = true;
            pretty_instruction_done.push(span);
        }
        pretty_instruction_done.reverse();

        pretty_instructions.push(Line::from(pretty_instruction_done));
    }
    Ok(pretty_instructions)
}

// TODO change to take String (with ownership)
/// Removes everything behind # or // from the string
pub fn remove_comment(instruction: &str) -> String {
    instruction
        .lines()
        .map(|line| {
            if let Some(index) = line.find("//") {
                line[..index].trim()
            } else if let Some(index) = line.find('#') {
                line[..index].trim()
            } else {
                line.trim()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Returns the comment inside the string including the delimiter.
/// Otherwise returns `None`.
pub fn get_comment(instruction: &str) -> Option<String> {
    let comment = instruction
        .lines()
        .map(|line| {
            if let Some(index) = line.find("//") {
                line[index..].trim()
            } else if let Some(index) = line.find('#') {
                line[index..].trim()
            } else {
                ""
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if comment.is_empty() {
        None
    } else {
        Some(comment)
    }
}

// TODO remove
/// Removes comments and labels from the string.
pub fn cleanup_instruction_line(mut line: String) -> String {
    // Remove comments
    line = remove_comment(&line);
    // Check for labels
    let splits = line.split_whitespace().collect::<Vec<&str>>();
    if splits.is_empty() {
        return String::new();
    }
    if splits[0].ends_with(':') {
        return splits
            .into_iter()
            .skip(1)
            .collect::<Vec<&str>>()
            .join(" ")
            .to_string();
    }
    line
}

/// Builds a hash set of allowed instruction identifiers, by parsing each line in the input instructions as instruction
/// and storing the id.
pub fn build_instruction_whitelist(
    instructions: Vec<String>,
    path: &str,
) -> Result<HashSet<String>> {
    let instructions = prepare_whitelist_file(instructions);
    let mut whitelisted_instructions = HashSet::new();
    for (idx, s) in instructions.iter().enumerate() {
        match Instruction::try_from(s.as_str()) {
            Ok(i) => {
                let _ = whitelisted_instructions.insert(i.identifier());
            }
            Err(e) => {
                // Workaround for wrong end_range value depending on error.
                // For the line to be printed when more then one character is affected for some reason the range needs to be increased by one.
                let end_range = match e {
                    InstructionParseError::InvalidExpression(_, _) => e.range().1 - e.range().0 + 1,
                    InstructionParseError::UnknownInstruction(_, _) => {
                        e.range().1 - e.range().0 + 1
                    }
                    InstructionParseError::NotANumber(_, _) => e.range().1 - e.range().0,
                    InstructionParseError::UnknownComparison(_, _) => e.range().1 - e.range().0,
                    InstructionParseError::UnknownOperation(_, _) => e.range().1 - e.range().0,
                    InstructionParseError::MissingExpression { range: _, help: _ } => {
                        e.range().1 - e.range().0
                    }
                };
                let file_contents = instructions.join("\n");
                Err(BuildAllowedInstructionsError {
                    src: NamedSource::new(path, instructions.clone().join("\n")),
                    bad_bit: SourceSpan::new(
                        SourceOffset::from_location(
                            file_contents.clone(),
                            idx + 1,
                            e.range().0 + 1,
                        ),
                        end_range,
                    ),
                    reason: e,
                })?;
            }
        }
    }
    Ok(whitelisted_instructions)
}

/// Prepares the whitelist file for parsing to instructions by replacing placeholders with correct alpha notation code.
///
/// The following is replaced:
/// A - a0
/// M - p(h1)
/// C - 0
/// Y - y
/// OP - +
/// CMP - ==
pub fn prepare_whitelist_file(content: Vec<String>) -> Vec<String> {
    let mut prepared = Vec::new();
    for line in content {
        let mut new_chunks = Vec::new();
        match line.as_str() {
            "goto" => {
                prepared.push("goto loop".to_string());
                continue;
            }
            "call" => {
                prepared.push("call loop".to_string());
                continue;
            }
            _ => (),
        }
        let chunks = line.split(' ');
        for chunk in chunks {
            match chunk {
                "A" => new_chunks.push("a0"),
                "M" => new_chunks.push("p(h1)"),
                "C" => new_chunks.push("0"),
                "Y" => new_chunks.push("y"),
                "OP" => new_chunks.push("+"),
                "stackOP" => new_chunks.push("stack+"),
                "CMP" => new_chunks.push("=="),
                "goto" => new_chunks.push("goto loop"),
                _ => new_chunks.push(chunk),
            }
        }
        prepared.push(new_chunks.join(" "));
    }
    prepared
}

#[cfg(test)]
pub mod test_utils {
    use crate::{
        cli::{GlobalArgs, InstructionLimitingArgs},
        runtime::{builder::RuntimeBuilder, Runtime},
    };

    /// Creates a string vector from a &str.
    pub fn string_literal_to_vec(input: &str) -> Vec<String> {
        input.split('\n').map(|f| f.to_string()).collect()
    }

    /// Constructs a runtime using the input string.
    pub fn runtime_from_str(input: &str) -> miette::Result<Runtime> {
        RuntimeBuilder::new(&string_literal_to_vec(input), "test")
            .unwrap()
            .build()
    }

    /// Constructs a new runtime using the input string and applies default global args.
    pub fn runtime_from_str_with_default_cli_args(input: &str) -> miette::Result<Runtime> {
        let mut rb = RuntimeBuilder::new(&string_literal_to_vec(input), "test").unwrap();
        rb.apply_global_cli_args(&GlobalArgs::default()).unwrap();
        rb.build()
    }

    /// Constructs a runtime using the input string.
    pub fn runtime_from_str_with_disable_memory_detection(input: &str) -> miette::Result<Runtime> {
        let mut rb = RuntimeBuilder::new(&string_literal_to_vec(input), "test").unwrap();

        let mut ila = InstructionLimitingArgs::default();
        ila.disable_memory_detection = true;
        rb.apply_instruction_limiting_args(&ila).unwrap();
        rb.build()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{get_comment, prepare_whitelist_file, remove_comment};

    use super::format_instructions;

    #[test]
    fn test_pretty_format_instructions() {
        let instructions = vec![
            "p(a) := 8 // Configure amount of times the loops should run".to_string(),
            "a0 := 2".to_string(),
            "p(b) := p(a)".to_string(),
            "loop_a:".to_string(),
            "push".to_string(),
            "p(b) := p(b) - 1".to_string(),
            "if p(b) > 0 then goto loop_a".to_string(),
            "p(b) := p(a) - 1".to_string(),
            "loop_b:".to_string(),
            "stack*".to_string(),
            "p(b) := p(b) - 1".to_string(),
            "if p(b) > 0 then goto loop_b".to_string(),
            "pop".to_string(),
        ];
        let formatted_instructions = vec![
            "         p(a) := 8                     // Configure amount of times the loops should run".to_string(),
            "         a0 := 2".to_string(),
            "         p(b) := p(a)".to_string(),                              
            "loop_a:".to_string(),                              
            "         push".to_string(),                              
            "         p(b) := p(b) - 1".to_string(),                              
            "         if p(b) > 0 then goto loop_a".to_string(),                              
            "         p(b) := p(a) - 1".to_string(),                              
            "loop_b:".to_string(),                              
            "         stack*".to_string(),                              
            "         p(b) := p(b) - 1".to_string(),                              
            "         if p(b) > 0 then goto loop_b".to_string(),                              
            "         pop".to_string()
        ];
        let pretty_instructions = format_instructions(&instructions, true, false, false).unwrap();
        for (idx, _) in pretty_instructions.iter().enumerate() {
            assert_eq!(
                pretty_instructions[idx].to_string(),
                formatted_instructions[idx]
            );
        }
        assert_eq!(
            format_instructions(&instructions, true, false, false)
                .unwrap()
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>(),
            formatted_instructions
        );
    }

    #[test]
    fn test_remove_comments() {
        assert_eq!(
            remove_comment("a := 5 # Some comment"),
            String::from("a := 5")
        );
        assert_eq!(
            remove_comment("a := 5 // Some comment"),
            String::from("a := 5")
        );
        assert_eq!(remove_comment("a #:= 5"), String::from("a"));
        assert_eq!(remove_comment("a //:= 5"), String::from("a"));
        assert_eq!(remove_comment("#a := 5"), String::from(""));
        assert_eq!(remove_comment("//a := 5"), String::from(""));
    }

    #[test]
    fn test_get_comment() {
        assert_eq!(
            get_comment("a := 5 # Some comment"),
            Some(String::from("# Some comment"))
        );
        assert_eq!(
            get_comment("a := 5 // Some comment"),
            Some(String::from("// Some comment"))
        );
        assert_eq!(get_comment("a #:= 5"), Some(String::from("#:= 5")));
        assert_eq!(get_comment("a //:= 5"), Some(String::from("//:= 5")));
        assert_eq!(get_comment("#a := 5"), Some(String::from("#a := 5")));
        assert_eq!(get_comment("//a := 5"), Some(String::from("//a := 5")));
        assert_eq!(get_comment("a := 5"), None);
    }

    #[test]
    fn test_prepare_whitelist_file() {
        let contents = "A := M\nA := C\nM := A\nY := A OP M\nif A CMP M then goto\ngoto\ncall";
        let contents = prepare_whitelist_file(
            contents
                .split('\n')
                .map(String::from)
                .collect::<Vec<String>>(),
        );
        let after = vec![
            "a0 := p(h1)".to_string(),
            "a0 := 0".to_string(),
            "p(h1) := a0".to_string(),
            "y := a0 + p(h1)".to_string(),
            "if a0 == p(h1) then goto loop".to_string(),
            "goto loop".to_string(),
            "call loop".to_string(),
        ];
        assert_eq!(*contents, after);
    }

    // TODO fix
    //#[test]
    //fn test_remove_special_commented_lines() {
    //    let input = vec![
    //        "a := 5".to_string(),
    //        "# a:= 5".to_string(),
    //        "       # a:= 5".to_string(),
    //        "// a := 5".to_string(),
    //        "       // a := 5".to_string(),
    //        "a := 5 # comment".to_string(),
    //        "a := 5 // comment".to_string(),
    //    ];
    //    let res = format_instructions(&input, false, false, true).unwrap();
    //    assert_eq!(
    //        res.iter().map(|f| f.to_string()).collect::<Vec<String>>(),
    //        vec![
    //            "a := 5",
    //            "// a := 5",
    //            "       // a := 5",
    //            "a := 5 # comment",
    //            "a := 5 // comment"
    //        ]
    //        .iter()
    //        .map(|f| f.to_string())
    //        .collect::<Vec<String>>()
    //    );
    //}
}
