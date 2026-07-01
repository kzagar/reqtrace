use crate::config::Config;
use anyhow::Result;
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

// @IMP2.1@ (FROM: @REQ1.3@)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RawItem {
    pub id: String,
    pub file_path: PathBuf,
    pub line_range: LineRange,
    pub title: String,
    pub derived_from: Vec<String>,
}

pub struct Scanner {
    config: Config,
    tag_regex: Regex,
    from_regex: Regex,
    id_regex: Regex,
}

impl Scanner {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            config,
            tag_regex: Regex::new(r"(?P<full>@(?P<id>[A-Z]+[0-9\.]+)@)")?,
            from_regex: Regex::new(r"FROM:\s*(@?[A-Z0-9\.]+(?:\s*,\s*@?[A-Z0-9\.]+)*)")?,
            id_regex: Regex::new(r"@?([A-Z]+[0-9\.]+)@?")?,
        })
    }

    pub fn scan_all(&self) -> Result<Vec<RawItem>> {
        let mut items = Vec::new();
        for root in &self.config.paths.scan {
            for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if self.should_ignore(path) {
                        continue;
                    }
                    match self.scan_file(path) {
                        Ok(res) => items.extend(res),
                        Err(e) => eprintln!("Warning: failed to scan {:?}: {}", path, e),
                    }
                }
            }
        }
        Ok(items)
    }

    fn should_ignore(&self, path: &Path) -> bool {
        if let Some(ignore) = &self.config.paths.ignore {
            let path_str = path.to_string_lossy();
            return ignore.iter().any(|p| path_str.contains(p));
        }
        false
    }

    pub fn scan_file(&self, path: &Path) -> Result<Vec<RawItem>> {
        let content = std::fs::read_to_string(path)?;
        let mut file_items = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let parser = crate::languages::get_parser(ext);
        let has_parser = parser.is_some();

        let parsed_items = if let Some(ref p) = parser {
            p.parse(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        let mut in_raw_string = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("r#\"") {
                in_raw_string = true;
            }
            if in_raw_string && trimmed.contains("\"#") {
                in_raw_string = false;
                continue;
            }
            if in_raw_string {
                continue;
            }

            if let Some(caps) = self.tag_regex.captures(line) {
                let full_match = &caps["full"];
                let id = &caps["id"];
                let line_num = i + 1;

                // Check if this is a definition or just a reference.
                let is_def = (trimmed.starts_with("<!--") && trimmed.contains(full_match))
                    || (trimmed.starts_with("//") && trimmed.contains(full_match))
                    || (trimmed.starts_with("/*") && trimmed.contains(full_match))
                    || (trimmed.starts_with('#') && trimmed.contains(full_match))
                    || (!has_parser && trimmed.starts_with(full_match));

                if !is_def {
                    continue;
                }

                let mut title = String::new();
                let mut line_range = LineRange {
                    start: line_num,
                    end: line_num,
                };

                if has_parser {
                    let best_item = parsed_items
                        .iter()
                        .filter(|it| it.start_line > line_num || it.end_line >= line_num)
                        .min_by_key(|it| {
                            let distance = if it.start_line <= line_num && it.end_line >= line_num {
                                0
                            } else {
                                it.start_line - line_num
                            };
                            let range_size = it.end_line - it.start_line;
                            (distance, range_size)
                        });

                    if let Some(item) = best_item {
                        title = item.name.clone();
                        line_range = LineRange {
                            start: item.start_line,
                            end: item.end_line,
                        };
                    }
                } else {
                    for j in (i + 1)..lines.len() {
                        let trimmed = lines[j]
                            .trim()
                            .trim_start_matches('#')
                            .trim_start_matches('*')
                            .trim_start_matches('-')
                            .trim();
                        if !trimmed.is_empty() {
                            title = trimmed.to_string();
                            line_range.start = j + 1;
                            let mut end = j + 1;
                            for (k, k_line) in lines.iter().enumerate().skip(j + 1) {
                                if let Some(caps) = self.tag_regex.captures(k_line) {
                                    let full_match = &caps["full"];
                                    let k_trimmed = k_line.trim();
                                    let k_is_def = (k_trimmed.starts_with("<!--")
                                        && k_trimmed.contains(full_match))
                                        || (k_trimmed.starts_with("//")
                                            && k_trimmed.contains(full_match))
                                        || (k_trimmed.starts_with("/*")
                                            && k_trimmed.contains(full_match));
                                    if k_is_def {
                                        break;
                                    }
                                }
                                end = k + 1;
                            }
                            line_range.end = end;
                            break;
                        }
                    }
                }

                let mut derived_from = Vec::new();
                if let Some(from_caps) = self.from_regex.captures(line) {
                    let tags_str = &from_caps[1];
                    for tag_match in self.id_regex.find_iter(tags_str) {
                        derived_from.push(tag_match.as_str().trim_matches('@').to_string());
                    }
                }

                let portable_path = PathBuf::from(path.to_string_lossy().replace('\\', "/"));

                file_items.push(RawItem {
                    id: id.to_string(),
                    file_path: portable_path,
                    line_range,
                    title,
                    derived_from,
                });
            }
        }

        Ok(file_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Paths, TypeMapping};

    fn mock_config() -> Config {
        Config {
            paths: Paths {
                scan: vec![], // Not used in individual file scan test
                ignore: None,
                db: PathBuf::from("db.json"),
            },
            types: vec![TypeMapping {
                prefix: "REQ".into(),
                item_type: "Requirement".into(),
                requirement_type: Some("Functional".into()),
            }],
        }
    }

    // @UT2@ (FROM: @REQ1.3@)
    #[test]
    fn test_scan_markdown() {
        let content = r#"
<!-- @REQ1.1@ -->
### Login feature
Allows users to login.

<!-- @REQ1.2@ (FROM: @REQ1.1@) -->
### Password hashing
Passwords must be hashed.
"#;
        let temp_file = "test_scan.md";
        std::fs::write(temp_file, content).unwrap();

        let scanner = Scanner::new(mock_config()).unwrap();
        let items = scanner.scan_file(Path::new(temp_file)).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "REQ1.1");
        assert_eq!(items[0].title, "Login feature");
        assert_eq!(items[1].id, "REQ1.2");
        assert_eq!(items[1].derived_from, vec!["REQ1.1"]);

        std::fs::remove_file(temp_file).unwrap();
    }

    // @UT19@ (FROM: @REQ1.4@)
    #[test]
    fn test_scan_python() {
        let content = r#"
# @REQ1.1@
class Parent:
    def method_1(self):
        pass
"#;
        let temp_file = "test_scan.py";
        std::fs::write(temp_file, content).unwrap();

        let scanner = Scanner::new(mock_config()).unwrap();
        let items = scanner.scan_file(Path::new(temp_file)).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "REQ1.1");
        assert_eq!(items[0].title, "Parent");

        std::fs::remove_file(temp_file).unwrap();
    }
}
