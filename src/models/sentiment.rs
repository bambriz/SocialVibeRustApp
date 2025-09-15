use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SentimentType {
    Sad,
    Angry,
    Sarcastic,
    Happy,
    Affection,
    Calm,
    Fear,
    // Mixed sentiments (e.g., Sarcastic + Angry)
    Mixed(Vec<SentimentType>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentiment {
    pub sentiment_type: SentimentType,
    pub confidence: f64, // 0.0 to 1.0
    pub color_code: String, // Hex color code
}

impl SentimentType {
    pub fn color_code(&self) -> String {
        match self {
            SentimentType::Sad => "#1e3a8a".to_string(), // Dark blue
            SentimentType::Angry => "#dc2626".to_string(), // Red
            SentimentType::Sarcastic => "#7c3aed".to_string(), // Purple
            SentimentType::Happy => "#fbbf24".to_string(), // Bright yellow
            SentimentType::Affection => "#ec4899".to_string(), // Pink
            SentimentType::Calm => "#059669".to_string(), // Green
            SentimentType::Fear => "#374151".to_string(), // Dark grey
            SentimentType::Mixed(types) => {
                // For mixed sentiments, create a gradient or return primary sentiment
                if let Some(primary) = types.first() {
                    primary.color_code()
                } else {
                    "#6b7280".to_string() // Default grey
                }
            }
        }
    }

    pub fn from_analysis(analysis_result: &str) -> Vec<Self> {
        // This will be implemented to parse sentiment analysis results from Python
        // For now, return a default
        vec![SentimentType::Calm]
    }
}