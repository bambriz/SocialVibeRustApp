use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub author_username: String, // Denormalized for performance
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub comment_count: u32,
    pub sentiment_score: Option<f64>, // -1.0 to 1.0
    pub sentiment_colors: Vec<String>, // Color codes for sentiment
    pub popularity_score: f64, // Calculated score for feed ranking
    pub is_blocked: bool, // Content moderation flag
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub author_username: String,
    pub created_at: DateTime<Utc>,
    pub comment_count: u32,
    pub sentiment_colors: Vec<String>,
    pub popularity_score: f64,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            author_username: post.author_username,
            created_at: post.created_at,
            comment_count: post.comment_count,
            sentiment_colors: post.sentiment_colors,
            popularity_score: post.popularity_score,
        }
    }
}