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
    // Migration support methods
    async fn get_posts_with_old_sentiment_types(&self) -> Result<Vec<Post>>;
    async fn update_post_sentiment(&self, post_id: Uuid, sentiment_type: Option<String>, sentiment_colors: Vec<String>, sentiment_score: Option<f64>) -> Result<()>;
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

// CSV-based post repository for fallback storage
pub struct CsvPostRepository {
    csv_file_path: String,
    posts_cache: Arc<Mutex<HashMap<Uuid, Post>>>, // In-memory cache for performance
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CsvPost {
    id: String,
    title: String,
    content: String,
    author_id: String,
    author_username: String,
    created_at: String,
    updated_at: String,
    comment_count: u32,
    sentiment_score: String, // Serialize Option<f64> as string
    sentiment_colors: String, // Serialize Vec<String> as JSON string
    sentiment_type: String, // Serialize Option<String> as string
    popularity_score: f64,
    is_blocked: bool,
    toxicity_tags: String, // Serialize Vec<String> as JSON string
    toxicity_scores: String, // Serialize Option<serde_json::Value> as JSON string
}

impl CsvPostRepository {
    pub fn new(csv_file_path: Option<String>) -> Self {
        let path = csv_file_path.unwrap_or_else(|| "posts_backup.csv".to_string());
        let repo = Self {
            csv_file_path: path.clone(),
            posts_cache: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Initialize CSV file with headers if it doesn't exist
        if !Path::new(&path).exists() {
            if let Err(e) = repo.initialize_csv_file() {
                eprintln!("Warning: Failed to initialize CSV file: {}", e);
            }
        }
        
        // Load existing posts into cache
        if let Err(e) = repo.load_posts_from_csv() {
            eprintln!("Warning: Failed to load existing posts from CSV: {}", e);
        }
        
        repo
    }
    
    fn initialize_csv_file(&self) -> Result<()> {
        let file = File::create(&self.csv_file_path)
            .map_err(|e| crate::AppError::InternalError(format!("Failed to create CSV file: {}", e)))?;
        let mut writer = Writer::from_writer(file);
        
        // Write header
        writer.write_record(&[
            "id", "title", "content", "author_id", "author_username", 
            "created_at", "updated_at", "comment_count", "sentiment_score",
            "sentiment_colors", "sentiment_type", "popularity_score", "is_blocked",
            "toxicity_tags", "toxicity_scores"
        ])
        .map_err(|e| crate::AppError::InternalError(format!("Failed to write CSV header: {}", e)))?;
        
        writer.flush()
            .map_err(|e| crate::AppError::InternalError(format!("Failed to flush CSV file: {}", e)))?;
        
        Ok(())
    }
    
    fn load_posts_from_csv(&self) -> Result<()> {
        if !Path::new(&self.csv_file_path).exists() {
            return Ok(());
        }
        
        let file = File::open(&self.csv_file_path)
            .map_err(|e| crate::AppError::InternalError(format!("Failed to open CSV file: {}", e)))?;
        
        let mut reader = Reader::from_reader(BufReader::new(file));
        let mut cache = self.posts_cache.lock().unwrap();
        
        for result in reader.deserialize() {
            let csv_post: CsvPost = result
                .map_err(|e| crate::AppError::InternalError(format!("Failed to parse CSV record: {}", e)))?;
            
            let post = self.csv_post_to_post(csv_post)?;
            cache.insert(post.id, post);
        }
        
        Ok(())
    }
    
    fn save_posts_to_csv(&self) -> Result<()> {
        let cache = self.posts_cache.lock().unwrap();
        
        // Create a temporary file to ensure atomic writes and prevent corruption
        let temp_path = format!("{}.tmp", self.csv_file_path);
        let file = File::create(&temp_path)
            .map_err(|e| crate::AppError::InternalError(format!("Failed to create temporary CSV file: {}", e)))?;
        
        {
            let mut writer = Writer::from_writer(BufWriter::new(file));
            
            // Only write header if file is being created for the first time
            // (this method is called for updates, so we need to check if we should write header)
            let should_write_header = !std::path::Path::new(&self.csv_file_path).exists();
            
            if should_write_header {
                writer.write_record(&[
                    "id", "title", "content", "author_id", "author_username", 
                    "created_at", "updated_at", "comment_count", "sentiment_score",
                    "sentiment_colors", "sentiment_type", "popularity_score", "is_blocked",
                    "toxicity_tags", "toxicity_scores"
                ])
                .map_err(|e| crate::AppError::InternalError(format!("Failed to write CSV header: {}", e)))?;
            }
            
            // Write all posts
            for post in cache.values() {
                let csv_post = self.post_to_csv_post(post)?;
                writer.serialize(&csv_post)
                    .map_err(|e| crate::AppError::InternalError(format!("Failed to write CSV record: {}", e)))?;
            }
            
            // Ensure all data is written before moving file
            writer.flush()
                .map_err(|e| crate::AppError::InternalError(format!("Failed to flush CSV file: {}", e)))?;
        } // Writer is dropped here, ensuring file is closed
        
        // Atomically replace the original file with the temporary file
        std::fs::rename(&temp_path, &self.csv_file_path)
            .map_err(|e| crate::AppError::InternalError(format!("Failed to replace CSV file: {}", e)))?;
        
        Ok(())
    }
    
    fn post_to_csv_post(&self, post: &Post) -> Result<CsvPost> {
        Ok(CsvPost {
            id: post.id.to_string(),
            title: post.title.clone(),
            content: post.content.clone(),
            author_id: post.author_id.to_string(),
            author_username: post.author_username.clone(),
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
            comment_count: post.comment_count,
            sentiment_score: post.sentiment_score.map(|s| s.to_string()).unwrap_or_else(|| "".to_string()),
            sentiment_colors: serde_json::to_string(&post.sentiment_colors)
                .map_err(|e| crate::AppError::InternalError(format!("Failed to serialize sentiment colors: {}", e)))?,
            sentiment_type: post.sentiment_type.clone().unwrap_or_else(|| "".to_string()),
            popularity_score: post.popularity_score,
            is_blocked: post.is_blocked,
            toxicity_tags: serde_json::to_string(&post.toxicity_tags)
                .map_err(|e| crate::AppError::InternalError(format!("Failed to serialize toxicity tags: {}", e)))?,
            toxicity_scores: post.toxicity_scores.as_ref()
                .map(|scores| serde_json::to_string(scores)
                    .map_err(|e| crate::AppError::InternalError(format!("Failed to serialize toxicity scores: {}", e))))
                .transpose()?
                .unwrap_or_else(|| "".to_string()),
        })
    }
    
    fn csv_post_to_post(&self, csv_post: CsvPost) -> Result<Post> {
        let id = Uuid::parse_str(&csv_post.id)
            .map_err(|e| crate::AppError::InternalError(format!("Invalid UUID in CSV: {}", e)))?;
        let author_id = Uuid::parse_str(&csv_post.author_id)
            .map_err(|e| crate::AppError::InternalError(format!("Invalid author UUID in CSV: {}", e)))?;
        let created_at = DateTime::parse_from_rfc3339(&csv_post.created_at)
            .map_err(|e| crate::AppError::InternalError(format!("Invalid created_at date in CSV: {}", e)))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&csv_post.updated_at)
            .map_err(|e| crate::AppError::InternalError(format!("Invalid updated_at date in CSV: {}", e)))?
            .with_timezone(&Utc);
        let sentiment_score = if csv_post.sentiment_score.is_empty() {
            None
        } else {
            Some(csv_post.sentiment_score.parse()
                .map_err(|e| crate::AppError::InternalError(format!("Invalid sentiment score in CSV: {}", e)))?)
        };
        let sentiment_colors: Vec<String> = serde_json::from_str(&csv_post.sentiment_colors)
            .map_err(|e| crate::AppError::InternalError(format!("Failed to parse sentiment colors from CSV: {}", e)))?;
        let sentiment_type = if csv_post.sentiment_type.is_empty() {
            None
        } else {
            Some(csv_post.sentiment_type)
        };
        
        // Parse toxicity tags from JSON
        let toxicity_tags: Vec<String> = if csv_post.toxicity_tags.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&csv_post.toxicity_tags)
                .map_err(|e| crate::AppError::InternalError(format!("Failed to parse toxicity tags from CSV: {}", e)))?
        };
        
        // Parse toxicity scores from JSON
        let toxicity_scores: Option<serde_json::Value> = if csv_post.toxicity_scores.is_empty() {
            None
        } else {
            Some(serde_json::from_str(&csv_post.toxicity_scores)
                .map_err(|e| crate::AppError::InternalError(format!("Failed to parse toxicity scores from CSV: {}", e)))?)
        };
        
        Ok(Post {
            id,
            title: csv_post.title,
            content: csv_post.content,
            author_id,
            author_username: csv_post.author_username,
            created_at,
            updated_at,
            comment_count: csv_post.comment_count,
            sentiment_score,
            sentiment_colors,
            sentiment_type,
            popularity_score: csv_post.popularity_score,
            is_blocked: csv_post.is_blocked,
            toxicity_tags,
            toxicity_scores,
        })
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

#[async_trait]
impl PostRepository for CsvPostRepository {
    async fn create_post(&self, post: &Post) -> Result<Post> {
        let mut cache = self.posts_cache.lock().unwrap();
        cache.insert(post.id, post.clone());
        drop(cache); // Release lock before file I/O
        
        self.save_posts_to_csv()?;
        Ok(post.clone())
    }

    async fn get_post_by_id(&self, id: Uuid) -> Result<Option<Post>> {
        let cache = self.posts_cache.lock().unwrap();
        Ok(cache.get(&id).cloned())
    }

    async fn get_posts_paginated(&self, limit: u32, offset: u32) -> Result<Vec<Post>> {
        let cache = self.posts_cache.lock().unwrap();
        let mut posts: Vec<Post> = cache.values().cloned().collect();
        
        // Sort by created_at descending (newest first)
        posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        let start = offset as usize;
        let end = std::cmp::min(start + limit as usize, posts.len());
        
        if start >= posts.len() {
            Ok(vec![])
        } else {
            Ok(posts[start..end].to_vec())
        }
    }

    async fn get_posts_by_popularity(&self, limit: u32, offset: u32) -> Result<Vec<Post>> {
        let cache = self.posts_cache.lock().unwrap();
        let mut posts: Vec<Post> = cache.values().cloned().collect();
        
        // Sort by popularity score descending
        posts.sort_by(|a, b| b.popularity_score.partial_cmp(&a.popularity_score).unwrap_or(std::cmp::Ordering::Equal));
        
        let start = offset as usize;
        let end = std::cmp::min(start + limit as usize, posts.len());
        
        if start >= posts.len() {
            Ok(vec![])
        } else {
            Ok(posts[start..end].to_vec())
        }
    }

    async fn update_post(&self, post: &Post) -> Result<Post> {
        let mut cache = self.posts_cache.lock().unwrap();
        cache.insert(post.id, post.clone());
        drop(cache); // Release lock before file I/O
        
        self.save_posts_to_csv()?;
        Ok(post.clone())
    }

    async fn delete_post(&self, id: Uuid) -> Result<()> {
        let mut cache = self.posts_cache.lock().unwrap();
        cache.remove(&id);
        drop(cache); // Release lock before file I/O
        
        self.save_posts_to_csv()
    }

    async fn increment_comment_count(&self, post_id: Uuid) -> Result<()> {
        let mut cache = self.posts_cache.lock().unwrap();
        if let Some(post) = cache.get_mut(&post_id) {
            post.comment_count += 1;
        }
        drop(cache); // Release lock before file I/O
        
        self.save_posts_to_csv()
    }

    async fn update_popularity_score(&self, post_id: Uuid, score: f64) -> Result<()> {
        let mut cache = self.posts_cache.lock().unwrap();
        if let Some(post) = cache.get_mut(&post_id) {
            post.popularity_score = score;
        }
        drop(cache); // Release lock before file I/O
        
        self.save_posts_to_csv()
    }

    async fn get_posts_with_old_sentiment_types(&self) -> Result<Vec<Post>> {
        let cache = self.posts_cache.lock().unwrap();
        let old_sentiment_types = ["happy", "excited", "calm"];
        
        let posts_with_old_sentiments: Vec<Post> = cache.values()
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
        let mut cache = self.posts_cache.lock().unwrap();
        if let Some(post) = cache.get_mut(&post_id) {
            post.sentiment_type = sentiment_type;
            post.sentiment_colors = sentiment_colors;
            post.sentiment_score = sentiment_score;
            post.updated_at = chrono::Utc::now();
        }
        drop(cache); // Release lock before file I/O
        
        self.save_posts_to_csv()
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