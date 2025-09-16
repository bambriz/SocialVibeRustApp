use crate::models::Post;
use crate::models::post::{CreatePostRequest, PostResponse};
use crate::db::repository::{PostRepository, MockPostRepository};
use crate::services::{SentimentService, ModerationService};
use crate::{AppError, Result};
use uuid::Uuid;
use std::sync::Arc;
use chrono::Utc;

pub struct PostService {
    post_repo: Arc<MockPostRepository>,
    sentiment_service: Arc<SentimentService>,
    moderation_service: Arc<ModerationService>,
}

impl PostService {
    pub fn new(
        post_repo: Arc<MockPostRepository>, 
        sentiment_service: Arc<SentimentService>,
        moderation_service: Arc<ModerationService>
    ) -> Self {
        Self { 
            post_repo, 
            sentiment_service,
            moderation_service 
        }
    }

    pub async fn create_post(&self, request: CreatePostRequest, author_id: Uuid) -> Result<PostResponse> {
        // Combine title and content for analysis
        let full_text = format!("{} {}", request.title, request.content);
        
        // Check content moderation first
        let is_blocked = self.moderation_service.check_content(&full_text).await
            .map_err(|e| AppError::InternalError(format!("Content moderation failed: {}", e)))?;
        
        if is_blocked {
            return Err(AppError::ValidationError("Content violates community guidelines and has been blocked".to_string()));
        }
        
        // Run sentiment analysis
        let sentiments = self.sentiment_service.analyze_sentiment(&full_text).await
            .map_err(|e| AppError::InternalError(format!("Sentiment analysis failed: {}", e)))?;
        
        // Calculate sentiment score and extract colors
        let sentiment_score = if !sentiments.is_empty() {
            Some(sentiments.iter().map(|s| s.confidence).sum::<f64>() / sentiments.len() as f64)
        } else {
            None
        };
        
        let sentiment_colors: Vec<String> = sentiments.iter()
            .flat_map(|s| s.sentiment_type.colors_array())
            .collect();
        
        // Calculate popularity score based on sentiment
        let popularity_score = self.calculate_popularity_score_from_sentiment(&sentiments);
        
        let post = Post {
            id: uuid::Uuid::new_v4(),
            title: request.title,
            content: request.content,
            author_id,
            author_username: "User".to_string(), // TODO: Get from user service
            created_at: Utc::now(),
            updated_at: Utc::now(),
            comment_count: 0,
            sentiment_score,
            sentiment_colors,
            popularity_score,
            is_blocked: false, // Already checked above
        };
        
        let created_post = self.post_repo.create_post(&post).await?;
        Ok(PostResponse::from(created_post))
    }

    pub async fn get_post(&self, post_id: Uuid) -> Result<Option<PostResponse>> {
        let post = self.post_repo.get_post_by_id(post_id).await?;
        Ok(post.map(PostResponse::from))
    }

    pub async fn get_posts_feed(&self, limit: u32, offset: u32) -> Result<Vec<PostResponse>> {
        let posts = self.post_repo.get_posts_by_popularity(limit, offset).await?;
        Ok(posts.into_iter().map(PostResponse::from).collect())
    }

    pub async fn calculate_popularity_score(&self, post: &Post) -> f64 {
        // Base score from sentiment analysis
        let sentiment_score = post.popularity_score;
        
        // Factor in engagement metrics
        let comment_boost = (post.comment_count as f64) * 0.1;
        let recency_hours = (Utc::now() - post.created_at).num_hours() as f64;
        let recency_decay = 1.0 / (1.0 + recency_hours * 0.01); // Decay over time
        
        (sentiment_score + comment_boost) * recency_decay
    }
    
    fn calculate_popularity_score_from_sentiment(&self, sentiments: &[crate::models::sentiment::Sentiment]) -> f64 {
        if sentiments.is_empty() {
            return 1.0; // Default neutral score
        }
        
        use crate::models::sentiment::SentimentType;
        
        let mut total_score = 0.0;
        let mut total_confidence = 0.0;
        
        for sentiment in sentiments {
            let sentiment_multiplier = match sentiment.sentiment_type {
                SentimentType::Happy => 1.2,
                SentimentType::Affection => 1.1,
                SentimentType::Calm => 1.0,
                SentimentType::Sarcastic => 0.9,
                SentimentType::Sad => 0.8,
                SentimentType::Fear => 0.7,
                SentimentType::Angry => 0.6,
                SentimentType::Mixed(ref types) => {
                    // For mixed sentiments, average the scores
                    if types.is_empty() {
                        1.0
                    } else {
                        types.iter().map(|t| match t {
                            SentimentType::Happy => 1.2,
                            SentimentType::Affection => 1.1,
                            SentimentType::Calm => 1.0,
                            SentimentType::Sarcastic => 0.9,
                            SentimentType::Sad => 0.8,
                            SentimentType::Fear => 0.7,
                            SentimentType::Angry => 0.6,
                            _ => 1.0,
                        }).sum::<f64>() / types.len() as f64
                    }
                }
            };
            
            total_score += sentiment_multiplier * sentiment.confidence;
            total_confidence += sentiment.confidence;
        }
        
        if total_confidence > 0.0 {
            total_score / total_confidence
        } else {
            1.0
        }
    }
}