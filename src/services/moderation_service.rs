use tokio::process::Command;
use std::process::Stdio;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ModerationResult {
    pub is_blocked: bool,
    pub violation_type: Option<String>,
    pub details: Option<String>,
    pub toxicity_tags: Vec<String>,
    pub all_scores: Option<serde_json::Value>,
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
                    toxicity_tags: Vec::new(),
                    all_scores: None,
                });
            }
        };
        
        // Check if the result is in the new JSON format
        if result.starts_with("new_format:") {
            // Parse the JSON response from the new format
            let json_str = &result[11..]; // Remove "new_format:" prefix
            match serde_json::from_str::<serde_json::Value>(json_str) {
                Ok(parsed) => {
                    let is_blocked = parsed["is_blocked"].as_bool().unwrap_or(false);
                    let violation_type = parsed["violation_type"].as_str().map(|s| s.to_string());
                    let details = parsed["details"].as_str().map(|s| s.to_string());
                    
                    // Extract toxicity tags
                    let toxicity_tags: Vec<String> = parsed["toxicity_tags"]
                        .as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default();
                    
                    // Extract all scores
                    let all_scores = parsed["all_scores"].clone();
                    
                    return Ok(ModerationResult {
                        is_blocked,
                        violation_type,
                        details,
                        toxicity_tags,
                        all_scores: if all_scores.is_null() { None } else { Some(all_scores) },
                    });
                }
                Err(e) => {
                    eprintln!("Failed to parse new format JSON: {}", e);
                    // Fall through to old format parsing
                }
            }
        }
        
        // Handle old string-based format for backward compatibility
        let trimmed = result.trim();
        
        if trimmed == "allowed" {
            return Ok(ModerationResult {
                is_blocked: false,
                violation_type: None,
                details: None,
                toxicity_tags: Vec::new(),
                all_scores: None,
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
                toxicity_tags: Vec::new(), // Fallback format doesn't have toxicity tags
                all_scores: None, // Fallback format doesn't have all scores
            })
        } else {
            // Fallback for old format or unexpected responses
            Ok(ModerationResult {
                is_blocked: trimmed == "blocked",
                violation_type: Some("unknown_violation".to_string()),
                details: None,
                toxicity_tags: Vec::new(), // Fallback format doesn't have toxicity tags
                all_scores: None, // Fallback format doesn't have all scores
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
                
                // Check if the response has the new format with toxicity_tags and all_scores
                if result.get("toxicity_tags").is_some() && result.get("all_scores").is_some() {
                    // NEW FORMAT: Parse the enhanced response
                    let is_blocked = result["is_blocked"].as_bool().unwrap_or(false);
                    let violation_type = result["violation_type"].as_str().map(|s| s.to_string());
                    let details = result["details"].as_str().map(|s| s.to_string());
                    
                    // Extract toxicity tags (new field)
                    let toxicity_tags: Vec<String> = result["toxicity_tags"]
                        .as_array()
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default();
                    
                    // Extract all scores for diagnostic data (new field)
                    let all_scores = result["all_scores"].clone();
                    
                    let moderation_result = ModerationResult {
                        is_blocked,
                        violation_type,
                        details,
                        toxicity_tags,
                        all_scores: if all_scores.is_null() { None } else { Some(all_scores) },
                    };
                    
                    return Ok(format!("new_format:{}", serde_json::to_string(&moderation_result).unwrap_or_default()));
                } else {
                    // OLD FORMAT: Backward compatibility
                    let is_blocked = result["is_blocked"].as_bool().unwrap_or(false);
                    
                    if is_blocked {
                        let violation_type = result["violation_type"].as_str().unwrap_or("unknown");
                        let details = result["details"].as_str().unwrap_or("");
                        return Ok(format!("blocked:{}:{}", violation_type, details));
                    } else {
                        return Ok("allowed".to_string());
                    }
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