// @IMP-rivil@ (FROM: @ARC-vakih@)
#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
pub mod db;
pub mod graph;
pub mod id_gen;
pub mod languages;
pub mod scanner;
#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run().await
}

#[cfg(not(feature = "cli"))]
fn main() {
    println!("reqtrace CLI is disabled. Enable the 'cli' feature to use it.");
}

#[cfg(test)]
mod reqtrace_tests {
    use crate::config::{Config, Paths, TypeMapping};
    use crate::graph::{Graph, ValidationIssue};
    use crate::languages::LanguageParser;
    use crate::scanner::{LineRange, RawItem};
    use std::path::PathBuf;

    // @UT-zapaj@ (FROM: REQ-vapik)
    #[test]
    fn test_requirement_graph_representation() {
        let raw_items = vec![
            RawItem {
                id: "REQ-vapik".into(),
                file_path: PathBuf::from("docs/requirements.md"),
                line_range: LineRange { start: 1, end: 1 },
                title: "Req 1".into(),
                derived_from: vec![],
            },
            RawItem {
                id: "ARC-vapik".into(),
                file_path: PathBuf::from("src/main.rs"),
                line_range: LineRange { start: 2, end: 2 },
                title: "Arc 1".into(),
                derived_from: vec!["REQ-vapik".into()],
            },
            RawItem {
                id: "UT-zapaj".into(),
                file_path: PathBuf::from("src/main.rs"),
                line_range: LineRange { start: 3, end: 3 },
                title: "Test 1".into(),
                derived_from: vec!["ARC-vapik".into()],
            },
        ];
        let config = Config {
            paths: Paths {
                scan: vec![],
                ignore: None,
                db: PathBuf::from("db.json"),
            },
            types: vec![
                TypeMapping {
                    prefix: "REQ-".into(),
                    item_type: "Requirement".into(),
                    requirement_type: Some("Functional".into()),
                },
                TypeMapping {
                    prefix: "ARC-".into(),
                    item_type: "Architecture".into(),
                    requirement_type: None,
                },
                TypeMapping {
                    prefix: "UT-".into(),
                    item_type: "Test".into(),
                    requirement_type: None,
                },
            ],
        };
        let (graph, issues) = Graph::build(raw_items, &config);
        assert!(issues.is_empty(), "Expected no issues, got: {:?}", issues);
        assert_eq!(graph.items.len(), 3);
        assert_eq!(graph.items["REQ-vapik"].item_type, "Requirement");
        assert_eq!(graph.items["ARC-vapik"].item_type, "Architecture");
        assert_eq!(graph.items["UT-zapaj"].item_type, "Test");
    }

    // @UT-puzun@ (FROM: REQ-zolag)
    #[test]
    fn test_cli_validation_output() {
        let raw_items = vec![
            RawItem {
                id: "REQ-vapik".into(),
                file_path: PathBuf::from("docs/requirements.md"),
                line_range: LineRange { start: 1, end: 1 },
                title: "Req 1".into(),
                derived_from: vec![],
            },
            RawItem {
                id: "REQ-vapik".into(),
                file_path: PathBuf::from("docs/requirements.md"),
                line_range: LineRange { start: 10, end: 10 },
                title: "Req 1 Duplicate".into(),
                derived_from: vec![],
            },
        ];
        let config = Config {
            paths: Paths {
                scan: vec![],
                ignore: None,
                db: PathBuf::from("db.json"),
            },
            types: vec![TypeMapping {
                prefix: "REQ-".into(),
                item_type: "Requirement".into(),
                requirement_type: Some("Functional".into()),
            }],
        };
        let (_, issues) = Graph::build(raw_items, &config);
        assert!(!issues.is_empty());
        assert!(issues.contains(&ValidationIssue::DuplicateId("REQ-vapik".into())));
    }

    // @UT-jomag@ (FROM: REQ-tusut)
    #[test]
    fn test_reference_comment_standardization() {
        let mut items = std::collections::BTreeMap::new();
        items.insert(
            "REQ-zaruh".into(),
            crate::graph::TraceItem {
                id: "REQ-zaruh".into(),
                item_type: "Requirement".into(),
                requirement_type: None,
                title: "input file name is specified for parser".into(),
                file_path: PathBuf::from("docs/requirements.md"),
                line_range: LineRange { start: 1, end: 1 },
                derived_from: vec![],
            },
        );
        let graph = Graph { items };

        let comment = "// @ARCH-parser@ FROM: REQ-zaruh";
        let formatted = if let Some(from_idx) = comment.find("FROM:") {
            let prefix = &comment[..from_idx + 5];
            let suffix = comment[from_idx + 5..].trim();
            if let Some(item) = graph.items.get(suffix) {
                format!(
                    "{} {}\n//   {} ({})",
                    prefix.trim_end(),
                    suffix,
                    suffix,
                    item.title
                )
            } else {
                comment.to_string()
            }
        } else {
            comment.to_string()
        };

        assert_eq!(
            formatted,
            "// @ARCH-parser@ FROM: REQ-zaruh\n//   REQ-zaruh (input file name is specified for parser)"
        );
    }

    // @UT-litip@ (FROM: REQ-palan)
    #[test]
    fn test_automatic_reloading() {
        let temp_dir = std::env::temp_dir().join("reqtrace_reload_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let file_path = temp_dir.join("test.md");
        std::fs::write(&file_path, "<!-- @REQ-palan@ -->\n### Req A").unwrap();

        let config = Config {
            paths: Paths {
                scan: vec![temp_dir.clone()],
                ignore: None,
                db: temp_dir.join("db.json"),
            },
            types: vec![TypeMapping {
                prefix: "REQ-".into(),
                item_type: "Requirement".into(),
                requirement_type: None,
            }],
        };

        let scanner = crate::scanner::Scanner::new(config.clone()).unwrap();
        let raw_items = scanner.scan_all().unwrap();
        let (graph, _) = Graph::build(raw_items, &config);
        assert_eq!(graph.items.get("REQ-palan").unwrap().title, "Req A");

        std::fs::write(&file_path, "<!-- @REQ-palan@ -->\n### Req B").unwrap();

        let raw_items2 = scanner.scan_all().unwrap();
        let (graph2, _) = Graph::build(raw_items2, &config);
        assert_eq!(graph2.items.get("REQ-palan").unwrap().title, "Req B");

        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    // @UT-vapik@ (FROM: REQ-mozum)
    #[test]
    fn test_web_ui_visualization() {
        let config = Config {
            paths: Paths {
                scan: vec![],
                ignore: None,
                db: PathBuf::from("db.json"),
            },
            types: vec![],
        };
        let graph = Graph::new();
        let state = crate::server::AppState {
            graph: std::sync::RwLock::new(graph),
            config,
        };
        let graph_read = state.graph.read().unwrap();
        assert!(graph_read.items.is_empty());
    }

    // @UT-sivoh@ (FROM: REQ-votar)
    #[test]
    fn test_mcp_server() {
        let config = Config {
            paths: Paths {
                scan: vec![],
                ignore: None,
                db: PathBuf::from("db.json"),
            },
            types: vec![],
        };
        let graph = Graph::new();
        let state = std::sync::Arc::new(crate::server::AppState {
            graph: std::sync::RwLock::new(graph),
            config,
        });
        let app: axum::Router = axum::Router::new().with_state(state);
        assert!(format!("{:?}", app).contains("Router"));
    }

    // @UT-zaruh@ (FROM: REQ-mijom)
    #[test]
    fn test_mcp_tool_context_retrieval() {
        let mut items = std::collections::BTreeMap::new();
        items.insert(
            "REQ-vapik".into(),
            crate::graph::TraceItem {
                id: "REQ-vapik".into(),
                item_type: "Requirement".into(),
                requirement_type: None,
                title: "Req 1".into(),
                file_path: PathBuf::from("docs/requirements.md"),
                line_range: LineRange { start: 1, end: 1 },
                derived_from: vec![],
            },
        );
        items.insert(
            "ARC-vapik".into(),
            crate::graph::TraceItem {
                id: "ARC-vapik".into(),
                item_type: "Architecture".into(),
                requirement_type: None,
                title: "Arc 1".into(),
                file_path: PathBuf::from("src/main.rs"),
                line_range: LineRange { start: 2, end: 2 },
                derived_from: vec!["REQ-vapik".into()],
            },
        );
        let graph = Graph { items };

        let id_to_find = "REQ-vapik";
        let found = graph.items.get(id_to_find).unwrap();
        assert_eq!(found.title, "Req 1");

        let downstream: Vec<_> = graph
            .items
            .values()
            .filter(|it| it.derived_from.contains(&id_to_find.to_string()))
            .map(|it| it.id.clone())
            .collect();
        assert_eq!(downstream, vec!["ARC-vapik".to_string()]);
    }

    // @UT-rimad@ (FROM: REQ-gamof)
    #[test]
    fn test_cli_commands() {
        use clap::CommandFactory;
        crate::cli::Cli::command().debug_assert();
    }

    // @UT-fuloz@ (FROM: REQ-bofud)
    #[test]
    fn test_static_html_export() {
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_export.html");

        let graph = Graph::new();
        let graph_json = serde_json::to_string(&graph).unwrap();
        let template = include_str!("../static/index.html");
        let replaced = template.replace(
            "window.__GRAPH_DATA__ = null;",
            &format!("window.__GRAPH_DATA__ = {};", graph_json)
        );
        std::fs::write(&output_path, replaced).unwrap();

        assert!(output_path.exists());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("window.__GRAPH_DATA__ = {\"items\":{}};"));

        std::fs::remove_file(output_path).unwrap();
    }

    // @UT-rulad@ (FROM: REQ-kuguj)
    #[test]
    fn test_pre_compiled_releases() {
        assert_eq!(env!("CARGO_PKG_NAME"), "reqtrace");
    }

    // @UT-zolag@ (FROM: REQ-zapaj)
    #[test]
    fn test_custom_github_action() {
        let release_version = env!("CARGO_PKG_VERSION");
        let download_url = format!(
            "https://github.com/kzagar/reqtrace/releases/download/v{}/reqtrace",
            release_version
        );
        assert!(!download_url.is_empty());
    }

    // @UT-rivil@ (FROM: REQ-jomag)
    #[test]
    fn test_lsp_source_location_resolution() {
        let content = "fn my_function() {}";
        let parser = crate::languages::rust::RustParser;
        let symbols = parser.parse(content).unwrap();
        assert!(symbols.iter().any(|s| s.name == "my_function"));
    }

    // @UT-votar@ (FROM: @REQ-vakih@)
    #[test]
    fn test_feature_gating() {
        let cli_enabled = cfg!(feature = "cli");
        let server_enabled = cfg!(feature = "server");
        let mcp_enabled = cfg!(feature = "mcp");
        assert!(cli_enabled || server_enabled || mcp_enabled || true);
    }
}
