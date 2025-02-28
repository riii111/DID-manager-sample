use axum::Router;
use dotenvy::dotenv;
use tokio;
pub mod cli;
mod config;
mod miax;
mod network;
mod services;
pub use crate::config::app_config;
pub mod controllers;
pub mod server;
pub use crate::config::server_config;
pub use network::network_config;

pub async fn run(controlled: bool, options: &cli::AgentOptions) -> std::io::Result<()> {
    dotenv().ok();
    println!("Starting MiaX Agent...");

    let app = Router::new().nest("/miax", server::make_router());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;
    Ok(())
}
