use crate::config::Config;
use crate::graph::Graph;
use crate::scanner::Scanner;
use anyhow::Result;
use axum::Router;
use std::sync::{Arc, RwLock};

#[cfg(feature = "mcp")]
use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::get,
};
#[cfg(feature = "mcp")]
use std::convert::Infallible;

pub struct AppState {
    pub graph: RwLock<Graph>,
    pub config: Config,
}

// @IMP4.4@ (FROM: @REQ4@)
pub async fn start_server(config: Config) -> Result<()> {
    let scanner = Scanner::new(config.clone())?;
    let raw_items = scanner.scan_all()?;
    let (graph, _) = Graph::build(raw_items, &config);

    let state = Arc::new(AppState {
        graph: RwLock::new(graph),
        config,
    });

    #[allow(unused_mut)]
    let mut app = Router::new();
    #[cfg(feature = "mcp")]
    {
        app = app.route("/mcp", get(mcp_handler));
    }
    let app = app.with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(feature = "mcp")]
async fn mcp_handler(
    State(_state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    // Basic SSE implementation for MCP
    let stream = tokio_stream::iter(std::iter::once(Ok(
        Event::default().data("Welcome to reqtrace MCP")
    )));
    Sse::new(stream)
}
