pub mod config;
pub mod scanner;
pub mod graph;
pub mod db;
pub mod server;
pub mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run().await
}
