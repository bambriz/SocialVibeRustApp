pub mod cosmos;
pub mod repository;

// Database connection and state management
use crate::config::AppConfig;
use repository::{MockUserRepository, MockPostRepository, MockCommentRepository};
use std::sync::Arc;

#[derive(Clone)]
pub struct DatabaseClient {
    // Mock repositories for development
    pub user_repo: Arc<MockUserRepository>,
    pub post_repo: Arc<MockPostRepository>, 
    pub comment_repo: Arc<MockCommentRepository>,
    // TODO: Add Cosmos DB client when reintroduced
    // pub cosmos_client: CosmosClient,
}

impl DatabaseClient {
    pub async fn new(_config: &AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            user_repo: Arc::new(MockUserRepository),
            post_repo: Arc::new(MockPostRepository),
            comment_repo: Arc::new(MockCommentRepository),
        })
    }

    pub async fn health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement real health check when Cosmos is integrated
        Ok(())
    }
}