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
        let output = Command::new("python3")
            .arg("-u")  // Unbuffered output
            .arg("python_scripts/content_moderation.py")
            .arg(text)
            .stdin(Stdio::null())  // Prevent stdin waits
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(format!("Python content moderation failed: {}", String::from_utf8_lossy(&output.stderr)).into())
        }
    }
}