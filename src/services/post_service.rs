use crate::models::Post;
use crate::models::post::{CreatePostRequest, PostResponse};
use crate::models::sentiment::{Sentiment, SentimentType};
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

    pub async fn create_post(&self, request: CreatePostRequest, author_id: Uuid, author_username: String) -> Result<PostResponse> {
        // Analyze title and body separately with body bias
        // Body gets 4x weight compared to title for sentiment determination
        
        // Check content moderation first (use combined text for moderation)
        let combined_text = format!("{} {}", request.title, request.content);
        let is_blocked = self.moderation_service.check_content(&combined_text).await
            .map_err(|e| AppError::InternalError(format!("Content moderation failed: {}", e)))?;
        
        if is_blocked {
            return Err(AppError::ValidationError("Content violates community guidelines and has been blocked".to_string()));
        }
        
        // Run sentiment analysis on title and body separately
        let title_sentiments = self.sentiment_service.analyze_sentiment(&request.title).await
            .map_err(|e| AppError::InternalError(format!("Title sentiment analysis failed: {}", e)))?;
        let body_sentiments = self.sentiment_service.analyze_sentiment(&request.content).await
            .map_err(|e| AppError::InternalError(format!("Body sentiment analysis failed: {}", e)))?;
        
        // Combine sentiments with body bias (body weight = 4x title weight)
        let sentiments = self.combine_sentiments_with_bias(&title_sentiments, &body_sentiments, 0.2, 0.8);
        
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
            author_username,
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

    // Helper method to combine sentiments with weighting bias
    fn combine_sentiments_with_bias(&self, title_sentiments: &[Sentiment], body_sentiments: &[Sentiment], title_weight: f64, body_weight: f64) -> Vec<Sentiment> {
        // Filter out fallback Calm sentiments (parser failures)
        let filtered_title: Vec<&Sentiment> = title_sentiments.iter()
            .filter(|s| !(matches!(s.sentiment_type, SentimentType::Calm) && (s.confidence - 0.5).abs() < 0.01))
            .collect();
        let filtered_body: Vec<&Sentiment> = body_sentiments.iter()
            .filter(|s| !(matches!(s.sentiment_type, SentimentType::Calm) && (s.confidence - 0.5).abs() < 0.01))
            .collect();
        
        // If both are empty after filtering, default to calm
        if filtered_body.is_empty() && filtered_title.is_empty() {
            return vec![Sentiment {
                sentiment_type: SentimentType::Calm,
                confidence: 0.5,
                color_code: SentimentType::Calm.color_code(),
            }];
        }
        
        // If only body has real sentiments, use those (body bias)
        if !filtered_body.is_empty() && filtered_title.is_empty() {
            return filtered_body.into_iter().cloned().collect();
        }
        
        // If only title has real sentiments, use those with reduced confidence
        if !filtered_title.is_empty() && filtered_body.is_empty() {
            return filtered_title.iter().map(|s| Sentiment {
                sentiment_type: s.sentiment_type.clone(),
                confidence: s.confidence * title_weight,
                color_code: s.color_code.clone(),
            }).collect();
        }
        
        // If both have sentiments, prioritize body with weighting
        let mut combined_score_map = std::collections::HashMap::new();
        
        // Add title sentiments with title weight
        for sentiment in filtered_title {
            let key = format!("{:?}", sentiment.sentiment_type);
            *combined_score_map.entry(key).or_insert(0.0) += sentiment.confidence * title_weight;
        }
        
        // Add body sentiments with body weight (higher priority)
        for sentiment in filtered_body {
            let key = format!("{:?}", sentiment.sentiment_type);
            *combined_score_map.entry(key).or_insert(0.0) += sentiment.confidence * body_weight;
        }
        
        // Return the highest scoring sentiment
        if let Some((dominant_type, score)) = combined_score_map.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
            let sentiment_type = match dominant_type.as_str() {
                "Happy" => SentimentType::Happy,
                "Sad" => SentimentType::Sad,
                "Angry" => SentimentType::Angry,
                "Excited" => SentimentType::Excited,
                "Confused" => SentimentType::Confused,
                "Fear" => SentimentType::Fear,
                "Calm" => SentimentType::Calm,
                "Affection" => SentimentType::Affection,
                "Sarcastic" => SentimentType::Sarcastic,
                _ => SentimentType::Calm,
            };
            
            vec![Sentiment {
                sentiment_type: sentiment_type.clone(),
                confidence: *score,
                color_code: sentiment_type.color_code(),
            }]
        } else {
            vec![Sentiment {
                sentiment_type: SentimentType::Calm,
                confidence: 0.5,
                color_code: SentimentType::Calm.color_code(),
            }]
        }
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
                SentimentType::Excited => 1.3,  // Highest positive boost
                SentimentType::Affection => 1.1,
                SentimentType::Calm => 1.0,
                SentimentType::Confused => 0.95, // Slightly lower than neutral
                SentimentType::Sarcastic => 0.9,
                SentimentType::Sad => 0.8,
                SentimentType::Fear => 0.7,
                SentimentType::Angry => 0.6,
                SentimentType::SarcasticCombination(ref base_type) => {
                    // For sarcastic combinations, reduce the base sentiment score
                    let base_score = match **base_type {
                        SentimentType::Happy => 1.2,
                        SentimentType::Excited => 1.3,
                        SentimentType::Affection => 1.1,
                        SentimentType::Calm => 1.0,
                        SentimentType::Confused => 0.95,
                        SentimentType::Sad => 0.8,
                        SentimentType::Fear => 0.7,
                        SentimentType::Angry => 0.6,
                        _ => 1.0,
                    };
                    base_score * 0.8  // Reduce by 20% due to sarcasm
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