use axum::{
    routing::{get, post, put, delete}, 
    Router, Json, middleware,
    extract::State,
};
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::{users, posts, auth, comments, vote_routes};

// Function to create protected routes without middleware (middleware applied at top level)
pub fn protected_routes_with_auth() -> Router<AppState> {
    Router::new()
        .route("/posts", post(posts::create_post))
        .route("/posts/:post_id", put(posts::update_post))
        .route("/posts/:post_id", delete(posts::delete_post))
        .merge(comments::protected_routes())
        .merge(vote_routes::protected_vote_routes())
}

pub fn routes(app_state: &AppState) -> Router<AppState> {
    let public_routes = Router::new()
        .route("/health", get(api_health))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/users", get(users::get_users))
        .route("/posts", get(posts::get_posts))
        .route("/posts/:post_id", get(posts::get_post))
        .route("/posts/user/:user_id", get(posts::get_user_posts));

    let protected_routes = protected_routes_with_auth()
        .layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware_with_state));

    public_routes
        .merge(protected_routes)
        .merge(comments::public_routes()) 
        .merge(vote_routes::public_vote_routes())
}

// Auth middleware that receives AppState directly as parameter
async fn auth_middleware_with_state(
    State(app_state): State<AppState>,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let headers = request.headers().clone();
    
    // Extract Authorization header
    let auth_header = match headers.get(axum::http::header::AUTHORIZATION) {
        Some(header) => header,
        None => {
            return axum::response::Response::builder()
                .status(401)
                .header("content-type", "application/json")
                .body(r#"{"error": "Missing authorization header"}"#.into())
                .unwrap_or_else(|_| axum::response::Response::default());
        }
    };
    
    // Parse Bearer token
    let auth_str = match auth_header.to_str() {
        Ok(s) => s,
        Err(_) => {
            return axum::response::Response::builder()
                .status(401)
                .header("content-type", "application/json")
                .body(r#"{"error": "Invalid authorization header format"}"#.into())
                .unwrap_or_else(|_| axum::response::Response::default());
        }
    };
    
    if !auth_str.starts_with("Bearer ") {
        return axum::response::Response::builder()
            .status(401)
            .header("content-type", "application/json")
            .body(r#"{"error": "Authorization header must start with 'Bearer '"}"#.into())
            .unwrap_or_else(|_| axum::response::Response::default());
    }
    
    let token = &auth_str[7..]; // Remove "Bearer " prefix
    
    // Validate JWT token
    let claims = match app_state.auth_service.verify_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return axum::response::Response::builder()
                .status(401)
                .header("content-type", "application/json")
                .body(r#"{"error": "Invalid or expired token"}"#.into())
                .unwrap_or_else(|_| axum::response::Response::default());
        }
    };
    
    // Parse user_id from string to UUID
    let user_id = match uuid::Uuid::parse_str(&claims.user_id) {
        Ok(id) => id,
        Err(_) => {
            return axum::response::Response::builder()
                .status(401)
                .header("content-type", "application/json")
                .body(r#"{"error": "Invalid user ID in token"}"#.into())
                .unwrap_or_else(|_| axum::response::Response::default());
        }
    };
    
    // Add both user context and claims to request extensions
    let user_context = crate::auth::middleware::UserContext {
        user_id,
        username: claims.username.clone(),
    };
    
    request.extensions_mut().insert(user_context);
    request.extensions_mut().insert(claims); // Insert Claims for handlers
    
    let response = next.run(request).await;
    response
}

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