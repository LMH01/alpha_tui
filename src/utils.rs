use std::{
    fs::File,
    io::{BufRead, BufReader},
};

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
        if parts[0].ends_with(":") {
            // label detected
            if max_label_length < parts[0].len() {
                max_label_length = parts[0].len();
            }
            parts.remove(0);
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
        let mut pretty_instruction = String::new();

        // Check for labels
        let mut parts = instruction.split_whitespace().collect::<Vec<&str>>();
        if parts.is_empty() {
            pretty_instructions.push(String::new());
            continue;
        }
        if parts[0].ends_with(":") {
            // label detected
            let label = parts.remove(0);
            let padding = " ".repeat(max_label_length - label.len());
            pretty_instruction.push_str(&format!("{}{} ", label, padding));
        } else {
            pretty_instruction.push_str(&format!("{} ", " ".repeat(max_label_length)));
        }

        let without_label = parts.join(" ");
        let comment = get_comment(&without_label);
        let instruction = match get_comment(&without_label) {
            Some(c) => without_label.replace(&c, ""),
            None => without_label,
        };

        // instruction printing
        let padding = " ".repeat(max_instruction_length - instruction.len());
        pretty_instruction.push_str(&format!("{}{}", instruction, padding));

        // comment printing
        if let Some(c) = comment {
            pretty_instruction.push_str(&format!(" {}", c));
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
    use crate::utils::{remove_comment, get_comment};

    fn test_pretty_format_instructions() {
        let instructions = vec!["", ""];
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
