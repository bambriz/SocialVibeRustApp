pub mod cosmos;
pub mod repository;

// Database connection and state management
use crate::config::AppConfig;
use repository::{MockUserRepository, MockPostRepository, MockCommentRepository, MockVoteRepository, CsvUserRepository, UserRepository, VoteRepository};
use std::sync::Arc;

#[derive(Clone)]
pub struct DatabaseClient {
    // Repositories for development with CSV persistence for users
    pub user_repo: Arc<dyn UserRepository>,
    pub post_repo: Arc<MockPostRepository>, 
    pub comment_repo: Arc<MockCommentRepository>,
    pub vote_repo: Arc<dyn VoteRepository>,
    // TODO: Add Cosmos DB client when reintroduced
    // pub cosmos_client: CosmosClient,
}

impl DatabaseClient {
    pub async fn new(_config: &AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            user_repo: Arc::new(CsvUserRepository::new(None)), // Uses default "users_backup.csv"
            post_repo: Arc::new(MockPostRepository::new()),
            comment_repo: Arc::new(MockCommentRepository::new()),
            vote_repo: Arc::new(MockVoteRepository::new()) as Arc<dyn VoteRepository>,
        })
    }

    pub async fn health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement real health check when Cosmos is integrated
        Ok(())
    }
}