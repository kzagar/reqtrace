pub mod cli;
pub mod config;
pub mod db;
pub mod graph;
pub mod scanner;
pub mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run().await
}
