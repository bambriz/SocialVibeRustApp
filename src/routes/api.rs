use axum::{routing::{get, post}, Router, Json, middleware};
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::{users, posts, auth};
use crate::auth::middleware::auth_middleware;

pub fn routes() -> Router<AppState> {
    let public_routes = Router::new()
        .route("/health", get(api_health))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/users", get(users::get_users))
        .route("/posts", get(posts::get_posts))
        .route("/posts/:post_id", get(posts::get_post));

    let protected_routes = Router::new()
        .route("/posts", post(posts::create_post));

    public_routes.merge(protected_routes)
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

// Removed old placeholder endpoints - now using dedicated route handlers