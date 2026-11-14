use super::{LanguageParser, ParsedSymbol};
use anyhow::Result;
use regex::Regex;
use std::sync::OnceLock;

// @IMP-vakih@ (FROM: @ARC-bofud@)
pub struct TypeScriptParser;

fn decl_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    CELL.get_or_init(|| {
        Regex::new(
            r"^\s*(?:export\s+(?:default\s+)?)?(?:declare\s+)?(?:async\s+)?(?:(function|class|interface|type|enum)\s+(?P<name1>[a-zA-Z_$][a-zA-Z0-9_$]*)|(?:const|let|var)\s+(?P<name2>[a-zA-Z_$][a-zA-Z0-9_$]*)\s*(?::[^=]+)?=)"
        )
        .unwrap()
    })
}

fn test_regex() -> &'static Regex {
    static CELL: OnceLock<Regex> = OnceLock::new();
    CELL.get_or_init(|| {
        Regex::new(
            r#"^\s*(?:describe|it|test)(?:\.(?:only|skip|todo))?\(\s*['"`](?P<name>[^'"`]+)['"`]"#,
        )
        .unwrap()
    })
}

fn count_braces(line: &str) -> (i32, i32) {
    let mut open = 0;
    let mut close = 0;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_backtick = false;
    let mut escape = false;

    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if escape {
            escape = false;
            i += 1;
            continue;
        }
        if c == '\\' && (in_single || in_double || in_backtick) {
            escape = true;
            i += 1;
            continue;
        }
        if !in_single
            && !in_double
            && !in_backtick
            && c == '/'
            && i + 1 < chars.len()
            && chars[i + 1] == '/'
        {
            break; // line comment
        }
        match c {
            '\'' if !in_double && !in_backtick => in_single = !in_single,
            '"' if !in_single && !in_backtick => in_double = !in_double,
            '`' if !in_single && !in_double => in_backtick = !in_backtick,
            '{' if !in_single && !in_double && !in_backtick => open += 1,
            '}' if !in_single && !in_double && !in_backtick => close += 1,
            _ => {}
        }
        i += 1;
    }
    (open, close)
}

struct ActiveSymbol {
    name: String,
    start_line: usize,
    entry_depth: i32,
    saw_open_brace: bool,
}

impl LanguageParser for TypeScriptParser {
    fn parse(&self, content: &str) -> Result<Vec<ParsedSymbol>> {
        let mut symbols = Vec::new();
        let mut stack: Vec<ActiveSymbol> = Vec::new();
        let decl_re = decl_regex();
        let test_re = test_regex();

        let mut brace_depth: i32 = 0;

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed.starts_with('*')
            {
                let (open, close) = count_braces(line);
                brace_depth += open - close;
                continue;
            }

            let matched_name = if let Some(caps) = test_re.captures(line) {
                Some(caps["name"].to_string())
            } else if let Some(caps) = decl_re.captures(line) {
                caps.name("name1")
                    .or_else(|| caps.name("name2"))
                    .map(|m| m.as_str().to_string())
            } else {
                None
            };

            if let Some(name) = matched_name {
                stack.push(ActiveSymbol {
                    name,
                    start_line: line_num,
                    entry_depth: brace_depth,
                    saw_open_brace: false,
                });
            }

            let (open, close) = count_braces(line);
            if open > 0 {
                for sym in stack.iter_mut() {
                    if !sym.saw_open_brace {
                        sym.saw_open_brace = true;
                    }
                }
            }
            brace_depth += open - close;

            while let Some(top) = stack.last() {
                if top.saw_open_brace && brace_depth <= top.entry_depth {
                    let popped = stack.pop().unwrap();
                    symbols.push(ParsedSymbol {
                        name: popped.name,
                        start_line: popped.start_line,
                        end_line: line_num,
                    });
                } else {
                    break;
                }
            }
        }

        let total_lines = content.lines().count().max(1);
        while let Some(popped) = stack.pop() {
            symbols.push(ParsedSymbol {
                name: popped.name,
                start_line: popped.start_line,
                end_line: total_lines,
            });
        }

        Ok(symbols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // @UT-polaf@ (FROM: @REQ-habot@)
    #[test]
    fn test_typescript_parsing() {
        let content = r#"
// @IMP-aaaaa@ (FROM: @ARC-aaaaa@)
export function calculateTotal(items: number[]): number {
    return items.reduce((a, b) => a + b, 0);
}

// @IMP-bbbbb@ (FROM: @ARC-bbbbb@)
const TaskDialog: React.FC<Props> = ({ id }) => {
    return <div>{id}</div>;
};

// @UT-ccccc@ (FROM: @REQ-ccccc@)
describe('TaskDialog', () => {
    it('renders id', () => {
        expect(true).toBe(true);
    });
});
"#;
        let parser = TypeScriptParser;
        let symbols = parser.parse(content).unwrap();

        let calc = symbols.iter().find(|s| s.name == "calculateTotal").unwrap();
        assert_eq!(calc.start_line, 3);
        assert_eq!(calc.end_line, 5);

        let dlg = symbols
            .iter()
            .find(|s| s.name == "TaskDialog" && s.start_line == 8)
            .unwrap();
        assert_eq!(dlg.start_line, 8);
        assert_eq!(dlg.end_line, 10);

        let desc = symbols
            .iter()
            .find(|s| s.name == "TaskDialog" && s.start_line == 13)
            .unwrap();
        assert_eq!(desc.start_line, 13);
        assert_eq!(desc.end_line, 17);

        let it_blk = symbols.iter().find(|s| s.name == "renders id").unwrap();
        assert_eq!(it_blk.start_line, 14);
        assert_eq!(it_blk.end_line, 16);
    }
}
