use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub post_id: Uuid, // Partition key for Cosmos DB
    pub content: String,
    pub author_id: Uuid,
    pub author_username: String, // Denormalized for performance
    pub parent_id: Option<Uuid>, // None for top-level comments
    pub thread_path: String, // Materialized path for nested structure (e.g., "001.002.003")
    pub depth: u8, // Comment nesting depth
    pub created_at: DateTime<Utc>,
    pub sentiment_score: Option<f64>,
    pub sentiment_colors: Vec<String>,
    pub is_blocked: bool, // Content moderation flag
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub content: String,
    pub parent_id: Option<Uuid>, // Reply to another comment
}

#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub author_id: Uuid,
    pub author_username: String,
    pub parent_id: Option<Uuid>,
    pub depth: u8,
    pub created_at: DateTime<Utc>,
    pub sentiment_colors: Vec<String>,
    pub replies: Vec<CommentResponse>, // Nested replies
}

impl From<Comment> for CommentResponse {
    fn from(comment: Comment) -> Self {
        Self {
            id: comment.id,
            post_id: comment.post_id,
            content: comment.content,
            author_id: comment.author_id,
            author_username: comment.author_username,
            parent_id: comment.parent_id,
            depth: comment.depth,
            created_at: comment.created_at,
            sentiment_colors: comment.sentiment_colors,
            replies: vec![], // Will be populated by service layer
        }
    }
}