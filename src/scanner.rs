use crate::config::Config;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

// @IMP-zaruh@ (FROM: @REQ-zaruh@)
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
    ignore_globset: GlobSet,
}

impl Scanner {
    pub fn new(config: Config) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        if let Some(ref ignore_patterns) = config.paths.ignore {
            for raw_pat in ignore_patterns {
                let pat = raw_pat.trim().replace('\\', "/");
                if pat.is_empty() {
                    continue;
                }
                if let Ok(glob) = Glob::new(&pat) {
                    builder.add(glob);
                }
                if !pat.contains('/')
                    && let Ok(glob) = Glob::new(&format!("**/{}", pat))
                {
                    builder.add(glob);
                }
                let trimmed = pat.trim_end_matches('/');
                if !trimmed.ends_with('*') {
                    if let Ok(glob) = Glob::new(&format!("{}/**", trimmed)) {
                        builder.add(glob);
                    }
                    if let Ok(glob) =
                        Glob::new(&format!("**/{}/**", trimmed.trim_start_matches('/')))
                    {
                        builder.add(glob);
                    }
                    if let Ok(glob) = Glob::new(&format!("**/{}", trimmed.trim_start_matches('/')))
                    {
                        builder.add(glob);
                    }
                } else if pat.ends_with("/**") || pat.ends_with("/*") {
                    let dir_prefix = pat.trim_end_matches('*').trim_end_matches('/');
                    if !dir_prefix.is_empty() {
                        if let Ok(glob) = Glob::new(dir_prefix) {
                            builder.add(glob);
                        }
                        if let Ok(glob) =
                            Glob::new(&format!("**/{}", dir_prefix.trim_start_matches('/')))
                        {
                            builder.add(glob);
                        }
                    }
                }
            }
        }
        let ignore_globset = builder.build()?;

        Ok(Self {
            config,
            tag_regex: Regex::new(r"(?P<full>@(?P<id>[a-zA-Z0-9][a-zA-Z0-9\.\-]*)@)")?,
            from_regex: Regex::new(r"FROM:\s*(@?[a-zA-Z0-9\.\-]+(?:\s*,\s*@?[a-zA-Z0-9\.\-]+)*)")?,
            id_regex: Regex::new(r"@?([a-zA-Z0-9][a-zA-Z0-9\.\-]*)@?")?,
            ignore_globset,
        })
    }

    pub fn scan_all(&self) -> Result<Vec<RawItem>> {
        let mut items = Vec::new();
        for root in &self.config.paths.scan {
            for entry in WalkDir::new(root)
                .into_iter()
                .filter_entry(|e| !self.should_ignore(e.path()))
                .filter_map(|e| e.ok())
            {
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

    pub fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().replace('\\', "/");
        let clean_path = path_str.trim_start_matches("./");

        if self.ignore_globset.is_match(clean_path) {
            return true;
        }

        if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            && self.ignore_globset.is_match(file_name)
        {
            return true;
        }

        false
    }

    pub fn scan_file(&self, path: &Path) -> Result<Vec<RawItem>> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                return Ok(Vec::new());
            }
            Err(e) => return Err(e.into()),
        };
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
                prefix: "REQ-".into(),
                item_type: "Requirement".into(),
                requirement_type: Some("Functional".into()),
            }],
        }
    }

    // @UT-siris@ (FROM: @REQ-zaruh@)
    #[test]
    fn test_scan_markdown() {
        let content = r#"
<!-- @REQ-vapik@ -->
### Login feature
Allows users to login.

<!-- @REQ-sivoh@ (FROM: @REQ-vapik@) -->
### Password hashing
Passwords must be hashed.
"#;
        let temp_file = "test_scan.md";
        std::fs::write(temp_file, content).unwrap();

        let scanner = Scanner::new(mock_config()).unwrap();
        let items = scanner.scan_file(Path::new(temp_file)).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "REQ-vapik");
        assert_eq!(items[0].title, "Login feature");
        assert_eq!(items[1].id, "REQ-sivoh");
        assert_eq!(items[1].derived_from, vec!["REQ-vapik"]);

        std::fs::remove_file(temp_file).unwrap();
    }

    // @UT-palan@ (FROM: @REQ-zaruh@)
    #[test]
    fn test_scan_proquint() {
        let content = r#"
<!-- @REQ-lusab-babad@ -->
### Proquint ID
Testing proquint IDs.

<!-- @ARCH-1@ (FROM: @REQ-lusab-babad@) -->
### Legacy ID
Testing legacy ID with proquint as parent.
"#;
        let temp_file = "test_proquint.md";
        std::fs::write(temp_file, content).unwrap();

        let scanner = Scanner::new(mock_config()).unwrap();
        let items = scanner.scan_file(Path::new(temp_file)).unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "REQ-lusab-babad");
        assert_eq!(items[1].id, "ARCH-1");
        assert_eq!(items[1].derived_from, vec!["REQ-lusab-babad"]);

        std::fs::remove_file(temp_file).unwrap();
    }

    // @UT-pisap@ (FROM: @REQ-rimad@)
    #[test]
    fn test_scan_python() {
        let content = r#"
# @REQ-vapik@
class Parent:
    def method_1(self):
        pass
"#;
        let temp_file = "test_scan.py";
        std::fs::write(temp_file, content).unwrap();

        let scanner = Scanner::new(mock_config()).unwrap();
        let items = scanner.scan_file(Path::new(temp_file)).unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "REQ-vapik");
        assert_eq!(items[0].title, "Parent");

        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_should_ignore_glob_patterns() {
        let config = Config {
            paths: Paths {
                scan: vec![],
                ignore: Some(vec![
                    "target/**".into(),
                    ".git/**".into(),
                    "node_modules/**".into(),
                    "dist/**".into(),
                    ".venv/**".into(),
                    "venv/**".into(),
                    "**/*.pyc".into(),
                    "**/__pycache__/**".into(),
                ]),
                db: PathBuf::from("db.json"),
            },
            types: vec![],
        };

        let scanner = Scanner::new(config).unwrap();

        assert!(scanner.should_ignore(Path::new(
            "src/appmgr/admin/__pycache__/__init__.cpython-313.pyc"
        )));
        assert!(scanner.should_ignore(Path::new("src/appmgr/__pycache__")));
        assert!(scanner.should_ignore(Path::new("tests/unit/test.pyc")));
        assert!(scanner.should_ignore(Path::new("target/debug/reqtrace")));
        assert!(scanner.should_ignore(Path::new(".git/HEAD")));
        assert!(scanner.should_ignore(Path::new(".venv/lib/python3.12/site-packages/pkg")));

        assert!(!scanner.should_ignore(Path::new("src/appmgr/admin/app.py")));
        assert!(!scanner.should_ignore(Path::new("docs/requirements.md")));
        assert!(!scanner.should_ignore(Path::new("tests/unit/test_app.py")));
    }
}
