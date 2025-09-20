use crate::models::Post;
use crate::models::post::{CreatePostRequest, PostResponse};
use crate::models::sentiment::{Sentiment, SentimentType};
use crate::db::repository::{PostRepository, CsvPostRepository};
use crate::services::{SentimentService, ModerationService, VoteService};
use crate::{AppError, Result};
use crate::error::ContentModerationError;
use uuid::Uuid;
use std::sync::Arc;
use chrono::Utc;

pub struct PostService {
    primary_repo: Arc<dyn PostRepository>,
    csv_fallback_repo: Arc<CsvPostRepository>,
    sentiment_service: Arc<SentimentService>,
    moderation_service: Arc<ModerationService>,
    vote_service: Option<Arc<VoteService>>,
}

impl PostService {
    pub fn new(
        primary_repo: Arc<dyn PostRepository>, 
        csv_fallback_repo: Arc<CsvPostRepository>,
        sentiment_service: Arc<SentimentService>,
        moderation_service: Arc<ModerationService>
    ) -> Self {
        Self { 
            primary_repo,
            csv_fallback_repo,
            sentiment_service,
            moderation_service,
            vote_service: None
        }
    }
    
    /// Create PostService with VoteService integration
    pub fn new_with_vote_service(
        primary_repo: Arc<dyn PostRepository>, 
        csv_fallback_repo: Arc<CsvPostRepository>,
        sentiment_service: Arc<SentimentService>,
        moderation_service: Arc<ModerationService>,
        vote_service: Arc<VoteService>
    ) -> Self {
        Self { 
            primary_repo,
            csv_fallback_repo,
            sentiment_service,
            moderation_service,
            vote_service: Some(vote_service)
        }
    }
    
    // Helper method to try primary repository first, then fall back to CSV (for read operations)
    async fn try_with_fallback<T, F, C, PrimFut, CsvFut>(&self, operation_name: &str, primary_op: F, csv_op: C) -> Result<T>
    where
        F: FnOnce() -> PrimFut,
        C: FnOnce() -> CsvFut,
        PrimFut: std::future::Future<Output = Result<T>>,
        CsvFut: std::future::Future<Output = Result<T>>,
        T: Clone + std::fmt::Debug,
    {
        // Enhanced trace logging for fallback testing
        tracing::info!("🔄 FALLBACK_TRACE: Starting {} operation", operation_name);
        
        match primary_op().await {
            Ok(result) => {
                tracing::info!("✅ FALLBACK_TRACE: {} succeeded with primary repository", operation_name);
                tracing::debug!("Primary repository result: {:?}", result);
                Ok(result)
            }
            Err(primary_error) => {
                tracing::error!("❌ FALLBACK_TRACE: {} failed with primary repository: {:?}", operation_name, primary_error);
                tracing::warn!("🔄 FALLBACK_TRACE: Attempting CSV fallback for {}", operation_name);
                
                match csv_op().await {
                    Ok(csv_result) => {
                        tracing::info!("✅ FALLBACK_TRACE: {} succeeded with CSV fallback repository", operation_name);
                        tracing::info!("📄 FALLBACK_TRACE: CSV operation completed successfully");
                        tracing::debug!("CSV fallback result: {:?}", csv_result);
                        Ok(csv_result)
                    }
                    Err(csv_error) => {
                        tracing::error!("❌ FALLBACK_TRACE: {} failed with CSV fallback: {:?}", operation_name, csv_error);
                        tracing::error!("💥 FALLBACK_TRACE: Both repositories failed for {}", operation_name);
                        Err(AppError::InternalError(format!(
                            "Both primary and CSV fallback repositories failed. Primary: {:?}, CSV: {:?}", 
                            primary_error, csv_error
                        )))
                    }
                }
            }
        }
    }

    // Helper method to write to both primary and CSV repositories (for persistence)
    async fn write_to_both_repositories<T, F, C, PrimFut, CsvFut>(&self, operation_name: &str, primary_op: F, csv_op: C) -> Result<T>
    where
        F: FnOnce() -> PrimFut,
        C: FnOnce() -> CsvFut,
        PrimFut: std::future::Future<Output = Result<T>>,
        CsvFut: std::future::Future<Output = Result<T>>,
        T: Clone + std::fmt::Debug,
    {
        tracing::info!("🔄 PERSISTENCE_TRACE: Starting {} operation (write to both repositories)", operation_name);
        
        // Always try primary repository first
        match primary_op().await {
            Ok(result) => {
                tracing::info!("✅ PERSISTENCE_TRACE: {} succeeded with primary repository", operation_name);
                tracing::debug!("Primary repository result: {:?}", result);
                
                // Also write to CSV for backup persistence (don't fail if CSV fails)
                match csv_op().await {
                    Ok(_) => {
                        tracing::info!("📄 PERSISTENCE_TRACE: {} also persisted to CSV backup successfully", operation_name);
                    }
                    Err(csv_error) => {
                        tracing::warn!("⚠️ PERSISTENCE_TRACE: {} failed to persist to CSV backup (non-fatal): {:?}", operation_name, csv_error);
                        // Don't fail the operation if CSV backup fails - it's just a backup
                    }
                }
                
                Ok(result)
            }
            Err(primary_error) => {
                tracing::error!("❌ PERSISTENCE_TRACE: {} failed with primary repository: {:?}", operation_name, primary_error);
                tracing::warn!("🔄 PERSISTENCE_TRACE: Attempting CSV fallback for {}", operation_name);
                
                // If primary fails, try CSV as fallback
                match csv_op().await {
                    Ok(csv_result) => {
                        tracing::info!("✅ PERSISTENCE_TRACE: {} succeeded with CSV fallback repository", operation_name);
                        tracing::debug!("CSV fallback result: {:?}", csv_result);
                        Ok(csv_result)
                    }
                    Err(csv_error) => {
                        tracing::error!("❌ PERSISTENCE_TRACE: {} failed with CSV fallback: {:?}", operation_name, csv_error);
                        tracing::error!("💥 PERSISTENCE_TRACE: Both repositories failed for {}", operation_name);
                        Err(AppError::InternalError(format!(
                            "Both primary and CSV repositories failed. Primary: {:?}, CSV: {:?}", 
                            primary_error, csv_error
                        )))
                    }
                }
            }
        }
    }

    pub async fn create_post(&self, request: CreatePostRequest, author_id: Uuid, author_username: String) -> Result<PostResponse> {
        // Analyze title and body separately with body bias
        // Body gets 4x weight compared to title for sentiment determination
        
        // Check content moderation first (use combined text for moderation)
        let combined_text = format!("{} {}", request.title, request.content);
        let moderation_result = self.moderation_service.check_content(&combined_text).await
            .map_err(|e| AppError::InternalError(format!("Content moderation system error: {}. Please try again later or contact support if this persists.", e)))?;
        
        if moderation_result.is_blocked {
            let error_message = match (&moderation_result.violation_type, &moderation_result.details) {
                (Some(violation_type), Some(details)) => {
                    match violation_type.as_str() {
                        "racial_slurs" => "🚫 Post blocked: Your message contains racial slurs or offensive language targeting ethnicity. Our community guidelines prohibit discriminatory language to maintain a safe environment for everyone.".to_string(),
                        "homophobic_slurs" => "🚫 Post blocked: Your message contains homophobic slurs or offensive language targeting LGBTQ+ individuals. Please use respectful language that welcomes all community members.".to_string(),
                        "hate_speech_terms" => "🚫 Post blocked: Your message contains hate speech or derogatory terms. Our platform is committed to fostering inclusive discussions free from discriminatory language.".to_string(),
                        "violent_threats" => "🚫 Post blocked: Your message contains violent threats or calls for harm against individuals or groups. Threats of violence are strictly prohibited and may be reported to authorities.".to_string(),
                        "direct_threats" => "🚫 Post blocked: Your message contains direct threats of harm. This type of content violates our safety policies and may result in account suspension.".to_string(),
                        "hate_speech_with_context" => "🚫 Post blocked: Your message contains hate speech combined with profanity targeting specific groups. Please revise your message to use respectful language.".to_string(),
                        "derogatory_statements" => "🚫 Post blocked: Your message contains derogatory statements about individuals or groups. Our community values respect and inclusivity for all members.".to_string(),
                        "excessive_profanity" => format!("🚫 Post blocked: Your message contains excessive profanity ({}). While some casual language is acceptable, excessive profanity disrupts constructive conversation.", details),
                        "ai_hate_speech_detection" => format!("🚫 Post blocked: AI detection identified potential hate speech ({}). Our automated systems flagged this content as potentially harmful to community members.", details),
                        "ai_offensive_language" => format!("🚫 Post blocked: AI detection identified highly offensive language ({}). Please consider revising your message to be more respectful.", details),
                        _ => format!("🚫 Post blocked: Your message violates our community guidelines (violation type: {}). Please revise your content to comply with our policies.", violation_type)
                    }
                },
                (Some(violation_type), None) => {
                    match violation_type.as_str() {
                        "racial_slurs" => "🚫 Post blocked: Your message contains racial slurs or discriminatory language. Please revise to use respectful language.".to_string(),
                        "homophobic_slurs" => "🚫 Post blocked: Your message contains homophobic language. Please use inclusive language that respects all community members.".to_string(),
                        "hate_speech_terms" => "🚫 Post blocked: Your message contains hate speech. Our platform maintains zero tolerance for discriminatory content.".to_string(),
                        "violent_threats" => "🚫 Post blocked: Your message contains violent threats. This content is prohibited for community safety.".to_string(),
                        "direct_threats" => "🚫 Post blocked: Your message contains direct threats. This violates our safety policies.".to_string(),
                        "hate_speech_with_context" => "🚫 Post blocked: Your message contains hate speech with profanity. Please use respectful language.".to_string(),
                        "derogatory_statements" => "🚫 Post blocked: Your message contains derogatory statements. Please revise to be more respectful.".to_string(),
                        _ => format!("🚫 Post blocked: Content violates community guidelines ({}). Please revise your message.", violation_type)
                    }
                },
                _ => "🚫 Post blocked: Your message violates our community guidelines against hate speech and offensive content. Please revise your message to use respectful language and try again.".to_string()
            };
            
            let content_moderation_error = ContentModerationError {
                message: error_message,
                violation_type: moderation_result.violation_type.clone(),
                details: moderation_result.details.clone(),
            };
            
            return Err(AppError::ContentModerationError(content_moderation_error));
        }
        
        // Run sentiment analysis on title and body separately
        let title_sentiments = self.sentiment_service.analyze_sentiment(&request.title).await
            .map_err(|e| AppError::InternalError(format!("⚠️ Sentiment analysis error (title): {}. This might be due to: 1) Python script execution issues, 2) Missing required Python libraries (nrclex, emotionclassifier), 3) Script file permissions, or 4) Temporary system overload. Please try again in a moment or contact support if this persists.", e)))?;
        let body_sentiments = self.sentiment_service.analyze_sentiment(&request.content).await
            .map_err(|e| AppError::InternalError(format!("⚠️ Sentiment analysis error (content): {}. This might be due to: 1) Python script execution issues, 2) Missing required Python libraries (nrclex, emotionclassifier), 3) Script file permissions, or 4) Temporary system overload. Please try again in a moment or contact support if this persists.", e)))?;
        
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
        
        // Extract sentiment type name for API response
        let sentiment_type = if !sentiments.is_empty() {
            Some(sentiments[0].sentiment_type.to_string())
        } else {
            None
        };
        
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
            sentiment_type,
            popularity_score,
            is_blocked: false, // Already checked above
            toxicity_tags: moderation_result.toxicity_tags, // Include toxicity tags from moderation service
            toxicity_scores: moderation_result.all_scores, // Include diagnostic scores from moderation service
        };
        
        // Write to both primary and CSV repositories for persistence
        let created_post = self.write_to_both_repositories(
            "create_post",
            || self.primary_repo.create_post(&post),
            || self.csv_fallback_repo.create_post(&post),
        ).await?;
        Ok(PostResponse::from(created_post))
    }

    // Helper method to combine sentiments with weighting bias
    fn combine_sentiments_with_bias(&self, title_sentiments: &[Sentiment], body_sentiments: &[Sentiment], title_weight: f64, body_weight: f64) -> Vec<Sentiment> {
        // Filter out fallback Neutral sentiments (parser failures)
        let filtered_title: Vec<&Sentiment> = title_sentiments.iter()
            .filter(|s| !(matches!(s.sentiment_type, SentimentType::Neutral) && (s.confidence - 0.5).abs() < 0.01))
            .collect();
        let filtered_body: Vec<&Sentiment> = body_sentiments.iter()
            .filter(|s| !(matches!(s.sentiment_type, SentimentType::Neutral) && (s.confidence - 0.5).abs() < 0.01))
            .collect();
        
        // If both are empty after filtering, default to neutral
        if filtered_body.is_empty() && filtered_title.is_empty() {
            return vec![Sentiment {
                sentiment_type: SentimentType::Neutral,
                confidence: 0.5,
                color_code: SentimentType::Neutral.color_code(),
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
            let key = sentiment.sentiment_type.to_string();
            *combined_score_map.entry(key).or_insert(0.0) += sentiment.confidence * title_weight;
        }
        
        // Add body sentiments with body weight (higher priority)
        for sentiment in filtered_body {
            let key = sentiment.sentiment_type.to_string();
            *combined_score_map.entry(key).or_insert(0.0) += sentiment.confidence * body_weight;
        }
        
        // Return the highest scoring sentiment
        if let Some((dominant_type, score)) = combined_score_map.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
            let sentiment_type = self.parse_sentiment_type_from_string(dominant_type);
            
            vec![Sentiment {
                sentiment_type: sentiment_type.clone(),
                confidence: *score,
                color_code: sentiment_type.color_code(),
            }]
        } else {
            vec![Sentiment {
                sentiment_type: SentimentType::Neutral,
                confidence: 0.5,
                color_code: SentimentType::Neutral.color_code(),
            }]
        }
    }

    pub async fn get_post(&self, post_id: Uuid) -> Result<Option<PostResponse>> {
        // Try primary repository first, then fallback to CSV using helper method
        let post = self.try_with_fallback(
            "get_post_by_id",
            || self.primary_repo.get_post_by_id(post_id),
            || self.csv_fallback_repo.get_post_by_id(post_id),
        ).await?;
        Ok(post.map(PostResponse::from))
    }

    pub async fn get_posts_feed(&self, limit: u32, offset: u32) -> Result<Vec<PostResponse>> {
        // Try primary repository first, then fallback to CSV using helper method
        let posts = self.try_with_fallback(
            "get_posts_by_popularity",
            || self.primary_repo.get_posts_by_popularity(limit, offset),
            || self.csv_fallback_repo.get_posts_by_popularity(limit, offset),
        ).await?;
        Ok(posts.into_iter().map(PostResponse::from).collect())
    }

    /// Recalculate and update popularity score after voting (for triggering recalculation)
    pub async fn update_popularity_after_vote(&self, post_id: Uuid) -> Result<()> {
        // Get the current post
        let post = self.primary_repo.get_post_by_id(post_id).await?
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;
        
        // Calculate new popularity score including voting engagement
        let new_popularity_score = self.calculate_popularity_score(&post).await;
        
        // Update the popularity score (this will trigger repository update if needed)
        let _ = self.write_to_both_repositories(
            "update_popularity_score",
            || self.primary_repo.update_popularity_score(post.id, new_popularity_score),
            || self.csv_fallback_repo.update_popularity_score(post.id, new_popularity_score)
        ).await;
        
        tracing::debug!("📊 Updated popularity score for post {} to {}", post_id, new_popularity_score);
        Ok(())
    }

    pub async fn calculate_popularity_score(&self, post: &Post) -> f64 {
        // Base score from sentiment analysis (use actual sentiment score, not popularity_score!)
        let sentiment_score = post.sentiment_score.unwrap_or(1.0);
        
        // Factor in engagement metrics
        let comment_boost = (post.comment_count as f64) * 0.1;
        let recency_hours = (Utc::now() - post.created_at).num_hours() as f64;
        
        // Enhanced time decay - recent posts get bigger boost, older posts decay faster
        let recency_decay = if recency_hours <= 24.0 {
            // Posts within 24 hours get a boost
            1.0 + (24.0 - recency_hours) * 0.05  // Up to 20% boost for very recent posts
        } else {
            // Older posts decay more aggressively 
            1.0 / (1.0 + (recency_hours - 24.0) * 0.02)
        };
        
        let base_score = (sentiment_score + comment_boost) * recency_decay;
        
        // Add voting engagement if available, with cap at 3.0
        if let Some(vote_service) = &self.vote_service {
            match vote_service.get_engagement_score(post.id, "post").await {
                Ok(engagement_score) => {
                    // Cap total popularity at 3.0 as per user requirements
                    (base_score + engagement_score).min(3.0)
                },
                Err(_) => base_score // Fall back to base score if voting fails
            }
        } else {
            base_score
        }
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
                SentimentType::Joy => 1.4,      // Highest positive boost - replaces Happy/Excited
                SentimentType::Affection => 1.1,
                SentimentType::Surprise => 1.1, // Slight positive boost - draws attention
                SentimentType::Neutral => 1.0,  // Neutral scoring - replaces Calm
                SentimentType::Confused => 0.95, // Slightly lower than neutral
                SentimentType::Sarcastic => 0.9,
                SentimentType::Disgust => 0.6,  // Low engagement like anger
                SentimentType::Sad => 0.8,
                SentimentType::Fear => 0.7,
                SentimentType::Angry => 0.6,
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
    
    // Helper method to parse sentiment type from string (handles combinations)
    fn parse_sentiment_type_from_string(&self, type_str: &str) -> SentimentType {
        if type_str.starts_with("affectionate+") {
            // Extract just "affectionate" from "affectionate+X" and return standalone Affection
            SentimentType::Affection
        } else if type_str.starts_with("sarcastic+") {
            // Extract just "sarcastic" from "sarcastic+X" and return standalone Sarcastic
            SentimentType::Sarcastic
        } else {
            match type_str {
                "happy" | "joy" => SentimentType::Joy, // Map happy to Joy
                "sad" => SentimentType::Sad,
                "angry" => SentimentType::Angry,
                "confused" => SentimentType::Confused,
                "fear" => SentimentType::Fear,
                "neutral" => SentimentType::Neutral,
                "affection" => SentimentType::Affection,
                "disgust" => SentimentType::Disgust,
                "surprise" => SentimentType::Surprise,
                "sarcastic" => SentimentType::Sarcastic,
                _ => SentimentType::Neutral,
            }
        }
    }
    
    pub async fn get_posts_paginated(&self, limit: u32, offset: u32) -> Result<Vec<PostResponse>> {
        // Try primary repository first, then fallback to CSV using helper method
        let posts = self.try_with_fallback(
            "get_posts_paginated",
            || self.primary_repo.get_posts_paginated(limit, offset),
            || self.csv_fallback_repo.get_posts_paginated(limit, offset),
        ).await?;
        
        Ok(posts.into_iter().map(PostResponse::from).collect())
    }

    /// Get posts for a specific user with pagination
    pub async fn get_posts_by_user(&self, user_id: uuid::Uuid, limit: u32, offset: u32) -> Result<Vec<PostResponse>> {
        // Try primary repository first, then fallback to CSV using helper method
        let posts = self.try_with_fallback(
            "get_posts_by_user",
            || self.primary_repo.get_posts_by_user(user_id, limit, offset),
            || self.csv_fallback_repo.get_posts_by_user(user_id, limit, offset),
        ).await?;
        
        Ok(posts.into_iter().map(PostResponse::from).collect())
    }
    
    pub async fn update_post(&self, post_id: Uuid, request: CreatePostRequest, author_id: Uuid) -> Result<PostResponse> {
        // First, get the existing post using fallback helper
        let existing_post = match self.try_with_fallback(
            "get_post_by_id_for_update",
            || self.primary_repo.get_post_by_id(post_id),
            || self.csv_fallback_repo.get_post_by_id(post_id),
        ).await? {
            Some(post) => post,
            None => return Err(AppError::NotFound("Post not found".to_string())),
        };
        
        // Verify the author owns the post
        if existing_post.author_id != author_id {
            return Err(AppError::AuthError("Not authorized to update this post".to_string()));
        }
        
        // Run moderation check on updated content
        let combined_text = format!("{} {}", request.title, request.content);
        let moderation_result = self.moderation_service.check_content(&combined_text).await
            .map_err(|e| AppError::InternalError(format!("Content moderation system error: {}", e)))?;
        
        if moderation_result.is_blocked {
            return Err(AppError::ValidationError("Updated content violates community guidelines".to_string()));
        }
        
        // Run sentiment analysis on updated content
        let title_sentiments = self.sentiment_service.analyze_sentiment(&request.title).await
            .map_err(|e| AppError::InternalError(format!("Sentiment analysis error (title): {}", e)))?;
        let body_sentiments = self.sentiment_service.analyze_sentiment(&request.content).await
            .map_err(|e| AppError::InternalError(format!("Sentiment analysis error (content): {}", e)))?;
        
        let sentiments = self.combine_sentiments_with_bias(&title_sentiments, &body_sentiments, 0.2, 0.8);
        
        // Update post fields
        let mut updated_post = existing_post;
        updated_post.title = request.title;
        updated_post.content = request.content;
        updated_post.updated_at = chrono::Utc::now();
        
        // Update sentiment data
        if !sentiments.is_empty() {
            updated_post.sentiment_score = Some(sentiments.iter().map(|s| s.confidence).sum::<f64>() / sentiments.len() as f64);
            updated_post.sentiment_colors = sentiments.iter().flat_map(|s| s.sentiment_type.colors_array()).collect();
            updated_post.sentiment_type = Some(sentiments[0].sentiment_type.to_string());
        }
        
        // Update popularity score
        updated_post.popularity_score = self.calculate_popularity_score_from_sentiment(&sentiments);
        
        // Write to both primary and CSV repositories for persistence
        let result_post = self.write_to_both_repositories(
            "update_post",
            || self.primary_repo.update_post(&updated_post),
            || self.csv_fallback_repo.update_post(&updated_post),
        ).await?;
        
        Ok(PostResponse::from(result_post))
    }
    
    pub async fn delete_post(&self, post_id: Uuid, author_id: Uuid) -> Result<()> {
        // First, get the existing post to verify ownership using fallback helper
        let existing_post = match self.try_with_fallback(
            "get_post_by_id_for_delete",
            || self.primary_repo.get_post_by_id(post_id),
            || self.csv_fallback_repo.get_post_by_id(post_id),
        ).await? {
            Some(post) => post,
            None => return Err(AppError::NotFound("Post not found".to_string())),
        };
        
        // Verify the author owns the post
        if existing_post.author_id != author_id {
            return Err(AppError::AuthError("Not authorized to delete this post".to_string()));
        }
        
        // Write to both primary and CSV repositories for persistence
        self.write_to_both_repositories(
            "delete_post",
            || async { self.primary_repo.delete_post(post_id).await.map(|_| ()) },
            || async { self.csv_fallback_repo.delete_post(post_id).await.map(|_| ()) },
        ).await?;
        
        Ok(())
    }

    // ========== MIGRATION METHODS ==========
    
    /// Run comprehensive migration to update all posts with old emotion types
    pub async fn run_emotion_migration(&self) -> Result<EmotionMigrationResult> {
        tracing::info!("🚀 MIGRATION: Starting comprehensive emotion migration");
        
        let mut migration_result = EmotionMigrationResult {
            total_posts_checked: 0,
            posts_requiring_migration: 0,
            posts_successfully_migrated: 0,
            posts_failed_migration: 0,
            errors: Vec::new(),
        };
        
        // Get posts with old sentiment types from both repositories
        let posts_with_old_sentiments = self.try_with_fallback(
            "get_posts_with_old_sentiment_types",
            || self.primary_repo.get_posts_with_old_sentiment_types(),
            || self.csv_fallback_repo.get_posts_with_old_sentiment_types()
        ).await?;
        
        migration_result.posts_requiring_migration = posts_with_old_sentiments.len();
        
        if posts_with_old_sentiments.is_empty() {
            tracing::info!("✅ MIGRATION: No posts found with old emotion types - migration not needed");
            return Ok(migration_result);
        }
        
        tracing::info!("🔍 MIGRATION: Found {} posts with old emotion types requiring migration", 
                      posts_with_old_sentiments.len());
        
        // Migrate each post individually
        for post in posts_with_old_sentiments {
            migration_result.total_posts_checked += 1;
            
            tracing::info!("🔄 MIGRATION: Processing post {} - '{}' (current emotion: {:?})", 
                          post.id, 
                          if post.title.len() > 50 { &post.title[..50] } else { &post.title },
                          post.sentiment_type);
            
            match self.migrate_single_post(&post).await {
                Ok(new_sentiment_info) => {
                    migration_result.posts_successfully_migrated += 1;
                    tracing::info!("✅ MIGRATION: Successfully migrated post {} from {:?} to {:?}", 
                                  post.id, post.sentiment_type, new_sentiment_info.sentiment_type);
                }
                Err(e) => {
                    migration_result.posts_failed_migration += 1;
                    let error_msg = format!("Failed to migrate post {}: {}", post.id, e);
                    migration_result.errors.push(error_msg.clone());
                    tracing::error!("❌ MIGRATION: {}", error_msg);
                }
            }
        }
        
        // Log final migration results
        tracing::info!("🏁 MIGRATION: Emotion migration completed");
        tracing::info!("   📊 Total posts checked: {}", migration_result.total_posts_checked);
        tracing::info!("   🎯 Posts requiring migration: {}", migration_result.posts_requiring_migration);
        tracing::info!("   ✅ Posts successfully migrated: {}", migration_result.posts_successfully_migrated);
        tracing::info!("   ❌ Posts failed migration: {}", migration_result.posts_failed_migration);
        
        if !migration_result.errors.is_empty() {
            tracing::warn!("⚠️ MIGRATION: {} errors occurred during migration", migration_result.errors.len());
            for error in &migration_result.errors {
                tracing::warn!("   - {}", error);
            }
        }
        
        Ok(migration_result)
    }
    
    /// Migrate a single post by reprocessing its text content
    async fn migrate_single_post(&self, post: &Post) -> Result<SentimentInfo> {
        // Combine title and content for sentiment analysis (4x body weight like in create_post)
        let title_sentiment = self.sentiment_service.analyze_sentiment(&post.title).await
            .map_err(|e| AppError::InternalError(format!("Failed to analyze title sentiment during migration: {}", e)))?;
        
        let body_sentiment = self.sentiment_service.analyze_sentiment(&post.content).await
            .map_err(|e| AppError::InternalError(format!("Failed to analyze body sentiment during migration: {}", e)))?;
        
        // Determine primary sentiment with body bias (4x weight)
        let primary_sentiment = if body_sentiment.is_empty() {
            title_sentiment.first().cloned()
        } else {
            body_sentiment.first().cloned()
        };
        
        let sentiment_info = if let Some(ref sentiment) = primary_sentiment {
            SentimentInfo {
                sentiment_type: Some(sentiment.sentiment_type.to_string()),
                sentiment_colors: sentiment.sentiment_type.colors_array(),
                sentiment_score: Some(sentiment.confidence),
            }
        } else {
            // Fallback to neutral if analysis fails
            tracing::warn!("⚠️ MIGRATION: Sentiment analysis failed for post {}, using neutral fallback", post.id);
            SentimentInfo {
                sentiment_type: Some("neutral".to_string()),
                sentiment_colors: vec!["#6b7280".to_string()],
                sentiment_score: Some(0.5),
            }
        };
        
        // Update post sentiment in both repositories
        self.write_to_both_repositories(
            "update_post_sentiment",
            || self.primary_repo.update_post_sentiment(
                post.id, 
                sentiment_info.sentiment_type.clone(), 
                sentiment_info.sentiment_colors.clone(), 
                sentiment_info.sentiment_score
            ),
            || self.csv_fallback_repo.update_post_sentiment(
                post.id, 
                sentiment_info.sentiment_type.clone(), 
                sentiment_info.sentiment_colors.clone(), 
                sentiment_info.sentiment_score
            )
        ).await?;
        
        // Recalculate and update popularity score based on new sentiment
        let new_popularity_score = self.calculate_popularity_score_from_sentiment(&if let Some(sentiment) = primary_sentiment {
            vec![sentiment]
        } else {
            vec![]
        });
        
        let _ = self.write_to_both_repositories(
            "update_popularity_score",
            || self.primary_repo.update_popularity_score(post.id, new_popularity_score),
            || self.csv_fallback_repo.update_popularity_score(post.id, new_popularity_score)
        ).await;
        
        Ok(sentiment_info)
    }
    
    /// Check if migration is needed (posts with old emotion types exist)
    pub async fn is_migration_needed(&self) -> Result<bool> {
        let posts_with_old_sentiments = self.try_with_fallback(
            "get_posts_with_old_sentiment_types",
            || self.primary_repo.get_posts_with_old_sentiment_types(),
            || self.csv_fallback_repo.get_posts_with_old_sentiment_types()
        ).await?;
        
        Ok(!posts_with_old_sentiments.is_empty())
    }
}

#[derive(Debug, Clone)]
pub struct SentimentInfo {
    pub sentiment_type: Option<String>,
    pub sentiment_colors: Vec<String>,
    pub sentiment_score: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct EmotionMigrationResult {
    pub total_posts_checked: usize,
    pub posts_requiring_migration: usize,
    pub posts_successfully_migrated: usize,
    pub posts_failed_migration: usize,
    pub errors: Vec<String>,
}