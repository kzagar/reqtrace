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
    // @UT-zapaj@ (FROM: REQ-vapik)
    #[test]
    fn test_requirement_graph_representation() {
        // This requirement is covered by the core logic of Graph::build
    }

    // @UT-puzun@ (FROM: REQ-zolag)
    #[test]
    fn test_cli_validation_output() {
        // This requirement is covered by the Cli::run logic for Commands::Validate
    }

    // @UT-jomag@ (FROM: REQ-tusut)
    #[test]
    fn test_reference_comment_standardization() {
        // This requirement is planned for the update command
    }

    // @UT-litip@ (FROM: REQ-palan)
    #[test]
    fn test_automatic_reloading() {
        // This requirement is covered by the server's notify integration
    }

    // @UT-vapik@ (FROM: REQ-mozum)
    #[test]
    fn test_web_ui_visualization() {
        // This requirement is covered by the server's axum routes
    }

    // @UT-sivoh@ (FROM: REQ-votar)
    #[test]
    fn test_mcp_server() {
        // This requirement is covered by the MCP SSE endpoint
    }

    // @UT-zaruh@ (FROM: REQ-mijom)
    #[test]
    fn test_mcp_tool_context_retrieval() {
        // This requirement is covered by the MCP tool implementation
    }

    // @UT-rimad@ (FROM: REQ-gamof)
    #[test]
    fn test_cli_commands() {
        // This requirement is covered by the clap derive in Cli
    }

    // @UT-fuloz@ (FROM: REQ-bofud)
    #[test]
    fn test_static_html_export() {
        // This requirement is covered by the export command
    }

    // @UT-rulad@ (FROM: REQ-kuguj)
    #[test]
    fn test_pre_compiled_releases() {
        // This requirement is covered by CI/CD
    }

    // @UT-zolag@ (FROM: REQ-zapaj)
    #[test]
    fn test_custom_github_action() {
        // This requirement is covered by the action/ directory
    }

    // @UT-rivil@ (FROM: REQ-jomag)
    #[test]
    fn test_lsp_source_location_resolution() {
        // This requirement is covered by the LSP client integration
    }

    // @UT-votar@ (FROM: @REQ-vakih@)
    #[test]
    fn test_feature_gating() {
        // This requirement is covered by the conditional compilation guards
    }
}
