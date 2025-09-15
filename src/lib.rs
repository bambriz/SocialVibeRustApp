// Library modules for the social media application
pub mod config;
pub mod models;
pub mod db;
pub mod routes;
pub mod services;
pub mod auth;
pub mod error;

// Re-export commonly used types
pub use error::{AppError, Result};
pub use config::AppConfig;

// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: db::DatabaseClient,
    // TODO: Add service instances when implemented
    // pub user_service: Arc<services::UserService>,
    // pub post_service: Arc<services::PostService>,
    // pub comment_service: Arc<services::CommentService>,
    // pub sentiment_service: Arc<services::SentimentService>,
    // pub moderation_service: Arc<services::ModerationService>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let db = db::DatabaseClient::new(&config).await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(Self {
            config,
            db,
        })
    }
}