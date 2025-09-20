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
pub use config::{AppConfig, PythonServerMode};

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
    pub vote_service: std::sync::Arc<services::VoteService>,
    pub python_manager: Option<std::sync::Arc<services::PythonManager>>,
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
        
        // Create CSV fallback repository
        let csv_fallback_repo = std::sync::Arc::new(db::repository::CsvPostRepository::new(None)); // Uses default "posts_backup.csv"
        
        let vote_service = std::sync::Arc::new(services::VoteService::new(db.vote_repo.clone()));
        
        let comment_service = std::sync::Arc::new(services::CommentService::new_with_ai(
            db.comment_repo.clone(),
            Some(sentiment_service.clone()),
            Some(moderation_service.clone()),
            Some(vote_service.clone())
        ));
        
        let post_service = std::sync::Arc::new(services::PostService::new_with_vote_service(
            db.post_repo.clone() as std::sync::Arc<dyn db::repository::PostRepository>,
            csv_fallback_repo,
            sentiment_service.clone(),
            moderation_service.clone(),
            vote_service.clone()
        ));

        // Initialize PythonManager based on configuration
        let python_manager = match config.python_server_mode {
            PythonServerMode::Subprocess => {
                let manager = services::PythonManager::new(None); // Use default config
                Some(std::sync::Arc::new(manager))
            }
            PythonServerMode::External => None,
        };

        Ok(Self {
            config,
            db,
            user_service,
            post_service,
            comment_service,
            sentiment_service,
            moderation_service,
            auth_service,
            vote_service,
            python_manager,
        })
    }
}