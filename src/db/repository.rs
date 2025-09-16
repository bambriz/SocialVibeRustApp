// Repository trait abstractions for database operations
use crate::models::{User, Post, Comment};
use crate::{AppError, Result};
use uuid::Uuid;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: &User) -> Result<User>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn update_user(&self, user: &User) -> Result<User>;
    async fn delete_user(&self, id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create_post(&self, post: &Post) -> Result<Post>;
    async fn get_post_by_id(&self, id: Uuid) -> Result<Option<Post>>;
    async fn get_posts_paginated(&self, limit: u32, offset: u32) -> Result<Vec<Post>>;
    async fn get_posts_by_popularity(&self, limit: u32, offset: u32) -> Result<Vec<Post>>;
    async fn update_post(&self, post: &Post) -> Result<Post>;
    async fn delete_post(&self, id: Uuid) -> Result<()>;
    async fn increment_comment_count(&self, post_id: Uuid) -> Result<()>;
    async fn update_popularity_score(&self, post_id: Uuid, score: f64) -> Result<()>;
}

#[async_trait]
pub trait CommentRepository: Send + Sync {
    async fn create_comment(&self, comment: &Comment) -> Result<Comment>;
    async fn get_comment_by_id(&self, id: Uuid) -> Result<Option<Comment>>;
    async fn get_comments_by_post_id(&self, post_id: Uuid) -> Result<Vec<Comment>>;
    async fn get_comments_by_parent_id(&self, parent_id: Uuid) -> Result<Vec<Comment>>;
    async fn update_comment(&self, comment: &Comment) -> Result<Comment>;
    async fn delete_comment(&self, id: Uuid) -> Result<()>;
    async fn get_max_sibling_count(&self, post_id: Uuid, parent_path: Option<&str>) -> Result<u32>;
}

// Mock implementations for development (before Cosmos DB integration)
pub struct MockUserRepository;
pub struct MockPostRepository;
pub struct MockCommentRepository;

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn create_user(&self, user: &User) -> Result<User> {
        Ok(user.clone())
    }

    async fn get_user_by_id(&self, _id: Uuid) -> Result<Option<User>> {
        Ok(None)
    }

    async fn get_user_by_email(&self, _email: &str) -> Result<Option<User>> {
        Ok(None)
    }

    async fn update_user(&self, user: &User) -> Result<User> {
        Ok(user.clone())
    }

    async fn delete_user(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl PostRepository for MockPostRepository {
    async fn create_post(&self, post: &Post) -> Result<Post> {
        Ok(post.clone())
    }

    async fn get_post_by_id(&self, _id: Uuid) -> Result<Option<Post>> {
        Ok(None)
    }

    async fn get_posts_paginated(&self, _limit: u32, _offset: u32) -> Result<Vec<Post>> {
        Ok(vec![])
    }

    async fn get_posts_by_popularity(&self, _limit: u32, _offset: u32) -> Result<Vec<Post>> {
        Ok(vec![])
    }

    async fn update_post(&self, post: &Post) -> Result<Post> {
        Ok(post.clone())
    }

    async fn delete_post(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn increment_comment_count(&self, _post_id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn update_popularity_score(&self, _post_id: Uuid, _score: f64) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl CommentRepository for MockCommentRepository {
    async fn create_comment(&self, comment: &Comment) -> Result<Comment> {
        Ok(comment.clone())
    }

    async fn get_comment_by_id(&self, _id: Uuid) -> Result<Option<Comment>> {
        Ok(None)
    }

    async fn get_comments_by_post_id(&self, _post_id: Uuid) -> Result<Vec<Comment>> {
        Ok(vec![])
    }

    async fn get_comments_by_parent_id(&self, _parent_id: Uuid) -> Result<Vec<Comment>> {
        Ok(vec![])
    }

    async fn update_comment(&self, comment: &Comment) -> Result<Comment> {
        Ok(comment.clone())
    }

    async fn delete_comment(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn get_max_sibling_count(&self, _post_id: Uuid, _parent_path: Option<&str>) -> Result<u32> {
        Ok(0)
    }
}