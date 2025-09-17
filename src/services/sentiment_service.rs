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

    // Method to call Python sentiment analysis script
    async fn call_python_analyzer(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("python3")
            .arg("-u")  // Unbuffered output
            .arg("python_scripts/custom_sentiment_analysis.py")
            .arg(text)
            .stdin(Stdio::null())  // Prevent stdin waits
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