use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub python_server_mode: PythonServerMode,
}

#[derive(Debug, Clone)]
pub enum PythonServerMode {
    Subprocess,
    External,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let python_mode = env::var("PYTHON_SERVER_MODE")
            .unwrap_or_else(|_| "subprocess".to_string())
            .to_lowercase();
        
        let python_server_mode = match python_mode.as_str() {
            "external" => PythonServerMode::External,
            _ => PythonServerMode::Subprocess, // Default to subprocess mode
        };
        
        Self {
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("PORT")
                .or_else(|_| env::var("SERVER_PORT"))
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
            python_server_mode,
        }
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}