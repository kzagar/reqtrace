use axum::{
    extract::State,
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use crate::graph::Graph;
use crate::scanner::Scanner;
use crate::config::Config;
use std::sync::{Arc, RwLock};
use anyhow::Result;
use std::convert::Infallible;

pub struct AppState {
    pub graph: RwLock<Graph>,
    pub config: Config,
}

// @ARC4.4@ (FROM: @REQ4.3@)
pub async fn start_server(config: Config) -> Result<()> {
    let scanner = Scanner::new(config.clone())?;
    let raw_items = scanner.scan_all()?;
    let (graph, _) = Graph::build(raw_items, &config);

    let state = Arc::new(AppState {
        graph: RwLock::new(graph),
        config,
    });

    let app = Router::new()
        .route("/mcp", get(mcp_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn mcp_handler(
    State(_state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    // Basic SSE implementation for MCP
    let stream = tokio_stream::iter(std::iter::once(Ok(Event::default().data("Welcome to reqtrace MCP"))));
    Sse::new(stream)
}
