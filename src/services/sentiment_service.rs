use crate::models::{Sentiment, SentimentType};
use tokio::process::Command;
use std::process::Stdio;
use std::time::Duration;
use tokio::time::timeout;

pub struct SentimentService;

impl SentimentService {
    pub fn new() -> Self {
        Self
    }

    pub async fn analyze_sentiment(&self, text: &str) -> Result<Vec<Sentiment>, Box<dyn std::error::Error>> {
        // Add timeout to prevent hanging on model downloads
        let result = match timeout(Duration::from_secs(5), self.call_python_analyzer(text)).await {
            Ok(res) => res?,
            Err(_) => {
                eprintln!("Sentiment analysis timed out, returning default");
                return Ok(vec![Sentiment {
                    sentiment_type: SentimentType::Neutral,
                    confidence: 0.30,
                    color_code: SentimentType::Neutral.color_code(),
                }]);
            }
        };
        
        // Robust parsing: handle different output formats
        let clean_result = result.lines().last().unwrap_or("").trim();
        if clean_result.is_empty() {
            return Ok(vec![]); // Return empty instead of default Neutral
        }
        
        // Parse the result (format: "sentiment_type:confidence") with resilient parsing
        let (sentiment_label, confidence_str) = if let Some((label, conf)) = clean_result.split_once(':') {
            (label.trim().to_lowercase(), conf.trim())
        } else {
            // Handle missing confidence - default to 0.70
            (clean_result.trim().to_lowercase(), "0.70")
        };
        
        let sentiment_type = if sentiment_label.starts_with("sarcastic+") || sentiment_label.starts_with("sarcasm+") {
            // Handle sarcasm combinations like "sarcastic+happy" or "sarcasm+happy"
            // Now return standalone Sarcastic instead of combo
            SentimentType::Sarcastic
        } else if sentiment_label.starts_with("affectionate+") {
            // Handle affectionate combinations like "affectionate+happy" 
            // Now return standalone Affection instead of combo
            SentimentType::Affection
        } else {
            match sentiment_label.as_str() {
                "happy" | "joy" => SentimentType::Joy, // Map happy to Joy
                "confused" => SentimentType::Confused,
                "sad" => SentimentType::Sad,
                "angry" => SentimentType::Angry,
                "fear" => SentimentType::Fear,
                "disgust" => SentimentType::Disgust,
                "surprise" => SentimentType::Surprise,
                "neutral" => SentimentType::Neutral,
                "affection" => SentimentType::Affection,
                "sarcastic" => SentimentType::Sarcastic,
                _ => SentimentType::Neutral // Default to Neutral instead of returning empty
            }
        };
        
        let confidence: f64 = confidence_str.parse().unwrap_or(0.70);
        
        let sentiment = Sentiment {
            sentiment_type: sentiment_type.clone(),
            confidence,
            color_code: sentiment_type.color_code(),
        };

        Ok(vec![sentiment])
    }

    // Method to call Python sentiment analysis server (persistent, faster)
    async fn call_python_analyzer(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
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
            
            let request_payload = serde_json::json!({ "text": text });
            println!("🔄 DIAGNOSTIC: Making request to Python server");
            println!("   📤 URL: http://127.0.0.1:8001/analyze");
            println!("   📄 Text: \"{}{}\"", 
                if text.len() > 100 { &text[..100] } else { text },
                if text.len() > 100 { "..." } else { "" });
            println!("   🔢 Attempt: {}/{}", attempts, max_attempts);
            
            match client
                .post("http://127.0.0.1:8001/analyze")  // Use IPv4 explicitly
                .json(&request_payload)
                .send()
                .await 
            {
                Ok(response) if response.status().is_success() => {
                    println!("   ✅ Response received successfully");
                    let result: serde_json::Value = response.json().await?;
                    
                    println!("   📥 Raw response: {}", result);
                    
                    // Extract sentiment info from server response
                    let sentiment_type = result["sentiment_type"].as_str().unwrap_or("neutral");
                    let confidence = result["confidence"].as_f64().unwrap_or(0.5);
                    let is_combo = result["is_combo"].as_bool().unwrap_or(false);
                    
                    println!("   🎯 Parsed sentiment: {} (confidence: {:.2})", sentiment_type, confidence);
                    if is_combo {
                        let combo_type = result["combo_type"].as_str().unwrap_or("unknown");
                        println!("   🎭 Combo detected: {}", combo_type);
                    }
                    
                    // Return in the expected format for existing parsing logic
                    let formatted_result = format!("{}:{:.2}", sentiment_type, confidence);
                    println!("   📤 Returning to Rust: {}", formatted_result);
                    return Ok(formatted_result);
                },
                Ok(response) => {
                    // Server responded but with error status
                    println!("   ❌ Server error response: {}", response.status());
                    if attempts == max_attempts {
                        eprintln!("Python sentiment server responded with error after {} attempts", max_attempts);
                        break;
                    }
                },
                Err(e) => {
                    // Connection failed
                    println!("   ❌ Connection failed: {}", e);
                    if attempts == max_attempts {
                        eprintln!("Python sentiment server connection failed after {} attempts", max_attempts);
                        break;
                    }
                    // Brief delay before retry
                    tokio::time::sleep(std::time::Duration::from_millis(100 * attempts as u64)).await;
                }
            }
        }
        
        // Fallback to script if server is not available
        eprintln!("Python sentiment server not available, falling back to script");
        let output = Command::new("python3")
            .arg("-u")
            .arg("python_scripts/custom_sentiment_analysis.py")
            .arg(text)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(format!("Both sentiment server and script failed: {}", String::from_utf8_lossy(&output.stderr)).into())
        }
    }
}