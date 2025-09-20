use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a user's vote on emotion or content filter tags for posts/comments
/// Users can vote on whether they agree with AI-detected sentiment or toxicity tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub id: Uuid,
    pub user_id: Uuid,
    /// Post or comment being voted on
    pub target_id: Uuid, 
    /// "post" or "comment" 
    pub target_type: String,
    /// "emotion" or "content_filter"
    pub vote_type: String,
    /// The specific tag being voted on (e.g. "joy", "sarcastic", "insult")
    pub tag: String,
    /// User agrees (true) or disagrees (false) with the tag
    pub is_upvote: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to cast a vote on emotion or content tags
#[derive(Debug, Clone, Deserialize)]
pub struct CreateVoteRequest {
    pub target_id: Uuid,
    pub target_type: String, // "post" or "comment"
    pub vote_type: String,   // "emotion" or "content_filter"
    pub tag: String,         // specific tag like "joy", "insult"
    pub is_upvote: bool,     // true = agree with tag, false = disagree
}

/// Aggregated vote counts for a specific tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagVoteCount {
    pub tag: String,
    pub upvotes: i64,
    pub downvotes: i64,
    pub total_votes: i64,
    pub agreement_ratio: f64, // upvotes / total_votes
    /// Formatted display like "1.2k", "45M"
    pub display_count: String,
}

/// Vote summary for a post or comment with all tag votes
#[derive(Debug, Serialize)]
pub struct VoteSummary {
    pub target_id: Uuid,
    pub emotion_votes: Vec<TagVoteCount>,
    pub content_filter_votes: Vec<TagVoteCount>,
    pub total_engagement: i64,
}

impl TagVoteCount {
    /// Format numbers with k/M abbreviations per user requirements
    /// 1000 -> 1k, 1115 -> 1.1k, 25320 -> 25.3k, 234987 -> 235k, etc.
    pub fn format_number(count: i64) -> String {
        match count {
            n if n >= 1_000_000 => {
                let millions = n as f64 / 1_000_000.0;
                if millions >= 100.0 {
                    format!("{}M", millions.round() as i64)
                } else if millions >= 10.0 {
                    let formatted = format!("{:.1}", millions);
                    if formatted.ends_with(".0") {
                        format!("{}M", formatted.trim_end_matches(".0"))
                    } else {
                        format!("{}M", formatted)
                    }
                } else {
                    let formatted = format!("{:.2}", millions);
                    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
                    format!("{}M", trimmed)
                }
            },
            n if n >= 1_000 => {
                let thousands = n as f64 / 1_000.0;
                if thousands >= 100.0 {
                    format!("{}k", thousands.round() as i64)
                } else {
                    let formatted = format!("{:.1}", thousands);
                    if formatted.ends_with(".0") {
                        format!("{}k", formatted.trim_end_matches(".0"))
                    } else {
                        format!("{}k", formatted)
                    }
                }
            },
            n => n.to_string()
        }
    }
    
    pub fn new(tag: String, upvotes: i64, downvotes: i64) -> Self {
        let total_votes = upvotes + downvotes;
        let agreement_ratio = if total_votes > 0 {
            upvotes as f64 / total_votes as f64
        } else {
            0.0
        };
        let display_count = Self::format_number(total_votes);
        
        Self {
            tag,
            upvotes,
            downvotes,
            total_votes,
            agreement_ratio,
            display_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_formatting() {
        assert_eq!(TagVoteCount::format_number(999), "999");
        assert_eq!(TagVoteCount::format_number(1000), "1k");
        assert_eq!(TagVoteCount::format_number(1115), "1.1k");
        assert_eq!(TagVoteCount::format_number(25320), "25.3k");
        assert_eq!(TagVoteCount::format_number(234987), "235k");
        assert_eq!(TagVoteCount::format_number(1_000_000), "1M");
        assert_eq!(TagVoteCount::format_number(1_045_560), "1.04M");
        assert_eq!(TagVoteCount::format_number(235_234_432), "235M");
    }
}