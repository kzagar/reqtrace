use crate::config::Config;
use crate::db::Serializer;
use crate::graph::Graph;
use crate::scanner::Scanner;
use anyhow::Result;
use clap::{Parser, Subcommand};

// @IMP-rulad@ (FROM: ARC-siris)
#[derive(Parser)]
#[command(name = "reqtrace")]
#[command(version)]
#[command(about = "Systems and software engineering tooling for establishing traceability to requirements", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the background server
    #[cfg(feature = "server")]
    Server,
    /// Sync database and rewrite comments
    Update,
    /// Lint the graph
    Validate,
    /// Generate standalone visualization
    Export {
        #[arg(short, long)]
        output: String,
    },
    /// Generate proquint identifiers
    GenId {
        /// The prefix for the ID (e.g., REQ-, ARC-)
        #[arg(short, long, default_value = "REQ-")]
        prefix: String,
        /// Number of IDs to generate
        #[arg(short, long, short_alias = 'n', default_value_t = 1)]
        count: usize,
    },
    /// List traceability items from .reqtrace/db.json
    List {
        /// Filter by item type (requirement, architecture, implementation, test)
        #[arg(short = 't', long = "type")]
        item_type: Option<String>,
        /// Case-insensitive substring filter on id, title, or file_path
        #[arg(short, long)]
        search: Option<String>,
        /// Show only requirements missing architecture or test coverage
        #[arg(long)]
        gaps: bool,
    },
    /// Show item details, upstream/downstream links, and optional source snippets
    Show {
        /// One or more item IDs (e.g., REQ-kuzur)
        #[arg(required = true)]
        ids: Vec<String>,
        /// Include all transitive upstream parents
        #[arg(short, long)]
        upstream: bool,
        /// Include all transitive downstream children
        #[arg(short, long)]
        downstream: bool,
        /// Print source/documentation line ranges from files
        #[arg(short, long)]
        snippets: bool,
    },
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load_default().or_else(|_| -> Result<Config> {
        // Fallback or error if no config found
        println!("Warning: .reqtrace/config.toml not found, using defaults");
        Ok(Config {
            paths: crate::config::Paths {
                scan: vec!["src".into(), "docs".into()],
                ignore: Some(vec!["target".into(), ".git".into()]),
                db: ".reqtrace/db.json".into(),
            },
            types: vec![
                crate::config::TypeMapping {
                    prefix: "REQ-".into(),
                    item_type: "Requirement".into(),
                    requirement_type: Some("Functional".into()),
                },
                crate::config::TypeMapping {
                    prefix: "ARC-".into(),
                    item_type: "Architecture".into(),
                    requirement_type: None,
                },
            ],
        })
    })?;

    match &cli.command {
        Commands::Validate => {
            let scanner = Scanner::new(config.clone())?;
            let raw_items = scanner.scan_all()?;
            let (_, issues) = Graph::build(raw_items, &config);

            if issues.is_empty() {
                println!("Traceability graph is valid!");
            } else {
                for issue in issues {
                    eprintln!("Issue: {:?}", issue);
                }
                std::process::exit(1);
            }
        }
        Commands::Update => {
            let scanner = Scanner::new(config.clone())?;
            let raw_items = scanner.scan_all()?;
            let (graph, _) = Graph::build(raw_items, &config);

            if let Some(parent) = config.paths.db.parent() {
                std::fs::create_dir_all(parent)?;
            }
            Serializer::save(&graph, &config.paths.db)?;
            println!("Database updated at {:?}", config.paths.db);
        }
        #[cfg(feature = "server")]
        Commands::Server => {
            crate::server::start_server(config).await?;
        }
        Commands::GenId { prefix, count } => {
            let scanner = Scanner::new(config.clone())?;
            let raw_items = scanner.scan_all()?;
            let existing_ids: std::collections::HashSet<String> =
                raw_items.into_iter().map(|it| it.id).collect();

            let ids = crate::id_gen::generate_ids(prefix, &existing_ids, *count)?;
            for id in ids {
                println!("{}", id);
            }
        }
        // @IMP-mozum@ (FROM: @ARC-mozum@)
        Commands::Export { output } => {
            let scanner = Scanner::new(config.clone())?;
            let raw_items = scanner.scan_all()?;
            let (graph, _) = Graph::build(raw_items, &config);

            let graph_json = serde_json::to_string(&graph)?;
            let template = include_str!("../static/index.html");
            let replaced = template.replace(
                "window.__GRAPH_DATA__ = null;",
                &format!("window.__GRAPH_DATA__ = {};", graph_json),
            );

            std::fs::write(output, replaced)?;
            println!("Graph successfully exported to {}", output);
        }
        Commands::List {
            item_type,
            search,
            gaps,
        } => {
            let graph = load_or_build_graph(&config)?;
            let children_map = build_children_map(&graph);
            let type_filter = item_type.as_deref().map(normalize_type_filter);
            let search_lower = search.as_deref().map(|s| s.to_lowercase());

            for item in graph.items.values() {
                if let Some(ref tf) = type_filter
                    && item.item_type.to_lowercase() != *tf
                {
                    continue;
                }
                if let Some(ref q) = search_lower {
                    let hay = format!("{} {} {}", item.id, item.title, item.file_path.display())
                        .to_lowercase();
                    if !hay.contains(q) {
                        continue;
                    }
                }
                let children = children_map.get(&item.id).cloned().unwrap_or_default();
                if *gaps {
                    if item.item_type != "Requirement" {
                        continue;
                    }
                    let has_arch = children.iter().any(|cid| {
                        graph
                            .items
                            .get(cid)
                            .is_some_and(|c| c.item_type == "Architecture")
                    });
                    let has_test = children
                        .iter()
                        .any(|cid| graph.items.get(cid).is_some_and(|c| c.item_type == "Test"));
                    if has_arch && has_test {
                        continue;
                    }
                }
                let from_str = if item.derived_from.is_empty() {
                    "-".to_string()
                } else {
                    item.derived_from.join(",")
                };
                let to_str = if children.is_empty() {
                    "-".to_string()
                } else {
                    children.join(",")
                };
                println!(
                    "{}\t{}\t{}:{}-{}\t{}\tfrom:[{}]\tto:[{}]",
                    item.id,
                    item.item_type,
                    item.file_path.display(),
                    item.line_range.start,
                    item.line_range.end,
                    item.title,
                    from_str,
                    to_str
                );
            }
        }
        Commands::Show {
            ids,
            upstream,
            downstream,
            snippets,
        } => {
            let graph = load_or_build_graph(&config)?;
            let children_map = build_children_map(&graph);
            let mut selected = std::collections::BTreeSet::new();

            for id in ids {
                if !graph.items.contains_key(id) {
                    eprintln!("Warning: item '{}' not found in graph", id);
                    continue;
                }
                selected.insert(id.clone());

                if *upstream || (!*upstream && !*downstream) {
                    let mut stack = vec![id.clone()];
                    let mut visited = std::collections::HashSet::new();
                    while let Some(cur) = stack.pop() {
                        if !visited.insert(cur.clone()) {
                            continue;
                        }
                        if let Some(it) = graph.items.get(&cur) {
                            for p in &it.derived_from {
                                selected.insert(p.clone());
                                if *upstream {
                                    stack.push(p.clone());
                                }
                            }
                        }
                    }
                }

                if *downstream || (!*upstream && !*downstream) {
                    let mut stack = vec![id.clone()];
                    let mut visited = std::collections::HashSet::new();
                    while let Some(cur) = stack.pop() {
                        if !visited.insert(cur.clone()) {
                            continue;
                        }
                        if let Some(ch) = children_map.get(&cur) {
                            for c in ch {
                                selected.insert(c.clone());
                                if *downstream {
                                    stack.push(c.clone());
                                }
                            }
                        }
                    }
                }
            }

            for id in selected {
                if let Some(item) = graph.items.get(&id) {
                    let children = children_map.get(&item.id).cloned().unwrap_or_default();
                    println!(
                        "{} [{}] {} ({}:{}-{})",
                        item.id,
                        item.item_type,
                        item.title,
                        item.file_path.display(),
                        item.line_range.start,
                        item.line_range.end
                    );
                    if !item.derived_from.is_empty() {
                        println!("  from: {}", item.derived_from.join(", "));
                    }
                    if !children.is_empty() {
                        println!("  to:   {}", children.join(", "));
                    }
                    if *snippets && let Ok(content) = std::fs::read_to_string(&item.file_path) {
                        let lines: Vec<&str> = content.lines().collect();
                        let start = item.line_range.start.saturating_sub(1);
                        let end = item.line_range.end.min(lines.len());
                        for (idx, line) in lines.iter().enumerate().take(end).skip(start) {
                            println!("  {:>4} | {}", idx + 1, line);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

// @IMP-vugul@ (FROM: @ARC-kitir@)
fn load_or_build_graph(config: &Config) -> Result<Graph> {
    if config.paths.db.exists() {
        Serializer::load(&config.paths.db)
    } else {
        let scanner = Scanner::new(config.clone())?;
        let raw_items = scanner.scan_all()?;
        let (graph, _) = Graph::build(raw_items, config);
        Ok(graph)
    }
}

fn build_children_map(graph: &Graph) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut children_map: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for item in graph.items.values() {
        for parent in &item.derived_from {
            children_map
                .entry(parent.clone())
                .or_default()
                .push(item.id.clone());
        }
    }
    children_map
}

fn normalize_type_filter(raw: &str) -> String {
    match raw.to_lowercase().as_str() {
        "req" | "requirement" | "requirements" => "requirement".to_string(),
        "arc" | "arch" | "architecture" => "architecture".to_string(),
        "imp" | "impl" | "implementation" => "implementation".to_string(),
        "ut" | "it" | "test" | "tests" => "test".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // @UT-jubid@ (FROM: @REQ-polaf@)
    #[test]
    fn test_cli_list_and_show_parsing() {
        let list_cli = Cli::try_parse_from([
            "reqtrace", "list", "--type", "req", "--search", "auth", "--gaps",
        ])
        .unwrap();
        match list_cli.command {
            Commands::List {
                item_type,
                search,
                gaps,
            } => {
                assert_eq!(item_type.as_deref(), Some("req"));
                assert_eq!(search.as_deref(), Some("auth"));
                assert!(gaps);
            }
            _ => panic!("Expected List"),
        }

        let show_cli =
            Cli::try_parse_from(["reqtrace", "show", "REQ-aaaaa", "--upstream", "--snippets"])
                .unwrap();
        match show_cli.command {
            Commands::Show {
                ids,
                upstream,
                downstream,
                snippets,
            } => {
                assert_eq!(ids, vec!["REQ-aaaaa"]);
                assert!(upstream);
                assert!(!downstream);
                assert!(snippets);
            }
            _ => panic!("Expected Show"),
        }
    }
}
