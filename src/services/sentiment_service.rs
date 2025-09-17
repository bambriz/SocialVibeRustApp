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
                    sentiment_type: SentimentType::Calm,
                    confidence: 0.30,
                    color_code: SentimentType::Calm.color_code(),
                }]);
            }
        };
        
        // Robust parsing: handle different output formats
        let clean_result = result.lines().last().unwrap_or("").trim();
        if clean_result.is_empty() {
            return Ok(vec![]); // Return empty instead of default Calm
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
            let base_sentiment = sentiment_label.strip_prefix("sarcastic+")
                .or_else(|| sentiment_label.strip_prefix("sarcasm+"))
                .unwrap_or("calm");
            let base_type = match base_sentiment {
                "happy" => SentimentType::Happy,
                "joy" => SentimentType::Joy,
                "sad" => SentimentType::Sad,
                "angry" => SentimentType::Angry,
                "excited" => SentimentType::Excited,
                "confused" => SentimentType::Confused,
                "fear" => SentimentType::Fear,
                "disgust" => SentimentType::Disgust,
                "surprise" => SentimentType::Surprise,
                "calm" => SentimentType::Calm,
                "affection" => SentimentType::Affection,
                _ => SentimentType::Calm, // Default to Calm instead of returning empty
            };
            SentimentType::SarcasticCombination(Box::new(base_type))
        } else {
            match sentiment_label.as_str() {
                "happy" => SentimentType::Happy,
                "joy" => SentimentType::Joy,
                "excited" => SentimentType::Excited,
                "confused" => SentimentType::Confused,
                "sad" => SentimentType::Sad,
                "angry" => SentimentType::Angry,
                "fear" => SentimentType::Fear,
                "disgust" => SentimentType::Disgust,
                "surprise" => SentimentType::Surprise,
                "calm" => SentimentType::Calm,
                "affection" => SentimentType::Affection,
                "sarcastic" => SentimentType::Sarcastic,
                _ => SentimentType::Calm // Default to Calm instead of returning empty
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
            
            match client
                .post("http://127.0.0.1:8001/analyze")  // Use IPv4 explicitly
                .json(&serde_json::json!({ "text": text }))
                .send()
                .await 
            {
                Ok(response) if response.status().is_success() => {
            let result: serde_json::Value = response.json().await?;
            
            // Extract sentiment info from server response
            let sentiment_type = result["sentiment_type"].as_str().unwrap_or("calm");
            let confidence = result["confidence"].as_f64().unwrap_or(0.5);
            
                    // Return in the expected format for existing parsing logic
                    return Ok(format!("{}:{:.2}", sentiment_type, confidence));
                },
                Ok(_) => {
                    // Server responded but with error status
                    if attempts == max_attempts {
                        eprintln!("Python sentiment server responded with error after {} attempts", max_attempts);
                        break;
                    }
                },
                Err(_) => {
                    // Connection failed
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