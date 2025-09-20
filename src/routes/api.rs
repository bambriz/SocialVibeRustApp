use axum::{
    routing::{get, post, put, delete}, 
    Router, Json, middleware,
    extract::{State, Request},
    middleware::Next,
    response::Response,
    http::HeaderMap,
};
use serde_json::{json, Value};
use crate::{AppState, AppError};
use crate::routes::{users, posts, auth, comments, vote_routes};
use crate::auth::middleware::auth_middleware;

// Function to create protected routes with auth middleware applied
pub fn protected_routes_with_auth() -> Router<AppState> {
    Router::new()
        .route("/posts", post(posts::create_post))
        .route("/posts/:post_id", put(posts::update_post))
        .route("/posts/:post_id", delete(posts::delete_post))
        .merge(comments::protected_routes())
        .layer(middleware::from_fn(auth_middleware))
}

pub fn routes() -> Router<AppState> {
    let public_routes = Router::new()
        .route("/health", get(api_health))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/users", get(users::get_users))
        .route("/posts", get(posts::get_posts))
        .route("/posts/:post_id", get(posts::get_post))
        .route("/posts/user/:user_id", get(posts::get_user_posts));

    public_routes
        .merge(protected_routes_with_auth())
        .merge(comments::public_routes()) 
        .merge(vote_routes::vote_routes())
}

// Remove the wrapper function - using auth_middleware directly with from_fn_with_state

async fn api_health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "social_media_app",
        "version": "0.1.0",
        "features": {
            "authentication": "implemented",
            "posts": "implemented",
            "comments": "implemented",
            "sentiment_analysis": "implemented",
            "content_moderation": "implemented"
        },
        "endpoints": {
            "auth": {
                "register": "/api/auth/register",
                "login": "/api/auth/login"
            },
            "posts": {
                "create": "POST /api/posts (requires auth)",
                "list": "GET /api/posts",
                "get": "GET /api/posts/:id",
                "user_posts": "GET /api/posts/user/:user_id",
                "update": "PUT /api/posts/:id (requires auth)",
                "delete": "DELETE /api/posts/:id (requires auth)"
            },
            "comments": {
                "list": "GET /api/posts/:post_id/comments",
                "create": "POST /api/posts/:post_id/comments (requires auth)",
                "get": "GET /api/comments/:comment_id",
                "thread": "GET /api/comments/:comment_id/thread",
                "update": "PUT /api/comments/:comment_id (requires auth)",
                "delete": "DELETE /api/comments/:comment_id (requires auth)"
            }
        }
    }))
}

// Removed old placeholder endpoints - now using dedicated route handlers