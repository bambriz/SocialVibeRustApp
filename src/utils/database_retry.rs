use std::time::Duration;
use tokio::time::sleep;
use crate::{AppError, Result};

/// Configuration for database operation retries
pub struct DatabaseRetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for DatabaseRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry a database operation with exponential backoff
pub async fn retry_database_operation<F, Fut, T>(
    operation: F,
    config: DatabaseRetryConfig,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = config.initial_delay_ms;
    let mut last_error = AppError::DatabaseError("Operation failed".to_string());

    for attempt in 1..=config.max_retries {
        tracing::debug!("🔄 DATABASE_RETRY: Attempt {}/{}", attempt, config.max_retries);
        
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    tracing::info!("✅ DATABASE_RETRY: Operation succeeded on attempt {}", attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                last_error = e;
                
                if attempt < config.max_retries {
                    tracing::warn!("⚠️ DATABASE_RETRY: Attempt {} failed: {}, retrying in {}ms", 
                                  attempt, last_error, delay);
                    
                    sleep(Duration::from_millis(delay)).await;
                    
                    // Exponential backoff with jitter
                    delay = ((delay as f64) * config.backoff_multiplier) as u64;
                    delay = delay.min(config.max_delay_ms);
                    
                    // Add jitter (±10%) - using simple timestamp-based jitter
                    let timestamp_jitter = (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or(Duration::from_millis(0))
                        .as_millis() % 100) as f64;
                    let jitter = (delay as f64) * 0.1 * ((timestamp_jitter / 100.0) - 0.5) * 2.0;
                    delay = ((delay as f64) + jitter).max(10.0) as u64;
                }
            }
        }
    }
    
    tracing::error!("❌ DATABASE_RETRY: All {} attempts failed, last error: {}", 
                   config.max_retries, last_error);
    Err(last_error)
}

/// Convenience function for operations that might need retrying
pub async fn with_database_retry<F, Fut, T>(operation: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    retry_database_operation(operation, DatabaseRetryConfig::default()).await
}