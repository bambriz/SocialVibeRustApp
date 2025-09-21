// Repository trait abstractions for database operations
use crate::models::{User, Post, Comment, Vote, TagVoteCount, VoteSummary};
use crate::Result;
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
    async fn get_posts_by_user(&self, user_id: Uuid, limit: u32, offset: u32) -> Result<Vec<Post>>;
    async fn update_post(&self, post: &Post) -> Result<Post>;
    async fn delete_post(&self, id: Uuid) -> Result<()>;
    async fn increment_comment_count(&self, post_id: Uuid) -> Result<()>;
    async fn update_popularity_score(&self, post_id: Uuid, score: f64) -> Result<()>;
    // Migration support methods
    async fn get_posts_with_old_sentiment_types(&self) -> Result<Vec<Post>>;
    async fn update_post_sentiment(&self, post_id: Uuid, sentiment_type: Option<String>, sentiment_colors: Vec<String>, sentiment_score: Option<f64>) -> Result<()>;
}

#[async_trait]
pub trait CommentRepository: Send + Sync {
    async fn create_comment(&self, comment: &Comment) -> Result<Comment>;
    async fn create_comment_atomic(&self, post_id: Uuid, parent_id: Option<Uuid>, comment: &Comment) -> Result<Comment>;
    async fn get_comment_by_id(&self, id: Uuid) -> Result<Option<Comment>>;
    async fn get_comments_by_post_id(&self, post_id: Uuid) -> Result<Vec<Comment>>;
    async fn get_comments_by_parent_id(&self, parent_id: Uuid) -> Result<Vec<Comment>>;
    async fn update_comment(&self, comment: &Comment) -> Result<Comment>;
    async fn delete_comment(&self, id: Uuid) -> Result<()>;
    async fn get_max_sibling_count(&self, post_id: Uuid, parent_path: Option<&str>) -> Result<u32>;
    async fn allocate_next_sibling_index(&self, post_id: Uuid, parent_id: Option<Uuid>) -> Result<i32>;
    async fn increment_reply_count(&self, comment_id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait VoteRepository: Send + Sync {
    /// Cast or update a vote on a post/comment tag
    async fn cast_vote(&self, vote: &Vote) -> Result<Vote>;
    /// Get user's vote on a specific target and tag
    async fn get_user_vote(&self, user_id: Uuid, target_id: Uuid, vote_type: &str, tag: &str) -> Result<Option<Vote>>;
    /// Get vote counts for all tags on a target (post/comment)
    async fn get_vote_counts(&self, target_id: Uuid, target_type: &str) -> Result<Vec<TagVoteCount>>;
    /// Get comprehensive vote summary for a target
    async fn get_vote_summary(&self, target_id: Uuid, target_type: &str) -> Result<VoteSummary>;
    /// Remove a user's vote
    async fn remove_vote(&self, user_id: Uuid, target_id: Uuid, vote_type: &str, tag: &str) -> Result<()>;
    /// Get total engagement score for popularity calculation
    async fn get_engagement_score(&self, target_id: Uuid, target_type: &str) -> Result<f64>;
}

// In-memory mock implementations for development/testing
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use chrono::{DateTime, Utc};
use csv::{Reader, Writer};

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

// Mock vote repository for development
pub struct MockVoteRepository {
    votes: Arc<Mutex<HashMap<String, Vote>>>, // "user_id:target_id:vote_type:tag" -> Vote
}

// CSV-based user repository for persistent storage
pub struct CsvUserRepository {
    csv_file_path: String,
    users_cache: Arc<Mutex<HashMap<String, User>>>, // email -> User
    users_by_id_cache: Arc<Mutex<HashMap<Uuid, User>>>, // id -> User
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CsvUser {
    id: String,
    username: String,
    email: String,
    password_hash: String,
    created_at: String,
    updated_at: String,
    is_active: bool,
}


impl CsvUserRepository {
    pub fn new(csv_file_path: Option<String>) -> Self {
        let path = csv_file_path.unwrap_or_else(|| "users_backup.csv".to_string());
        let repo = Self {
            csv_file_path: path.clone(),
            users_cache: Arc::new(Mutex::new(HashMap::new())),
            users_by_id_cache: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Initialize CSV file with headers if it doesn't exist
        if !Path::new(&path).exists() {
            if let Err(e) = repo.initialize_csv_file() {
                eprintln!("Warning: Failed to initialize user CSV file: {}", e);
            }
        }
        
        // Load existing users into cache
        if let Err(e) = repo.load_users_from_csv() {
            eprintln!("Warning: Failed to load existing users from CSV: {}", e);
        }
        
        repo
    }
    
    fn initialize_csv_file(&self) -> Result<()> {
        let file = File::create(&self.csv_file_path)
            .map_err(|e| crate::AppError::InternalError(format!("Failed to create user CSV file: {}", e)))?;
        let mut writer = Writer::from_writer(file);
        
        // Write header
        writer.write_record(&[
            "id", "username", "email", "password_hash", 
            "created_at", "updated_at", "is_active"
        ])
        .map_err(|e| crate::AppError::InternalError(format!("Failed to write user CSV header: {}", e)))?;
        
        writer.flush()
            .map_err(|e| crate::AppError::InternalError(format!("Failed to flush user CSV writer: {}", e)))?;
        
        Ok(())
    }
    
    fn load_users_from_csv(&self) -> Result<()> {
        if !Path::new(&self.csv_file_path).exists() {
            return Ok(()); // File doesn't exist yet, that's OK
        }
        
        let file = File::open(&self.csv_file_path)
            .map_err(|e| crate::AppError::InternalError(format!("Failed to open user CSV file: {}", e)))?;
        let mut reader = Reader::from_reader(BufReader::new(file));
        
        let mut users_cache = self.users_cache.lock().unwrap();
        let mut users_by_id_cache = self.users_by_id_cache.lock().unwrap();
        
        for result in reader.deserialize() {
            let csv_user: CsvUser = result
                .map_err(|e| crate::AppError::InternalError(format!("Failed to deserialize user from CSV: {}", e)))?;
            let user = self.csv_user_to_user(csv_user)?;
            users_cache.insert(user.email.clone(), user.clone());
            users_by_id_cache.insert(user.id, user);
        }
        
        Ok(())
    }
    
    fn save_users_to_csv(&self) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.csv_file_path)
            .map_err(|e| crate::AppError::InternalError(format!("Failed to open user CSV file for writing: {}", e)))?;
            
        let mut writer = Writer::from_writer(BufWriter::new(file));
        
        // Write header
        writer.write_record(&[
            "id", "username", "email", "password_hash", 
            "created_at", "updated_at", "is_active"
        ])
        .map_err(|e| crate::AppError::InternalError(format!("Failed to write user CSV header: {}", e)))?;
        
        let users_cache = self.users_cache.lock().unwrap();
        for user in users_cache.values() {
            let csv_user = self.user_to_csv_user(user);
            writer.serialize(&csv_user)
                .map_err(|e| crate::AppError::InternalError(format!("Failed to serialize user to CSV: {}", e)))?;
        }
        
        writer.flush()
            .map_err(|e| crate::AppError::InternalError(format!("Failed to flush user CSV writer: {}", e)))?;
        
        Ok(())
    }
    
    fn user_to_csv_user(&self, user: &User) -> CsvUser {
        CsvUser {
            id: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            password_hash: user.password_hash.clone(),
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
            is_active: user.is_active,
        }
    }
    
    fn csv_user_to_user(&self, csv_user: CsvUser) -> Result<User> {
        let id = Uuid::parse_str(&csv_user.id)
            .map_err(|e| crate::AppError::InternalError(format!("Invalid user ID in CSV: {}", e)))?;
        let created_at = DateTime::parse_from_rfc3339(&csv_user.created_at)
            .map_err(|e| crate::AppError::InternalError(format!("Invalid created_at in CSV: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&csv_user.updated_at)
            .map_err(|e| crate::AppError::InternalError(format!("Invalid updated_at in CSV: {}", e)))?
            .with_timezone(&Utc);
        
        Ok(User {
            id,
            username: csv_user.username,
            email: csv_user.email,
            password_hash: csv_user.password_hash,
            display_name: None,
            bio: None,
            avatar_url: None,
            created_at,
            updated_at,
            is_active: csv_user.is_active,
        })
    }
}


#[async_trait]
impl UserRepository for CsvUserRepository {
    async fn create_user(&self, user: &User) -> Result<User> {
        let mut users_cache = self.users_cache.lock().unwrap();
        let mut users_by_id_cache = self.users_by_id_cache.lock().unwrap();
        
        users_cache.insert(user.email.clone(), user.clone());
        users_by_id_cache.insert(user.id, user.clone());
        drop(users_cache); // Release locks before file I/O
        drop(users_by_id_cache);
        
        self.save_users_to_csv()?;
        Ok(user.clone())
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let users_by_id_cache = self.users_by_id_cache.lock().unwrap();
        Ok(users_by_id_cache.get(&id).cloned())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let users_cache = self.users_cache.lock().unwrap();
        Ok(users_cache.get(email).cloned())
    }

    async fn update_user(&self, user: &User) -> Result<User> {
        let mut users_cache = self.users_cache.lock().unwrap();
        let mut users_by_id_cache = self.users_by_id_cache.lock().unwrap();
        
        users_cache.insert(user.email.clone(), user.clone());
        users_by_id_cache.insert(user.id, user.clone());
        drop(users_cache); // Release locks before file I/O
        drop(users_by_id_cache);
        
        self.save_users_to_csv()?;
        Ok(user.clone())
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        let mut users_by_id_cache = self.users_by_id_cache.lock().unwrap();
        if let Some(user) = users_by_id_cache.remove(&id) {
            let mut users_cache = self.users_cache.lock().unwrap();
            users_cache.remove(&user.email);
            drop(users_cache); // Release locks before file I/O
        }
        drop(users_by_id_cache);
        
        self.save_users_to_csv()?;
        Ok(())
    }
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

    async fn get_posts_by_user(&self, user_id: Uuid, limit: u32, offset: u32) -> Result<Vec<Post>> {
        let posts_list = self.posts_list.lock().unwrap();
        
        // Filter posts by user_id and sort by created_at in descending order
        let mut user_posts: Vec<Post> = posts_list
            .iter()
            .filter(|post| post.author_id == user_id)
            .cloned()
            .collect();
        
        user_posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        let start = offset as usize;
        let end = std::cmp::min(start + limit as usize, user_posts.len());
        
        if start >= user_posts.len() {
            Ok(vec![])
        } else {
            Ok(user_posts[start..end].to_vec())
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

    async fn get_posts_with_old_sentiment_types(&self) -> Result<Vec<Post>> {
        let posts_list = self.posts_list.lock().unwrap();
        let old_sentiment_types = ["happy", "excited", "calm"];
        
        let posts_with_old_sentiments: Vec<Post> = posts_list.iter()
            .filter(|post| {
                if let Some(ref sentiment_type) = post.sentiment_type {
                    // Check for direct old sentiment types
                    if old_sentiment_types.contains(&sentiment_type.as_str()) {
                        return true;
                    }
                    // Check for combination sentiment types that contain old base emotions
                    if sentiment_type.contains("happy") || sentiment_type.contains("excited") || sentiment_type.contains("calm") {
                        return true;
                    }
                }
                false
            })
            .cloned()
            .collect();
        
        Ok(posts_with_old_sentiments)
    }

    async fn update_post_sentiment(&self, post_id: Uuid, sentiment_type: Option<String>, sentiment_colors: Vec<String>, sentiment_score: Option<f64>) -> Result<()> {
        let mut posts = self.posts.lock().unwrap();
        let mut posts_list = self.posts_list.lock().unwrap();
        
        if let Some(post) = posts.get_mut(&post_id) {
            post.sentiment_type = sentiment_type;
            post.sentiment_colors = sentiment_colors;
            post.sentiment_score = sentiment_score;
            post.updated_at = chrono::Utc::now();
            
            // Update in the list as well
            if let Some(pos) = posts_list.iter().position(|p| p.id == post_id) {
                posts_list[pos].sentiment_type = post.sentiment_type.clone();
                posts_list[pos].sentiment_colors = post.sentiment_colors.clone();
                posts_list[pos].sentiment_score = post.sentiment_score;
                posts_list[pos].updated_at = post.updated_at;
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
    
    /// Count total comments for path generation
    pub async fn count_comments(&self) -> Result<usize> {
        let comments = self.comments.lock().unwrap();
        Ok(comments.len())
    }
    
    /// Count replies to a specific parent comment for path generation
    pub async fn count_replies(&self, parent_id: Uuid) -> Result<usize> {
        let comments = self.comments.lock().unwrap();
        let count = comments
            .values()
            .filter(|comment| comment.parent_id == Some(parent_id))
            .count();
        Ok(count)
    }
    
    /// Atomically allocate next sibling index for a specific parent (including post-level)
    /// This prevents race conditions in path generation
    pub async fn allocate_next_sibling_index(&self, post_id: Uuid, parent_id: Option<Uuid>) -> Result<usize> {
        let comments = self.comments.lock().unwrap();
        let count = match parent_id {
            None => {
                // Root-level: Count top-level comments for this specific post
                comments
                    .values()
                    .filter(|comment| comment.post_id == post_id && comment.parent_id.is_none())
                    .count()
            },
            Some(parent_id) => {
                // Reply: Count replies to this specific parent
                comments
                    .values()
                    .filter(|comment| comment.parent_id == Some(parent_id))
                    .count()
            }
        };
        Ok(count + 1) // Return next available index (1-based)
    }
    
    /// Increment reply count for a parent comment atomically
    pub async fn increment_reply_count(&self, parent_id: Uuid) -> Result<()> {
        let mut comments = self.comments.lock().unwrap();
        if let Some(parent_comment) = comments.get_mut(&parent_id) {
            parent_comment.reply_count += 1;
            parent_comment.updated_at = chrono::Utc::now();
        }
        Ok(())
    }
}

#[async_trait]
impl CommentRepository for MockCommentRepository {
    async fn create_comment(&self, comment: &Comment) -> Result<Comment> {
        let mut comments = self.comments.lock().unwrap();
        comments.insert(comment.id, comment.clone());
        Ok(comment.clone())
    }

    async fn create_comment_atomic(&self, _post_id: Uuid, _parent_id: Option<Uuid>, comment: &Comment) -> Result<Comment> {
        // Mock implementation: simply calls create_comment
        // In real implementation, this would do atomic path allocation + insert
        self.create_comment(comment).await
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

    async fn allocate_next_sibling_index(&self, post_id: Uuid, parent_id: Option<Uuid>) -> Result<i32> {
        let comments = self.comments.lock().unwrap();
        let count = match parent_id {
            None => {
                // Root-level: Count top-level comments for this specific post
                comments
                    .values()
                    .filter(|comment| comment.post_id == post_id && comment.parent_id.is_none())
                    .count()
            },
            Some(parent_id) => {
                // Reply: Count replies to this specific parent
                comments
                    .values()
                    .filter(|comment| comment.parent_id == Some(parent_id))
                    .count()
            }
        };
        Ok((count + 1) as i32) // Return next available index (1-based)
    }

    async fn increment_reply_count(&self, parent_id: Uuid) -> Result<()> {
        let mut comments = self.comments.lock().unwrap();
        if let Some(parent_comment) = comments.get_mut(&parent_id) {
            parent_comment.reply_count += 1;
            parent_comment.updated_at = chrono::Utc::now();
        }
        Ok(())
    }
}

// MockVoteRepository implementation
impl MockVoteRepository {
    pub fn new() -> Self {
        Self {
            votes: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    fn make_vote_key(&self, user_id: Uuid, target_id: Uuid, vote_type: &str, tag: &str) -> String {
        format!("{}:{}:{}:{}", user_id, target_id, vote_type, tag)
    }
}

#[async_trait]
impl VoteRepository for MockVoteRepository {
    async fn cast_vote(&self, vote: &Vote) -> Result<Vote> {
        let mut votes = self.votes.lock().unwrap();
        let key = self.make_vote_key(vote.user_id, vote.target_id, &vote.vote_type, &vote.tag);
        votes.insert(key, vote.clone());
        Ok(vote.clone())
    }
    
    async fn get_user_vote(&self, user_id: Uuid, target_id: Uuid, vote_type: &str, tag: &str) -> Result<Option<Vote>> {
        let votes = self.votes.lock().unwrap();
        let key = self.make_vote_key(user_id, target_id, vote_type, tag);
        Ok(votes.get(&key).cloned())
    }
    
    async fn get_vote_counts(&self, target_id: Uuid, target_type: &str) -> Result<Vec<TagVoteCount>> {
        let votes = self.votes.lock().unwrap();
        let mut tag_counts: HashMap<String, (i64, i64)> = HashMap::new();
        
        // Count votes for this target
        for vote in votes.values() {
            if vote.target_id == target_id && vote.target_type == target_type {
                let (upvotes, downvotes) = tag_counts.entry(vote.tag.clone()).or_insert((0, 0));
                if vote.is_upvote {
                    *upvotes += 1;
                } else {
                    *downvotes += 1;
                }
            }
        }
        
        Ok(tag_counts.into_iter()
            .map(|(tag, (upvotes, downvotes))| TagVoteCount::new(tag, upvotes, downvotes))
            .collect())
    }
    
    async fn get_vote_summary(&self, target_id: Uuid, target_type: &str) -> Result<VoteSummary> {
        let all_counts = self.get_vote_counts(target_id, target_type).await?;
        let (emotion_votes, content_filter_votes): (Vec<_>, Vec<_>) = all_counts.into_iter()
            .partition(|vote_count| {
                // Emotion tags: joy, sad, angry, fear, disgust, surprise, confused, neutral, sarcastic, affectionate
                matches!(vote_count.tag.as_str(), "joy" | "sad" | "angry" | "fear" | "disgust" | "surprise" | "confused" | "neutral" | "sarcastic" | "affectionate")
            });
        
        let total_engagement = emotion_votes.iter().chain(content_filter_votes.iter())
            .map(|vc| vc.total_votes)
            .sum();
            
        Ok(VoteSummary {
            target_id,
            emotion_votes,
            content_filter_votes,
            total_engagement,
        })
    }
    
    async fn remove_vote(&self, user_id: Uuid, target_id: Uuid, vote_type: &str, tag: &str) -> Result<()> {
        let mut votes = self.votes.lock().unwrap();
        let key = self.make_vote_key(user_id, target_id, vote_type, tag);
        votes.remove(&key);
        Ok(())
    }
    
    async fn get_engagement_score(&self, target_id: Uuid, target_type: &str) -> Result<f64> {
        let vote_counts = self.get_vote_counts(target_id, target_type).await?;
        let total_engagement = vote_counts.iter().map(|vc| vc.total_votes).sum::<i64>();
        
        // Cap engagement impact at 3.0 as per user requirements
        let engagement_score = (total_engagement as f64).ln().max(0.0).min(3.0);
        Ok(engagement_score)
    }
}