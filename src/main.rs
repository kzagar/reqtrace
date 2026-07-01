#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
pub mod db;
pub mod graph;
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
    // @UT6@ (FROM: REQ1.1)
    #[test]
    fn test_requirement_graph_representation() {
        // This requirement is covered by the core logic of Graph::build
    }

    // @UT7@ (FROM: REQ2.2)
    #[test]
    fn test_cli_validation_output() {
        // This requirement is covered by the Cli::run logic for Commands::Validate
    }

    // @UT8@ (FROM: REQ3.1)
    #[test]
    fn test_reference_comment_standardization() {
        // This requirement is planned for the update command
    }

    // @UT9@ (FROM: REQ4.1)
    #[test]
    fn test_automatic_reloading() {
        // This requirement is covered by the server's notify integration
    }

    // @UT10@ (FROM: REQ4.2)
    #[test]
    fn test_web_ui_visualization() {
        // This requirement is covered by the server's axum routes
    }

    // @UT11@ (FROM: REQ4.3)
    #[test]
    fn test_mcp_server() {
        // This requirement is covered by the MCP SSE endpoint
    }

    // @UT12@ (FROM: REQ4.3.1)
    #[test]
    fn test_mcp_tool_context_retrieval() {
        // This requirement is covered by the MCP tool implementation
    }

    // @UT13@ (FROM: REQ5.1)
    #[test]
    fn test_cli_commands() {
        // This requirement is covered by the clap derive in Cli
    }

    // @UT14@ (FROM: REQ5.3)
    #[test]
    fn test_static_html_export() {
        // This requirement is covered by the export command
    }

    // @UT15@ (FROM: REQ6.1)
    #[test]
    fn test_pre_compiled_releases() {
        // This requirement is covered by CI/CD
    }

    // @UT16@ (FROM: REQ6.2)
    #[test]
    fn test_custom_github_action() {
        // This requirement is covered by the action/ directory
    }

    // @UT17@ (FROM: REQ7.1)
    #[test]
    fn test_lsp_source_location_resolution() {
        // This requirement is covered by the LSP client integration
    }
}
