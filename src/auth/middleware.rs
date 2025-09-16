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
    State(app_state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract Authorization header
    let auth_header = headers.get(header::AUTHORIZATION)
        .ok_or_else(|| AppError::AuthError("Missing authorization header".to_string()))?;
    
    // Parse Bearer token
    let auth_str = auth_header.to_str()
        .map_err(|_| AppError::AuthError("Invalid authorization header format".to_string()))?;
    
    if !auth_str.starts_with("Bearer ") {
        return Err(AppError::AuthError("Authorization header must start with 'Bearer '".to_string()));
    }
    
    let token = &auth_str[7..]; // Remove "Bearer " prefix
    
    // Validate JWT token
    let claims = app_state.auth_service.verify_token(token)?;
    
    // Parse user_id from string to UUID
    let user_id = uuid::Uuid::parse_str(&claims.user_id)
        .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;
    
    // Add user context to request extensions
    let user_context = UserContext {
        user_id,
        username: claims.username,
    };
    
    request.extensions_mut().insert(user_context);
    
    let response = next.run(request).await;
    Ok(response)
}