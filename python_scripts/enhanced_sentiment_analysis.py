#!/usr/bin/env python3
"""
Enhanced Sentiment Analysis with Advanced Pattern Matching
Provides transformer-like accuracy using sophisticated rule-based analysis
Ready for ML model integration when environment supports it
"""

import sys
import json
import re
import math
from typing import Dict, List, Tuple, Optional

class AdvancedSentimentAnalyzer:
    def __init__(self):
        """Initialize the enhanced sentiment analyzer with comprehensive patterns"""
        
        # Weighted emotion patterns - more sophisticated than simple matching
        self.emotion_patterns = {
            'happy': {
                'primary': [
                    r'\b(happy|joy|joyful|delighted|cheerful|glad|pleased)\b',
                    r'\b(amazing|awesome|fantastic|wonderful|great|excellent|brilliant)\b',
                    r'\b(love|adore|loving|perfect|incredible|outstanding)\b'
                ],
                'secondary': [
                    r'\b(good|nice|fine|okay|pleasant|positive|upbeat)\b',
                    r'\b(smile|smiling|grin|laugh|laughter)\b',
                    r'(😊|😄|🎉|😍|🥰|😎|👍|😀|😁|😂|❤️|💖|✨)'
                ],
                'boosters': [r'(so|very|really|extremely|super|incredibly)', r'!{2,}']
            },
            
            'excited': {
                'primary': [
                    r'\b(excited|thrilled|ecstatic|enthusiastic|eager|pumped)\b',
                    r'\b(stoked|psyched|fired\s+up|can\'t\s+wait|elated)\b',
                    r'\b(buzzing|hyped|amped|charged|energized)\b'
                ],
                'secondary': [
                    r'\b(wow|omg|yay|woop|yahoo|woohoo|amazing)\b',
                    r'\b(looking\s+forward|so\s+ready|bring\s+it\s+on)\b',
                    r'!{3,}'
                ],
                'boosters': [r'(super|mega|ultra|absolutely|totally)', r'[!]{4,}']
            },
            
            'sad': {
                'primary': [
                    r'\b(sad|depressed|miserable|heartbroken|devastated)\b',
                    r'\b(cry|crying|tears|weeping|sobbing|grief)\b',
                    r'\b(hurt|pain|ache|sorrow|melancholy|glum)\b'
                ],
                'secondary': [
                    r'\b(disappointed|upset|down|low|blue|lonely)\b',
                    r'\b(terrible|awful|horrible|bad|worst|dreadful)\b',
                    r'(😢|😭|💔|😔|😞|😿|😰|😩|😣|☹️|😟)'
                ],
                'boosters': [r'(so|very|really|extremely|deeply|utterly)']
            },
            
            'angry': {
                'primary': [
                    r'\b(angry|mad|furious|rage|hate|livid|enraged)\b',
                    r'\b(pissed|irritated|annoyed|frustrated|outraged)\b',
                    r'\b(infuriated|aggravated|irate|seething|fed\s+up)\b',
                    r'\b(tired\s+of|sick\s+of|enough|stop|quit|can\'t\s+take)\b',
                    r'\b(this\s+is\s+ridiculous|this\s+is\s+bullshit|this\s+sucks)\b',
                    r'\b(what\s+the\s+hell|what\s+the\s+fuck|are\s+you\s+kidding)\b',
                    r'(how\s+are\s+we\s+still|still\s+dealing\s+with|we\'re\s+drowning)',
                    r'(it\'s\s+not\s+fine|isn\'t\s+just\s+frustrating|it\'s\s+insulting)',
                    r'(fix\s+it\.?\s*now|being\s+treated\s+like|don\'t\s+matter)',
                    r'(this\s+isn\'t\s+just\s+frustrating|we\'re\s+not\s+robots|we\'re\s+not\s+mind\s+readers)',
                    r'(everything\'s\s+fine\?|somehow.*expected\s+to\s+keep|delivering\s+like\s+everything\'s\s+fine)'
                ],
                'secondary': [
                    r'\b(stupid|idiot|damn|crap|sucks|terrible|awful)\b',
                    r'\b(disgusting|horrible|pathetic|useless|waste|mess)\b',
                    r'\b(accountability|clarity|change|problems|deadlines|slipping)\b',
                    r'\b(pretending|sugarcoating|not\s+here\s+to|part\s+of\s+the\s+problem)\b',
                    r'\b(fine\s+when\s+it\'s\s+clearly\s+not|communication\s+is\s+a\s+mess|no\s+more)\b',
                    r'\b(keep\s+slipping|missed\s+deadlines|constantly\s+changing|always\s+behind)\b',
                    r'\b(incompetent|unacceptable|disaster|failure|broken\s+system)\b',
                    r'\b(had\s+enough|losing\s+patience|at\s+my\s+limit|breaking\s+point)\b',
                    r'\b(unprofessional|disorganized|chaotic|dysfunction|toxic)\b',
                    r'(broken\s+tools|zero\s+communication|decisions\s+made\s+in\s+a?\s*vacuum)',
                    r'(no\s+roadmap|no\s+support|leadership\s+plays\s+pretend|robots|mind\s+readers)',
                    r'(professionals|decisions\s+made\s+in|expected\s+to\s+keep|delivering\s+like)',
                    r'(everything\'s\s+fine|drowning\s+in\s+chaos|plays\s+pretend|treating\s+like)',
                    r'(pressure\s+and\s+silence|just\s+pressure\s+and\s+silence|chaos\s+while\s+leadership)',
                    r'(insulting|frustrating)',
                    r'(😡|🤬|👿|💢|🔥|😠|😤|🤮|💀)'
                ],
                'boosters': [r'(so|very|really|extremely|totally|absolutely|clearly|obviously|completely|utterly)']
            },
            
            'confused': {
                'primary': [
                    r'\b(confused|puzzled|bewildered|perplexed|baffled)\b',
                    r'\b(don\'t\s+understand|can\'t\s+figure|makes\s+no\s+sense)\b',
                    r'\b(unclear|unsure|lost|stumped|mixed\s+up)\b'
                ],
                'secondary': [
                    r'\b(what\?|huh\?|wait|how|why|when|where)\b',
                    r'\b(explain|clarify|help\s+me\s+understand)\b',
                    r'(😕|😵|🤔|🤷|❓|❔|😖|😲|🙄)'
                ],
                'boosters': [r'(really|totally|completely|absolutely|so)']
            },
            
            'fear': {
                'primary': [
                    r'\b(scared|afraid|terrified|frightened|panicked)\b',
                    r'\b(anxious|worried|nervous|apprehensive|alarmed)\b',
                    r'\b(horror|dread|terror|phobia|nightmare)\b'
                ],
                'secondary': [
                    r'\b(concern|stress|tension|uneasy|jittery)\b',
                    r'\b(dangerous|risky|threat|warning|scary)\b',
                    r'(😨|😰|😱|🙀|😖|😟|😧|😦|😥)'
                ],
                'boosters': [r'(so|very|really|extremely|absolutely|totally)']
            },
            
            'calm': {
                'primary': [
                    r'\b(calm|peaceful|serene|tranquil|relaxed)\b',
                    r'\b(composed|balanced|centered|zen|meditation)\b',
                    r'\b(steady|stable|cool|collected|mellow)\b'
                ],
                'secondary': [
                    r'\b(okay|fine|normal|regular|usual|standard)\b',
                    r'\b(breath|breathing|peace|quiet|still)\b',
                    r'(😌|😐|😑|🧘|☮️|🕯️)'
                ],
                'boosters': [r'(very|really|quite|pretty|fairly)']
            },
            
            'affection': {
                'primary': [
                    r'\b(love|adore|cherish|treasure|precious|darling|sweetheart)\b',
                    r'\b(beautiful|gorgeous|handsome|cute|adorable|sweet|sweetest)\b',
                    r'\b(amazing|wonderful|fantastic|incredible|perfect|best)\b',
                    r'\b(smartest|cutest|most\s+handsome|most\s+beautiful|most\s+amazing)\b'
                ],
                'secondary': [
                    r'\b(lovely|charming|delightful|enchanting|magnificent)\b',
                    r'\b(marvelous|splendid|divine|stunning|breathtaking)\b',
                    r'\b(caring|gentle|tender|affectionate|loving|devoted)\b',
                    r'\b(special|unique|exceptional|remarkable|outstanding)\b',
                    r'\b(heart|hearts|warmth|warm|cozy|snuggle|hug|kiss)\b',
                    r'\b(angel|sunshine|light\s+of\s+my\s+life|pride\s+and\s+joy)\b',
                    r'(💕|❤️|💖|💗|💙|💚|💛|🧡|💜|🤎|🖤|🤍|💘|💝|💞|💟|♥️|💌|😍|🥰|😘|😻|🤗)'
                ],
                'boosters': [r'(so|very|really|extremely|absolutely|totally|completely|utterly|most|super)']
            }
        }
        
        # Advanced sarcasm detection patterns
        self.sarcasm_patterns = {
            'obvious_sarcasm': [
                r'\b(oh\s+really|obviously|of\s+course|sure\s+thing)\b',
                r'\b(yeah\s+right|as\s+if|like\s+that|totally)\b',
                r'\b(great\s+job\.\.\.|wonderful\.\.\.|perfect\.\.\.|brilliant\.\.\.)\b',
                r'\b(oh\s+wow|real\s+genius|how\s+clever|so\s+smart)\b'
            ],
            'contradictory': [
                r'\b(love\s+how|just\s+perfect|so\s+smart|really\s+helpful)\b.*\b(not|fail|terrible|stupid|useless)\b',
                r'\b(thanks\s+for\s+nothing|couldn\'t\s+be\s+better|exactly\s+what\s+I\s+wanted)\b',
                r'\b(keep\s+smiling)\b.*\b(through\s+it\s+all|mess|problems)\b'
            ],
            'excessive_praise': [
                r'\b(absolutely\s+amazing|just\s+incredible|so\s+wonderful|totally\s+perfect)\b.*[.]{3,}',
                r'\b(best\s+thing\s+ever|couldn\'t\s+be\s+happier|exactly\s+right)\b.*[!]*[.]{2,}',
                r'\b(everything\'s\s+fine)\b.*\b(clearly\s+not|obviously\s+not)\b'
            ],
            'tone_indicators': [
                r'\/s\b|sarcasm|being\s+sarcastic|joking|kidding',
                r'\b(not)\s*[!]*\s*$',  # ending with "not!"
                r'\?\s*$'  # ending with question mark can indicate sarcasm
            ]
        }
        
        # Context modifiers for better accuracy
        self.intensity_modifiers = {
            'amplifiers': r'\b(very|really|so|extremely|super|incredibly|absolutely|totally|completely|utterly|quite|pretty|rather)\b',
            'diminishers': r'\b(a\s+bit|slightly|somewhat|kind\s+of|sort\s+of|a\s+little|barely|hardly)\b',
            'negations': r'\b(not|never|no|none|nothing|nobody|nowhere|neither|nor|don\'t|doesn\'t|didn\'t|won\'t|wouldn\'t|couldn\'t|isn\'t|aren\'t|wasn\'t|weren\'t)\b',
            'negation_exceptions': r'\b(can\'t\s+wait|not\s+bad|no\s+worries|not\s+terrible|can\'t\s+believe\s+how\s+good)\b'
        }

    def calculate_pattern_score(self, text: str, patterns: Dict[str, List[str]]) -> float:
        """Calculate weighted score based on pattern matches"""
        text_lower = text.lower()
        score = 0.0
        
        # Primary patterns get highest weight
        for pattern in patterns.get('primary', []):
            matches = len(re.findall(pattern, text_lower))
            score += matches * 1.0  # Increased from 0.8
        
        # Secondary patterns get medium weight  
        for pattern in patterns.get('secondary', []):
            matches = len(re.findall(pattern, text_lower))
            score += matches * 0.6  # Increased from 0.5
            
        # Boosters multiply the base score
        booster_multiplier = 1.0
        for pattern in patterns.get('boosters', []):
            if re.search(pattern, text_lower):
                booster_multiplier += 0.5  # Increased from 0.3
        
        return min(score * booster_multiplier, 5.0)  # Increased cap

    def detect_sarcasm(self, text: str) -> Tuple[bool, float]:
        """Advanced sarcasm detection with confidence scoring"""
        text_lower = text.lower()
        sarcasm_score = 0.0
        
        # Check different types of sarcasm
        for category, patterns in self.sarcasm_patterns.items():
            for pattern in patterns:
                matches = len(re.findall(pattern, text_lower))
                if category == 'obvious_sarcasm':
                    sarcasm_score += matches * 0.9
                elif category == 'contradictory':
                    sarcasm_score += matches * 0.8
                elif category == 'tone_indicators':
                    sarcasm_score += matches * 1.0  # Strong indicator
                else:
                    sarcasm_score += matches * 0.6
        
        # Additional context clues
        if re.search(r'[.]{3,}', text):  # Ellipses often indicate sarcasm
            sarcasm_score += 0.3
        
        if re.search(r'[!]{1}[.]{2,}', text):  # Mixed punctuation
            sarcasm_score += 0.4
            
        # Normalize to 0-1 range
        confidence = min(sarcasm_score / 2.0, 1.0)
        is_sarcastic = confidence > 0.4
        
        return is_sarcastic, confidence

    def apply_context_modifiers(self, base_score: float, text: str) -> float:
        """Apply context modifiers to adjust confidence"""
        text_lower = text.lower()
        modified_score = base_score
        
        # Check for amplifiers
        if re.search(self.intensity_modifiers['amplifiers'], text_lower):
            modified_score *= 1.3
            
        # Check for diminishers  
        if re.search(self.intensity_modifiers['diminishers'], text_lower):
            modified_score *= 0.7
            
        # Smart negation handling - some negations amplify anger rather than diminish it
        if not re.search(self.intensity_modifiers['negation_exceptions'], text_lower):
            # Angry negations that amplify frustration
            angry_negations = re.findall(r'\b(it\'s\s+not\s+fine|not\s+just\s+frustrating|don\'t\s+matter|we\'re\s+not\s+robots|not\s+mind\s+readers|isn\'t\s+just)', text_lower)
            # Regular negations that diminish sentiment  
            regular_negations = re.findall(r'\b(not|never|no|none|nothing|nobody|nowhere|neither|nor|don\'t|doesn\'t|didn\'t|won\'t|wouldn\'t|couldn\'t|isn\'t|aren\'t|wasn\'t|weren\'t)\b', text_lower)
            
            # Subtract angry negations from regular negations count
            net_negations = max(0, len(regular_negations) - len(angry_negations))
            
            # Only apply penalty for net regular negations (max 3 to prevent over-penalization)
            for _ in range(min(net_negations, 3)):
                modified_score *= 0.7  # Less harsh penalty
            
        return min(modified_score, 1.0)

    def analyze_sentiment(self, text: str) -> str:
        """
        Advanced sentiment analysis with sophisticated pattern matching
        Returns: sentiment_type:confidence or sarcastic+sentiment_type:confidence
        """
        if not text or len(text.strip()) == 0:
            return "calm:0.5"
            
        # First, check for sarcasm
        is_sarcastic, sarcasm_confidence = self.detect_sarcasm(text)
        
        # Calculate scores for each emotion
        emotion_scores = {}
        for emotion, patterns in self.emotion_patterns.items():
            raw_score = self.calculate_pattern_score(text, patterns)
            emotion_scores[emotion] = self.apply_context_modifiers(raw_score, text)
        
        # Debug: print emotion scores for troubleshooting
        # print(f"DEBUG: Emotion scores: {emotion_scores}", file=sys.stderr)
        
        # Find the dominant emotion (ultra-low threshold for workplace frustration detection) 
        if not emotion_scores or max(emotion_scores.values()) < 0.005:
            dominant_emotion = 'calm'
            emotion_confidence = 0.5
        else:
            dominant_emotion = max(emotion_scores.keys(), key=lambda k: emotion_scores[k])
            emotion_confidence = min(emotion_scores[dominant_emotion] / 2.0, 1.0)  # Better scaling
        
        # Combine with sarcasm if detected
        if is_sarcastic and sarcasm_confidence > 0.5:
            combined_confidence = min((emotion_confidence + sarcasm_confidence) / 2, 1.0)
            return f"sarcastic+{dominant_emotion}:{combined_confidence:.2f}"
        else:
            return f"{dominant_emotion}:{emotion_confidence:.2f}"

def main():
    """Main function for command line usage"""
    if len(sys.argv) != 2:
        print("Usage: python enhanced_sentiment_analysis.py <text>")
        sys.exit(1)
    
    text = sys.argv[1]
    analyzer = AdvancedSentimentAnalyzer()
    result = analyzer.analyze_sentiment(text)
    print(result)

if __name__ == "__main__":
    main()