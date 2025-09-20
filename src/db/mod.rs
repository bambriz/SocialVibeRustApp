pub mod repository;
pub mod postgres;

// Database connection and state management
use crate::config::AppConfig;
use repository::{UserRepository, PostRepository, CommentRepository, VoteRepository};
use postgres::PostgresDatabase;
use std::sync::Arc;
use std::env;

#[derive(Clone)]
pub struct DatabaseClient {
    // PostgreSQL repositories for production-ready persistence
    pub user_repo: Arc<dyn UserRepository>,
    pub post_repo: Arc<dyn PostRepository>, 
    pub comment_repo: Arc<dyn CommentRepository>,
    pub vote_repo: Arc<dyn VoteRepository>,
}

impl DatabaseClient {
    pub async fn new(_config: &AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Get DATABASE_URL from environment
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable not set")?;
            
        // Create PostgreSQL database connection
        let postgres_db = PostgresDatabase::new(&database_url).await
            .map_err(|e| format!("Failed to connect to PostgreSQL: {}", e))?;
            
        Ok(Self {
            user_repo: Arc::new(postgres_db.user_repo()) as Arc<dyn UserRepository>,
            post_repo: Arc::new(postgres_db.post_repo()) as Arc<dyn PostRepository>,
            comment_repo: Arc::new(postgres_db.comment_repo()) as Arc<dyn CommentRepository>,
            vote_repo: Arc::new(postgres_db.vote_repo()) as Arc<dyn VoteRepository>,
        })
    }

    pub async fn health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Simple health check - confirms database client is initialized
        // Note: Does not verify active connection - implement actual DB ping if needed
        Ok(())
    }
}