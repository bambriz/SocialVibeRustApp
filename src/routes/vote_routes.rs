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
use crate::auth::AuthUser;
use crate::auth::middleware::auth_middleware;

/// Vote-related API routes
pub fn vote_routes() -> Router<AppState> {
    let public_vote_routes = Router::new()
        .route("/vote/:target_id/:target_type", get(get_vote_summary))
        .route("/vote/counts/:target_id/:target_type", get(get_vote_counts));
    
    let protected_vote_routes = Router::new()
        .route("/vote", post(cast_vote))
        .route("/vote/user/:target_id/:vote_type/:tag", get(get_user_vote))
        .layer(middleware::from_fn(auth_middleware));
    
    public_vote_routes.merge(protected_vote_routes)
}

/// Cast or update a vote on emotion/content tags
async fn cast_vote(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(request): Json<CreateVoteRequest>,
) -> Result<Json<serde_json::Value>> {
    match state.vote_service.cast_vote(user.id, request).await {
        Ok(vote) => Ok(Json(serde_json::json!({
            "success": true,
            "vote": vote,
            "message": "Vote cast successfully"
        }))),
        Err(crate::AppError::ValidationError(msg)) if msg == "Vote removed" => {
            Ok(Json(serde_json::json!({
                "success": true,
                "vote": null,
                "message": "Vote removed (toggled off)"
            })))
        },
        Err(e) => Err(e),
    }
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

/// Get user's vote on a specific tag (authenticated)
async fn get_user_vote(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((target_id, vote_type, tag)): Path<(Uuid, String, String)>,
) -> Result<Json<serde_json::Value>> {
    let vote = state.vote_service.get_user_vote(user.id, target_id, &vote_type, &tag).await?;
    Ok(Json(serde_json::json!({
        "vote": vote,
        "has_voted": vote.is_some()
    })))
}