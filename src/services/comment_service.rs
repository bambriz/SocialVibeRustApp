/*!
 * Comment Service for Social Pulse - Simplified Implementation
 * 
 * A basic comment service to get the system working.
 * Will be enhanced with full Reddit-style features later.
 */

use crate::models::comment::{Comment, CreateCommentRequest, CommentResponse};
use crate::db::repository::{CommentRepository, UserRepository};
use crate::services::sentiment_service::SentimentService;
use crate::services::moderation_service::ModerationService;
use crate::services::vote_service::VoteService;
use crate::{AppError, Result};
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;

/// Hierarchical path generation for Reddit-style nesting
const MAX_DEPTH: i32 = 10;
const PATH_SEPARATOR: &str = "/";
const DEPTH_INCREMENT: i32 = 1;

/// Enhanced comment service with sentiment analysis and hierarchy
pub struct CommentService {
    comment_repo: Arc<dyn CommentRepository>,
    user_repo: Arc<dyn UserRepository>,
    sentiment_service: Option<Arc<SentimentService>>,
    moderation_service: Option<Arc<ModerationService>>,
    vote_service: Option<Arc<VoteService>>,
}


impl CommentService {
    
    /// Create CommentService with AI services (enhanced version)
    pub fn new_with_ai(
        comment_repo: Arc<dyn CommentRepository>,
        user_repo: Arc<dyn UserRepository>,
        sentiment_service: Option<Arc<SentimentService>>,
        moderation_service: Option<Arc<ModerationService>>,
        vote_service: Option<Arc<VoteService>>,
    ) -> Self {
        Self { 
            comment_repo,
            user_repo,
            sentiment_service,
            moderation_service,
            vote_service,
        }
    }
    
    /// Generate materialized path with atomic sibling index allocation
    #[allow(dead_code)]
    async fn generate_comment_path(&self, post_id: Uuid, parent_id: Option<Uuid>) -> Result<(String, i32)> {
        match parent_id {
            None => {
                // Root-level comment: Use atomic per-post sibling allocation
                let sibling_index = self.comment_repo
                    .allocate_next_sibling_index(post_id, None)
                    .await?;
                let path_segment = format!("{:06}{}", sibling_index, PATH_SEPARATOR);
                Ok((path_segment, 0))
            },
            Some(parent_id) => {
                // Reply: extend parent's path with atomic sibling index
                let parent = self.comment_repo
                    .get_comment_by_id(parent_id)
                    .await?
                    .ok_or_else(|| AppError::ValidationError("Parent comment not found".to_string()))?;
                
                // Check depth limit
                if parent.depth >= MAX_DEPTH {
                    return Err(AppError::ValidationError(
                        format!("Maximum nesting depth ({}) exceeded", MAX_DEPTH)
                    ));
                }
                
                // Atomic sibling index allocation prevents race conditions
                let sibling_index = self.comment_repo
                    .allocate_next_sibling_index(post_id, Some(parent_id))
                    .await?;
                
                // Zero-padded path segment ensures correct lexicographic sorting
                let new_path = format!("{}{:06}{}", 
                    parent.path, 
                    sibling_index, 
                    PATH_SEPARATOR
                );
                
                Ok((new_path, parent.depth + DEPTH_INCREMENT))
            }
        }
    }
    
    /// Create a new comment with full AI processing pipeline
    pub async fn create_comment(
        &self, 
        post_id: Uuid,
        request: CreateCommentRequest, 
        user_id: Uuid
    ) -> Result<CommentResponse> {
        tracing::info!("🔄 COMMENT_SERVICE_DIAGNOSTIC: Starting comment creation process");
        tracing::info!("   📍 Post ID: {}", post_id);
        tracing::info!("   👤 User ID: {}", user_id);
        tracing::info!("   📄 Raw content length: {}", request.content.len());
        
        // Basic validation
        let content = request.content.trim();
        tracing::info!("   📝 Trimmed content length: {}", content.len());
        
        if content.is_empty() {
            tracing::error!("❌ Validation failed: Empty content after trimming");
            return Err(AppError::ValidationError("Comment content cannot be empty".to_string()));
        }
        
        if content.len() > 2000 {
            tracing::error!("❌ Validation failed: Content too long ({} > 2000)", content.len());
            return Err(AppError::ValidationError("Comment content exceeds 2000 character limit".to_string()));
        }
        
        tracing::debug!("   ✅ Basic validation passed");

        // Path and depth will be computed atomically in the repository
        
        // 2. Content moderation check
        tracing::info!("🛡️  COMMENT_SERVICE_DIAGNOSTIC: Starting content moderation");
        let moderation_start = std::time::Instant::now();
        
        let moderation_result = if let Some(moderation_service) = &self.moderation_service {
            tracing::debug!("   🔍 Using content moderation service");
            moderation_service.check_content(content).await
                .map_err(|e| {
                    tracing::error!("❌ Content moderation system error: {}", e);
                    AppError::InternalError(format!("Content moderation system error: {}. Please try again later or contact support if this persists.", e))
                })?
        } else {
            tracing::warn!("   ⚠️  No moderation service configured - allowing content");
            crate::services::moderation_service::ModerationResult {
                is_blocked: false,
                violation_type: None,
                details: None,
                toxicity_tags: Vec::new(),
                all_scores: None,
            }
        };
        
        let moderation_duration = moderation_start.elapsed();
        tracing::info!("   ✅ Moderation completed in {:?} - Blocked: {}", 
                      moderation_duration, moderation_result.is_blocked);
        
        if moderation_result.is_blocked {
            tracing::warn!("❌ Comment blocked by moderation: {:?}", moderation_result.violation_type);
            return Err(AppError::ValidationError(format!(
                "Comment blocked by moderation: {}",
                moderation_result.violation_type.clone().unwrap_or_else(|| "Policy violation".to_string())
            )));
        }

        // 3. Sentiment analysis - analyze content for emotions (same as posts)
        tracing::info!("🎭 COMMENT_SERVICE_DIAGNOSTIC: Starting sentiment analysis");
        let sentiment_start = std::time::Instant::now();
        
        let (sentiment_score, sentiment_colors, sentiment_type) = if let Some(sentiment_service) = &self.sentiment_service {
            tracing::debug!("   🔍 Using sentiment analysis service");
            let sentiments = sentiment_service
                .analyze_sentiment(content)
                .await
                .map_err(|e| {
                    tracing::error!("❌ Sentiment analysis failed: {}", e);
                    AppError::InternalError(format!("Sentiment analysis failed: {}", e))
                })?;
            
            if let Some(primary_sentiment) = sentiments.first() {
                let score = (primary_sentiment.confidence - 0.5) * 2.0; // Convert confidence to score
                let colors = vec![primary_sentiment.color_code.clone()];
                let sentiment_type = Some(primary_sentiment.sentiment_type.to_string());
                
                tracing::info!("   🎯 Sentiment detected: {:?} (confidence: {:.2})", 
                              primary_sentiment.sentiment_type, primary_sentiment.confidence);
                (Some(score), colors, sentiment_type)
            } else {
                tracing::debug!("   ⚪ No sentiment detected");
                (None, vec![], None)
            }
        } else {
            tracing::warn!("   ⚠️  No sentiment service configured - skipping analysis");
            (None, vec![], None)
        };
        
        let sentiment_duration = sentiment_start.elapsed();
        tracing::info!("   ✅ Sentiment analysis completed in {:?}", sentiment_duration);
        
        // 4. Calculate initial popularity score based on sentiment
        let popularity_score = self.calculate_popularity_score_from_sentiment(&sentiment_score, &sentiment_type);

        // 5. Create comment with full metadata (path and depth computed atomically)
        let comment = Comment {
            id: Uuid::new_v4(),
            post_id,
            user_id,
            parent_id: request.parent_id,
            content: request.content.clone(),
            path: String::new(), // Will be computed atomically
            depth: 0, // Will be computed atomically
            sentiment_score,
            sentiment_colors,
            sentiment_type,
            is_blocked: moderation_result.is_blocked,
            toxicity_tags: moderation_result.toxicity_tags,
            toxicity_scores: moderation_result.all_scores,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            reply_count: 0,
            popularity_score,
            sentiment_analysis: Some(serde_json::json!({
                "sentiment_score": sentiment_score,
                "sentiment_colors": sentiment_colors,
                "sentiment_type": sentiment_type
            })),
        };
        
        // 6. Save using atomic method (computes path + inserts + increments reply count in single transaction)
        let saved_comment = self.comment_repo
            .create_comment_atomic(post_id, request.parent_id, &comment)
            .await?;
        
        tracing::info!("✅ Created {} comment {} with sentiment analysis", 
            if request.parent_id.is_some() { "nested" } else { "top-level" },
            saved_comment.id
        );
        
        // Fetch author information
        let author = self.user_repo.get_user_by_id(saved_comment.user_id).await?;
        
        // ENHANCED: Log that comment was created for future post popularity updates
        tracing::debug!("📊 Comment created for post {} - popularity will be updated by periodic refresh", post_id);

        Ok(CommentResponse {
            comment: saved_comment,
            author,
            replies: vec![],
            can_modify: true,
            is_collapsed: false,
        })
    }
    
    /// Increment reply count for parent comment
    #[allow(dead_code)]
    async fn increment_reply_count(&self, parent_id: Uuid) -> Result<()> {
        // This would be handled by the repository layer
        // For now, we'll leave it as a placeholder
        tracing::debug!("📊 Incrementing reply count for parent comment {}", parent_id);
        Ok(())
    }

    /// Get comments for a post with sorting option
    pub async fn get_comments_for_post(&self, post_id: Uuid, sort_by: Option<&str>) -> Result<Vec<CommentResponse>> {
        let mut comments = self.comment_repo
            .get_comments_by_post_id(post_id)
            .await?;

        // Sort comments by popularity while preserving hierarchy if requested
        if let Some("popular") = sort_by {
            comments = self.sort_comments_by_popularity(comments).await;
        }
            
        // Fetch author information for all comments
        let mut comment_responses = Vec::new();
        for comment in comments {
            let author = self.user_repo.get_user_by_id(comment.user_id).await?;
            comment_responses.push(CommentResponse {
                comment,
                author,
                replies: vec![], // Will be populated in build_hierarchical_structure
                can_modify: false, // TODO: Check permissions
                is_collapsed: false,
            });
        }

        // Build hierarchical structure
        let hierarchical_comments = self.build_hierarchical_structure(comment_responses);

        Ok(hierarchical_comments)
    }

    /// Build hierarchical comment structure from flat list
    fn build_hierarchical_structure(&self, comments: Vec<CommentResponse>) -> Vec<CommentResponse> {
        use std::collections::HashMap;
        
        // Create a map of comment ID to comment for quick lookup
        let mut comment_map: HashMap<Uuid, CommentResponse> = HashMap::new();
        for comment in comments {
            comment_map.insert(comment.comment.id, comment);
        }
        
        // Build the hierarchy
        let mut root_comments = Vec::new();
        let mut comments_to_process: Vec<_> = comment_map.into_iter().collect();
        
        // First pass: identify root comments
        let mut i = 0;
        while i < comments_to_process.len() {
            let (id, comment) = &comments_to_process[i];
            if comment.comment.parent_id.is_none() {
                // This is a root comment
                let (_, root_comment) = comments_to_process.remove(i);
                root_comments.push(root_comment);
            } else {
                i += 1;
            }
        }
        
        // Recursive function to build children for each comment
        fn build_children(
            parent_id: Uuid,
            remaining_comments: &mut Vec<(Uuid, CommentResponse)>
        ) -> Vec<CommentResponse> {
            let mut children = Vec::new();
            let mut i = 0;
            
            while i < remaining_comments.len() {
                let (_, comment) = &remaining_comments[i];
                if comment.comment.parent_id == Some(parent_id) {
                    // This comment is a child of the current parent
                    let (child_id, mut child_comment) = remaining_comments.remove(i);
                    
                    // Recursively build this child's children
                    child_comment.replies = build_children(child_id, remaining_comments);
                    children.push(child_comment);
                } else {
                    i += 1;
                }
            }
            
            children
        }
        
        // Build children for each root comment
        for root_comment in &mut root_comments {
            root_comment.replies = build_children(root_comment.comment.id, &mut comments_to_process);
        }
        
        root_comments
    }
    
    /// Sort comments by popularity while preserving hierarchical structure
    /// Root comments are sorted by popularity, and replies within each parent are also sorted by popularity
    async fn sort_comments_by_popularity(&self, comments: Vec<Comment>) -> Vec<Comment> {
        use std::collections::HashMap;
        
        // Calculate current popularity scores for all comments including votes
        let mut comment_scores = HashMap::new();
        for comment in &comments {
            // Calculate popularity including current votes
            let score = if let Some(vote_service) = &self.vote_service {
                self.calculate_popularity_score(comment, Some(vote_service.as_ref())).await
            } else {
                self.calculate_popularity_score(comment, None).await
            };
            comment_scores.insert(comment.id, score);
        }
        
        // Group comments by depth and parent for hierarchical sorting
        let mut root_comments = Vec::new();
        let mut replies_by_parent: HashMap<String, Vec<Comment>> = HashMap::new();
        
        for comment in comments {
            if comment.depth == 0 {
                root_comments.push(comment);
            } else {
                // Extract parent path from current path to group replies
                let parent_path = self.extract_parent_path(&comment.path);
                replies_by_parent.entry(parent_path).or_insert_with(Vec::new).push(comment);
            }
        }
        
        // Sort root comments by popularity
        root_comments.sort_by(|a, b| {
            let score_a = comment_scores.get(&a.id).unwrap_or(&1.0);
            let score_b = comment_scores.get(&b.id).unwrap_or(&1.0);
            score_b.partial_cmp(score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Sort replies within each parent group by popularity
        for replies in replies_by_parent.values_mut() {
            replies.sort_by(|a, b| {
                let score_a = comment_scores.get(&a.id).unwrap_or(&1.0);
                let score_b = comment_scores.get(&b.id).unwrap_or(&1.0);
                score_b.partial_cmp(score_a).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        
        // Reconstruct the comment list maintaining hierarchy but with popularity sorting
        let mut sorted_comments = Vec::new();
        
        // Add root comments and their sorted replies recursively
        for root_comment in root_comments {
            sorted_comments.push(root_comment.clone());
            self.add_sorted_replies(&root_comment.path, &replies_by_parent, &mut sorted_comments);
        }
        
        sorted_comments
    }
    
    /// Extract parent path from a comment path (e.g., "000001/000002/" -> "000001/")
    fn extract_parent_path(&self, path: &str) -> String {
        let parts: Vec<&str> = path.trim_end_matches('/').split('/').collect();
        if parts.len() > 1 {
            format!("{}/", parts[..parts.len()-1].join("/"))
        } else {
            String::new()
        }
    }
    
    /// Recursively add sorted replies to the result list
    fn add_sorted_replies(&self, parent_path: &str, replies_by_parent: &HashMap<String, Vec<Comment>>, result: &mut Vec<Comment>) {
        if let Some(replies) = replies_by_parent.get(parent_path) {
            for reply in replies {
                result.push(reply.clone());
                // Recursively add replies to this reply
                self.add_sorted_replies(&reply.path, replies_by_parent, result);
            }
        }
    }
    
    /// Calculate popularity score from sentiment data (similar to posts)
    fn calculate_popularity_score_from_sentiment(&self, sentiment_score: &Option<f64>, sentiment_type: &Option<String>) -> f64 {
        // Default neutral score
        let mut base_score = 1.0;
        
        // Apply sentiment multiplier based on type (same logic as posts)
        if let Some(sentiment_type) = sentiment_type {
            let sentiment_multiplier = match sentiment_type.as_str() {
                "joy" => 1.4,       // Highest positive boost
                "affection" => 1.1,
                "surprise" => 1.1,  // Slight positive boost - draws attention
                "neutral" => 1.0,   // Neutral scoring
                "confused" => 0.95, // Slightly lower than neutral
                "sarcastic" => 0.9,
                "disgust" => 0.6,   // Low engagement like anger
                "sad" => 0.8,
                "fear" => 0.7,
                "angry" => 0.6,
                _ => 1.0, // Default for unknown types
            };
            
            // Apply confidence if available
            if let Some(confidence) = sentiment_score {
                base_score = sentiment_multiplier * confidence.abs().max(0.1); // Ensure minimum positive score
            } else {
                base_score = sentiment_multiplier;
            }
        }
        
        // Ensure score is always positive and reasonable
        base_score.max(0.1).min(2.0)
    }
    
    /// Calculate full popularity score including votes and engagement (similar to posts)
    pub async fn calculate_popularity_score(&self, comment: &Comment, vote_service: Option<&crate::services::vote_service::VoteService>) -> f64 {
        // Base score from sentiment analysis
        let sentiment_score = self.calculate_popularity_score_from_sentiment(&comment.sentiment_score, &comment.sentiment_type);
        
        // ENHANCED: Factor in engagement metrics with bigger reply boost
        let reply_boost = (comment.reply_count as f64) * 0.15; // Increased from 0.05 to 0.15
        let recency_hours = (Utc::now() - comment.created_at).num_hours() as f64;
        
        // Enhanced recency decay - newer comments get more boost
        let recency_decay = if recency_hours <= 6.0 {
            // Comments within 6 hours get a boost
            1.0 + (6.0 - recency_hours) * 0.1  // Up to 60% boost for very recent comments
        } else {
            // Older comments decay more than posts since conversation moves faster
            1.0 / (1.0 + (recency_hours - 6.0) * 0.03)
        };
        
        let base_score = (sentiment_score + reply_boost) * recency_decay;
        
        // Add voting engagement if available, with cap at 3.0
        let final_score = if let Some(vote_service) = vote_service {
            match vote_service.get_engagement_score(comment.id, "comment").await {
                Ok(engagement_score) => {
                    // Cap total popularity at 3.0 as per user requirements
                    (base_score + engagement_score).min(3.0)
                },
                Err(_) => base_score // Fall back to base score if voting fails
            }
        } else {
            base_score
        };
        
        // CRITICAL: Ensure popularity score is never negative - minimum value of 0.0
        final_score.max(0.0)
    }
    
    /// Recalculate and update popularity score after voting (for triggering recalculation)
    pub async fn update_popularity_after_vote(&self, comment_id: Uuid, vote_service: Option<&crate::services::vote_service::VoteService>) -> Result<()> {
        // Get the current comment
        let comment = self.comment_repo
            .get_comment_by_id(comment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
        
        // Calculate new popularity score including voting engagement
        let new_popularity_score = self.calculate_popularity_score(&comment, vote_service).await;
        
        // For now, PostgreSQL calculates popularity on demand, so no update needed
        // This method is here for consistency and future implementations
        tracing::debug!("📊 Calculated new popularity score for comment {} to {}", comment_id, new_popularity_score);
        Ok(())
    }

    /// Get a specific comment thread (placeholder)
    pub async fn get_comment_thread(&self, comment_id: Uuid) -> Result<Vec<CommentResponse>> {
        // For now, just return the single comment
        let comment = self.comment_repo
            .get_comment_by_id(comment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
            
        Ok(vec![CommentResponse {
            comment,
            author: None,
            replies: vec![],
            can_modify: false,
            is_collapsed: false,
        }])
    }
    
    /// Update a comment (placeholder)
    pub async fn update_comment(&self, id: Uuid, content: String, user_id: Uuid) -> Result<CommentResponse> {
        let mut comment = self.comment_repo
            .get_comment_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
            
        // Basic permission check
        if comment.user_id != user_id {
            return Err(AppError::ValidationError("You cannot modify this comment".to_string()));
        }
        
        // Basic validation
        if content.trim().is_empty() {
            return Err(AppError::ValidationError("Comment content cannot be empty".to_string()));
        }
        
        // Update comment
        comment.content = content;
        comment.updated_at = Utc::now();
        
        let updated_comment = self.comment_repo
            .update_comment(&comment)
            .await?;
            
        Ok(CommentResponse {
            comment: updated_comment,
            author: None,
            replies: vec![],
            can_modify: true,
            is_collapsed: false,
        })
    }
    
    /// Delete a comment (placeholder)
    pub async fn delete_comment(&self, id: Uuid, user_id: Uuid) -> Result<()> {
        let comment = self.comment_repo
            .get_comment_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;
            
        // Basic permission check
        if comment.user_id != user_id {
            return Err(AppError::ValidationError("You cannot delete this comment".to_string()));
        }
        
        self.comment_repo
            .delete_comment(id)
            .await?;
            
        Ok(())
    }

    // ========== MIGRATION METHODS ==========
    
    /// Check if comment migration is needed (placeholder for future comment sentiment migration)
    pub async fn is_migration_needed(&self) -> Result<bool> {
        // Comments don't currently have sentiment_type field like posts do
        // This method is here for future compatibility if comment sentiment analysis is added
        tracing::info!("🔍 MIGRATION: Checking if comment migration is needed");
        tracing::info!("✅ MIGRATION: Comments don't currently require migration (no sentiment_type field)");
        Ok(false)
    }
    
    /// Run comment migration (placeholder for future implementation)
    pub async fn run_emotion_migration(&self) -> Result<CommentMigrationResult> {
        tracing::info!("🚀 MIGRATION: Starting comment emotion migration");
        tracing::info!("✅ MIGRATION: No comment migration needed - comments don't have sentiment_type field yet");
        
        Ok(CommentMigrationResult {
            total_comments_checked: 0,
            comments_requiring_migration: 0,
            comments_successfully_migrated: 0,
            comments_failed_migration: 0,
            errors: Vec::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct CommentMigrationResult {
    pub total_comments_checked: usize,
    pub comments_requiring_migration: usize,
    pub comments_successfully_migrated: usize,
    pub comments_failed_migration: usize,
    pub errors: Vec<String>,
}