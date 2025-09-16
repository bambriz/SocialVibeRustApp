use axum::{
    extract::{State, Json, Path},
    http::{header, HeaderMap},
    response::Json as ResponseJson,
};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::{AppState, AppError, Result};
use crate::models::post::{CreatePostRequest, PostResponse};

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
) -> Result<ResponseJson<Value>> {
    let posts = app_state.post_service.get_posts_feed(20, 0).await?;
    
    Ok(ResponseJson(json!({
        "posts": posts,
        "total": posts.len(),
        "page": 1
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