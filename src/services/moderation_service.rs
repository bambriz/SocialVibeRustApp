use tokio::process::Command;
use std::process::Stdio;

pub struct ModerationService;

impl ModerationService {
    pub fn new() -> Self {
        Self
    }

    pub async fn check_content(&self, text: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let result = self.call_python_moderator(text).await?;
        let is_blocked = result.trim() == "blocked";
        Ok(is_blocked)
    }

    async fn call_python_moderator(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("python3")
            .arg("python_scripts/content_moderation.py")
            .arg(text)
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