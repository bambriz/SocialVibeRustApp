use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::db::repository::VoteRepository;
use crate::models::{Vote, CreateVoteRequest, TagVoteCount, VoteSummary};
use crate::{Result, AppError};

/// Service for handling emotion and content filter tag voting
pub struct VoteService {
    vote_repo: Arc<dyn VoteRepository>,
}

impl VoteService {
    pub fn new(vote_repo: Arc<dyn VoteRepository>) -> Self {
        Self { vote_repo }
    }

    /// Cast or update a vote on emotion/content tags
    pub async fn cast_vote(&self, user_id: Uuid, request: CreateVoteRequest) -> Result<Vote> {
        // Validate vote request
        self.validate_vote_request(&request)?;

        // Check if user already voted on this tag
        let existing_vote = self.vote_repo.get_user_vote(
            user_id,
            request.target_id,
            &request.vote_type,
            &request.tag
        ).await?;

        // Create new vote
        let vote = Vote {
            id: Uuid::new_v4(),
            user_id,
            target_id: request.target_id,
            target_type: request.target_type,
            vote_type: request.vote_type,
            tag: request.tag,
            is_upvote: request.is_upvote,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // If existing vote is the same, remove it (toggle behavior)
        if let Some(existing) = existing_vote {
            if existing.is_upvote == vote.is_upvote {
                self.vote_repo.remove_vote(user_id, vote.target_id, &vote.vote_type, &vote.tag).await?;
                return Err(AppError::ValidationError("Vote removed".to_string()));
            }
        }

        // Cast the vote
        self.vote_repo.cast_vote(&vote).await
    }

    /// Get vote counts for a target (post or comment)
    pub async fn get_vote_counts(&self, target_id: Uuid, target_type: &str) -> Result<Vec<TagVoteCount>> {
        self.vote_repo.get_vote_counts(target_id, target_type).await
    }

    /// Get comprehensive vote summary with emotion and content filter breakdown
    pub async fn get_vote_summary(&self, target_id: Uuid, target_type: &str) -> Result<VoteSummary> {
        self.vote_repo.get_vote_summary(target_id, target_type).await
    }

    /// Get user's vote on a specific tag
    pub async fn get_user_vote(&self, user_id: Uuid, target_id: Uuid, vote_type: &str, tag: &str) -> Result<Option<Vote>> {
        self.vote_repo.get_user_vote(user_id, target_id, vote_type, tag).await
    }

    /// Remove a user's vote
    pub async fn remove_vote(&self, user_id: Uuid, target_id: Uuid, vote_type: &str, tag: &str) -> Result<()> {
        self.vote_repo.remove_vote(user_id, target_id, vote_type, tag).await
    }

    /// Calculate engagement score for popularity ranking
    pub async fn get_engagement_score(&self, target_id: Uuid, target_type: &str) -> Result<f64> {
        self.vote_repo.get_engagement_score(target_id, target_type).await
    }

    /// Update popularity score based on engagement (cap at 3.0, then sort by timestamp)
    pub async fn update_popularity_with_engagement(&self, target_id: Uuid, target_type: &str, base_score: f64) -> Result<f64> {
        let engagement_score = self.get_engagement_score(target_id, target_type).await?;
        
        // Cap total popularity at 3.0 as per user requirements
        let total_score = (base_score + engagement_score).min(3.0);
        
        Ok(total_score)
    }

    /// Validate vote request
    fn validate_vote_request(&self, request: &CreateVoteRequest) -> Result<()> {
        // Validate target type
        if !matches!(request.target_type.as_str(), "post" | "comment") {
            return Err(AppError::ValidationError("target_type must be 'post' or 'comment'".to_string()));
        }

        // Validate vote type
        if !matches!(request.vote_type.as_str(), "emotion" | "content_filter") {
            return Err(AppError::ValidationError("vote_type must be 'emotion' or 'content_filter'".to_string()));
        }

        // Validate emotion tags
        if request.vote_type == "emotion" {
            let valid_emotions = ["joy", "sad", "angry", "fear", "disgust", "surprise", "confused", "neutral", "sarcastic", "affectionate"];
            if !valid_emotions.contains(&request.tag.as_str()) {
                return Err(AppError::ValidationError(format!("Invalid emotion tag: {}", request.tag)));
            }
        }

        // Validate content filter tags (based on existing toxicity detection)
        if request.vote_type == "content_filter" {
            let valid_filters = ["toxicity", "severe_toxicity", "identity_attack", "insult", "threat", "profanity", "spam", "harassment"];
            if !valid_filters.contains(&request.tag.as_str()) {
                return Err(AppError::ValidationError(format!("Invalid content filter tag: {}", request.tag)));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_number_formatting() {
        // Test the number formatting from TagVoteCount
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