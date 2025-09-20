/*!
 * Comment Routes for Social Pulse - Reddit-Style Comments API
 * 
 * This module handles HTTP endpoints for the hierarchical comment system.
 * Endpoints support creating, reading, updating, and deleting comments with
 * full Reddit-style nesting and sentiment analysis.
 */

use axum::{
    extract::{Path, Query, State},
    Extension,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::Claims,
    models::comment::{CreateCommentRequest, CommentResponse},
    AppError, Result,
};

/// Query parameters for comment listing and pagination
#[derive(Debug, Deserialize)]
pub struct CommentQuery {
    /// Maximum number of comments to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
    /// Filter by depth level
    pub max_depth: Option<u32>,
    /// Sort order: "newest", "oldest", "popular"
    pub sort: Option<String>,
}

/// Response for comment creation
#[derive(Debug, Serialize)]
pub struct CreateCommentResponse {
    pub success: bool,
    pub comment: CommentResponse,
    pub message: String,
}

/// Response for comment updates
#[derive(Debug, Serialize)]
pub struct UpdateCommentResponse {
    pub success: bool,
    pub comment: CommentResponse,
    pub message: String,
}

/// Generic success response
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

/// Create comment routes
/// Public comment routes (no authentication required)
pub fn public_routes() -> Router<crate::AppState> {
    Router::new()
        // Get comments for a specific post
        .route("/posts/:post_id/comments", get(get_post_comments))
        // Get a specific comment thread (for deep-linking)
        .route("/comments/:comment_id/thread", get(get_comment_thread))
        // Get a specific comment by ID
        .route("/comments/:comment_id", get(get_comment_by_id))
}

/// Protected comment routes (authentication required)
pub fn protected_routes() -> Router<crate::AppState> {
    Router::new()
        // Create a new comment on a post
        .route("/posts/:post_id/comments", post(create_comment))
        // Update a comment
        .route("/comments/:comment_id", put(update_comment))
        // Delete a comment
        .route("/comments/:comment_id", delete(delete_comment))
        // TODO: Add auth middleware - for now allowing access to protected routes
}

/// Legacy function for backward compatibility
pub fn create_routes() -> Router<crate::AppState> {
    public_routes().merge(protected_routes())
}

/// Get all comments for a post with Reddit-style hierarchy
/// 
/// GET /api/posts/{post_id}/comments
/// Query params: limit, offset, max_depth, sort
async fn get_post_comments(
    Path(post_id): Path<Uuid>,
    Query(query): Query<CommentQuery>,
    State(app_state): State<crate::AppState>,
) -> Result<Json<Vec<CommentResponse>>> {
    tracing::debug!("📝 Getting comments for post: {} with sort: {:?}", post_id, query.sort);
    
    let comments = app_state.comment_service
        .get_comments_for_post(post_id, query.sort.as_deref())
        .await?;
    
    // TODO: Apply query filters (limit, offset, max_depth) 
    // Sort is now implemented - "popular" sorting preserves hierarchy
    
    tracing::debug!("✅ Retrieved {} comments for post {}", comments.len(), post_id);
    Ok(Json(comments))
}

/// Create a new comment on a post
/// 
/// POST /api/posts/{post_id}/comments
/// Body: CreateCommentRequest
async fn create_comment(
    Path(post_id): Path<Uuid>,
    State(app_state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateCommentRequest>,
) -> Result<Json<CreateCommentResponse>> {
    let user_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;
    
    tracing::debug!("📝 Creating comment on post: {} by user: {}", post_id, user_id);
    
    // Validate that the post_id in the path matches the request
    if request.post_id != post_id {
        return Err(AppError::ValidationError(
            "Post ID in path must match post ID in request body".to_string()
        ));
    }
    
    let comment = app_state.comment_service
        .create_comment(post_id, request, user_id)
        .await?;
    
    tracing::info!("✅ Created comment {} on post {} by user {}", 
                   comment.comment.id, post_id, user_id);
    
    Ok(Json(CreateCommentResponse {
        success: true,
        comment,
        message: "Comment created successfully".to_string(),
    }))
}

/// Get a specific comment thread for deep-linking
/// 
/// GET /api/comments/{comment_id}/thread
async fn get_comment_thread(
    Path(comment_id): Path<Uuid>,
    State(app_state): State<crate::AppState>,
) -> Result<Json<Vec<CommentResponse>>> {
    tracing::debug!("📝 Getting thread for comment: {}", comment_id);
    
    let thread = app_state.comment_service
        .get_comment_thread(comment_id)
        .await?;
    
    tracing::debug!("✅ Retrieved thread for comment {}", comment_id);
    Ok(Json(thread))
}

/// Get a specific comment by ID
/// 
/// GET /api/comments/{comment_id}
async fn get_comment_by_id(
    Path(comment_id): Path<Uuid>,
    State(app_state): State<crate::AppState>,
) -> Result<Json<CommentResponse>> {
    tracing::debug!("📝 Getting comment by ID: {}", comment_id);
    
    // For now, get the thread and return the first comment
    // TODO: Implement direct comment lookup in service
    let thread = app_state.comment_service
        .get_comment_thread(comment_id)
        .await?;
    
    let comment = thread.into_iter()
        .find(|c| c.comment.id == comment_id)
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
    
    tracing::debug!("✅ Retrieved comment {}", comment_id);
    Ok(Json(comment))
}

/// Update a comment's content
/// 
/// PUT /api/comments/{comment_id}
/// Body: { "content": "new content" }
async fn update_comment(
    Path(comment_id): Path<Uuid>,
    State(app_state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
    Json(update_data): Json<serde_json::Value>,
) -> Result<Json<UpdateCommentResponse>> {
    let user_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;
    
    tracing::debug!("📝 Updating comment: {} by user: {}", comment_id, user_id);
    
    let content = update_data.get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("Content field is required".to_string()))?
        .to_string();
    
    let comment = app_state.comment_service
        .update_comment(comment_id, content, user_id)
        .await?;
    
    tracing::info!("✅ Updated comment {} by user {}", comment_id, user_id);
    
    Ok(Json(UpdateCommentResponse {
        success: true,
        comment,
        message: "Comment updated successfully".to_string(),
    }))
}

/// Delete a comment
/// 
/// DELETE /api/comments/{comment_id}
async fn delete_comment(
    Path(comment_id): Path<Uuid>,
    State(app_state): State<crate::AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<SuccessResponse>> {
    let user_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;
    
    tracing::debug!("📝 Deleting comment: {} by user: {}", comment_id, user_id);
    
    app_state.comment_service
        .delete_comment(comment_id, user_id)
        .await?;
    
    tracing::info!("✅ Deleted comment {} by user {}", comment_id, user_id);
    
    Ok(Json(SuccessResponse {
        success: true,
        message: "Comment deleted successfully".to_string(),
    }))
}

/*
 * FUTURE ENHANCEMENTS:
 * 
 * 1. REAL-TIME UPDATES:
 *    - WebSocket support for live comment updates
 *    - Server-Sent Events for comment notifications
 *    - Optimistic UI updates with rollback
 *    
 * 2. ADVANCED FILTERING:
 *    - Filter by emotion/sentiment
 *    - Filter by content moderation status
 *    - Search within comment text
 *    - Sort by engagement/popularity
 *    
 * 3. MODERATION ENDPOINTS:
 *    - Flag comment as inappropriate
 *    - Bulk moderation actions
 *    - Community moderation voting
 *    
 * 4. ANALYTICS ENDPOINTS:
 *    - Comment engagement metrics
 *    - Sentiment trend analysis
 *    - Thread activity summaries
 */