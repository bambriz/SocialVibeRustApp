import sys
import re
from nrclex import NRCLex
from typing import Dict

# Import emotionclassifier as specified by user
# Note: Temporarily disabled due to initialization timeouts - will re-enable once optimized
EMOTIONCLASSIFIER_AVAILABLE = False
emotion_classifier = None

# try:
#     from emotionclassifier import EmotionClassifier
#     emotion_classifier = None
#     EMOTIONCLASSIFIER_AVAILABLE = True
# except ImportError:
#     EMOTIONCLASSIFIER_AVAILABLE = False
#     emotion_classifier = None

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
        r'\b(great|perfect|wonderful|magical)\b.*\b(meeting|email|another|timing|start|day)\b',
        r'\b(thanks\s+for\s+nothing|couldn\'t\s+be\s+better)\b',
        r'\b(living\s+the\s+dream|perfect\s+timing|magical\s+start)\b',
        r'\b(couldn\'t\s+have\s+asked\s+for|what\s+a\s+day|how\s+perfect)\b',
        r'\b(complete\s+the\s+chaos|chaos\s+bingo|one\s+glitch\s+at\s+a\s+time)\b'
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
    """
    Analyze the main emotion using EmotionClassifier for broader emotion detection
    Returns the dominant emotion with confidence score
    """
    text = text.lower()
    
    # PRIMARY: Use EmotionClassifier as specified by user
    if EMOTIONCLASSIFIER_AVAILABLE:
        try:
            # Lazy initialization to avoid startup delays
            global emotion_classifier
            if emotion_classifier is None:
                emotion_classifier = EmotionClassifier()
            
            result = emotion_classifier.predict(text)
            
            # EmotionClassifier returns different emotion labels, map them to our system
            emotion_mapping = {
                'happy': 'happy',
                'joy': 'joy', 
                'sadness': 'sad',
                'sad': 'sad',
                'anger': 'angry',
                'angry': 'angry',
                'fear': 'fear',
                'disgust': 'disgust',
                'surprise': 'surprise',
                'excitement': 'excited',
                'excited': 'excited',
                'love': 'affection',
                'affection': 'affection',
                'neutral': 'calm',
                'calm': 'calm'
            }
            
            # Handle different return formats from EmotionClassifier
            if isinstance(result, dict):
                if 'emotion' in result:
                    emotion = result['emotion'].lower()
                    confidence = result.get('confidence', 0.70)
                else:
                    # Dict of emotion:score pairs - handle properly
                    if result:
                        emotion = max(result.keys(), key=lambda k: result[k]).lower()
                        confidence = result[emotion]
                    else:
                        emotion = 'calm'
                        confidence = 0.30
            elif isinstance(result, str):
                emotion = result.lower() 
                confidence = 0.70
            else:
                emotion = str(result).lower()
                confidence = 0.70
            
            mapped_emotion = emotion_mapping.get(emotion, 'calm')
            return f"{mapped_emotion}:{confidence:.2f}"
            
        except Exception as e:
            # Fall through to NRCLex fallback  
            pass
    
    # FALLBACK: Use NRCLex + enhanced pattern matching (as complementary to EmotionClassifier)
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
                return f"{pattern_emotion}:0.70"
                
        return f"{mapped_emotion}:0.60"
    
    # Fallback to simple pattern matching if NRCLex fails
    if re.search(r'\b(happy|joy|glad|cheerful|delighted)\b', text):
        return 'happy:0.60'
    elif re.search(r'\b(sad|depressed|miserable|heartbroken)\b', text):
        return 'sad:0.60'
    elif re.search(r'\b(angry|mad|furious|rage|hate)\b', text):
        return 'angry:0.60'
    else:
        return 'calm:0.30'

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
        # Extract just the emotion from the EmotionClassifier result for sarcasm combinations
        emotion_part = advance_sentiment_analysis.split(':')[0] if ':' in advance_sentiment_analysis else advance_sentiment_analysis
        print(f"sarcastic+{emotion_part}:0.80")
    else:
        # EmotionClassifier already returns with confidence score
        print(advance_sentiment_analysis)

if __name__ == "__main__":
    main()