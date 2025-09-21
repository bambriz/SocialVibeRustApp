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
    pub sentiment_type: Option<String>, // Human-readable sentiment name (e.g., "angry", "joy", "sarcastic+happy")
    pub sentiment_analysis: Option<serde_json::Value>, // JSONB field with full sentiment analysis data
    pub popularity_score: f64, // Calculated score for feed ranking
    pub is_blocked: bool, // Content moderation flag
    pub toxicity_tags: Vec<String>, // Toxicity categories (e.g., "toxicity", "insult", "threat")
    pub toxicity_scores: Option<serde_json::Value>, // Complete diagnostic data from moderation system
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
    pub sentiment_type: Option<String>,
    pub popularity_score: f64,
    pub toxicity_tags: Vec<String>, // Include toxicity tags for frontend
    pub toxicity_scores: Option<serde_json::Value>, // Include scores for diagnostic purposes
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        // Extract sentiment data from sentiment_analysis JSON field
        let (sentiment_type, sentiment_colors) = if let Some(sentiment_analysis) = &post.sentiment_analysis {
            // Try to parse the JSON and extract primary_emotion
            if let Ok(analysis) = serde_json::from_value::<serde_json::Value>(sentiment_analysis.clone()) {
                if let Some(primary_emotion) = analysis.get("primary_emotion").and_then(|v| v.as_str()) {
                    let emotion_color = match primary_emotion.to_lowercase().as_str() {
                        "joy" | "happy" => "#fbbf24",        // Bright yellow/gold
                        "sad" => "#1e3a8a",                   // Dark blue
                        "angry" => "#dc2626",                 // Red
                        "confused" => "#a16207",              // Brown/amber
                        "disgust" => "#84cc16",               // Lime green
                        "surprise" => "#f97316",              // Orange
                        "fear" => "#374151",                  // Dark grey
                        "neutral" => "#6b7280",               // Neutral gray
                        "affection" | "affectionate" => "#ec4899",  // Pink
                        "sarcastic" => "#7c3aed",             // Purple
                        _ => "#6b7280",                       // Default to neutral gray
                    };
                    (Some(primary_emotion.to_string()), vec![emotion_color.to_string()])
                } else {
                    // Fallback to legacy fields if primary_emotion not found
                    (post.sentiment_type.clone(), post.sentiment_colors.clone())
                }
            } else {
                // If JSON parsing fails, fallback to legacy fields
                (post.sentiment_type.clone(), post.sentiment_colors.clone())
            }
        } else {
            // If no sentiment_analysis, use legacy fields
            (post.sentiment_type.clone(), post.sentiment_colors.clone())
        };

        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            author_username: post.author_username,
            created_at: post.created_at,
            comment_count: post.comment_count,
            sentiment_colors,
            sentiment_type,
            popularity_score: post.popularity_score,
            toxicity_tags: post.toxicity_tags,
            toxicity_scores: post.toxicity_scores,
        }
    }
}