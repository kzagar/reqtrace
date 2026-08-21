use crate::config::Config;
use crate::db::Serializer;
use crate::graph::Graph;
use crate::scanner::Scanner;
use anyhow::Result;
use clap::{Parser, Subcommand};

// @IMP-rulad@ (FROM: ARC-siris)
#[derive(Parser)]
#[command(name = "reqtrace")]
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
    /// Generate the next proquint ID
    GenId {
        /// The prefix for the ID (e.g., REQ-, ARC-)
        #[arg(short, long, default_value = "REQ-")]
        prefix: String,
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
        Commands::GenId { prefix } => {
            let scanner = Scanner::new(config.clone())?;
            let raw_items = scanner.scan_all()?;
            let existing_ids: std::collections::HashSet<String> =
                raw_items.into_iter().map(|it| it.id).collect();

            let next_id = crate::id_gen::generate_next_id(prefix, &existing_ids)?;
            println!("{}", next_id);
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
    }

    Ok(())
}
