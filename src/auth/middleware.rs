// Authentication middleware for protecting routes
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::{AppError, AppState};
use crate::auth::Claims;

#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_id: uuid::Uuid,
    pub username: String,
}

pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract AppState from extensions (injected by router)
    let app_state = match request.extensions().get::<AppState>() {
        Some(state) => state.clone(),
        None => {
            // Return error response if AppState not found
            return axum::response::Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body("Internal Server Error".into())
                .unwrap_or_else(|_| axum::response::Response::default());
        }
    };
    
    let headers = request.headers().clone();
    // Extract Authorization header
    let auth_header = match headers.get(header::AUTHORIZATION) {
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
    let user_context = UserContext {
        user_id,
        username: claims.username.clone(),
    };
    
    request.extensions_mut().insert(user_context);
    request.extensions_mut().insert(claims); // Insert Claims for handlers
    
    let response = next.run(request).await;
    response
}