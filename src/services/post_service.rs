use crate::models::Post;
use crate::models::post::{CreatePostRequest, PostResponse};
use crate::models::sentiment::{Sentiment, SentimentType};
use crate::db::repository::{PostRepository, MockPostRepository, CsvPostRepository};
use crate::services::{SentimentService, ModerationService};
use crate::{AppError, Result};
use uuid::Uuid;
use std::sync::Arc;
use chrono::Utc;

pub struct PostService {
    primary_repo: Arc<dyn PostRepository>,
    csv_fallback_repo: Arc<CsvPostRepository>,
    sentiment_service: Arc<SentimentService>,
    moderation_service: Arc<ModerationService>,
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
            moderation_service 
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
            return Err(AppError::ValidationError(error_message));
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
        // PRIORITIZE COMBINATION SENTIMENTS - if either title or body has sarcasm or affection combos, use them
        // Check for affectionate combinations first (slightly higher priority)
        for sentiment in body_sentiments.iter() {
            if matches!(sentiment.sentiment_type, SentimentType::AffectionateCombination(_)) {
                return vec![sentiment.clone()];
            }
        }
        for sentiment in title_sentiments.iter() {
            if matches!(sentiment.sentiment_type, SentimentType::AffectionateCombination(_)) {
                return vec![sentiment.clone()];
            }
        }
        
        // Then check for sarcastic combinations
        for sentiment in body_sentiments.iter() {
            if matches!(sentiment.sentiment_type, SentimentType::SarcasticCombination(_)) {
                return vec![sentiment.clone()];
            }
        }
        for sentiment in title_sentiments.iter() {
            if matches!(sentiment.sentiment_type, SentimentType::SarcasticCombination(_)) {
                return vec![sentiment.clone()];
            }
        }
        
        // Filter out fallback Neutral sentiments (parser failures) but preserve sarcastic combinations
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
                SentimentType::SarcasticCombination(ref base_type) => {
                    // For sarcastic combinations, reduce the base sentiment score
                    let base_score = match **base_type {
                        SentimentType::Joy => 1.4,
                        SentimentType::Affection => 1.1,
                        SentimentType::Surprise => 1.1,
                        SentimentType::Neutral => 1.0,
                        SentimentType::Confused => 0.95,
                        SentimentType::Sarcastic => 0.9,
                        SentimentType::Disgust => 0.6,
                        SentimentType::Sad => 0.8,
                        SentimentType::Fear => 0.7,
                        SentimentType::Angry => 0.6,
                        _ => 1.0,
                    };
                    base_score * 0.8  // Reduce by 20% due to sarcasm
                }
                SentimentType::AffectionateCombination(ref base_type) => {
                    // For affectionate combinations, boost the base sentiment score
                    let base_score = match **base_type {
                        SentimentType::Joy => 1.4,
                        SentimentType::Affection => 1.1,
                        SentimentType::Surprise => 1.1,
                        SentimentType::Neutral => 1.0,
                        SentimentType::Confused => 0.95,
                        SentimentType::Sarcastic => 0.9,
                        SentimentType::Disgust => 0.6,
                        SentimentType::Sad => 0.8,
                        SentimentType::Fear => 0.7,
                        SentimentType::Angry => 0.6,
                        _ => 1.0,
                    };
                    base_score * 1.2  // Boost by 20% due to affection
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
    
    // Helper method to parse sentiment type from string (handles combinations)
    fn parse_sentiment_type_from_string(&self, type_str: &str) -> SentimentType {
        if type_str.starts_with("affectionate+") {
            let base_emotion = type_str.strip_prefix("affectionate+").unwrap_or("neutral");
            let base_type = match base_emotion {
                "happy" | "joy" => SentimentType::Joy, // Map happy to Joy
                "sad" => SentimentType::Sad,
                "angry" => SentimentType::Angry,
                "confused" => SentimentType::Confused,
                "fear" => SentimentType::Fear,
                "disgust" => SentimentType::Disgust,
                "surprise" => SentimentType::Surprise,
                "neutral" => SentimentType::Neutral,
                "affection" => SentimentType::Affection,
                _ => SentimentType::Neutral,
            };
            SentimentType::AffectionateCombination(Box::new(base_type))
        } else if type_str.starts_with("sarcastic+") {
            let base_emotion = type_str.strip_prefix("sarcastic+").unwrap_or("neutral");
            let base_type = match base_emotion {
                "happy" | "joy" => SentimentType::Joy, // Map happy to Joy
                "sad" => SentimentType::Sad,
                "angry" => SentimentType::Angry,
                "confused" => SentimentType::Confused,
                "fear" => SentimentType::Fear,
                "disgust" => SentimentType::Disgust,
                "surprise" => SentimentType::Surprise,
                "neutral" => SentimentType::Neutral,
                "affection" => SentimentType::Affection,
                _ => SentimentType::Neutral,
            };
            SentimentType::SarcasticCombination(Box::new(base_type))
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
}