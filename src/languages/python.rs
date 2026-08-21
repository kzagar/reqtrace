use super::{LanguageParser, ParsedSymbol};
use anyhow::Result;
use regex::Regex;
use std::sync::OnceLock;

pub struct PythonParser;

fn class_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    CELL.get_or_init(|| Regex::new(r"^\s*class\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)").unwrap())
}

fn def_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    CELL.get_or_init(|| {
        Regex::new(r"^\s*(?:async\s+)?def\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)").unwrap()
    })
}

fn get_indentation(line: &str) -> usize {
    let mut count = 0;
    for c in line.chars() {
        if c == ' ' {
            count += 1;
        } else if c == '\t' {
            count += 4;
        } else {
            break;
        }
    }
    count
}

fn is_empty_or_comment(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.is_empty() || trimmed.starts_with('#')
}

struct ActiveBlock {
    indent: usize,
    name: String,
    start_line: usize,
}

impl LanguageParser for PythonParser {
    fn parse(&self, content: &str) -> Result<Vec<ParsedSymbol>> {
        let mut symbols = Vec::new();
        let mut stack: Vec<ActiveBlock> = Vec::new();
        let class_re = class_regex();
        let def_re = def_regex();
        let mut last_non_empty_line = 1;

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;

            if is_empty_or_comment(line) {
                continue;
            }

            let indent = get_indentation(line);

            // Pop blocks that are deeper than or equal to current indentation
            while let Some(top) = stack.last() {
                if indent <= top.indent {
                    let popped = stack.pop().unwrap();
                    symbols.push(ParsedSymbol {
                        name: popped.name,
                        start_line: popped.start_line,
                        end_line: last_non_empty_line,
                    });
                } else {
                    break;
                }
            }

            // Check if it's a new declaration
            let is_class = class_re.captures(line);
            let is_def = def_re.captures(line);

            if let Some(caps) = is_class {
                let local_name = &caps["name"];
                let full_name = if let Some(parent) = stack.last() {
                    format!("{}::{}", parent.name, local_name)
                } else {
                    local_name.to_string()
                };
                stack.push(ActiveBlock {
                    indent,
                    name: full_name,
                    start_line: line_num,
                });
            } else if let Some(caps) = is_def {
                let local_name = &caps["name"];
                let full_name = if let Some(parent) = stack.last() {
                    format!("{}::{}", parent.name, local_name)
                } else {
                    local_name.to_string()
                };
                stack.push(ActiveBlock {
                    indent,
                    name: full_name,
                    start_line: line_num,
                });
            }

            last_non_empty_line = line_num;
        }

        // Pop remaining blocks
        while let Some(popped) = stack.pop() {
            symbols.push(ParsedSymbol {
                name: popped.name,
                start_line: popped.start_line,
                end_line: last_non_empty_line,
            });
        }

        Ok(symbols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // @UT-tusut@ (FROM: @REQ-rimad@)
    #[test]
    fn test_python_parsing() {
        let content = r#"
class Parent:
    def method_1(self):
        pass

def top_level_func():
    pass
"#;
        let parser = PythonParser;
        let symbols = parser.parse(content).unwrap();

        assert_eq!(symbols.len(), 3);

        // Find Parent::method_1
        let m1 = symbols
            .iter()
            .find(|s| s.name == "Parent::method_1")
            .unwrap();
        assert_eq!(m1.start_line, 3);
        assert_eq!(m1.end_line, 4);

        // Find Parent
        let p = symbols.iter().find(|s| s.name == "Parent").unwrap();
        assert_eq!(p.start_line, 2);
        assert_eq!(p.end_line, 4);

        // Find top_level_func
        let t = symbols.iter().find(|s| s.name == "top_level_func").unwrap();
        assert_eq!(t.start_line, 6);
        assert_eq!(t.end_line, 7);
    }
}
