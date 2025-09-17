use tokio::process::Command;
use std::process::Stdio;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct ModerationResult {
    pub is_blocked: bool,
    pub violation_type: Option<String>,
    pub details: Option<String>,
}

pub struct ModerationService;

impl ModerationService {
    pub fn new() -> Self {
        Self
    }

    pub async fn check_content(&self, text: &str) -> Result<ModerationResult, Box<dyn std::error::Error>> {
        // Add timeout to prevent hanging on model downloads
        let result = match timeout(Duration::from_secs(5), self.call_python_moderator(text)).await {
            Ok(res) => res?,
            Err(_) => {
                eprintln!("Content moderation timed out, failing open for safety");
                return Ok(ModerationResult {
                    is_blocked: false,
                    violation_type: Some("moderation_timeout".to_string()),
                    details: Some("system_timeout".to_string()),
                });
            }
        };
        let trimmed = result.trim();
        
        if trimmed == "allowed" {
            return Ok(ModerationResult {
                is_blocked: false,
                violation_type: None,
                details: None,
            });
        }
        
        if trimmed.starts_with("blocked:") {
            let parts: Vec<&str> = trimmed[8..].split(':').collect(); // Remove "blocked:" prefix
            let violation_type = parts[0].to_string();
            let details = if parts.len() > 1 {
                Some(parts[1..].join(":"))
            } else {
                None
            };
            
            Ok(ModerationResult {
                is_blocked: true,
                violation_type: Some(violation_type),
                details,
            })
        } else {
            // Fallback for old format or unexpected responses
            Ok(ModerationResult {
                is_blocked: trimmed == "blocked",
                violation_type: Some("unknown_violation".to_string()),
                details: None,
            })
        }
    }

    async fn call_python_moderator(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Configure client with timeouts and retry logic
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_millis(500))
            .timeout(std::time::Duration::from_secs(2))
            .build()?;
        
        // Try connecting to persistent Python server with retry
        let mut attempts = 0;
        let max_attempts = 3;
        
        while attempts < max_attempts {
            attempts += 1;
            
            match client
                .post("http://127.0.0.1:8001/moderate")  // Use IPv4 explicitly
                .json(&serde_json::json!({ "text": text }))
                .send()
                .await 
            {
                Ok(response) if response.status().is_success() => {
                let result: serde_json::Value = response.json().await?;
                
                let is_blocked = result["is_blocked"].as_bool().unwrap_or(false);
                
                    if is_blocked {
                        let violation_type = result["violation_type"].as_str().unwrap_or("unknown");
                        let details = result["details"].as_str().unwrap_or("");
                        return Ok(format!("blocked:{}:{}", violation_type, details));
                    } else {
                        return Ok("allowed".to_string());
                    }
                },
                Ok(_) => {
                    // Server responded but with error status
                    if attempts == max_attempts {
                        eprintln!("Python moderation server responded with error after {} attempts", max_attempts);
                        break;
                    }
                },
                Err(_) => {
                    // Connection failed
                    if attempts == max_attempts {
                        eprintln!("Python moderation server connection failed after {} attempts", max_attempts);
                        break;
                    }
                    // Brief delay before retry
                    tokio::time::sleep(std::time::Duration::from_millis(100 * attempts as u64)).await;
                }
            }
        }
        
        // Fallback to script if server is not available
        eprintln!("Python moderation server not available, falling back to script");
        let output = Command::new("python3")
            .arg("-u")
            .arg("python_scripts/content_moderation.py")
            .arg(text)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(format!("Both moderation server and script failed: {}", String::from_utf8_lossy(&output.stderr)).into())
        }
    }
}