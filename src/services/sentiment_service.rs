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
        
        // Robust parsing: handle different output formats
        let clean_result = result.lines().last().unwrap_or("").trim();
        if clean_result.is_empty() {
            return Ok(vec![]); // Return empty instead of default Calm
        }
        
        // Parse the result (format: "sentiment_type:confidence")
        let parts: Vec<&str> = clean_result.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Ok(vec![]); // Return empty instead of default Calm
        }
        
        let sentiment_label = parts[0].to_lowercase(); // Make case-insensitive
        
        let sentiment_type = if sentiment_label.starts_with("sarcastic+") {
            // Handle sarcasm combinations like "sarcastic+happy"
            let base_sentiment = sentiment_label.strip_prefix("sarcastic+").unwrap_or("calm");
            let base_type = match base_sentiment {
                "happy" => SentimentType::Happy,
                "sad" => SentimentType::Sad,
                "angry" => SentimentType::Angry,
                "excited" => SentimentType::Excited,
                "confused" => SentimentType::Confused,
                "fear" => SentimentType::Fear,
                "calm" => SentimentType::Calm,
                "affection" => SentimentType::Affection,
                _ => return Ok(vec![]), // Return empty for unknown types
            };
            SentimentType::SarcasticCombination(Box::new(base_type))
        } else {
            match sentiment_label.as_str() {
                "happy" => SentimentType::Happy,
                "excited" => SentimentType::Excited,
                "confused" => SentimentType::Confused,
                "sad" => SentimentType::Sad,
                "angry" => SentimentType::Angry,
                "fear" => SentimentType::Fear,
                "calm" => SentimentType::Calm,
                "affection" => SentimentType::Affection,
                "sarcastic" => SentimentType::Sarcastic,
                _ => return Ok(vec![]), // Return empty for unknown types
            }
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
            .arg("python_scripts/custom_sentiment_analysis.py")
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