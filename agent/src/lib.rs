use axum::{routing::get, Router};
use tokio;

pub mod cli;
pub mod controllers;
pub mod server;

pub async fn run(controlled: bool, options: &cli::AgentOptions) -> std::io::Result<()> {
    println!("Starting MiaX Agent...");

    let app = Router::new().nest("/miax", controllers::public::router());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;
    Ok(())
}
