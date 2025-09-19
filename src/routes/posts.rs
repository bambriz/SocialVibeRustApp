use axum::{
    extract::{State, Json, Path, Query},
    http::{header, HeaderMap},
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::{AppState, AppError, Result};
use crate::models::post::{CreatePostRequest, PostResponse};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    20
}

// Validation constants
const MIN_LIMIT: u32 = 1;
const MAX_LIMIT: u32 = 50;
const MAX_OFFSET: u32 = 10000;

pub async fn create_post(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreatePostRequest>,
) -> Result<ResponseJson<Value>> {
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
    
    // Verify JWT token using AuthService
    let claims = app_state.auth_service.verify_token(token)?;
    
    // Parse user_id from string to UUID
    let author_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;
    
    // Create post with authenticated user's ID and username
    let post = app_state.post_service.create_post(request, author_id, claims.username.clone()).await?;
    
    Ok(ResponseJson(json!({
        "post": post,
        "message": "Post created successfully with sentiment analysis and content moderation",
        "author": {
            "id": author_id,
            "username": claims.username
        }
    })))
}

pub async fn get_posts(
    State(app_state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<ResponseJson<Value>> {
    // Validate pagination parameters
    if params.limit < MIN_LIMIT || params.limit > MAX_LIMIT {
        return Err(AppError::ValidationError(format!(
            "Invalid limit. Must be between {} and {}", MIN_LIMIT, MAX_LIMIT
        )));
    }
    
    if params.offset > MAX_OFFSET {
        return Err(AppError::ValidationError(format!(
            "Invalid offset. Must be <= {}", MAX_OFFSET
        )));
    }
    
    let posts = app_state.post_service.get_posts_feed(params.limit, params.offset).await?;
    
    // Calculate pagination metadata (safe from division by zero)
    let page = if params.limit > 0 {
        (params.offset / params.limit) + 1
    } else {
        1
    };
    let has_more = posts.len() == params.limit as usize;
    
    Ok(ResponseJson(json!({
        "posts": posts,
        "total": posts.len(),
        "page": page,
        "limit": params.limit,
        "offset": params.offset,
        "has_more": has_more
    })))
}

pub async fn get_post(
    State(app_state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<ResponseJson<Value>> {
    let post = app_state.post_service.get_post(post_id).await?;
    
    match post {
        Some(post) => Ok(ResponseJson(json!({
            "post": post
        }))),
        None => Err(AppError::NotFound("Post not found".to_string())),
    }
}

pub async fn update_post(
    State(app_state): State<AppState>,
    Path(post_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<CreatePostRequest>,
) -> Result<ResponseJson<Value>> {
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
    
    // Verify JWT token using AuthService
    let claims = app_state.auth_service.verify_token(token)?;
    
    // Parse user_id from string to UUID
    let author_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;
    
    // Update post with authenticated user's ID
    let post = app_state.post_service.update_post(post_id, request, author_id).await?;
    
    Ok(ResponseJson(json!({
        "post": post,
        "message": "Post updated successfully"
    })))
}

pub async fn delete_post(
    State(app_state): State<AppState>,
    Path(post_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<ResponseJson<Value>> {
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
    
    // Verify JWT token using AuthService
    let claims = app_state.auth_service.verify_token(token)?;
    
    // Parse user_id from string to UUID
    let author_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;
    
    // Delete post with authenticated user's ID
    app_state.post_service.delete_post(post_id, author_id).await?;
    
    Ok(ResponseJson(json!({
        "message": "Post deleted successfully"
    })))
}