// PostgreSQL repository implementations using sqlx
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use std::sync::Arc;
use crate::{Result, AppError};
use crate::models::{User, Post, Comment, Vote, TagVoteCount, VoteSummary};
use crate::db::repository::{UserRepository, PostRepository, CommentRepository, VoteRepository};

// PostgreSQL connection pool wrapper
pub struct PostgresDatabase {
    pub pool: Arc<PgPool>,
}

impl PostgresDatabase {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to connect to PostgreSQL: {}", e)))?;
        
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

#[async_trait]
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
            comment_count: 0,
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
                comment_count: 0, // Will be calculated separately
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
                   p.is_flagged, p.created_at, p.updated_at, p.view_count, u.username
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
                comment_count: 0,
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
                comment_count: 0,
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
        // Comment count not stored in database - calculated on demand
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

#[async_trait]
impl CommentRepository for PostgresCommentRepository {
    async fn create_comment(&self, comment: &Comment) -> Result<Comment> {
        // Simplified implementation - will be enhanced later
        Ok(comment.clone())
    }
    
    async fn get_comment_by_id(&self, _id: Uuid) -> Result<Option<Comment>> {
        Ok(None) // Simplified - will implement later
    }
    
    async fn get_comments_by_post_id(&self, _post_id: Uuid) -> Result<Vec<Comment>> {
        Ok(vec![]) // Simplified - will implement later
    }
    
    async fn get_comments_by_parent_id(&self, _parent_id: Uuid) -> Result<Vec<Comment>> {
        Ok(vec![]) // Simplified - will implement later
    }
    
    async fn update_comment(&self, comment: &Comment) -> Result<Comment> {
        Ok(comment.clone()) // Simplified - will implement later
    }
    
    async fn delete_comment(&self, _id: Uuid) -> Result<()> {
        Ok(()) // Simplified - will implement later
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
        let next_index = match parent_id {
            None => {
                // Root-level comment: Count top-level comments for this post and add 1
                let result = sqlx::query!(
                    "SELECT COUNT(*) as count FROM comments WHERE post_id = $1 AND parent_id IS NULL",
                    post_id
                )
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| crate::AppError::DatabaseError(format!("Failed to allocate root sibling index: {}", e)))?;
                
                (result.count.unwrap_or(0) + 1) as i32
            },
            Some(parent_id) => {
                // Reply: Count replies to this specific parent and add 1
                let result = sqlx::query!(
                    "SELECT COUNT(*) as count FROM comments WHERE parent_id = $1",
                    parent_id
                )
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| crate::AppError::DatabaseError(format!("Failed to allocate reply sibling index: {}", e)))?;
                
                (result.count.unwrap_or(0) + 1) as i32
            }
        };
        
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

// PostgreSQL Vote Repository - simplified implementation
pub struct PostgresVoteRepository {
    pool: Arc<PgPool>,
}

#[async_trait]
impl VoteRepository for PostgresVoteRepository {
    async fn cast_vote(&self, _vote: &Vote) -> Result<Vote> {
        Err(AppError::InternalError("Vote system temporarily disabled".to_string()))
    }
    
    async fn get_user_vote(&self, _user_id: Uuid, _target_id: Uuid, _vote_type: &str, _tag: &str) -> Result<Option<Vote>> {
        Ok(None)
    }
    
    async fn get_vote_counts(&self, _target_id: Uuid, _target_type: &str) -> Result<Vec<TagVoteCount>> {
        Ok(vec![])
    }
    
    async fn get_vote_summary(&self, _target_id: Uuid, _target_type: &str) -> Result<VoteSummary> {
        Ok(VoteSummary {
            target_id: _target_id,
            emotion_votes: vec![],
            content_filter_votes: vec![],
            total_engagement: 0,
        })
    }
    
    async fn remove_vote(&self, _user_id: Uuid, _target_id: Uuid, _vote_type: &str, _tag: &str) -> Result<()> {
        Ok(())
    }
    
    async fn get_engagement_score(&self, _target_id: Uuid, _target_type: &str) -> Result<f64> {
        Ok(1.0)
    }
}