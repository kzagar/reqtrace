use crate::config::Config;
use anyhow::Result;
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// @ARC2.1@ (FROM: @REQ1.3@)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RawItem {
    pub id: String,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub title: String,
    pub derived_from: Vec<String>,
}

pub struct Scanner {
    config: Config,
    tag_regex: Regex,
    from_regex: Regex,
}

impl Scanner {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            config,
            tag_regex: Regex::new(r"@([A-Z]+[0-9\.]+)@")?,
            from_regex: Regex::new(r"FROM:\s*(@[A-Z0-9\.]+@(?:\s*,\s*@[A-Z0-9\.]+@)*)")?,
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
                    items.extend(self.scan_file(path)?);
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

    fn scan_file(&self, path: &Path) -> Result<Vec<RawItem>> {
        let content = std::fs::read_to_string(path)?;
        let mut file_items = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if let Some(caps) = self.tag_regex.captures(line) {
                let id = caps[1].to_string();

                // Simplified title extraction: next line
                let title = if i + 1 < lines.len() {
                    lines[i + 1]
                        .trim()
                        .trim_start_matches('#')
                        .trim()
                        .to_string()
                } else {
                    "".to_string()
                };

                let mut derived_from = Vec::new();
                if let Some(from_caps) = self.from_regex.captures(line) {
                    let tags_str = &from_caps[1];
                    for tag_match in self.tag_regex.find_iter(tags_str) {
                        derived_from.push(tag_match.as_str().trim_matches('@').to_string());
                    }
                }

                file_items.push(RawItem {
                    id,
                    file_path: path.to_path_buf(),
                    line_number: i + 1,
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
}
