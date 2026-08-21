use crate::config::Config;
use crate::scanner::{LineRange, RawItem};
use std::collections::BTreeMap;

// @IMP-gizih@ (FROM: ARC-vapik)
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
    pub items: BTreeMap<String, TraceItem>,
}

// @IMP-fuloz@ (FROM: ARC-tusut)
#[derive(Debug, PartialEq)]
pub enum ValidationIssue {
    Orphan(String),
    BrokenLink(String, String), // source_id, target_id
    DuplicateId(String),
    UntestedRequirement(String),
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
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
        let mut children_map: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for item in self.items.values() {
            for target_id in &item.derived_from {
                children_map
                    .entry(target_id.clone())
                    .or_default()
                    .push(item.id.clone());
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

        // Untested Requirement detection: Requirements that have no Tests deriving from them (directly or transitively)
        for item in self.items.values() {
            if item.item_type == "Requirement" && !self.has_test_transitive(&item.id, &children_map)
            {
                issues.push(ValidationIssue::UntestedRequirement(item.id.clone()));
            }
        }

        issues
    }

    fn has_test_transitive(&self, id: &str, children_map: &BTreeMap<String, Vec<String>>) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![id.to_string()];

        while let Some(current_id) = stack.pop() {
            if !visited.insert(current_id.clone()) {
                continue;
            }

            if self
                .items
                .get(&current_id)
                .is_some_and(|it| it.item_type == "Test")
            {
                return true;
            }

            if let Some(children) = children_map.get(&current_id) {
                for child in children {
                    stack.push(child.clone());
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // @UT-kitir@ (FROM: REQ-rulad)
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

    // @UT-kuguj@ (FROM: REQ-rulad)
    #[test]
    fn test_validation_untested_requirement() {
        let raw_items = vec![
            RawItem {
                id: "REQ-1".into(),
                file_path: PathBuf::from("reqs.md"),
                line_range: LineRange { start: 1, end: 1 },
                title: "Requirement 1".into(),
                derived_from: vec![],
            },
            RawItem {
                id: "REQ-2".into(),
                file_path: PathBuf::from("reqs.md"),
                line_range: LineRange { start: 10, end: 10 },
                title: "Requirement 2".into(),
                derived_from: vec![],
            },
            RawItem {
                id: "UT-1".into(),
                file_path: PathBuf::from("test.rs"),
                line_range: LineRange { start: 1, end: 1 },
                title: "Unit Test 1".into(),
                derived_from: vec!["REQ-1".into()],
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
                    prefix: "UT".into(),
                    item_type: "Test".into(),
                    requirement_type: None,
                },
            ],
        };

        let (_, issues) = Graph::build(raw_items, &config);

        // REQ-1 is tested by UT-1
        assert!(!issues.contains(&ValidationIssue::UntestedRequirement("REQ-1".into())));
        // REQ-2 is not tested
        assert!(issues.contains(&ValidationIssue::UntestedRequirement("REQ-2".into())));
    }
}
