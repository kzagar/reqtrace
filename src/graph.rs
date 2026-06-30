use crate::config::Config;
use crate::scanner::{LineRange, RawItem};
use std::collections::HashMap;

// @ARC1.1@ (FROM: @REQ1.1@)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraceItem {
    pub id: String,
    pub item_type: String,
    pub requirement_type: Option<String>,
    pub title: String,
    pub file_path: std::path::PathBuf,
    pub line_range: LineRange,
    pub derived_from: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Graph {
    pub items: HashMap<String, TraceItem>,
}

// @ARC3.1@ (FROM: @REQ2.1@)
#[derive(Debug, PartialEq)]
pub enum ValidationIssue {
    Orphan(String),
    BrokenLink(String, String), // source_id, target_id
    DuplicateId(String),
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn build(raw_items: Vec<RawItem>, config: &Config) -> (Self, Vec<ValidationIssue>) {
        let mut graph = Graph::new();
        let mut issues = Vec::new();

        for raw in raw_items {
            if graph.items.contains_key(&raw.id) {
                issues.push(ValidationIssue::DuplicateId(raw.id.clone()));
                continue;
            }

            let item_type_info = config.types.iter().find(|t| raw.id.starts_with(&t.prefix));
            let (item_type, requirement_type) = match item_type_info {
                Some(info) => (info.item_type.clone(), info.requirement_type.clone()),
                None => ("Unknown".to_string(), None),
            };

            graph.items.insert(
                raw.id.clone(),
                TraceItem {
                    id: raw.id,
                    item_type,
                    requirement_type,
                    title: raw.title,
                    file_path: raw.file_path,
                    line_range: raw.line_range,
                    derived_from: raw.derived_from,
                },
            );
        }

        issues.extend(graph.validate());

        (graph, issues)
    }

    pub fn validate(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let mut child_ids = std::collections::HashSet::new();

        for item in self.items.values() {
            for target_id in &item.derived_from {
                child_ids.insert(item.id.clone());
                if !self.items.contains_key(target_id) {
                    issues.push(ValidationIssue::BrokenLink(
                        item.id.clone(),
                        target_id.clone(),
                    ));
                }
            }
        }

        // Orphan detection: Architecture or Test items that don't derive from anything
        // Simplified: items that have no parents and are not Requirements
        for item in self.items.values() {
            if item.item_type != "Requirement" && item.derived_from.is_empty() {
                issues.push(ValidationIssue::Orphan(item.id.clone()));
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // @UT3@ (FROM: @REQ2.1@)
    #[test]
    fn test_validation_orphan_and_broken() {
        let raw_items = vec![
            RawItem {
                id: "ARCH-1".into(),
                file_path: PathBuf::from("arch.md"),
                line_range: LineRange { start: 1, end: 1 },
                title: "Arch 1".into(),
                derived_from: vec!["REQ-1".into()], // Broken link
            },
            RawItem {
                id: "ARCH-2".into(),
                file_path: PathBuf::from("arch.md"),
                line_range: LineRange { start: 10, end: 10 },
                title: "Arch 2".into(),
                derived_from: vec![], // Orphan
            },
        ];

        let config = Config {
            paths: crate::config::Paths {
                scan: vec![],
                ignore: None,
                db: PathBuf::from("db.json"),
            },
            types: vec![
                crate::config::TypeMapping {
                    prefix: "REQ".into(),
                    item_type: "Requirement".into(),
                    requirement_type: None,
                },
                crate::config::TypeMapping {
                    prefix: "ARCH".into(),
                    item_type: "Architecture".into(),
                    requirement_type: None,
                },
            ],
        };

        let (_, issues) = Graph::build(raw_items, &config);

        assert!(issues.contains(&ValidationIssue::BrokenLink(
            "ARCH-1".into(),
            "REQ-1".into()
        )));
        assert!(issues.contains(&ValidationIssue::Orphan("ARCH-2".into())));
    }
}
