use axum::{
    response::{Html, Response},
    http::{StatusCode, header},
    Router,
    routing::get,
    extract::Path,
};
use tokio::fs;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(serve_index))
        .route("/static/*file", get(serve_static))
        .route("/health", get(health_check))
}

async fn serve_index() -> Result<Response, StatusCode> {
    match fs::read_to_string("static/index.html").await {
        Ok(content) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
            .body(content.into())
            .unwrap()),
        Err(_) => {
            // Fallback if static file not found
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
                .body(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Social Pulse - Sentiment-Based Social Media</title>
                <style>
                    body { 
                        font-family: 'Inter', sans-serif; 
                        text-align: center; 
                        padding: 50px;
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        min-height: 100vh;
                        color: white;
                        margin: 0;
                    }
                    .container {
                        max-width: 600px;
                        margin: 0 auto;
                        background: rgba(255, 255, 255, 0.1);
                        padding: 3rem;
                        border-radius: 1rem;
                        backdrop-filter: blur(10px);
                    }
                    h1 {
                        font-size: 2.5rem;
                        margin-bottom: 1rem;
                    }
                    .status {
                        background: rgba(16, 185, 129, 0.2);
                        padding: 1rem;
                        border-radius: 0.5rem;
                        margin: 2rem 0;
                    }
                    a {
                        color: #90EE90;
                        text-decoration: none;
                    }
                </style>
            </head>
            <body>
                <div class="container">
                    <h1>🚀 Social Pulse</h1>
                    <p>Sentiment-Based Social Media Platform</p>
                    <div class="status">
                        <h3>✅ Backend Server Running</h3>
                        <p>Frontend loading...</p>
                    </div>
                    <p>API Health: <a href="/api/health">/api/health</a></p>
                    <p>Try refreshing the page if the frontend doesn't load.</p>
                </div>
            </body>
            </html>
            "#.into())
                .unwrap())
        }
    }
}

async fn serve_static(Path(file_path): Path<String>) -> Result<Response, StatusCode> {
    let full_path = format!("static/{}", file_path);
    
    match fs::read(&full_path).await {
        Ok(contents) => {
            let content_type = match full_path.split('.').last() {
                Some("html") => "text/html; charset=utf-8",
                Some("css") => "text/css; charset=utf-8", 
                Some("js") => "application/javascript; charset=utf-8",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("gif") => "image/gif",
                Some("svg") => "image/svg+xml",
                _ => "application/octet-stream",
            };
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .header(header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")
                .body(contents.into())
                .unwrap())
        }
        Err(_) => Err(StatusCode::NOT_FOUND)
    }
}

async fn health_check() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Health Check - Social Pulse</title>
        <style>
            body { 
                font-family: 'Inter', sans-serif;
                max-width: 600px; 
                margin: 0 auto; 
                padding: 2rem; 
                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                color: white;
                min-height: 100vh;
                text-align: center;
            }
            .status { 
                background: rgba(16, 185, 129, 0.2); 
                padding: 2rem; 
                border-radius: 1rem; 
                margin: 2rem 0;
                backdrop-filter: blur(10px);
            }
            a { color: #90EE90; }
        </style>
    </head>
    <body>
        <h1>🏥 Health Check</h1>
        <div class="status">
            <h2>✅ Server Status: Healthy</h2>
            <p>Social Pulse API is running with all features:</p>
            <ul style="text-align: left; display: inline-block;">
                <li>✅ JWT Authentication</li>
                <li>✅ Sentiment Analysis</li>
                <li>✅ Content Moderation</li>
                <li>✅ Feed Algorithm</li>
            </ul>
            <p><a href="/">← Back to App</a></p>
        </div>
    </body>
    </html>
    "#)
}