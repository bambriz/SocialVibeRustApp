use axum::{routing::get, Router, Json};
use serde_json::{json, Value};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(api_health))
        .route("/posts", get(get_posts))
        .route("/users", get(get_users))
        // TODO: Add POST routes for creating users, posts, comments
        // TODO: Add authentication middleware
}

async fn api_health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "social_media_app",
        "version": "0.1.0",
        "features": {
            "authentication": "not_implemented",
            "posts": "not_implemented",
            "comments": "not_implemented",
            "sentiment_analysis": "not_implemented",
            "content_moderation": "not_implemented"
        }
    }))
}

async fn get_posts() -> Json<Value> {
    // TODO: Implement with database
    Json(json!({
        "posts": [],
        "message": "Post retrieval not implemented yet - database integration pending"
    }))
}

async fn get_users() -> Json<Value> {
    // TODO: Implement with database and authentication
    Json(json!({
        "users": [],
        "message": "User endpoints not implemented yet - authentication system pending"
    }))
}