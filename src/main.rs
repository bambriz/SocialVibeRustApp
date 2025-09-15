use axum::{
    response::Html,
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber;
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Social Media App server...");

    // Build our application with routes
    let app = Router::new()
        .route("/", get(home))
        .route("/health", get(health))
        .layer(CorsLayer::permissive());

    // Bind to port 5000 on all interfaces as required by Replit
    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    let listener = TcpListener::bind(&addr).await?;
    
    info!("Server running on http://0.0.0.0:5000");
    
    // Start serving requests
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn home() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Social Media App</title>
        <style>
            body { 
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                max-width: 800px; 
                margin: 0 auto; 
                padding: 2rem; 
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                min-height: 100vh;
            }
            .card { 
                background: rgba(255, 255, 255, 0.1); 
                padding: 2rem; 
                border-radius: 10px; 
                backdrop-filter: blur(10px);
                margin: 1rem 0;
            }
            h1 { text-align: center; margin-bottom: 2rem; }
            .feature { margin: 1rem 0; padding: 1rem; background: rgba(255, 255, 255, 0.05); border-radius: 5px; }
        </style>
    </head>
    <body>
        <h1>🚀 Social Media App</h1>
        <div class="card">
            <h2>Welcome to your Rust Social Media Platform!</h2>
            <p>This application is built with:</p>
            <div class="feature">🦀 <strong>Rust & Axum</strong> - High-performance web framework</div>
            <div class="feature">🗃️ <strong>Azure Cosmos DB</strong> - Scalable NoSQL database</div>
            <div class="feature">🧠 <strong>AI Sentiment Analysis</strong> - Color-coded emotional analysis</div>
            <div class="feature">🛡️ <strong>Content Moderation</strong> - Automated hate speech detection</div>
            <div class="feature">💬 <strong>Nested Comments</strong> - Rich discussion threading</div>
            <div class="feature">📊 <strong>Smart Feed Algorithm</strong> - Popularity-based post ranking</div>
        </div>
        <div class="card">
            <h3>🎯 Core Features Coming Soon:</h3>
            <ul>
                <li>User authentication system</li>
                <li>Post creation and management</li>
                <li>Nested comment system</li>
                <li>Sentiment analysis with color mapping</li>
                <li>Automated content moderation</li>
                <li>Infinite scroll feed</li>
            </ul>
        </div>
        <div class="card">
            <p><strong>Status:</strong> ✅ Server is running successfully on Rust!</p>
            <p><strong>Health Check:</strong> <a href="/health" style="color: #90EE90;">Check Server Health</a></p>
        </div>
    </body>
    </html>
    "#)
}

async fn health() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Health Check</title>
        <style>
            body { 
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                max-width: 600px; 
                margin: 0 auto; 
                padding: 2rem; 
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                min-height: 100vh;
                text-align: center;
            }
            .status { 
                background: rgba(0, 255, 0, 0.2); 
                padding: 2rem; 
                border-radius: 10px; 
                margin: 2rem 0;
            }
        </style>
    </head>
    <body>
        <h1>🏥 Health Check</h1>
        <div class="status">
            <h2>✅ Server Status: Healthy</h2>
            <p>Rust Social Media App is running perfectly!</p>
            <p><a href="/" style="color: #90EE90;">← Back to Home</a></p>
        </div>
    </body>
    </html>
    "#)
}