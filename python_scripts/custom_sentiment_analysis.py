import sys
import re
from typing import Dict, List

# Fallback implementation when heavy libraries are slow to load
def lightweight_nrc_analysis(text: str) -> Dict[str, float]:
    """
    Lightweight emotion detection based on NRC-style patterns
    """
    text_lower = text.lower()
    emotions = {
        'joy': 0.0,
        'anger': 0.0,
        'sadness': 0.0,
        'fear': 0.0,
        'trust': 0.0,
        'disgust': 0.0,
        'surprise': 0.0,
        'anticipation': 0.0
    }
    
    # Pattern-based emotion scoring (similar to NRCLex but lighter)
    emotion_patterns = {
        'joy': [r'\b(happy|joy|joyful|delighted|cheerful|glad|pleased|wonderful|amazing|fantastic|great|excellent)\b'],
        'anger': [r'\b(angry|mad|furious|rage|hate|pissed|irritated|annoyed|frustrated|fed\s+up|ridiculous|stupid)\b'],
        'sadness': [r'\b(sad|depressed|miserable|heartbroken|crying|tears|hurt|disappointed|awful|terrible)\b'],
        'fear': [r'\b(scared|afraid|terrified|worried|anxious|nervous|fear|dread|panic)\b'],
        'trust': [r'\b(trust|believe|faith|confidence|reliable|honest|caring|love|affection)\b'],
        'disgust': [r'\b(disgusting|gross|revolting|sick|nauseating|yuck|ew|horrible|awful|nasty)\b'],
        'surprise': [r'\b(surprised|shocked|wow|omg|amazing|incredible|unexpected|unbelievable)\b'],
        'anticipation': [r'\b(excited|looking\s+forward|can\'t\s+wait|eager|anticipating|hope|expect)\b']
    }
    
    for emotion, patterns in emotion_patterns.items():
        for pattern in patterns:
            matches = len(re.findall(pattern, text_lower))
            emotions[emotion] += matches * 0.3
    
    return emotions

def lightweight_emotion_classification(text: str) -> str:
    """
    Lightweight emotion classification based on pattern matching
    """
    text_lower = text.lower()
    
    # Enhanced pattern matching for better accuracy
    emotion_scores = {
        'happy': 0.0,
        'joy': 0.0,
        'excited': 0.0,
        'sad': 0.0,
        'angry': 0.0,
        'fear': 0.0,
        'disgust': 0.0,
        'surprise': 0.0,
        'confused': 0.0,
        'affection': 0.0,
        'calm': 0.0
    }
    
    # Comprehensive emotion patterns
    patterns = {
        'happy': [r'\b(happy|glad|pleased|cheerful|delighted|content|positive|good|nice|fine)\b'],
        'joy': [r'\b(joy|joyful|celebration|celebrate|party|triumph|victory|jubilant|ecstatic|elated)\b'],
        'excited': [r'\b(excited|thrilled|pumped|hyped|stoked|energized|enthusiastic|eager)\b'],
        'sad': [r'\b(sad|depressed|miserable|heartbroken|devastated|crying|tears|hurt|disappointed)\b'],
        'angry': [r'\b(angry|mad|furious|rage|hate|pissed|irritated|annoyed|frustrated|fed\s+up|ridiculous|stupid|bullshit)\b'],
        'fear': [r'\b(scared|afraid|terrified|frightened|worried|anxious|nervous|fear|panic|dread)\b'],
        'disgust': [r'\b(disgusting|gross|revolting|repulsive|sick|nauseating|yuck|ew|horrible|awful|nasty|foul)\b'],
        'surprise': [r'\b(surprised|shocked|astonished|amazed|wow|omg|incredible|unbelievable|unexpected)\b'],
        'confused': [r'\b(confused|puzzled|bewildered|perplexed|don\'t\s+understand|what\s+the|unclear)\b'],
        'affection': [r'\b(love|adore|cherish|treasure|affection|caring|tender|sweet|beloved|darling|sweetheart)\b'],
        'calm': [r'\b(calm|peaceful|serene|tranquil|relaxed|okay|fine|alright|normal|quiet)\b']
    }
    
    for emotion, emotion_patterns in patterns.items():
        for pattern in emotion_patterns:
            matches = len(re.findall(pattern, text_lower))
            emotion_scores[emotion] += matches
    
    # Return the highest scoring emotion
    if max(emotion_scores.values()) == 0:
        return 'calm'
    
    return max(emotion_scores, key=emotion_scores.get)

def enhanced_sarcasm_affection_analysis(text):
    """
    Enhanced sentiment analysis to detect sarcasm, affection, and other complex emotions
    Using lightweight implementations for better performance
    """
    text_lower = text.lower()
    
    # Use lightweight NRC-style analysis
    emotions = lightweight_nrc_analysis(text)
    
    # Enhanced sarcasm detection patterns
    sarcasm_patterns = [
        r'\b(oh\s+great|oh\s+really|obviously|of\s+course|sure\s+thing)\b',
        r'\b(yeah\s+right|as\s+if|like\s+that|totally)\b', 
        r'\b(just\s+perfect|just\s+great|how\s+wonderful|absolutely\s+perfect)\b',
        r'\b(great|perfect|wonderful)\b.*\b(meeting|email|another)\b',
        r'\b(thanks\s+for\s+nothing|couldn\'t\s+be\s+better)\b'
    ]
    
    # Enhanced affection detection patterns
    affection_patterns = [
        r'\b(love\s+you|adore\s+you|cherish\s+you|treasure\s+you)\b',
        r'\b(mean\s+everything|mean\s+the\s+world|so\s+important)\b',
        r'\b(darling|sweetheart|honey|dear|beloved)\b',
        r'\b(affection|affectionate|tender|caring|loving)\b'
    ]
    
    # Check for explicit sarcasm patterns first
    sarcasm_detected = any(re.search(pattern, text_lower) for pattern in sarcasm_patterns)
    
    # Check for explicit affection patterns
    affection_detected = any(re.search(pattern, text_lower) for pattern in affection_patterns)
    
    if affection_detected:
        # Strong affection indicators override other emotions
        if emotions.get("joy", 0) > 0.3 or emotions.get("trust", 0) > 0.2:
            return "affection"
        # Fallback affection detection
        return "affection"
    
    # Enhanced sarcasm detection using emotion contradictions + patterns
    if sarcasm_detected or (emotions.get("joy", 0) > 0 and emotions.get("anger", 0) > 0):
        if sarcasm_detected or abs(emotions.get("joy", 0) - emotions.get("anger", 0)) < 0.15:
            # Determine base emotion for sarcastic combination
            base_emotion = "happy"  # default
            if emotions.get("anger", 0) > 0.3:
                base_emotion = "angry"
            elif emotions.get("sadness", 0) > 0.3:
                base_emotion = "sad"
            elif emotions.get("disgust", 0) > 0.3:
                base_emotion = "disgust"
            elif emotions.get("fear", 0) > 0.3:
                base_emotion = "fear"
            return f"sarcastic+{base_emotion}"
    
    # Check other sarcasm combinations
    if emotions.get("sadness", 0) > 0 and emotions.get("joy", 0) > 0:
        if abs(emotions.get("sadness", 0) - emotions.get("joy", 0)) < 0.1:
            return "sarcastic+sad"
    if emotions.get("disgust", 0) > 0 and emotions.get("joy", 0) > 0:
        if abs(emotions.get("disgust", 0) - emotions.get("joy", 0)) < 0.1:
            return "sarcastic+disgust"

    return "neutral"

def enhanced_emotion_analysis(text):
    """Enhanced emotion analysis using lightweight classification"""
    emotion = lightweight_emotion_classification(text)
    
    # Map to our expanded emotion set
    emotion_mapping = {
        'happy': 'happy',
        'joy': 'joy', 
        'excited': 'excited',
        'affection': 'affection',
        'sad': 'sad',
        'angry': 'angry',
        'fear': 'fear',
        'disgust': 'disgust',
        'surprise': 'surprise',
        'confused': 'confused',
        'calm': 'calm'
    }
    
    return emotion_mapping.get(emotion, 'calm')

def main():
    """Main function for command line usage with enhanced emotion detection using lightweight libraries"""
    if len(sys.argv) != 2:
        print("Usage: python custom_sentiment_analysis.py <text>")
        sys.exit(1)

    text = sys.argv[1]
    
    # First check for complex emotions (sarcasm, affection)
    initial_sentiment_analysis = enhanced_sarcasm_affection_analysis(text)
    
    if initial_sentiment_analysis == "affection":
        print(f"{initial_sentiment_analysis}:0.75")
        return
    elif initial_sentiment_analysis.startswith("sarcastic+"):
        print(f"{initial_sentiment_analysis}:0.80")
        return
    elif initial_sentiment_analysis != "neutral":
        print(f"{initial_sentiment_analysis}:0.70")
        return
    
    # If no complex emotions detected, use enhanced emotion analysis
    advance_sentiment_analysis = enhanced_emotion_analysis(text)
    confidence = 0.65  # Default confidence
    
    # Adjust confidence based on emotion strength
    if advance_sentiment_analysis in ['angry', 'sad', 'happy', 'excited']:
        confidence = 0.70
    elif advance_sentiment_analysis in ['joy', 'disgust', 'surprise', 'fear']:
        confidence = 0.65
    else:
        confidence = 0.50  # calm, confused
    
    print(f"{advance_sentiment_analysis}:{confidence:.2f}")

if __name__ == "__main__":
    main()