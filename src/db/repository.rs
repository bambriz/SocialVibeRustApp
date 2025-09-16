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
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct MockUserRepository {
    users: Arc<Mutex<HashMap<String, User>>>, // email -> User
    users_by_id: Arc<Mutex<HashMap<Uuid, User>>>, // id -> User
}

pub struct MockPostRepository {
    posts: Arc<Mutex<HashMap<Uuid, Post>>>, // id -> Post
    posts_list: Arc<Mutex<Vec<Post>>>, // for listing and sorting
}

pub struct MockCommentRepository {
    comments: Arc<Mutex<HashMap<Uuid, Comment>>>, // id -> Comment
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            users_by_id: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn create_user(&self, user: &User) -> Result<User> {
        let mut users = self.users.lock().unwrap();
        let mut users_by_id = self.users_by_id.lock().unwrap();
        
        users.insert(user.email.clone(), user.clone());
        users_by_id.insert(user.id, user.clone());
        
        Ok(user.clone())
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let users_by_id = self.users_by_id.lock().unwrap();
        Ok(users_by_id.get(&id).cloned())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let users = self.users.lock().unwrap();
        Ok(users.get(email).cloned())
    }

    async fn update_user(&self, user: &User) -> Result<User> {
        let mut users = self.users.lock().unwrap();
        let mut users_by_id = self.users_by_id.lock().unwrap();
        
        users.insert(user.email.clone(), user.clone());
        users_by_id.insert(user.id, user.clone());
        
        Ok(user.clone())
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        let mut users_by_id = self.users_by_id.lock().unwrap();
        if let Some(user) = users_by_id.remove(&id) {
            let mut users = self.users.lock().unwrap();
            users.remove(&user.email);
        }
        Ok(())
    }
}

impl MockPostRepository {
    pub fn new() -> Self {
        Self {
            posts: Arc::new(Mutex::new(HashMap::new())),
            posts_list: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl PostRepository for MockPostRepository {
    async fn create_post(&self, post: &Post) -> Result<Post> {
        let mut posts = self.posts.lock().unwrap();
        let mut posts_list = self.posts_list.lock().unwrap();
        
        posts.insert(post.id, post.clone());
        posts_list.push(post.clone());
        
        Ok(post.clone())
    }

    async fn get_post_by_id(&self, id: Uuid) -> Result<Option<Post>> {
        let posts = self.posts.lock().unwrap();
        Ok(posts.get(&id).cloned())
    }

    async fn get_posts_paginated(&self, limit: u32, offset: u32) -> Result<Vec<Post>> {
        let posts_list = self.posts_list.lock().unwrap();
        let start = offset as usize;
        let end = std::cmp::min(start + limit as usize, posts_list.len());
        
        if start >= posts_list.len() {
            Ok(vec![])
        } else {
            Ok(posts_list[start..end].to_vec())
        }
    }

    async fn get_posts_by_popularity(&self, limit: u32, offset: u32) -> Result<Vec<Post>> {
        let mut posts_list = self.posts_list.lock().unwrap();
        
        // Sort by popularity score in descending order
        posts_list.sort_by(|a, b| b.popularity_score.partial_cmp(&a.popularity_score).unwrap_or(std::cmp::Ordering::Equal));
        
        let start = offset as usize;
        let end = std::cmp::min(start + limit as usize, posts_list.len());
        
        if start >= posts_list.len() {
            Ok(vec![])
        } else {
            Ok(posts_list[start..end].to_vec())
        }
    }

    async fn update_post(&self, post: &Post) -> Result<Post> {
        let mut posts = self.posts.lock().unwrap();
        let mut posts_list = self.posts_list.lock().unwrap();
        
        posts.insert(post.id, post.clone());
        
        // Update in the list as well
        if let Some(pos) = posts_list.iter().position(|p| p.id == post.id) {
            posts_list[pos] = post.clone();
        }
        
        Ok(post.clone())
    }

    async fn delete_post(&self, id: Uuid) -> Result<()> {
        let mut posts = self.posts.lock().unwrap();
        let mut posts_list = self.posts_list.lock().unwrap();
        
        posts.remove(&id);
        posts_list.retain(|p| p.id != id);
        
        Ok(())
    }

    async fn increment_comment_count(&self, post_id: Uuid) -> Result<()> {
        let mut posts = self.posts.lock().unwrap();
        let mut posts_list = self.posts_list.lock().unwrap();
        
        if let Some(post) = posts.get_mut(&post_id) {
            post.comment_count += 1;
            
            // Update in the list as well
            if let Some(pos) = posts_list.iter().position(|p| p.id == post_id) {
                posts_list[pos].comment_count += 1;
            }
        }
        
        Ok(())
    }

    async fn update_popularity_score(&self, post_id: Uuid, score: f64) -> Result<()> {
        let mut posts = self.posts.lock().unwrap();
        let mut posts_list = self.posts_list.lock().unwrap();
        
        if let Some(post) = posts.get_mut(&post_id) {
            post.popularity_score = score;
            
            // Update in the list as well
            if let Some(pos) = posts_list.iter().position(|p| p.id == post_id) {
                posts_list[pos].popularity_score = score;
            }
        }
        
        Ok(())
    }
}

impl MockCommentRepository {
    pub fn new() -> Self {
        Self {
            comments: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl CommentRepository for MockCommentRepository {
    async fn create_comment(&self, comment: &Comment) -> Result<Comment> {
        let mut comments = self.comments.lock().unwrap();
        comments.insert(comment.id, comment.clone());
        Ok(comment.clone())
    }

    async fn get_comment_by_id(&self, id: Uuid) -> Result<Option<Comment>> {
        let comments = self.comments.lock().unwrap();
        Ok(comments.get(&id).cloned())
    }

    async fn get_comments_by_post_id(&self, post_id: Uuid) -> Result<Vec<Comment>> {
        let comments = self.comments.lock().unwrap();
        let result: Vec<Comment> = comments.values()
            .filter(|c| c.post_id == post_id)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn get_comments_by_parent_id(&self, parent_id: Uuid) -> Result<Vec<Comment>> {
        let comments = self.comments.lock().unwrap();
        let result: Vec<Comment> = comments.values()
            .filter(|c| c.parent_id == Some(parent_id))
            .cloned()
            .collect();
        Ok(result)
    }

    async fn update_comment(&self, comment: &Comment) -> Result<Comment> {
        let mut comments = self.comments.lock().unwrap();
        comments.insert(comment.id, comment.clone());
        Ok(comment.clone())
    }

    async fn delete_comment(&self, id: Uuid) -> Result<()> {
        let mut comments = self.comments.lock().unwrap();
        comments.remove(&id);
        Ok(())
    }

    async fn get_max_sibling_count(&self, post_id: Uuid, _parent_path: Option<&str>) -> Result<u32> {
        let comments = self.comments.lock().unwrap();
        let count = comments.values()
            .filter(|c| c.post_id == post_id && c.parent_id.is_none())
            .count();
        Ok(count as u32)
    }
}