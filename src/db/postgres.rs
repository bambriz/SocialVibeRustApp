// PostgreSQL repository implementations using sqlx
use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgPoolOptions};
use uuid::Uuid;
use std::sync::Arc;
use std::time::Duration;
use crate::{Result, AppError};
use crate::models::{User, Post, Comment, Vote, TagVoteCount, VoteSummary};
use crate::db::repository::{UserRepository, PostRepository, CommentRepository, VoteRepository};

// PostgreSQL connection pool wrapper
pub struct PostgresDatabase {
    pub pool: Arc<PgPool>,
}

impl PostgresDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        tracing::info!("🔗 DATABASE_RESILIENCE: Configuring connection pool");
        tracing::info!("   📊 Max connections: 20");
        tracing::info!("   ⏰ Connection timeout: 30s");
        tracing::info!("   ⏳ Idle timeout: 10m");
        
        let pool = PgPoolOptions::new()
            .max_connections(20)  // Configure maximum concurrent connections
            .idle_timeout(Duration::from_secs(600)) // 10 minutes idle timeout  
            .max_lifetime(Duration::from_secs(3600)) // 1 hour max connection lifetime
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await
            .map_err(|e| {
                tracing::error!("❌ DATABASE_RESILIENCE: Failed to create connection pool: {}", e);
                AppError::DatabaseError(format!("Failed to connect to PostgreSQL: {}", e))
            })?;
        
        tracing::info!("✅ DATABASE_RESILIENCE: Connection pool configured successfully");
        
        Ok(Self {
            pool: Arc::new(pool),
        })
    }
    
    pub fn user_repo(&self) -> PostgresUserRepository {
        PostgresUserRepository { pool: self.pool.clone() }
    }
    
    pub fn post_repo(&self) -> PostgresPostRepository {
        PostgresPostRepository { pool: self.pool.clone() }
    }
    
    pub fn comment_repo(&self) -> PostgresCommentRepository {
        PostgresCommentRepository { pool: self.pool.clone() }
    }
    
    pub fn vote_repo(&self) -> PostgresVoteRepository {
        PostgresVoteRepository { pool: self.pool.clone() }
    }
}

// PostgreSQL User Repository
pub struct PostgresUserRepository {
    pool: Arc<PgPool>,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create_user(&self, user: &User) -> Result<User> {
        let row = sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash, display_name, bio, avatar_url, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, username, email, password_hash, display_name, bio, avatar_url, created_at, updated_at, is_active
            "#,
            user.id,
            user.username,
            user.email,
            user.password_hash,
            user.display_name,
            user.bio,
            user.avatar_url,
            user.is_active
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create user: {}", e)))?;
        
        Ok(User {
            id: row.id,
            username: row.username,
            email: row.email,
            password_hash: row.password_hash,
            display_name: row.display_name,
            bio: row.bio,
            avatar_url: row.avatar_url,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_active: row.is_active.unwrap_or(true),
        })
    }
    
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query!(
            r#"
            SELECT id, username, email, password_hash, display_name, bio, avatar_url, created_at, updated_at, is_active
            FROM users WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get user by id: {}", e)))?;
        
        Ok(row.map(|r| User {
            id: r.id,
            username: r.username,
            email: r.email,
            password_hash: r.password_hash,
            display_name: r.display_name,
            bio: r.bio,
            avatar_url: r.avatar_url,
            created_at: r.created_at,
            updated_at: r.updated_at,
            is_active: r.is_active.unwrap_or(true),
        }))
    }
    
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query!(
            r#"
            SELECT id, username, email, password_hash, display_name, bio, avatar_url, created_at, updated_at, is_active
            FROM users WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get user by email: {}", e)))?;
        
        Ok(row.map(|r| User {
            id: r.id,
            username: r.username,
            email: r.email,
            password_hash: r.password_hash,
            display_name: r.display_name,
            bio: r.bio,
            avatar_url: r.avatar_url,
            created_at: r.created_at,
            updated_at: r.updated_at,
            is_active: r.is_active.unwrap_or(true),
        }))
    }
    
    async fn update_user(&self, user: &User) -> Result<User> {
        let row = sqlx::query!(
            r#"
            UPDATE users 
            SET username = $2, email = $3, password_hash = $4, display_name = $5, bio = $6, avatar_url = $7, is_active = $8, updated_at = NOW()
            WHERE id = $1
            RETURNING id, username, email, password_hash, display_name, bio, avatar_url, created_at, updated_at, is_active
            "#,
            user.id,
            user.username,
            user.email,
            user.password_hash,
            user.display_name,
            user.bio,
            user.avatar_url,
            user.is_active
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update user: {}", e)))?;
        
        Ok(User {
            id: row.id,
            username: row.username,
            email: row.email,
            password_hash: row.password_hash,
            display_name: row.display_name,
            bio: row.bio,
            avatar_url: row.avatar_url,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_active: row.is_active.unwrap_or(true),
        })
    }
    
    async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(&*self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete user: {}", e)))?;
        
        Ok(())
    }
}

// PostgreSQL Post Repository
pub struct PostgresPostRepository {
    pool: Arc<PgPool>,
}

impl PostgresPostRepository {
    // Helper function to parse sentiment_analysis JSON into Post fields
    fn parse_sentiment_json(value: &Option<serde_json::Value>) -> (Option<f64>, Vec<String>, Option<String>) {
        if let Some(json) = value {
            let sentiment_score = json.get("sentiment_score").and_then(|v| v.as_f64());
            let sentiment_colors = json.get("sentiment_colors")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let sentiment_type = json.get("sentiment_type").and_then(|v| v.as_str().map(String::from));
            (sentiment_score, sentiment_colors, sentiment_type)
        } else {
            (None, vec![], None)
        }
    }

    // Helper function to parse moderation_result JSON into Post fields
    fn parse_moderation_json(value: &Option<serde_json::Value>) -> (Vec<String>, Option<serde_json::Value>) {
        if let Some(json) = value {
            let toxicity_tags = json.get("toxicity_tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let toxicity_scores = json.get("toxicity_scores").cloned();
            (toxicity_tags, toxicity_scores)
        } else {
            (vec![], None)
        }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create_post(&self, post: &Post) -> Result<Post> {
        let row = sqlx::query!(
            r#"
            INSERT INTO posts (id, user_id, content, title, sentiment_analysis, moderation_result, is_flagged, view_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, content, title, sentiment_analysis, moderation_result, is_flagged, 
                      created_at, updated_at, view_count
            "#,
            post.id,
            post.author_id,
            post.content,
            post.title,
            serde_json::to_value(serde_json::json!({
                "sentiment_score": post.sentiment_score,
                "sentiment_colors": post.sentiment_colors,
                "sentiment_type": post.sentiment_type
            })).ok(),
            serde_json::to_value(serde_json::json!({
                "toxicity_tags": post.toxicity_tags,
                "toxicity_scores": post.toxicity_scores,
                "is_blocked": post.is_blocked
            })).ok(),
            post.is_blocked,
            0i32
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to create post: {}", e)))?;
        
        Ok(Post {
            id: row.id,
            title: post.title.clone(),
            content: post.content.clone(),
            author_id: row.user_id,
            author_username: post.author_username.clone(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            comment_count: 0, // Will be calculated separately
            sentiment_score: post.sentiment_score,
            sentiment_colors: post.sentiment_colors.clone(),
            sentiment_type: post.sentiment_type.clone(),
            popularity_score: post.popularity_score,
            is_blocked: row.is_flagged.unwrap_or(false),
            toxicity_tags: post.toxicity_tags.clone(),
            toxicity_scores: post.toxicity_scores.clone(),
        })
    }
    
    async fn get_post_by_id(&self, id: Uuid) -> Result<Option<Post>> {
        let row = sqlx::query!(
            r#"
            SELECT p.id, p.user_id, p.content, p.title, p.sentiment_analysis, p.moderation_result, 
                   p.is_flagged, p.created_at, p.updated_at, p.view_count, u.username
            FROM posts p
            JOIN users u ON p.user_id = u.id
            WHERE p.id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get post: {}", e)))?;
        
        Ok(row.map(|r| {
            let (sentiment_score, sentiment_colors, sentiment_type) = Self::parse_sentiment_json(&r.sentiment_analysis);
            let (toxicity_tags, toxicity_scores) = Self::parse_moderation_json(&r.moderation_result);
            
            Post {
                id: r.id,
                title: r.title.unwrap_or_default(),
                content: r.content,
                author_id: r.user_id,
                author_username: r.username,
                created_at: r.created_at,
                updated_at: r.updated_at,
                comment_count: 0, // Will be calculated separately // Will be calculated separately
                sentiment_score,
                sentiment_colors,
                sentiment_type,
                popularity_score: 1.0,
                is_blocked: r.is_flagged.unwrap_or(false),
                toxicity_tags,
                toxicity_scores,
            }
        }))
    }
    
    async fn get_posts_paginated(&self, limit: u32, offset: u32) -> Result<Vec<Post>> {
        let rows = sqlx::query!(
            r#"
            SELECT p.id, p.user_id, p.content, p.title, p.sentiment_analysis, p.moderation_result, 
                   p.is_flagged, p.created_at, p.updated_at, p.view_count, u.username,
                   (SELECT COUNT(*) FROM comments c WHERE c.post_id = p.id AND c.depth = 0) as root_comment_count
            FROM posts p
            JOIN users u ON p.user_id = u.id
            ORDER BY p.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get posts: {}", e)))?;
        
        Ok(rows.into_iter().map(|r| {
            let (sentiment_score, sentiment_colors, sentiment_type) = Self::parse_sentiment_json(&r.sentiment_analysis);
            let (toxicity_tags, toxicity_scores) = Self::parse_moderation_json(&r.moderation_result);
            
            Post {
                id: r.id,
                title: r.title.unwrap_or_default(),
                content: r.content,
                author_id: r.user_id,
                author_username: r.username,
                created_at: r.created_at,
                updated_at: r.updated_at,
                comment_count: 0, // Will be calculated separately
                sentiment_score,
                sentiment_colors,
                sentiment_type,
                popularity_score: 1.0,
                is_blocked: r.is_flagged.unwrap_or(false),
                toxicity_tags,
                toxicity_scores,
            }
        }).collect())
    }
    
    async fn get_posts_by_popularity(&self, limit: u32, offset: u32) -> Result<Vec<Post>> {
        // For now, same as paginated until we add popularity scoring
        self.get_posts_paginated(limit, offset).await
    }
    
    async fn get_posts_by_user(&self, user_id: Uuid, limit: u32, offset: u32) -> Result<Vec<Post>> {
        let rows = sqlx::query!(
            r#"
            SELECT p.id, p.user_id, p.content, p.title, p.sentiment_analysis, p.moderation_result, 
                   p.is_flagged, p.created_at, p.updated_at, p.view_count, u.username
            FROM posts p
            JOIN users u ON p.user_id = u.id
            WHERE p.user_id = $1
            ORDER BY p.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit as i64,
            offset as i64
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get user posts: {}", e)))?;
        
        Ok(rows.into_iter().map(|r| {
            let (sentiment_score, sentiment_colors, sentiment_type) = Self::parse_sentiment_json(&r.sentiment_analysis);
            let (toxicity_tags, toxicity_scores) = Self::parse_moderation_json(&r.moderation_result);
            
            Post {
                id: r.id,
                title: r.title.unwrap_or_default(),
                content: r.content,
                author_id: r.user_id,
                author_username: r.username,
                created_at: r.created_at,
                updated_at: r.updated_at,
                comment_count: 0, // Will be calculated separately
                sentiment_score,
                sentiment_colors,
                sentiment_type,
                popularity_score: 1.0,
                is_blocked: r.is_flagged.unwrap_or(false),
                toxicity_tags,
                toxicity_scores,
            }
        }).collect())
    }
    
    async fn update_post(&self, post: &Post) -> Result<Post> {
        let row = sqlx::query!(
            r#"
            UPDATE posts 
            SET content = $2, title = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, user_id, content, title, sentiment_analysis, moderation_result, is_flagged, 
                      created_at, updated_at, view_count
            "#,
            post.id,
            post.content,
            post.title
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to update post: {}", e)))?;
        
        Ok(Post {
            id: row.id,
            title: row.title.unwrap_or_default(),
            content: row.content,
            author_id: row.user_id,
            author_username: post.author_username.clone(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            comment_count: post.comment_count,
            sentiment_score: post.sentiment_score,
            sentiment_colors: post.sentiment_colors.clone(),
            sentiment_type: post.sentiment_type.clone(),
            popularity_score: post.popularity_score,
            is_blocked: row.is_flagged.unwrap_or(false),
            toxicity_tags: post.toxicity_tags.clone(),
            toxicity_scores: post.toxicity_scores.clone(),
        })
    }
    
    async fn delete_post(&self, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM posts WHERE id = $1", id)
            .execute(&*self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete post: {}", e)))?;
        
        Ok(())
    }
    
    async fn increment_comment_count(&self, _post_id: Uuid) -> Result<()> {
        // Comment count calculated on-demand in get_posts_by_popularity 
        Ok(())
    }
    
    async fn update_popularity_score(&self, _post_id: Uuid, _score: f64) -> Result<()> {
        // Popularity score not stored in database - calculated on demand
        Ok(())
    }
    
    // Migration support methods - minimal implementation
    async fn get_posts_with_old_sentiment_types(&self) -> Result<Vec<Post>> {
        Ok(vec![]) // Not needed for PostgreSQL
    }
    
    async fn update_post_sentiment(&self, _post_id: Uuid, _sentiment_type: Option<String>, _sentiment_colors: Vec<String>, _sentiment_score: Option<f64>) -> Result<()> {
        Ok(()) // Not needed for PostgreSQL
    }
}

// PostgreSQL Comment Repository - simplified implementation
pub struct PostgresCommentRepository {
    pool: Arc<PgPool>,
}

impl PostgresCommentRepository {
    // Helper function to parse sentiment_analysis JSON into Comment fields
    fn parse_sentiment_json(value: &Option<serde_json::Value>) -> (Option<f64>, Vec<String>, Option<String>) {
        if let Some(json) = value {
            let sentiment_score = json.get("sentiment_score").and_then(|v| v.as_f64());
            let sentiment_colors = json.get("sentiment_colors")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let sentiment_type = json.get("sentiment_type").and_then(|v| v.as_str().map(String::from));
            (sentiment_score, sentiment_colors, sentiment_type)
        } else {
            (None, vec![], None)
        }
    }

    // Helper function to parse moderation_result JSON into Comment fields
    fn parse_moderation_json(value: &Option<serde_json::Value>) -> (Vec<String>, Option<serde_json::Value>) {
        if let Some(json) = value {
            let toxicity_tags = json.get("toxicity_tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let toxicity_scores = json.get("toxicity_scores").cloned();
            (toxicity_tags, toxicity_scores)
        } else {
            (vec![], None)
        }
    }
}

#[async_trait]
impl CommentRepository for PostgresCommentRepository {
    // Atomic comment creation with path allocation in single transaction
    async fn create_comment_atomic(&self, post_id: Uuid, parent_id: Option<Uuid>, comment: &Comment) -> Result<Comment> {
        // Start atomic transaction for entire operation
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| crate::AppError::DatabaseError(format!("Failed to start atomic comment transaction: {}", e)))?;
            
        // Compute path and depth within transaction (with locking)
        let (computed_path, computed_depth) = match parent_id {
            None => {
                // Root-level comment: Lock post and use MAX-based calculation
                sqlx::query!(
                    "SELECT id FROM posts WHERE id = $1 FOR UPDATE",
                    post_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| crate::AppError::DatabaseError(format!("Failed to lock post for atomic comment creation: {}", e)))?;
                
                // Use MAX instead of COUNT to handle deleted comments correctly
                let result = sqlx::query!(
                    r#"
                    SELECT COALESCE(MAX(
                        CAST(SPLIT_PART(TRIM(TRAILING '/' FROM path), '/', 1) AS INTEGER)
                    ), 0) as max_index 
                    FROM comments 
                    WHERE post_id = $1 AND parent_id IS NULL
                    "#,
                    post_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| crate::AppError::DatabaseError(format!("Failed to get max root index: {}", e)))?;
                
                let next_index = result.max_index.unwrap_or(0) + 1;
                let path = format!("{:06}/", next_index);
                (path, 0)
            },
            Some(parent_id) => {
                // Reply: Lock parent comment and get path+depth in single query
                let parent_result = sqlx::query!(
                    "SELECT path, depth FROM comments WHERE id = $1 FOR UPDATE",
                    parent_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| crate::AppError::DatabaseError(format!("Failed to lock parent and get metadata: {}", e)))?;
                
                // Use MAX to find highest sibling index under this parent
                let result = sqlx::query!(
                    r#"
                    SELECT COALESCE(MAX(
                        CAST(SPLIT_PART(TRIM(TRAILING '/' FROM SUBSTRING(path FROM LENGTH($2) + 1)), '/', 1) AS INTEGER)
                    ), 0) as max_index
                    FROM comments 
                    WHERE parent_id = $1
                    "#,
                    parent_id,
                    parent_result.path
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| crate::AppError::DatabaseError(format!("Failed to get max reply index: {}", e)))?;
                
                let next_index = result.max_index.unwrap_or(0) + 1;
                let parent_path = parent_result.path;
                let new_path = format!("{}{:06}/", parent_path, next_index);
                let child_depth = parent_result.depth + 1;
                (new_path, child_depth)
            }
        };
        
        // Increment parent reply count if this is a reply (within same transaction)
        if let Some(parent_id) = parent_id {
            sqlx::query!(
                "UPDATE comments SET reply_count = reply_count + 1 WHERE id = $1",
                parent_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::AppError::DatabaseError(format!("Failed to increment parent reply count: {}", e)))?;
        }
        
        // Insert comment with atomically computed path - all within same transaction!
        let row = sqlx::query!(
            r#"
            INSERT INTO comments (id, post_id, user_id, parent_id, content, path, depth, sentiment_analysis, moderation_result, is_flagged, created_at, updated_at, reply_count, popularity_score)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, post_id, user_id, parent_id, content, path, depth, sentiment_analysis, moderation_result, is_flagged, created_at, updated_at, reply_count, popularity_score
            "#,
            comment.id,
            comment.post_id,
            comment.user_id,
            comment.parent_id,
            comment.content,
            computed_path,
            computed_depth,
            // Convert individual sentiment fields to JSON for database storage (same as posts)
            serde_json::to_value(serde_json::json!({
                "sentiment_score": comment.sentiment_score,
                "sentiment_colors": comment.sentiment_colors,
                "sentiment_type": comment.sentiment_type
            })).ok(),
            serde_json::to_value(serde_json::json!({
                "toxicity_tags": comment.toxicity_tags,
                "toxicity_scores": comment.toxicity_scores,
                "is_blocked": comment.is_blocked
            })).ok(),
            comment.is_blocked,
            comment.created_at,
            comment.updated_at,
            comment.reply_count,
            comment.popularity_score
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| crate::AppError::DatabaseError(format!("Failed to insert comment atomically: {}", e)))?;
        
        // Commit entire atomic operation
        tx.commit()
            .await
            .map_err(|e| crate::AppError::DatabaseError(format!("Failed to commit atomic comment creation: {}", e)))?;
        
        // Parse sentiment and moderation JSON to individual fields (same as posts)
        let (sentiment_score, sentiment_colors, sentiment_type) = Self::parse_sentiment_json(&row.sentiment_analysis);
        let (toxicity_tags, toxicity_scores) = Self::parse_moderation_json(&row.moderation_result);
        
        Ok(Comment {
            id: row.id,
            post_id: row.post_id,
            user_id: row.user_id,
            parent_id: row.parent_id,
            content: row.content,
            path: row.path,
            depth: row.depth,
            sentiment_score,
            sentiment_colors,
            sentiment_type,
            is_blocked: row.is_flagged.unwrap_or(false),
            toxicity_tags,
            toxicity_scores,
            created_at: row.created_at,
            updated_at: row.updated_at,
            reply_count: row.reply_count.unwrap_or(0),
            popularity_score: row.popularity_score.unwrap_or(1.0),
        })
    }

    async fn create_comment(&self, comment: &Comment) -> Result<Comment> {
        // Simplified implementation - will be enhanced later
        Ok(comment.clone())
    }
    
    async fn get_comment_by_id(&self, _id: Uuid) -> Result<Option<Comment>> {
        Ok(None) // Simplified - will implement later
    }
    
    async fn get_comments_by_post_id(&self, post_id: Uuid) -> Result<Vec<Comment>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, post_id, user_id, parent_id, content, path, depth, 
                   sentiment_analysis, moderation_result, is_flagged, 
                   created_at, updated_at, reply_count, popularity_score
            FROM comments 
            WHERE post_id = $1 
            ORDER BY path ASC
            "#,
            post_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| crate::AppError::DatabaseError(format!("Failed to get comments for post: {}", e)))?;

        let mut comments = Vec::new();
        for row in rows {
            // Parse sentiment and moderation JSON to individual fields (same as create_comment_atomic)
            let (sentiment_score, sentiment_colors, sentiment_type) = Self::parse_sentiment_json(&row.sentiment_analysis);
            let (toxicity_tags, toxicity_scores) = Self::parse_moderation_json(&row.moderation_result);
            
            comments.push(Comment {
                id: row.id,
                post_id: row.post_id,
                user_id: row.user_id,
                parent_id: row.parent_id,
                content: row.content,
                path: row.path,
                depth: row.depth,
                sentiment_score,
                sentiment_colors,
                sentiment_type,
                is_blocked: row.is_flagged.unwrap_or(false),
                toxicity_tags,
                toxicity_scores,
                created_at: row.created_at,
                updated_at: row.updated_at,
                reply_count: row.reply_count.unwrap_or(0),
                popularity_score: row.popularity_score.unwrap_or(1.0),
            });
        }

        Ok(comments)
    }
    
    async fn get_comments_by_parent_id(&self, _parent_id: Uuid) -> Result<Vec<Comment>> {
        Ok(vec![]) // Simplified - will implement later
    }
    
    async fn update_comment(&self, comment: &Comment) -> Result<Comment> {
        Ok(comment.clone()) // Simplified - will implement later
    }
    
    async fn delete_comment(&self, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM comments WHERE id = $1", id)
            .execute(&*self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to delete comment: {}", e)))?;
        
        Ok(())
    }
    
    async fn get_max_sibling_count(&self, post_id: Uuid, parent_path: Option<&str>) -> Result<u32> {
        let count = if parent_path.is_none() {
            // Root-level: Count top-level comments for this post
            sqlx::query!(
                "SELECT COUNT(*) as count FROM comments WHERE post_id = $1 AND parent_id IS NULL",
                post_id
            )
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| crate::AppError::DatabaseError(format!("Failed to get max sibling count: {}", e)))?
            .count
        } else {
            // For now, simplified - could implement path-based counting later
            sqlx::query!(
                "SELECT COUNT(*) as count FROM comments WHERE post_id = $1",
                post_id
            )
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| crate::AppError::DatabaseError(format!("Failed to get max sibling count: {}", e)))?
            .count
        };
        
        Ok(count.unwrap_or(0) as u32)
    }

    async fn allocate_next_sibling_index(&self, post_id: Uuid, parent_id: Option<Uuid>) -> Result<i32> {
        // Use atomic transaction with locking to prevent race conditions
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| crate::AppError::DatabaseError(format!("Failed to start transaction: {}", e)))?;
        
        let next_index = match parent_id {
            None => {
                // Root-level comment: Lock post and atomically count + allocate
                // First, lock the post to prevent concurrent root comment creation
                sqlx::query!(
                    "SELECT id FROM posts WHERE id = $1 FOR UPDATE",
                    post_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| crate::AppError::DatabaseError(format!("Failed to lock post for root comment allocation: {}", e)))?;
                
                // Now count top-level comments for this post
                let result = sqlx::query!(
                    "SELECT COUNT(*) as count FROM comments WHERE post_id = $1 AND parent_id IS NULL",
                    post_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| crate::AppError::DatabaseError(format!("Failed to count root sibling comments: {}", e)))?;
                
                (result.count.unwrap_or(0) + 1) as i32
            },
            Some(parent_id) => {
                // Reply: Lock parent comment and atomically count + allocate
                // First, lock the parent comment to prevent concurrent reply creation
                sqlx::query!(
                    "SELECT id FROM comments WHERE id = $1 FOR UPDATE",
                    parent_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| crate::AppError::DatabaseError(format!("Failed to lock parent comment for reply allocation: {}", e)))?;
                
                // Now count replies to this specific parent
                let result = sqlx::query!(
                    "SELECT COUNT(*) as count FROM comments WHERE parent_id = $1",
                    parent_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| crate::AppError::DatabaseError(format!("Failed to count reply sibling comments: {}", e)))?;
                
                (result.count.unwrap_or(0) + 1) as i32
            }
        };
        
        // Commit the transaction to release locks
        tx.commit()
            .await
            .map_err(|e| crate::AppError::DatabaseError(format!("Failed to commit sibling allocation transaction: {}", e)))?;
        
        Ok(next_index)
    }

    async fn increment_reply_count(&self, comment_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE comments SET reply_count = reply_count + 1, updated_at = NOW() WHERE id = $1",
            comment_id
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| crate::AppError::DatabaseError(format!("Failed to increment reply count: {}", e)))?;
        
        Ok(())
    }
}

// PostgreSQL Vote Repository - full implementation
pub struct PostgresVoteRepository {
    pool: Arc<PgPool>,
}

impl PostgresVoteRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VoteRepository for PostgresVoteRepository {
    async fn cast_vote(&self, vote: &Vote) -> Result<Vote> {
        let result = sqlx::query!(
            r#"
            INSERT INTO votes (id, user_id, target_id, target_type, vote_type, tag, is_upvote, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (user_id, target_id, vote_type, tag) 
            DO UPDATE SET 
                is_upvote = EXCLUDED.is_upvote,
                updated_at = EXCLUDED.updated_at
            RETURNING id, user_id, target_id, target_type, vote_type, tag, is_upvote, created_at, updated_at
            "#,
            vote.id,
            vote.user_id,
            vote.target_id,
            vote.target_type,
            vote.vote_type,
            vote.tag,
            vote.is_upvote,
            vote.created_at,
            vote.updated_at
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to cast vote: {}", e)))?;

        Ok(Vote {
            id: result.id,
            user_id: result.user_id,
            target_id: result.target_id,
            target_type: result.target_type,
            vote_type: result.vote_type,
            tag: result.tag,
            is_upvote: result.is_upvote,
            created_at: result.created_at,
            updated_at: result.updated_at,
        })
    }
    
    async fn get_user_vote(&self, user_id: Uuid, target_id: Uuid, vote_type: &str, tag: &str) -> Result<Option<Vote>> {
        let result = sqlx::query!(
            "SELECT id, user_id, target_id, target_type, vote_type, tag, is_upvote, created_at, updated_at 
             FROM votes 
             WHERE user_id = $1 AND target_id = $2 AND vote_type = $3 AND tag = $4",
            user_id, target_id, vote_type, tag
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get user vote: {}", e)))?;

        Ok(result.map(|row| Vote {
            id: row.id,
            user_id: row.user_id,
            target_id: row.target_id,
            target_type: row.target_type,
            vote_type: row.vote_type,
            tag: row.tag,
            is_upvote: row.is_upvote,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }
    
    async fn get_vote_counts(&self, target_id: Uuid, target_type: &str) -> Result<Vec<TagVoteCount>> {
        let results = sqlx::query!(
            r#"
            SELECT 
                vote_type,
                tag,
                SUM(CASE WHEN is_upvote THEN 1 ELSE 0 END) as upvotes,
                SUM(CASE WHEN NOT is_upvote THEN 1 ELSE 0 END) as downvotes,
                COUNT(*) as total_votes
            FROM votes 
            WHERE target_id = $1 AND target_type = $2 
            GROUP BY vote_type, tag
            "#,
            target_id, target_type
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get vote counts: {}", e)))?;

        Ok(results.into_iter().map(|row| TagVoteCount {
            tag: row.tag,
            upvotes: row.upvotes.unwrap_or(0) as i64,
            downvotes: row.downvotes.unwrap_or(0) as i64,
            total_votes: row.total_votes.unwrap_or(0) as i64,
            agreement_ratio: if row.total_votes.unwrap_or(0) > 0 {
                row.upvotes.unwrap_or(0) as f64 / row.total_votes.unwrap_or(1) as f64
            } else {
                0.0
            },
            display_count: format!("{}", row.total_votes.unwrap_or(0)),
        }).collect())
    }
    
    async fn get_vote_summary(&self, target_id: Uuid, target_type: &str) -> Result<VoteSummary> {
        // Get vote counts separated by vote type
        let emotion_results = sqlx::query!(
            r#"
            SELECT 
                tag,
                SUM(CASE WHEN is_upvote THEN 1 ELSE 0 END) as upvotes,
                SUM(CASE WHEN NOT is_upvote THEN 1 ELSE 0 END) as downvotes,
                COUNT(*) as total_votes
            FROM votes 
            WHERE target_id = $1 AND target_type = $2 AND vote_type = 'emotion'
            GROUP BY tag
            "#,
            target_id, target_type
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get emotion vote counts: {}", e)))?;

        let content_filter_results = sqlx::query!(
            r#"
            SELECT 
                tag,
                SUM(CASE WHEN is_upvote THEN 1 ELSE 0 END) as upvotes,
                SUM(CASE WHEN NOT is_upvote THEN 1 ELSE 0 END) as downvotes,
                COUNT(*) as total_votes
            FROM votes 
            WHERE target_id = $1 AND target_type = $2 AND vote_type = 'content_filter'
            GROUP BY tag
            "#,
            target_id, target_type
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get content filter vote counts: {}", e)))?;

        let emotion_votes: Vec<TagVoteCount> = emotion_results.into_iter().map(|row| TagVoteCount {
            tag: row.tag,
            upvotes: row.upvotes.unwrap_or(0) as i64,
            downvotes: row.downvotes.unwrap_or(0) as i64,
            total_votes: row.total_votes.unwrap_or(0) as i64,
            agreement_ratio: if row.total_votes.unwrap_or(0) > 0 {
                row.upvotes.unwrap_or(0) as f64 / row.total_votes.unwrap_or(1) as f64
            } else {
                0.0
            },
            display_count: format!("{}", row.total_votes.unwrap_or(0)),
        }).collect();

        let content_filter_votes: Vec<TagVoteCount> = content_filter_results.into_iter().map(|row| TagVoteCount {
            tag: row.tag,
            upvotes: row.upvotes.unwrap_or(0) as i64,
            downvotes: row.downvotes.unwrap_or(0) as i64,
            total_votes: row.total_votes.unwrap_or(0) as i64,
            agreement_ratio: if row.total_votes.unwrap_or(0) > 0 {
                row.upvotes.unwrap_or(0) as f64 / row.total_votes.unwrap_or(1) as f64
            } else {
                0.0
            },
            display_count: format!("{}", row.total_votes.unwrap_or(0)),
        }).collect();

        let total_engagement = emotion_votes.iter().map(|v| v.total_votes).sum::<i64>() + 
                              content_filter_votes.iter().map(|v| v.total_votes).sum::<i64>();
        
        Ok(VoteSummary {
            target_id,
            emotion_votes,
            content_filter_votes,
            total_engagement: total_engagement,
        })
    }
    
    async fn remove_vote(&self, user_id: Uuid, target_id: Uuid, vote_type: &str, tag: &str) -> Result<()> {
        sqlx::query!(
            "DELETE FROM votes WHERE user_id = $1 AND target_id = $2 AND vote_type = $3 AND tag = $4",
            user_id, target_id, vote_type, tag
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to remove vote: {}", e)))?;

        Ok(())
    }
    
    async fn get_engagement_score(&self, target_id: Uuid, target_type: &str) -> Result<f64> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as vote_count FROM votes WHERE target_id = $1 AND target_type = $2",
            target_id, target_type
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to get engagement score: {}", e)))?;

        // Simple engagement calculation: 0.1 points per vote, capped at 2.0
        let engagement = (result.vote_count.unwrap_or(0) as f64 * 0.1).min(2.0);
        Ok(engagement)
    }
}