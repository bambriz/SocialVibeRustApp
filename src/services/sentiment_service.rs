use crate::models::{Sentiment, SentimentType};
use tokio::process::Command;
use std::process::Stdio;

pub struct SentimentService;

impl SentimentService {
    pub fn new() -> Self {
        Self
    }

    pub async fn analyze_sentiment(&self, text: &str) -> Result<Vec<Sentiment>, Box<dyn std::error::Error>> {
        let result = self.call_python_analyzer(text).await?;
        
        // Parse the result (format: "sentiment_type:confidence")
        let parts: Vec<&str> = result.trim().split(':').collect();
        if parts.len() != 2 {
            return Ok(vec![Sentiment {
                sentiment_type: SentimentType::Calm,
                confidence: 0.5,
                color_code: SentimentType::Calm.color_code(),
            }]);
        }
        
        let sentiment_type = match parts[0] {
            "happy" => SentimentType::Happy,
            "sad" => SentimentType::Sad,
            "angry" => SentimentType::Angry,
            "fear" => SentimentType::Fear,
            "calm" => SentimentType::Calm,
            "affection" => SentimentType::Affection,
            "sarcastic" => SentimentType::Sarcastic,
            _ => SentimentType::Calm,
        };
        
        let confidence: f64 = parts[1].parse().unwrap_or(0.5);
        
        let sentiment = Sentiment {
            sentiment_type: sentiment_type.clone(),
            confidence,
            color_code: sentiment_type.color_code(),
        };

        Ok(vec![sentiment])
    }

    // Method to call Python sentiment analysis script
    async fn call_python_analyzer(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("python3")
            .arg("python_scripts/sentiment_analysis.py")
            .arg(text)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(format!("Python sentiment analysis failed: {}", String::from_utf8_lossy(&output.stderr)).into())
        }
    }
}