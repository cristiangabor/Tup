mod api;
mod components;
mod handlers;
mod models;
mod routes;
mod state;

use reqwest::Client;
use std::{env, net::SocketAddr};
use tracing::info;

use routes::build_router;
use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "web_service=info".into()),
        )
        .init();

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3001);

    let state = AppState {
        client: Client::new(),
        farmer_api_url: env::var("FARMER_API_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        auth_service_url: env::var("AUTH_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:3002".to_string()),
        auth_public_url: env::var("AUTH_PUBLIC_URL")
            .unwrap_or_else(|_| "http://localhost:3002".to_string()),
    };

    let app = build_router(state);

    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(address).await?;
    info!("web-service listening on http://{address}");
    axum::serve(listener, app).await?;
    Ok(())
}
