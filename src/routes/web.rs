use axum::{response::Html, routing::get, Router};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/health", get(health))
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
            .api-links { display: flex; gap: 1rem; flex-wrap: wrap; margin-top: 1rem; }
            .api-link { 
                background: rgba(255, 255, 255, 0.2); 
                padding: 0.5rem 1rem; 
                border-radius: 5px; 
                color: #90EE90; 
                text-decoration: none; 
                font-size: 0.9rem;
            }
        </style>
    </head>
    <body>
        <h1>🚀 Social Media App</h1>
        <div class="card">
            <h2>Welcome to your Rust Social Media Platform!</h2>
            <p>This application is built with:</p>
            <div class="feature">🦀 <strong>Rust & Axum</strong> - High-performance modular web framework</div>
            <div class="feature">🗃️ <strong>Azure Cosmos DB</strong> - Scalable NoSQL database (coming soon)</div>
            <div class="feature">🧠 <strong>AI Sentiment Analysis</strong> - Color-coded emotional analysis</div>
            <div class="feature">🛡️ <strong>Content Moderation</strong> - Automated hate speech detection</div>
            <div class="feature">💬 <strong>Nested Comments</strong> - Rich discussion threading</div>
            <div class="feature">📊 <strong>Smart Feed Algorithm</strong> - Popularity-based post ranking</div>
        </div>
        <div class="card">
            <h3>🎯 Development Progress:</h3>
            <ul>
                <li>✅ Modular project structure</li>
                <li>✅ Error handling & configuration</li>
                <li>✅ Data models for users, posts, comments</li>
                <li>⏳ Database integration (Cosmos DB)</li>
                <li>⏳ Authentication system</li>
                <li>⏳ API endpoints</li>
                <li>⏳ Sentiment analysis pipeline</li>
                <li>⏳ Content moderation</li>
            </ul>
        </div>
        <div class="card">
            <h3>📡 API Endpoints (Coming Soon):</h3>
            <div class="api-links">
                <a href="/api/v1/health" class="api-link">Health Check</a>
                <a href="/api/v1/posts" class="api-link">Posts API</a>
                <a href="/api/v1/users" class="api-link">Users API</a>
            </div>
        </div>
        <div class="card">
            <p><strong>Status:</strong> ✅ Server is running with modular architecture!</p>
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
            <p>Rust Social Media App is running with modular architecture!</p>
            <p><a href="/" style="color: #90EE90;">← Back to Home</a></p>
        </div>
    </body>
    </html>
    "#)
}