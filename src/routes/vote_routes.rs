use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
    middleware,
};
use uuid::Uuid;

use crate::{AppState, Result};
use crate::models::{CreateVoteRequest, TagVoteCount, VoteSummary};
use crate::auth::Claims;

/// Vote-related API routes
pub fn vote_routes() -> Router<AppState> {
    Router::new()
        .route("/vote/:target_id/:target_type", get(get_vote_summary))
        .route("/vote/counts/:target_id/:target_type", get(get_vote_counts))
        // Protected routes will be added back after fixing handler signatures
}

/// Cast or update a vote on emotion/content tags
// Voting handlers temporarily disabled - will be restored after PostgreSQL setup
// async fn cast_vote() { ... }

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

// User vote handler temporarily disabled - will be restored after PostgreSQL setup
// async fn get_user_vote() { ... }