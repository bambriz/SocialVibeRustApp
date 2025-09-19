use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SentimentType {
    Sad,
    Angry,
    Sarcastic,
    Joy,        // Primary positive emotion (replaces Happy/Excited)
    Confused,
    Affection,
    Neutral,    // Fallback emotion (replaces Calm)
    Fear,
    Disgust,
    Surprise,
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
            SentimentType::Sad => "#1e3a8a".to_string(), // Dark blue - 😢
            SentimentType::Angry => "#dc2626".to_string(), // Red - 😠
            SentimentType::Sarcastic => "#7c3aed".to_string(), // Purple
            SentimentType::Joy => "#fbbf24".to_string(), // Bright yellow/gold - 😊 (replaces Happy/Excited)
            SentimentType::Confused => "#8b5cf6".to_string(), // Light purple
            SentimentType::Affection => "#ec4899".to_string(), // Pink
            SentimentType::Neutral => "#6b7280".to_string(), // Neutral gray (replaces Calm)
            SentimentType::Fear => "#374151".to_string(), // Dark grey - 😨
            SentimentType::Disgust => "#84cc16".to_string(), // Lime green - 🤢
            SentimentType::Surprise => "#f97316".to_string(), // Orange - 😲
        }
    }
    
    pub fn colors_array(&self) -> Vec<String> {
        vec![self.color_code()]
    }

    pub fn from_analysis(analysis_result: &str) -> Vec<Self> {
        // This will be implemented to parse sentiment analysis results from Python
        // For now, return a default
        vec![SentimentType::Neutral]
    }

    pub fn to_string(&self) -> String {
        match self {
            SentimentType::Sad => "sad".to_string(),
            SentimentType::Angry => "angry".to_string(),
            SentimentType::Sarcastic => "sarcastic".to_string(),
            SentimentType::Joy => "happy".to_string(), // Display as "happy" in frontend
            SentimentType::Confused => "confused".to_string(),
            SentimentType::Affection => "affection".to_string(),
            SentimentType::Neutral => "neutral".to_string(), // Replaces calm
            SentimentType::Fear => "fear".to_string(),
            SentimentType::Disgust => "disgust".to_string(),
            SentimentType::Surprise => "surprise".to_string(),
        }
    }
}