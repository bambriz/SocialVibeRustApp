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
    pub user_service: std::sync::Arc<services::UserService>,
    pub post_service: std::sync::Arc<services::PostService>,
    pub comment_service: std::sync::Arc<services::CommentService>,
    pub sentiment_service: std::sync::Arc<services::SentimentService>,
    pub moderation_service: std::sync::Arc<services::ModerationService>,
    pub auth_service: std::sync::Arc<auth::AuthService>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let db = db::DatabaseClient::new(&config).await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Initialize services with repository dependencies
        let auth_service = std::sync::Arc::new(auth::AuthService::new(config.jwt_secret.clone()));
        let user_service = std::sync::Arc::new(services::UserService::new(db.user_repo.clone()));
        let sentiment_service = std::sync::Arc::new(services::SentimentService::new());
        let moderation_service = std::sync::Arc::new(services::ModerationService::new());
        let post_service = std::sync::Arc::new(services::PostService::new(
            db.post_repo.clone(),
            sentiment_service.clone(),
            moderation_service.clone()
        ));
        let comment_service = std::sync::Arc::new(services::CommentService::new(db.comment_repo.clone()));

        Ok(Self {
            config,
            db,
            user_service,
            post_service,
            comment_service,
            sentiment_service,
            moderation_service,
            auth_service,
        })
    }
}