use std::{
    collections::HashSet,
    fs::{remove_file, File},
    io::{BufRead, BufReader, LineWriter, Write},
};

use miette::{miette, IntoDiagnostic, NamedSource, Result, SourceOffset, SourceSpan};

use crate::instructions::{
    error_handling::{BuildAllowedInstructionsError, InstructionParseError},
    Identifier, Instruction,
};

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
