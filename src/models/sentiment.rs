use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SentimentType {
    Sad,
    Angry,
    Sarcastic,
    Happy,
    Joy,        // Distinct from Happy - more intense positive emotion
    Excited,
    Confused,
    Affection,
    Calm,
    Fear,
    Disgust,    // New emotion type
    Surprise,   // New emotion type
    // Mixed sentiments (e.g., Sarcastic + Happy, Affectionate + Sad)
    SarcasticCombination(Box<SentimentType>),
    AffectionateCombination(Box<SentimentType>),
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
            SentimentType::Happy => "#fbbf24".to_string(), // Bright yellow/gold
            SentimentType::Joy => "#22d3ee".to_string(), // Bright cyan - 😊 
            SentimentType::Excited => "#f59e0b".to_string(), // Bright orange
            SentimentType::Confused => "#8b5cf6".to_string(), // Light purple
            SentimentType::Affection => "#ec4899".to_string(), // Pink
            SentimentType::Calm => "#059669".to_string(), // Green
            SentimentType::Fear => "#374151".to_string(), // Dark grey - 😨
            SentimentType::Disgust => "#84cc16".to_string(), // Lime green - 🤢
            SentimentType::Surprise => "#f97316".to_string(), // Orange - 😲
            SentimentType::SarcasticCombination(base_type) => {
                // Create a gradient effect by combining sarcasm purple with base sentiment
                format!("linear-gradient(45deg, #7c3aed, {})", base_type.color_code())
            }
            SentimentType::AffectionateCombination(base_type) => {
                // Create a gradient effect by combining affection pink with base sentiment
                format!("linear-gradient(45deg, #ec4899, {})", base_type.color_code())
            }
        }
    }
    
    pub fn colors_array(&self) -> Vec<String> {
        match self {
            SentimentType::SarcasticCombination(base_type) => {
                vec!["#7c3aed".to_string(), base_type.color_code()]
            }
            SentimentType::AffectionateCombination(base_type) => {
                vec!["#ec4899".to_string(), base_type.color_code()]
            }
            _ => vec![self.color_code()]
        }
    }

    pub fn from_analysis(analysis_result: &str) -> Vec<Self> {
        // This will be implemented to parse sentiment analysis results from Python
        // For now, return a default
        vec![SentimentType::Calm]
    }
}