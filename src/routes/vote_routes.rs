use axum::{
    extract::{State, Path},
    Extension,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use uuid::Uuid;

use crate::{AppState, Result, AppError};
use crate::models::{CreateVoteRequest, TagVoteCount, VoteSummary};
use crate::models::vote::Vote;
use crate::auth::Claims;

/// Vote-related API routes
// Public voting routes (no auth required)
pub fn public_vote_routes() -> Router<AppState> {
    Router::new()
        .route("/vote/:target_id/:target_type", get(get_vote_summary))
        .route("/vote/counts/:target_id/:target_type", get(get_vote_counts))
}

// Protected voting routes (auth required)
pub fn protected_vote_routes() -> Router<AppState> {
    Router::new()
        .route("/vote/user/:target_id/:target_type/:vote_type/:tag", get(get_user_vote))
        .route("/vote", post(cast_vote))
        .route("/vote/:target_id/:target_type/:vote_type/:tag", delete(remove_vote))
}

// Legacy function for backward compatibility
pub fn vote_routes() -> Router<AppState> {
    public_vote_routes()
}

/// Cast or update a vote on emotion/content tags
async fn cast_vote(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateVoteRequest>,
) -> Result<Json<VoteSummary>> {
    let user_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;

    // VoteService handles the toggle logic internally
    match state.vote_service.cast_vote(user_id, request.clone()).await {
        Ok(_) => {
            // Vote was cast or updated successfully
        },
        Err(AppError::ValidationError(msg)) if msg == "Vote removed" => {
            // Vote was toggled off (removed)
        },
        Err(e) => return Err(e),
    }

    // Trigger popularity recalculation after voting
    match request.target_type.as_str() {
        "post" => {
            if let Err(e) = state.post_service.update_popularity_after_vote(request.target_id).await {
                tracing::warn!("Failed to update post popularity after vote: {}", e);
            }
        },
        "comment" => {
            if let Err(e) = state.comment_service.update_popularity_after_vote(request.target_id, Some(&*state.vote_service)).await {
                tracing::warn!("Failed to update comment popularity after vote: {}", e);
            }
        },
        _ => {} // Ignore unknown target types
    }

    // Return updated vote summary
    let summary = state.vote_service.get_vote_summary(request.target_id, &request.target_type).await?;
    Ok(Json(summary))
}

/// Get comprehensive vote summary for a target (post or comment)
async fn get_vote_summary(
    State(state): State<AppState>,
    Path((target_id, target_type)): Path<(Uuid, String)>,
) -> Result<Json<VoteSummary>> {
    let summary = state.vote_service.get_vote_summary(target_id, &target_type).await?;
    Ok(Json(summary))
}

/// Get vote counts for all tags on a target
async fn get_vote_counts(
    State(state): State<AppState>,
    Path((target_id, target_type)): Path<(Uuid, String)>,
) -> Result<Json<Vec<TagVoteCount>>> {
    let counts = state.vote_service.get_vote_counts(target_id, &target_type).await?;
    Ok(Json(counts))
}

/// Get user's specific vote on a target and tag
async fn get_user_vote(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((target_id, target_type, vote_type, tag)): Path<(Uuid, String, String, String)>,
) -> Result<Json<Option<Vote>>> {
    let user_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;

    let vote = state.vote_service
        .get_user_vote(user_id, target_id, &vote_type, &tag)
        .await?;
    Ok(Json(vote))
}

/// Remove a user's vote
async fn remove_vote(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((target_id, target_type, vote_type, tag)): Path<(Uuid, String, String, String)>,
) -> Result<Json<VoteSummary>> {
    let user_id = Uuid::parse_str(&claims.user_id)
        .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;

    state.vote_service
        .remove_vote(user_id, target_id, &vote_type, &tag)
        .await?;

    // Trigger popularity recalculation after vote removal
    match target_type.as_str() {
        "post" => {
            if let Err(e) = state.post_service.update_popularity_after_vote(target_id).await {
                tracing::warn!("Failed to update post popularity after vote removal: {}", e);
            }
        },
        "comment" => {
            if let Err(e) = state.comment_service.update_popularity_after_vote(target_id, Some(&*state.vote_service)).await {
                tracing::warn!("Failed to update comment popularity after vote removal: {}", e);
            }
        },
        _ => {} // Ignore unknown target types
    }

    // Return updated vote summary
    let summary = state.vote_service.get_vote_summary(target_id, &target_type).await?;
    Ok(Json(summary))
}