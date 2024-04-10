use std::{
    collections::HashSet,
    fs::{remove_file, File},
    io::{BufRead, BufReader, LineWriter, Write},
};

use miette::{miette, IntoDiagnostic, NamedSource, Result, SourceOffset, SourceSpan};

use crate::{
    instructions::{
        error_handling::{BuildAllowedInstructionsError, InstructionParseError},
        Identifier, Instruction,
    },
    runtime::builder::RuntimeBuilder,
};

/// How many spaces should be between labels, instructions and comments when pretty formatting them
const SPACING: usize = 2;

/// Reads a file into a string vector.
///
/// Each  line is a new entry.
pub fn read_file(path: &str) -> Result<Vec<String>, String> {
    let mut content = Vec::new();
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e.to_string()),
    };
    let reader = BufReader::new(file);

    for line in reader.lines() {
        match line {
            Ok(l) => content.push(l),
            Err(e) => return Err(e.to_string()),
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

pub fn pretty_format_instructions(instructions: &[String]) -> Vec<String> {
    // determine spacings
    let mut max_label_length = 0;
    let mut max_instruction_length = 0;
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

    // apply spacing
    let mut pretty_instructions = Vec::new();
    for instruction in instructions {
        let mut label: Option<String> = None;

        // Check if instruction is empty string
        if instruction.is_empty() {
            pretty_instructions.push(String::new());
            continue;
        }

        // Check for labels
        let mut parts = instruction.split_whitespace().collect::<Vec<&str>>();
        if parts[0].ends_with(':') {
            // label detected
            label = Some(parts.remove(0).trim().to_string());
        }

        // Detect comment
        let without_label = parts.join(" ");
        let comment = get_comment(&without_label);

        // Detect instruction
        // remove comment from instruction line, if comment exists
        let instruction_txt = match comment {
            Some(ref c) => without_label.replace(c, "").trim().to_string(),
            None => without_label,
        };

        // Create pretty instruction from gathered parts
        let mut pretty_instruction = String::new();
        // label
        match label.clone() {
            Some(l) => pretty_instruction.push_str(&format!(
                "{}{}",
                l,
                &" ".repeat(max_label_length - l.chars().count() + SPACING)
            )),
            None => pretty_instruction.push_str(&" ".repeat(max_label_length + SPACING)),
        }
        // instruction
        pretty_instruction.push_str(&format!(
            "{}{}",
            instruction_txt,
            " ".repeat(max_instruction_length - instruction_txt.chars().count() + SPACING)
        ));
        // comment
        if let Some(ref c) = comment {
            pretty_instruction.push_str(&c.to_string());
        } else {
            pretty_instruction.push_str(&" ".repeat(max_instruction_length + SPACING));
            pretty_instruction = pretty_instruction.trim_end().to_string();
        }

        pretty_instructions.push(pretty_instruction);
    }
    pretty_instructions
}

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
/// Otherwise returns an empty string.
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

/// Builds instructions by checking if all used instructions are allowed.
///
/// For that the read in file is first formatted to be a compilable program by using `prepare_whitelist_file()`
#[allow(clippy::match_same_arms)]
pub fn build_instructions_with_whitelist(
    rb: &mut RuntimeBuilder,
    instructions: &[String],
    file: &str,
    whitelist_file: &str,
) -> Result<()> {
    // Instruction whitelist is provided
    let mut whitelisted_instructions_file_contents = match read_file(whitelist_file) {
        Ok(i) => i,
        Err(e) => {
            return Err(miette!(
                "Unable to read whitelisted instruction file [{}]: {}",
                whitelist_file,
                e
            ))
        }
    };
    whitelisted_instructions_file_contents =
        prepare_whitelist_file(whitelisted_instructions_file_contents);
    let mut whitelisted_instructions = HashSet::new();
    for (idx, s) in whitelisted_instructions_file_contents.iter().enumerate() {
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
                let file_contents = whitelisted_instructions_file_contents.join("\n");
                Err(BuildAllowedInstructionsError {
                    src: NamedSource::new(
                        whitelist_file,
                        whitelisted_instructions_file_contents.clone().join("\n"),
                    ),
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
    rb.build_instructions_whitelist(
        &instructions
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>(),
        file,
        &whitelisted_instructions,
    )?;
    Ok(())
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
mod tests {
    use crate::utils::{get_comment, prepare_whitelist_file, remove_comment};

    use super::pretty_format_instructions;

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
        let pretty_instructions = pretty_format_instructions(&instructions);
        for (idx, _) in pretty_instructions.iter().enumerate() {
            assert_eq!(pretty_instructions[idx], formatted_instructions[idx]);
        }
        assert_eq!(
            pretty_format_instructions(&instructions),
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
}
