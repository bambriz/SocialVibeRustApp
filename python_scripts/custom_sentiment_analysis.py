import sys
import re
from nrclex import NRCLex
from typing import Dict

def sarcasm_affection_analysis(text):
    """
    Initial sentiment analysis to detect sarcasm or affection
    Returns: "sarcasm", "affection", or "neutral"
    """
    text_lower = text.lower()
    emotions = NRCLex(text_lower)
    # sarcasm is detected by contradictory emotions such as joy and anger being near each other in value
    # out of emotions.top_emotions is a list of tuples with tuple index 0 being the emotion and index 1 being the value
    emotion_dict: Dict[str,float] = dict(emotions.top_emotions)  # type: ignore
    
    # Enhanced sarcasm detection with pattern matching (second pass as requested)
    sarcasm_patterns = [
        r'\b(oh\s+great|oh\s+really|obviously|of\s+course|sure\s+thing)\b',
        r'\b(yeah\s+right|as\s+if|like\s+that|totally)\b',
        r'\b(just\s+perfect|just\s+great|how\s+wonderful|absolutely\s+perfect)\b',
        r'\b(great|perfect|wonderful)\b.*\b(meeting|email|another)\b',
        r'\b(thanks\s+for\s+nothing|couldn\'t\s+be\s+better)\b'
    ]
    
    pattern_sarcasm_detected = any(re.search(pattern, text_lower) for pattern in sarcasm_patterns)
    
    # Your original sarcasm detection logic with enhanced thresholds
    if "joy" in emotion_dict and "anger" in emotion_dict:
        if abs(emotion_dict["joy"] - emotion_dict["anger"]) < 0.15:  # Enhanced threshold
            return "sarcasm"
    if "sadness" in emotion_dict and "joy" in emotion_dict:
        if abs(emotion_dict["sadness"] - emotion_dict["joy"]) < 0.15:  # Enhanced threshold
            return "sarcasm"
    if "disgust" in emotion_dict and "joy" in emotion_dict:
        if abs(emotion_dict["disgust"] - emotion_dict["joy"]) < 0.15:  # Enhanced threshold
            return "sarcasm"
    
    # Pattern-based sarcasm detection (second pass enhancement) - prioritize this
    if pattern_sarcasm_detected:
        return "sarcasm"
    
    # Enhanced affection detection with pattern matching
    affection_patterns = [
        r'\b(love\s+you|adore\s+you|cherish\s+you|treasure\s+you)\b',
        r'\b(mean\s+everything|mean\s+the\s+world|so\s+important)\b',
        r'\b(darling|sweetheart|honey|dear|beloved)\b'
    ]
    
    pattern_affection_detected = any(re.search(pattern, text_lower) for pattern in affection_patterns)
    
    # affection is detected by presence of positive emotions like joy, trust, and anticipation
    if "joy" in emotion_dict and "trust" in emotion_dict:
        if emotion_dict["joy"] > 0.15 and emotion_dict["trust"] > 0.15 and "sadness" not in emotion_dict and "anger" not in emotion_dict and "fear" not in emotion_dict and "disgust" not in emotion_dict:  # Enhanced thresholds
            return "affection" if abs(emotion_dict["joy"] - emotion_dict["trust"]) < 0.25 else "neutral"  # Enhanced difference threshold
    
    # Pattern-based affection fallback (second pass enhancement) - but not for celebration/party words
    celebration_words = r'\b(celebration|celebrate|party|wonderful)\b'
    if pattern_affection_detected and ("joy" in emotion_dict or "trust" in emotion_dict) and not re.search(celebration_words, text_lower):
        return "affection"

    return "neutral"

def main_emotion_analysis(text):
    """Enhanced emotion analysis using NRCLex + pattern matching fallback"""
    text = text.lower()
    
    # Use NRCLex for base emotion detection (faster than EmotionClassifier)
    emotions = NRCLex(text)
    emotion_scores = emotions.raw_emotion_scores  # type: ignore
    
    if emotion_scores:
        # Get the dominant emotion from NRCLex
        dominant_emotion = max(emotion_scores, key=emotion_scores.get)
        
        # Map NRCLex emotions to your expanded set
        emotion_mapping = {
            'joy': 'joy',
            'sadness': 'sad',
            'anger': 'angry',
            'fear': 'fear',
            'disgust': 'disgust',
            'surprise': 'surprise',
            'anticipation': 'excited',
            'trust': 'affection',
            'positive': 'happy',
            'negative': 'sad'
        }
        
        mapped_emotion = emotion_mapping.get(dominant_emotion, 'calm')
        
        # Pattern-based enhancement for missing emotions (second pass)
        text_patterns = {
            'joy': [r'\b(celebration|celebrate|party|joyful|jubilant|triumph|victory)\b'],
            'disgust': [r'\b(disgusting|gross|revolting|nauseating|yuck|ew|horrible)\b'],
            'surprise': [r'\b(surprised|shocked|wow|omg|unexpected|amazing|incredible)\b'],
            'excited': [r'\b(excited|thrilled|pumped|hyped|stoked|energized)\b'],
            'confused': [r'\b(confused|puzzled|bewildered|don\'t\s+understand)\b']
        }
        
        for pattern_emotion, patterns in text_patterns.items():
            if any(re.search(pattern, text) for pattern in patterns):
                return pattern_emotion
                
        return mapped_emotion
    
    # Fallback to simple pattern matching if NRCLex fails
    if re.search(r'\b(happy|joy|glad|cheerful|delighted)\b', text):
        return 'happy'
    elif re.search(r'\b(sad|depressed|miserable|heartbroken)\b', text):
        return 'sad'
    elif re.search(r'\b(angry|mad|furious|rage|hate)\b', text):
        return 'angry'
    else:
        return 'calm'

def main():
    """Main function for command line usage"""
    if len(sys.argv) != 2:
        print("Usage: python custom_sentiment_analysis.py <text>")
        sys.exit(1)

    text = sys.argv[1]
    initial_sentiment_analysis = sarcasm_affection_analysis(text)
    if initial_sentiment_analysis in ["affection"]:
        print(f"{initial_sentiment_analysis}:0.75")
        return
    advance_sentiment_analysis = main_emotion_analysis(text)
    if initial_sentiment_analysis == "sarcasm":
        print(f"sarcastic+{advance_sentiment_analysis}:0.80")
    else:
        # Add confidence scores based on emotion strength
        confidence = 0.70 if advance_sentiment_analysis in ['angry', 'sad', 'happy', 'excited'] else 0.50
        print(f"{advance_sentiment_analysis}:{confidence:.2f}")

if __name__ == "__main__":
    main()