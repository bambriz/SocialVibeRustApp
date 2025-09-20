/*!
 * Comment Service for Social Pulse - Simplified Implementation
 * 
 * A basic comment service to get the system working.
 * Will be enhanced with full Reddit-style features later.
 */

use crate::models::comment::{Comment, CreateCommentRequest, CommentResponse};
use crate::db::repository::CommentRepository;
use crate::services::sentiment_service::SentimentService;
use crate::services::moderation_service::ModerationService;
use crate::{AppError, Result};
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;

/// Enhanced comment service with sentiment analysis and hierarchy
pub struct CommentService {
    comment_repo: Arc<dyn CommentRepository>,
    sentiment_service: Option<Arc<SentimentService>>,
    moderation_service: Option<Arc<ModerationService>>,
}

/// Hierarchical path generation for Reddit-style nesting
const MAX_DEPTH: i32 = 10;
const PATH_SEPARATOR: &str = "/";
const DEPTH_INCREMENT: i32 = 1;

impl CommentService {
    /// Create a new CommentService (basic version for now)
    pub fn new(comment_repo: Arc<dyn CommentRepository>) -> Self {
        Self { 
            comment_repo,
            sentiment_service: None, // TODO: Connect to sentiment service
            moderation_service: None, // TODO: Connect to moderation service
        }
    }
    
    /// Create CommentService with AI services (enhanced version)
    pub fn new_with_ai(
        comment_repo: Arc<dyn CommentRepository>,
        sentiment_service: Option<Arc<SentimentService>>,
        moderation_service: Option<Arc<ModerationService>>,
    ) -> Self {
        Self { 
            comment_repo,
            sentiment_service,
            moderation_service,
        }
    }
    
    /// Generate materialized path with atomic sibling index allocation
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
        // Basic validation
        let content = request.content.trim();
        if content.is_empty() {
            return Err(AppError::ValidationError("Comment content cannot be empty".to_string()));
        }
        
        if content.len() > 2000 {
            return Err(AppError::ValidationError("Comment content exceeds 2000 character limit".to_string()));
        }

        // Path and depth will be computed atomically in the repository
        
        // 2. Content moderation check
        let (moderation_result, is_flagged) = if let Some(moderation_service) = &self.moderation_service {
            let mod_result = moderation_service
                .check_content(content)
                .await
                .map_err(|e| AppError::InternalError(format!("Moderation failed: {}", e)))?;
            
            if mod_result.is_blocked {
                return Err(AppError::ValidationError(format!(
                    "Comment blocked by moderation: {}",
                    mod_result.violation_type.clone().unwrap_or_else(|| "Policy violation".to_string())
                )));
            }
            
            // Separate flagging logic: flag based on toxicity score thresholds, not blocking
            let is_flagged = !mod_result.toxicity_tags.is_empty() || 
                mod_result.all_scores
                    .as_ref()
                    .and_then(|v| v.as_object())
                    .map(|scores| scores.values().any(|v| v.as_f64().unwrap_or(0.0) > 0.5))
                    .unwrap_or(false);
            
            (serde_json::to_value(&mod_result).ok(), is_flagged)
        } else {
            (None, false)
        };
        
        // 3. Sentiment analysis
        let sentiment_analysis = if let Some(sentiment_service) = &self.sentiment_service {
            let sentiments = sentiment_service
                .analyze_sentiment(content)
                .await
                .map_err(|e| AppError::InternalError(format!("Sentiment analysis failed: {}", e)))?;
            serde_json::to_value(&sentiments).ok()
        } else {
            None
        };
        
        // 4. Create comment with full metadata (path and depth computed atomically)
        let comment = Comment {
            id: Uuid::new_v4(),
            post_id,
            user_id,
            parent_id: request.parent_id,
            content: request.content.clone(),
            path: String::new(), // Will be computed atomically
            depth: 0, // Will be computed atomically
            sentiment_analysis,
            moderation_result,
            is_flagged: is_flagged, // Store flagging state separate from blocking
            created_at: Utc::now(),
            updated_at: Utc::now(),
            reply_count: 0,
        };
        
        // 5. Save using atomic method (computes path + inserts + increments reply count in single transaction)
        let saved_comment = self.comment_repo
            .create_comment_atomic(post_id, request.parent_id, &comment)
            .await?;
        
        tracing::info!("✅ Created {} comment {} with sentiment analysis", 
            if request.parent_id.is_some() { "nested" } else { "top-level" },
            saved_comment.id
        );
        
        Ok(CommentResponse {
            comment: saved_comment,
            author: None, // TODO: Fetch author info from user service
            replies: vec![],
            can_modify: true,
            is_collapsed: false,
        })
    }
    
    /// Increment reply count for parent comment
    async fn increment_reply_count(&self, parent_id: Uuid) -> Result<()> {
        // This would be handled by the repository layer
        // For now, we'll leave it as a placeholder
        tracing::debug!("📊 Incrementing reply count for parent comment {}", parent_id);
        Ok(())
    }

    /// Get comments for a post (simplified - no tree structure yet)
    pub async fn get_comments_for_post(&self, post_id: Uuid) -> Result<Vec<CommentResponse>> {
        let comments = self.comment_repo
            .get_comments_by_post_id(post_id)
            .await?;
            
        let responses = comments
            .into_iter()
            .map(|comment| CommentResponse {
                comment,
                author: None, // TODO: Fetch author info
                replies: vec![], // TODO: Build hierarchical structure
                can_modify: false, // TODO: Check permissions
                is_collapsed: false,
            })
            .collect();
            
        Ok(responses)
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