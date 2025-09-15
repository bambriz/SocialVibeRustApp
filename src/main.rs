use social_media_app::{AppState, AppConfig};
use social_media_app::routes::create_routes;
// Removed unused import
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Social Media App server...");

    // Load configuration from environment
    let config = AppConfig::from_env();
    info!("Server configuration loaded");

    // Initialize application state
    let app_state = AppState::new(config.clone()).await?;
    info!("Application state initialized");

    // Build our application with routes
    let app = create_routes()
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Bind to port 5000 on all interfaces as required by Replit
    let addr: SocketAddr = config.server_address().parse()?;
    let listener = TcpListener::bind(&addr).await?;
    
    info!("Server running on http://{}", config.server_address());
    
    // Start serving requests
    axum::serve(listener, app).await?;
    
    Ok(())
}