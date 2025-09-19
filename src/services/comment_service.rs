/*!
 * Comment Service for Social Pulse - Reddit-Style Hierarchical Comments
 * 
 * LEARNING FOCUS: Advanced Rust concepts for building comment systems
 * 
 * This is your learning laboratory! This service demonstrates:
 * - Database operations with async/await
 * - Error handling with Result types  
 * - Tree data structures and algorithms
 * - Ownership and borrowing patterns
 * - Integration with AI sentiment analysis
 * 
 * YOUR MISSION: Complete the TODO methods to build a full comment system
 */

use crate::models::comment::{Comment, CreateCommentRequest, CommentResponse, CommentTreeNode,
                             CommentTreeMetadata};
use crate::db::repository::{CommentRepository, MockCommentRepository};
use crate::services::user_service::UserService;
use crate::services::moderation_service::ModerationService;
use crate::services::sentiment_service::SentimentService; // Added
use crate::{AppError, Result};
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;

/// The CommentService handles all comment-related operations
/// 
/// LEARNING CONCEPTS:
/// - Arc<T> for shared ownership across threads
/// - async/await for non-blocking database operations  
/// - Result<T, E> for comprehensive error handling
/// - Generic programming with trait bounds
pub struct CommentService {
    /// Database repository for comment operations
    /// LEARNING: We use Arc for shared ownership and dyn for trait objects
    comment_repo: Arc<MockCommentRepository>, // TODO: Replace with dyn CommentRepository
    
    /// AI moderation service for content filtering
    /// LEARNING: Dependency injection pattern for testability
    moderation_service: Option<Arc<ModerationService>>, // Optional for now

    /// Sentiment analysis service (optional, mirrors posts)
    sentiment_service: Option<Arc<SentimentService>>,

    /// User service to fetch public author data
    user_service: Option<Arc<UserService>>,
}

impl CommentService {
    /// Create a new CommentService instance
    /// 
    /// LEARNING CONCEPTS:
    /// - Constructor patterns in Rust
    /// - Arc for shared ownership
    /// - Dependency injection for testability
    pub fn new(comment_repo: Arc<MockCommentRepository>) -> Self {
        Self { 
            comment_repo,
            moderation_service: None, // TODO: Inject moderation service
            sentiment_service: None,
            user_service: None,
        }
    }
    
    /// Create a new comment with full processing pipeline
    /// 
    /// LEARNING EXERCISE: Complete comment creation workflow
    /// YOUR TASK: Implement the full comment creation process
    /// 
    /// STEPS REQUIRED:
    /// 1. Validate the input request
    /// 2. Check if parent comment exists (for replies)
    /// 3. Generate materialized path and depth  
    /// 4. Run sentiment analysis
    /// 5. Run content moderation
    /// 6. Save to database
    /// 7. Return formatted response
    pub async fn create_comment(
        &self, 
        post_id: Uuid,
        request: CreateCommentRequest, 
        user_id: Uuid
    ) -> Result<CommentResponse> {
        // TODO: YOUR IMPLEMENTATION HERE
        // 
        // LEARNING GOALS:
        // - Understanding async/await patterns
        // - Error handling with ? operator
        // - Working with Option<T> for parent comments
        // - String manipulation for path generation
        // - Integration with external services (AI)
        
        // Step 1: Validate the request
        request.validate().map_err(|msg| AppError::ValidationError(msg))?;
        
        // Step 2: For now, create a basic comment structure
        // TODO: Implement full workflow with parent checking, path generation, AI analysis
        let (parent_path, depth, sibling_count) = if let Some(parent_id) = request.parent_id {
            // Fetch parent comment to get its path
            let parent = self.comment_repo.get_by_parent_id(parent_id).await
                .map_err(|e| AppError::InternalError(format!("Parent look up failed: {}", e)))?
                .ok_or_else(|| AppError::ValidationError("Parent comment not found".to_string()))?;

            // Count existing siblings to determine next path segment
            let siblings = self.comment_repo.count_siblings(parent_id).await
                .map_err(|e| AppError::InternalError(format!("Sibling count failed: {}", e)))?;

            (Some(parent.path.clone()),parent.depth + 1, siblings)
        } else {
            // Root comment: number of existing root comments for this post
            let root_count = self
                .comment_repo.
                count_siblings(post_id).
                await.map_err(|e| AppError::InternalError(format!("Root count failed: {}", e)))?;
            (None, 0, root_count)
        };

        // Materialized path
        let path = self.generate_thread_path(parent_path.as_deref(), sibling_count);

        // Run moderation if service is available. If blocked, surface a ContentModerationError
        let (moderation_result, is_flagged) = if let Some(moderation_service) = &self
            .moderation_service {
            let mod_result = moderation_service
                .check_content(&request.content)
                .await
                .map_err(|e| AppError::InternalError(format!("Moderation failed: {}", e)))?;
            if mod_result.is_blocked {
                return Err(AppError::ContentModerationError(
                    mod_result
                        .reason
                        .clone()
                        .unwrap_or_else(|| "Content blocked by moderation policy".to_string()),
                ));
            }
            (Some(mod_result), false)
        } else {
            (None, false)
        };

        // run sentiment analysis

        let sentiment_analysis = if let Some(sentiment_service) = &self.sentiment_service {
            let sentiments = sentiment_service.analyze_sentiment(&request.content).await
                .map_err(|e| AppError::InternalError(format!("Sentiment analysis failed: {}", e)))?;
            Some(sentiments)
        } else {
            None
        };

        // Construct the Comment object
        let comment = Comment {
            id: Uuid::new_v4(),
            post_id,
            user_id,
            parent_id: request.parent_id,
            content: request.content,
            path,
            depth,
            sentiment_analysis,
            moderation_result,
            is_flagged,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            reply_count: 0,
        };

        // Persist
        self.comment_repo
            .create_comment(comment.clone())
            .await
            .map_err(|e| AppError::InternalError(format!("Create comment failed: {e}")))?;
        // Let's get author
        let author = if let Some(user_service) = &self.user_service {
            match user_service.get_public_user(user_id).await {
                Ok(a) => Some(a),
                Err(e) => {
                    tracing::warn!("Author lookup failed (non-fatal): {e}");
                    None
                }
            }
        } else {
            None
        };
        Ok(CommentResponse {
            comment,
            author,
            replies: vec![],
            can_modify: true,
            is_collapsed: false,
        })
    }

    /// Get comments for a post with Reddit-style tree structure
    /// 
    /// LEARNING EXERCISE: Tree building and hierarchical data processing
    /// YOUR TASK: Build nested comment trees for frontend display
    pub async fn get_comments_for_post(&self, post_id: Uuid) -> Result<Vec<CommentResponse>> {
        // Fetch all comments for the post
        let mut comments = self.comment_repo
            .get_comments_by_post(post_id)
            .await
            .map_err(|e| AppError::InternalError(format!("Fetch comments failed: {}", e)))?;

        // Sort by materialized path to ensure correct order
        fn path_key(path: &str) -> Vec<u32> {
            path.trim_end_matches('/').split('/')
                .filter_map(|s| s.parse::<u32>().ok())
                .collect()
        }
        comments.sort_by_key(|c| path_key(&c.path));

        // Build parent -> children adjacency map
        let mut by_id: HashMap<Uuid, Comment> = HashMap::with_capacity(comments.len());
        let mut children: Hashmap<Option<Uuid>, Vec<Uuis>> = HashMap::new();
        for c in comments {
            let pid = c.parent_id;
            children.entry(pid).or_default().push(c.id);
            by_id.insert(c.id, c);
        }

        // Sort each siblings list by path
        for (_k, list) in children.iter_mut() {
            list.sort_by(|a, b| {
                let path_a = &by_id.get(a).unwrap().path;
                let path_b = &by_id.get(b).unwrap().path;
                path_key(path_a).cmp(&path_key(path_b))
            })
        }

        const MAX_DEPTH: u32 = 3;

        fn build_tree(
            id: Uuid,
            by_id: &HashMap<Uuid, Comment>,
            children: &HashMap<Option<Uuid>, Vec<Uuid>>,
        ) -> CommentResponse {
            let comment = by_id.get(&id).unwrap().clone();
            let depth = comment.depth;

            let mut replies = Vec::new();
            if depth < MAX_VISIBLE_DEPTH {
                if let Some(child_ids) = children.get(&Some(id)) {
                    replies = child_ids
                        .iter()
                        .map(|cid| build_tree(*cid, by_id, children))
                        .collect();
                }
            }

            let is_collapsed = depth >= MAX_VISIBLE_DEPTH
                && children
                .get(&Some(id))
                .map(|v| !v.is_empty())
                .unwrap_or(false);

            CommentResponse {
                comment,
                author: None,          // Lazy-load or enrich later
                replies,
                can_modify: false,     // Filled in higher layer (auth context)
                is_collapsed,
            }
        }

        // 5. Roots are entries whose parent_id = None
        let mut root_ids = children.get(&None).cloned().unwrap_or_default();
        root_ids.sort_by(|a, b| {
            let pa = &by_id.get(a).unwrap().path;
            let pb = &by_id.get(b).unwrap().path;
            path_key(pa).cmp(&path_key(pb))
        });

        let roots = root_ids
            .into_iter()
            .map(|rid| build_tree(rid, &by_id, &children))
            .collect();

        Ok(roots)
    }

    /// Generate materialized path for hierarchical comments
    /// 
    /// LEARNING EXERCISE: Path generation for hierarchical data  
    /// YOUR TASK: Implement this method to generate the next sequential path
    /// 
    /// HOW IT WORKS:
    /// - Root comment: "1/", "2/", "3/" etc.
    /// - Replies: "1/1/", "1/2/", "2/1/" etc. 
    /// - Deep replies: "1/2/3/", "1/2/4/" etc.
    fn generate_thread_path(&self, parent_path: Option<&str>, sibling_count: u32) -> String {
        let next = sibling_count + 1;
        match parent_path {
            None => format!("{next}/"),
            Some(p) => {
                let mut path = String::with_capacity(p.len() + 12);
                path.push_str(p);
                if !p.ends_with('/') {
                    path.push('/');
                }
                path.push_str(&next.to_string());
                path.push('/');
                path
            }
        }
    }

    /// Build a hierarchical tree from flat comment list
    /// 
    /// LEARNING EXERCISE: Tree construction algorithms
    /// YOUR TASK: Convert flat database results into nested trees
    fn build_comment_tree(&self, comments: Vec<Comment>) -> Vec<CommentResponse> {
        // Build hierarchical comment tree (O(n))
        fn path_key(path: &str) -> Vec<u32> {
            path.trim_end_matches('/')
                .split('/')
                .filter_map(|s| s.parse::<u32>().ok())
                .collect()
        }

        // 1. Index comments by id and 2. collect children per parent
        let mut by_id: HashMap<Uuid, Comment> = HashMap::new();
        let mut children_map: HashMap<Option<Uuid>, Vec<Uuid>> = HashMap::new();
        for c in comments {
            let pid = c.parent_id;
            children_map.entry(pid).or_default().push(c.id);
            by_id.insert(c.id, c);
        }

        // 3. Sort sibling lists deterministically by materialized path order
        for list in children_map.values_mut() {
            list.sort_by(|a, b| {
                let pa = &by_id.get(a).unwrap().path;
                let pb = &by_id.get(b).unwrap().path;
                path_key(pa).cmp(&path_key(pb))
            });
        }

        // Depth at which we auto-collapse (can be tuned)
        const AUTO_COLLAPSE_DEPTH: u32 = 6;

        // 4 & 5. Recursive construction
        fn build_node(
            id: Uuid,
            by_id: &HashMap<Uuid, Comment>,
            children: &HashMap<Option<Uuid>, Vec<Uuid>>,
        ) -> CommentResponse {
            let comment = by_id.get(&id).unwrap().clone();
            let depth = comment.depth;

            let replies = children
                .get(&Some(id))
                .map(|child_ids| {
                    child_ids
                        .iter()
                        .map(|cid| build_node(*cid, by_id, children))
                        .collect()
                })
                .unwrap_or_else(Vec::new);

            let is_collapsed = depth >= AUTO_COLLAPSE_DEPTH
                && children
                .get(&Some(id))
                .map(|v| !v.is_empty())
                .unwrap_or(false);

            CommentResponse {
                comment,
                author: None,
                replies,
                can_modify: false,
                is_collapsed,
            }
        }

        // Roots = parent_id == None
        let mut root_ids = children_map.get(&None).cloned().unwrap_or_default();
        root_ids.sort_by(|a, b| {
            let pa = &by_id.get(a).unwrap().path;
            let pb = &by_id.get(b).unwrap().path;
            path_key(pa).cmp(&path_key(pb))
        });

        root_ids
            .into_iter()
            .map(|rid| build_node(rid, &by_id, &children_map))
            .collect()
    }
    
    /// Get a specific comment thread (for deep-linking)
    /// 
    /// LEARNING EXERCISE: Path-based queries and thread focusing
    /// YOUR TASK: Implement Reddit-style thread focusing
    pub async fn get_comment_thread(&self, comment_id: Uuid) -> Result<Vec<CommentResponse>> {
        // Focused thread view (parent -> target -> target's direct children)
        let target = self.comment_repo
            .get_comment_by_id(comment_id)
            .await
            .map_err(|e| AppError::InternalError(format!("Fetch target comment failed: {e}")))?
            .ok_or_else(|| AppError::ValidationError("Comment not found".to_string()))?;

        // Fetch all comments for the post (we will slice out only what we need)
        let mut all = self.comment_repo
            .get_comments_by_post(target.post_id)
            .await
            .map_err(|e| AppError::InternalError(format!("Fetch post comments failed: {e}")))?;

        // Index by id + children adjacency
        let mut by_id: HashMap<Uuid, Comment> = HashMap::with_capacity(all.len());
        let mut children: HashMap<Option<Uuid>, Vec<Uuid>> = HashMap::new();
        for c in all.drain(..) {
            children.entry(c.parent_id).or_default().push(c.id);
            by_id.insert(c.id, c);
        }

        // Sort sibling lists deterministically by materialized path
        fn path_key(p: &str) -> Vec<u32> {
            p.trim_end_matches('/')
                .split('/')
                .filter_map(|s| s.parse::<u32>().ok())
                .collect()
        }
        for ids in children.values_mut() {
            ids.sort_by(|a, b| {
                let pa = &by_id.get(a).unwrap().path;
                let pb = &by_id.get(b).unwrap().path;
                path_key(pa).cmp(&path_key(pb))
            });
        }

        let target_path = target.path.clone();
        let target_id = target.id;
        let parent_id = target.parent_id;

        // Helper: build minimal chain from parent -> ... -> target
        fn build_chain(
            current_id: Uuid,
            target_id: Uuid,
            target_path: &str,
            by_id: &HashMap<Uuid, Comment>,
            children: &HashMap<Option<Uuid>, Vec<Uuid>>,
        ) -> CommentResponse {
            let comment = by_id.get(&current_id).unwrap().clone();
            // If this node is the target, do not traverse deeper here (children handled separately)
            if current_id == target_id {
                return CommentResponse {
                    comment,
                    author: None,
                    replies: vec![],
                    can_modify: false,
                    is_collapsed: false,
                };
            }

            // Narrow next child along the path to target
            let mut replies = Vec::new();
            if let Some(kids) = children.get(&Some(current_id)) {
                if let Some(next_id) = kids.iter().find(|cid| {
                    let child = by_id.get(cid).unwrap();
                    target_path.starts_with(&child.path)
                }) {
                    replies.push(build_chain(*next_id, target_id, target_path, by_id, children));
                }
            }

            CommentResponse {
                comment,
                author: None,
                replies,
                can_modify: false,
                is_collapsed: false,
            }
        }

        // Build base (either target alone, or parent->...->target chain)
        let mut roots: Vec<CommentResponse> = if let Some(pid) = parent_id {
            if by_id.get(&pid).is_some() {
                vec![build_chain(pid, target_id, &target_path, &by_id, &children)]
            } else {
                vec![] // Parent missing (data inconsistency) -> fall back to target only below
            }
        } else {
            vec![]
        };

        // Locate mutable reference to the target node inside roots (or create root if no parent)
        // Strategy: if parent exists we descend the single-child chain to find target.
        let mut target_node_opt: Option<*mut CommentResponse> = None;
        if roots.is_empty() {
            // Root comment is the target
            roots.push(CommentResponse {
                comment: target.clone(),
                author: None,
                replies: vec![],
                can_modify: false,
                is_collapsed: false,
            });
            target_node_opt = Some(&mut roots[0] as *mut _);
        } else {
            // Descend through replies until we reach target
            let mut ptr: *mut CommentResponse = &mut roots[0];
            unsafe {
                loop {
                    if (*ptr).comment.id == target_id {
                        target_node_opt = Some(ptr);
                        break;
                    }
                    if (*ptr).replies.len() == 1 {
                        let next_ptr: *mut CommentResponse = &mut (*ptr).replies[0];
                        ptr = next_ptr;
                    } else {
                        break; // Should not happen in focused chain
                    }
                }
            }
        }

        // Attach direct children (one extra level). Mark them collapsed if they have deeper descendants.
        if let Some(raw_target_ptr) = target_node_opt {
            unsafe {
                if let Some(child_ids) = children.get(&Some(target_id)) {
                    for cid in child_ids {
                        if let Some(child) = by_id.get(cid) {
                            let has_grandchildren = children
                                .get(&Some(*cid))
                                .map(|v| !v.is_empty())
                                .unwrap_or(false);
                            (*raw_target_ptr).replies.push(CommentResponse {
                                comment: child.clone(),
                                author: None,
                                replies: vec![], // Not expanding deeper
                                can_modify: false,
                                is_collapsed: has_grandchildren, // Indicate there is more to load
                            });
                        }
                    }
                }
            }
        }

        // If no parent and no explicit root (edge case), ensure target is present
        if roots.is_empty() {
            roots.push(CommentResponse {
                comment: target,
                author: None,
                replies: vec![],
                can_modify: false,
                is_collapsed: false,
            });
        }

        Ok(roots)
    }
    
    /// Update a comment with new content
    /// 
    /// LEARNING EXERCISE: Update operations with validation
    /// YOUR TASK: Implement safe comment updating
    pub async fn update_comment(&self, id: Uuid, content: String, user_id: Uuid) -> Result<CommentResponse> {
        // Update a comment\'s content with validation, auth, moderation, and sentiment re-processing
        let mut comment = self.comment_repo
            .get_comment_by_id(id)
            .await
            .map_err(|e| AppError::InternalError(format!("Fetch comment failed: {e}")))?
            .ok_or_else(|| AppError::ValidationError("Comment not found".to_string()))?;

        // 2. Permission check
        if comment.user_id != user_id {
            return Err(AppError::ValidationError("You cannot modify this comment".to_string()));
        }

        // 3. Validate new content (basic inline validation; could delegate to a shared validator)
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(AppError::ValidationError("Content cannot be empty".to_string()));
        }
        if trimmed.len() > 10_000 {
            return Err(AppError::ValidationError("Content exceeds 10k character limit".to_string()));
        }

        // 4. Re-run moderation (block if policy violation)
        let (moderation_result, is_flagged) = if let Some(moderation_service) = &self.moderation_service {
            let mod_result = moderation_service
                .check_content(trimmed)
                .await
                .map_err(|e| AppError::InternalError(format!("Moderation failed: {e}")))?;
            if mod_result.is_blocked {
                return Err(AppError::ContentModerationError(
                    mod_result.reason.clone().unwrap_or_else(|| "Content blocked by moderation policy".to_string()),
                ));
            }
            (Some(mod_result), false)
        } else {
            (comment.moderation_result.clone(), comment.is_flagged)
        };

        // 4b. Re-run sentiment (optional)
        let sentiment_analysis = if let Some(sentiment_service) = &self.sentiment_service {
            Some(
                sentiment_service
                    .analyze_sentiment(trimmed)
                    .await
                    .map_err(|e| AppError::InternalError(format!("Sentiment analysis failed: {e}")))?
            )
        } else {
            comment.sentiment_analysis.clone()
        };

        // 5. Apply updates
        comment.content = trimmed.to_string();
        comment.updated_at = Utc::now();
        comment.moderation_result = moderation_result;
        comment.is_flagged = is_flagged;
        comment.sentiment_analysis = sentiment_analysis;

        self.comment_repo
            .update_comment(comment.clone())
            .await
            .map_err(|e| AppError::InternalError(format!("Update failed: {e}")))?;

        // 6. Enrich with author (non-fatal on failure)
        let author = if let Some(user_service) = &self.user_service {
            match user_service.get_public_user(user_id).await {
                Ok(a) => Some(a),
                Err(e) => {
                    tracing::warn!("Author lookup failed (non-fatal): {e}");
                    None
                }
            }
        } else {
            None
        };

        Ok(CommentResponse {
            comment,
            author,
            replies: vec![],       // Not loading descendants in an update call
            can_modify: true,
            is_collapsed: false,
        })
    }
    
    /// Delete a comment and handle thread cleanup
    /// 
    /// LEARNING EXERCISE: Cascading operations and data consistency
    /// YOUR TASK: Implement safe comment deletion
    pub async fn delete_comment(&self, id: Uuid, user_id: Uuid) -> Result<()> {
        // Implementation: hybrid soft / hard delete with hierarchical count maintenance.
        // 1. Fetch comment
        let comment = self.comment_repo
            .get_comment_by_id(id)
            .await
            .map_err(|e| AppError::InternalError(format!("Fetch comment failed: {e}")))?
            .ok_or_else(|| AppError::ValidationError("Comment not found".to_string()))?;

        // 2. Permission (only author for now; extend later for moderators/admins)
        if comment.user_id != user_id {
            return Err(AppError::ValidationError("You cannot delete this comment".to_string()));
        }

        let post_id = comment.post_id;
        let parent_id = comment.parent_id;

        // 3. Strategy:
        //    - If comment has replies (reply_count > 0) -> soft delete (preserve thread & counts).
        //    - Else -> hard delete (remove row + decrement parent + decrement post aggregate count).
        // NOTE: In production this should ideally be wrapped in a DB transaction to keep counts consistent.
        if comment.reply_count > 0 {
            // Soft delete (id preserved to keep subtree reachable)
            if comment.content == "[deleted]" {
                return Ok(()); // already soft-deleted
            }
            let mut soft = comment.clone();
            soft.content = "[deleted]".to_string();
            soft.updated_at = Utc::now();
            self.comment_repo
                .update_comment(soft)
                .await
                .map_err(|e| AppError::InternalError(format!("Soft delete failed: {e}")))?;
            // Counts intentionally NOT decremented (thread still logically occupies a slot)
        } else {
            // Hard delete (no children -> safe to remove)
            self.comment_repo
                .delete_comment(id)
                .await
                .map_err(|e| AppError::InternalError(format!("Hard delete failed: {e}")))?;

            // 3a. Decrement parent reply_count if there is a parent
            if let Some(pid) = parent_id {
                match self.comment_repo.get_comment_by_id(pid).await {
                    Ok(Some(mut parent)) => {
                        if parent.reply_count > 0 {
                            parent.reply_count -= 1;
                            parent.updated_at = Utc::now();
                            if let Err(e) = self.comment_repo.update_comment(parent).await {
                                tracing::warn!("Parent reply_count decrement failed (non-fatal): {e}");
                            }
                        }
                    }
                    Ok(None) => {
                        tracing::warn!("Parent comment {pid} not found while decrementing reply_count");
                    }
                    Err(e) => {
                        tracing::warn!("Parent lookup failed while decrementing reply_count: {e}");
                    }
                }
            }

            // 3b. Decrement post-level aggregate comment count
            // EXPECTED REPO API (implement in repository):
            //   async fn decrement_post_comment_count(&self, post_id: Uuid) -> anyhow::Result<()>;
            if let Err(e) = self.comment_repo.decrement_post_comment_count(post_id).await {
                tracing::warn!(
                    "Post comment count decrement failed for post {post_id} (non-fatal): {e}"
                );
            }
        }

        Ok(())
    }

    // ========== MIGRATION METHODS ==========
    
    /// Check if comment migration is needed (placeholder for future comment sentiment migration)
    pub async fn is_migration_needed(&self) -> Result<bool> {
        // Check if any comment uses a legacy / incompatible sentiment schema.
        // Currently comments were introduced after latest sentiment version, so we expect none.
        // Allow forcing a migration run via env var for future backfills.
        if std::env::var("FORCE_COMMENT_SENTIMENT_MIGRATION").is_ok() {
            tracing::info!("🔍 MIGRATION: Forced via env var");
            return Ok(true);
        }
        tracing::info!("✅ MIGRATION: No incompatible comment sentiment versions detected");
        Ok(false)
    }
    
    /// Run comment migration (placeholder for future implementation)
    pub async fn run_emotion_migration(&self) -> Result<CommentMigrationResult> {
        tracing::info!("🚀 MIGRATION: Starting comment emotion migration");

        // Require sentiment service
        let Some(sentiment_service) = self.sentiment_service.clone() else {
            tracing::warn!("⚠️ MIGRATION: Sentiment service not configured; skipping.");
            return Ok(CommentMigrationResult {
                total_comments_checked: 0,
                comments_requiring_migration: 0,
                comments_successfully_migrated: 0,
                comments_failed_migration: 0,
                errors: Vec::new(),
            });
        };

        // Fetch all comments. NOTE: Ensure repository provides get_all_comments()
        let all = self.comment_repo
            .get_all_comments()
            .await
            .map_err(|e| AppError::InternalError(format!("Migration fetch failed: {e}")))?;

        let total = all.len();
        let mut comments_requiring_migration = 0usize;
        let mut comments_successfully_migrated = 0usize;
        let mut comments_failed_migration = 0usize;
        let mut errors = Vec::new();

        for mut c in all {
            // Condition: no sentiment_analysis present
            if c.sentiment_analysis.is_none() {
                comments_requiring_migration += 1;
                match sentiment_service.analyze_sentiment(&c.content).await {
                    Ok(sa) => {
                        c.sentiment_analysis = Some(sa);
                        c.updated_at = Utc::now();
                        if let Err(e) = self.comment_repo.update_comment(c.clone()).await {
                            comments_failed_migration += 1;
                            errors.push(format!("Persist failure for {}: {e}", c.id));
                        } else {
                            comments_successfully_migrated += 1;
                        }
                    }
                    Err(e) => {
                        comments_failed_migration += 1;
                        errors.push(format!("Sentiment failure for {}: {e}", c.id));
                    }
                }
            }
        }

        tracing::info!(
            "✅ MIGRATION: checked={}, needed={}, migrated={}, failed={}",
            total,
            comments_requiring_migration,
            comments_successfully_migrated,
            comments_failed_migration
        );

        Ok(CommentMigrationResult {
            total_comments_checked: total,
            comments_requiring_migration,
            comments_successfully_migrated,
            comments_failed_migration,
            errors,
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