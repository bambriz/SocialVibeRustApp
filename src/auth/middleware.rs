// Authentication middleware for protecting routes
use axum::{
    extract::Request,
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::AppError;

pub async fn auth_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // TODO: Implement JWT token validation middleware
    
    let auth_header = headers.get(header::AUTHORIZATION);
    
    if auth_header.is_none() {
        return Err(AppError::AuthError("Missing authorization header".to_string()));
    }

    // TODO: Extract and validate JWT token
    // TODO: Add user context to request extensions
    
    let response = next.run(request).await;
    Ok(response)
}