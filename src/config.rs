use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub cosmos_endpoint: Option<String>,
    pub cosmos_key: Option<String>,
    pub cosmos_database: String,
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            cosmos_endpoint: env::var("COSMOS_ENDPOINT").ok(),
            cosmos_key: env::var("COSMOS_KEY").ok(),
            cosmos_database: env::var("COSMOS_DATABASE").unwrap_or_else(|_| "social_media".to_string()),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
        }
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}