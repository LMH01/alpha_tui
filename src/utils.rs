use std::{
    fs::{remove_file, File},
    io::{BufRead, BufReader, LineWriter, Write},
};

use miette::{IntoDiagnostic, Result};

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

#[cfg(test)]
mod tests {
    use crate::utils::{get_comment, remove_comment};

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
}
