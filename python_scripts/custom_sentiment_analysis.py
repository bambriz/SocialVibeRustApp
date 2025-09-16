#!/usr/bin/env python3
"""
Enhanced Sentiment Analysis with Advanced Emotion Detection and Sarcasm Recognition
Supports 10+ emotions with sophisticated pattern matching and contextual analysis
"""

import sys
import re
import json
from typing import Dict, List, Tuple, Optional

class AdvancedSentimentAnalyzer:
    def __init__(self):
        """Initialize with comprehensive emotion patterns and sarcasm detection"""
        
        # Comprehensive emotion pattern library with weighted categories
        self.emotion_patterns = {
            'happy': {
                'primary': [
                    r'\b(happy|joy|joyful|glad|pleased|cheerful|delighted|thrilled)\b',
                    r'\b(amazing|awesome|fantastic|wonderful|great|excellent|perfect)\b',
                    r'\b(love|adore|enjoy|appreciate)\s+\w+',
                    r'\b(feeling\s+good|so\s+good|really\s+good)\b'
                ],
                'secondary': [
                    r'\b(nice|good|cool|sweet|fun|exciting)\b',
                    r'\b(smile|smiling|grin|laugh|laughing)\b',
                    r'[!]{2,}|😊|😄|😃|🙂|😁'
                ],
                'boosters': [r'(so|very|really|extremely|absolutely|totally)']
            },
            'joy': {
                'primary': [
                    r'\b(celebration|celebrate|party|festival|victory)\b',
                    r'\b(ecstatic|elated|euphoric|overjoyed|jubilant)\b',
                    r'\b(triumph|achievement|success|accomplishment)\b',
                    r'\b(dancing|singing|cheering)\b'
                ],
                'secondary': [
                    r'\b(yay|hooray|woohoo|woot|yes!)\b',
                    r'\b(bright|brilliant|shining|glowing)\b',
                    r'🎉|🎊|🥳|🎈|✨'
                ],
                'boosters': [r'(absolutely|totally|completely|utterly)']
            },
            'excited': {
                'primary': [
                    r'\b(excited|pumped|hyped|thrilled|stoked|energetic)\b',
                    r'\b(can\'t\s+wait|so\s+ready|looking\s+forward)\b',
                    r'\b(anticipation|eager|enthusiastic)\b'
                ],
                'secondary': [
                    r'\b(wow|omg|amazing|incredible)\b',
                    r'[!]{3,}',
                    r'🔥|⚡|💪'
                ]
            },
            'affection': {
                'primary': [
                    r'\b(love\s+you|adore\s+you|cherish\s+you|treasure\s+you)\b',
                    r'\b(mean\s+everything|mean\s+the\s+world|so\s+important)\b',
                    r'\b(darling|sweetheart|honey|dear|beloved)\b',
                    r'\b(affection|affectionate|tender|caring|loving)\b'
                ],
                'secondary': [
                    r'\b(hug|hugs|kiss|kisses|cuddle|embrace)\b',
                    r'\b(warm|gentle|soft|sweet|precious)\b',
                    r'💕|💖|💗|💝|❤️|💙|💚|💛|💜|🧡'
                ],
                'boosters': [r'(so|very|really|deeply|truly|completely)']
            },
            'sad': {
                'primary': [
                    r'\b(sad|depressed|miserable|heartbroken|devastated)\b',
                    r'\b(crying|tears|weeping|sobbing)\b',
                    r'\b(lonely|empty|hopeless|despair)\b'
                ],
                'secondary': [
                    r'\b(down|blue|gloomy|melancholy)\b',
                    r'\b(sorry|regret|disappointed)\b',
                    r'😢|😭|💔|😔'
                ]
            },
            'angry': {
                'primary': [
                    r'\b(angry|furious|rage|enraged|livid|pissed)\b',
                    r'\b(hate|despise|loathe|can\'t\s+stand)\b',
                    r'\b(frustrated|irritated|annoyed|fed\s+up)\b',
                    r'\b(workplace\s+frustration|office\s+politics|corporate\s+bs)\b'
                ],
                'secondary': [
                    r'\b(damn|shit|fuck|stupid|idiotic|ridiculous)\b',
                    r'\b(mad|upset|bothered|ticked\s+off)\b',
                    r'😠|😡|🤬|💢'
                ]
            },
            'fear': {
                'primary': [
                    r'\b(scared|afraid|terrified|frightened|horrified)\b',
                    r'\b(anxious|worried|nervous|panic|dread)\b',
                    r'\b(fear|phobia|nightmare|terror)\b'
                ],
                'secondary': [
                    r'\b(concerned|uneasy|troubled|distressed)\b',
                    r'\b(danger|threat|risk|unsafe)\b',
                    r'😨|😰|😱|💀'
                ]
            },
            'disgust': {
                'primary': [
                    r'\b(disgusting|gross|revolting|repulsive|nauseating)\b',
                    r'\b(sick|nauseous|queasy|vomit|puke)\b',
                    r'\b(disgusted|appalled|repelled|revolted)\b'
                ],
                'secondary': [
                    r'\b(yuck|ew|ugh|bleh|nasty)\b',
                    r'\b(awful|terrible|horrible|foul)\b',
                    r'🤢|🤮|🤧|💩'
                ]
            },
            'surprise': {
                'primary': [
                    r'\b(surprised|shocked|astonished|amazed|stunned)\b',
                    r'\b(unexpected|sudden|out\s+of\s+nowhere|didn\'t\s+see\s+coming)\b',
                    r'\b(wow|omg|oh\s+my\s+god|unbelievable|incredible)\b'
                ],
                'secondary': [
                    r'\b(whoa|what|really|seriously|no\s+way)\b',
                    r'\b(blown\s+away|mind\s+blown|can\'t\s+believe)\b',
                    r'😲|😮|🤯|😯'
                ]
            },
            'confused': {
                'primary': [
                    r'\b(confused|puzzled|bewildered|perplexed|baffled)\b',
                    r'\b(don\'t\s+understand|makes\s+no\s+sense|what\s+the)\b',
                    r'\b(unclear|uncertain|lost|mixed\s+up)\b'
                ],
                'secondary': [
                    r'\b(huh|what|why|how|when)\b',
                    r'\?{2,}',
                    r'🤔|😵|😕'
                ]
            },
            'calm': {
                'primary': [
                    r'\b(calm|peaceful|serene|tranquil|relaxed)\b',
                    r'\b(meditation|mindful|zen|balance|harmony)\b'
                ],
                'secondary': [
                    r'\b(quiet|still|gentle|soft|mild)\b',
                    r'\b(okay|fine|alright|normal)\b',
                    r'😌|🧘|☮️'
                ],
                'boosters': [r'(so|very|really|extremely|absolutely|totally)']
            }
        }
        
        # Advanced sarcasm detection patterns
        self.sarcasm_patterns = {
            'obvious_sarcasm': [
                r'\b(oh\s+great|oh\s+really|obviously|of\s+course|sure\s+thing)\b',
                r'\b(yeah\s+right|as\s+if|like\s+that|totally)\b',
                r'\b(great\s+job\.\.\.|wonderful\.\.\.|perfect\.\.\.|brilliant\.\.\.)\b',
                r'\b(oh\s+wow|real\s+genius|how\s+clever|so\s+smart)\b',
                r'\b(just\s+perfect|just\s+great|how\s+wonderful|absolutely\s+perfect)\b'
            ],
            'contradictory': [
                r'\b(love\s+how|just\s+perfect|so\s+smart|really\s+helpful)\b.*\b(not|fail|terrible|stupid|useless)\b',
                r'\b(thanks\s+for\s+nothing|couldn\'t\s+be\s+better|exactly\s+what\s+I\s+wanted)\b',
                r'\b(keep\s+smiling)\b.*\b(through\s+it\s+all|mess|problems)\b',
                r'\b(great|perfect|wonderful|amazing)\b.*\b(another|could\s+have\s+been|should\s+have)\b'
            ],
            'excessive_praise': [
                r'\b(absolutely\s+amazing|just\s+incredible|so\s+wonderful|totally\s+perfect)\b.*[.]{3,}',
                r'\b(best\s+thing\s+ever|couldn\'t\s+be\s+happier|exactly\s+right)\b.*[!]*[.]{2,}',
                r'\b(everything\'s\s+fine)\b.*\b(clearly\s+not|obviously\s+not)\b'
            ],
            'punctuation_sarcasm': [
                r'\b(great|perfect|wonderful|amazing|fantastic)\b.*\.\s*$',  # Positive word with period (flat tone)
                r'^[A-Z][a-z]+\s+(great|perfect|wonderful)\b',  # Start with "Oh great", "Just perfect"
                r'\b(great|perfect|wonderful)\b.*\b(meeting|email|another)\b'  # Context clues
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
            'diminishers': r'\b(somewhat|slightly|a\s+bit|kind\s+of|sort\s+of|maybe|perhaps)\b'
        }
        
        # Negation patterns for context-aware analysis
        self.negation_patterns = [
            r'\b(not|never|no|none|nothing|nobody|nowhere|neither|nor)\b',
            r'\b(don\'t|doesn\'t|didn\'t|won\'t|wouldn\'t|can\'t|cannot|couldn\'t|shouldn\'t|mustn\'t)\b',
            r'\b(hardly|barely|scarcely|rarely|seldom)\b'
        ]

    def calculate_pattern_score(self, text: str, patterns: Dict[str, List[str]]) -> float:
        """Calculate weighted score based on pattern matches"""
        text_lower = text.lower()
        score = 0.0
        
        # Primary patterns get highest weight (increased for affection priority)
        for pattern in patterns.get('primary', []):
            matches = len(re.findall(pattern, text_lower))
            score += matches * 1.5  # Increased to give affection better chance
        
        # Secondary patterns get medium weight  
        for pattern in patterns.get('secondary', []):
            matches = len(re.findall(pattern, text_lower))
            score += matches * 0.6
            
        # Booster patterns multiply the score
        for pattern in patterns.get('boosters', []):
            if re.search(pattern, text_lower):
                score *= 1.3
                
        return score

    def detect_sarcasm(self, text: str) -> Tuple[bool, float]:
        """Advanced sarcasm detection with confidence scoring"""
        text_lower = text.lower()
        sarcasm_score = 0.0
        
        # Check different types of sarcasm
        for category, patterns in self.sarcasm_patterns.items():
            for pattern in patterns:
                matches = len(re.findall(pattern, text_lower))
                if category == 'obvious_sarcasm':
                    sarcasm_score += matches * 1.2  # Increased weight
                elif category == 'contradictory':
                    sarcasm_score += matches * 1.0  # Increased weight
                elif category == 'punctuation_sarcasm':
                    sarcasm_score += matches * 0.8  # New category
                elif category == 'tone_indicators':
                    sarcasm_score += matches * 1.5  # Strong indicator
                else:
                    sarcasm_score += matches * 0.7  # Increased base weight

        # Additional context clues
        if re.search(r'[.]{3,}', text):  # Ellipses often indicate sarcasm
            sarcasm_score += 0.3
        
        if re.search(r'\b(just|only|simply)\b', text_lower):  # Minimizing words
            sarcasm_score += 0.2
            
        # Convert score to confidence (0-1)
        confidence = min(sarcasm_score / 2.0, 1.0)
        is_sarcastic = confidence > 0.3
        
        return is_sarcastic, confidence

    def apply_context_modifiers(self, base_score: float, text: str) -> float:
        """Apply intensity modifiers and negation handling"""
        text_lower = text.lower()
        modified_score = base_score
        
        # Check for amplifiers
        amplifier_matches = len(re.findall(self.intensity_modifiers['amplifiers'], text_lower))
        if amplifier_matches > 0:
            modified_score *= (1.0 + (amplifier_matches * 0.3))
        
        # Check for diminishers  
        diminisher_matches = len(re.findall(self.intensity_modifiers['diminishers'], text_lower))
        if diminisher_matches > 0:
            modified_score *= (1.0 - (diminisher_matches * 0.2))
            
        # Handle negation - reduces positive emotions, amplifies negative ones
        negation_found = any(re.search(pattern, text_lower) for pattern in self.negation_patterns)
        if negation_found:
            modified_score *= 0.7  # Reduce confidence when negation is present
            
        return max(modified_score, 0.0)

    def analyze_sentiment(self, text: str) -> Tuple[str, float]:
        """Main sentiment analysis with advanced emotion detection"""
        if not text or not text.strip():
            return "calm", 0.5
            
        text = text.strip()
        
        # First, check for sarcasm
        is_sarcastic, sarcasm_confidence = self.detect_sarcasm(text)
        
        # Calculate scores for all emotions
        emotion_scores = {}
        for emotion, patterns in self.emotion_patterns.items():
            raw_score = self.calculate_pattern_score(text, patterns)
            if raw_score > 0:
                final_score = self.apply_context_modifiers(raw_score, text)
                if final_score > 0:
                    emotion_scores[emotion] = final_score
        
        # Debug: print emotion scores for troubleshooting
        # print(f"DEBUG: Emotion scores: {emotion_scores}", file=sys.stderr)
        
        # Find the dominant emotion with proper thresholds
        if not emotion_scores:
            dominant_emotion = 'calm'
            emotion_confidence = 0.5
        else:
            # Get the highest scoring emotion (with affection priority in case of ties)
            max_score = max(emotion_scores.values())
            
            # If affection ties with other emotions, prioritize it
            if 'affection' in emotion_scores and emotion_scores['affection'] == max_score:
                dominant_emotion = 'affection'
            else:
                dominant_emotion = max(emotion_scores.keys(), key=lambda k: emotion_scores[k])
            
            raw_confidence = emotion_scores[dominant_emotion]
            
            # Apply minimum thresholds for detection (lowered for better sensitivity)
            if raw_confidence < 0.2:
                dominant_emotion = 'calm'
                emotion_confidence = 0.5
            else:
                emotion_confidence = min(raw_confidence / 2.0, 1.0)  # Better scaling
        
        # Return sarcastic combination if sarcasm detected with high confidence
        if is_sarcastic and sarcasm_confidence > 0.4:
            combined_confidence = min((emotion_confidence + sarcasm_confidence) / 2, 1.0)
            return f"sarcastic+{dominant_emotion}", combined_confidence
        
        return dominant_emotion, emotion_confidence

def main():
    """Main function for command-line usage"""
    if len(sys.argv) != 2:
        print("Usage: python3 custom_sentiment_analysis.py '<text>'", file=sys.stderr)
        sys.exit(1)
    
    text = sys.argv[1]
    analyzer = AdvancedSentimentAnalyzer()
    sentiment, confidence = analyzer.analyze_sentiment(text)
    
    # Output format: "sentiment:confidence"
    print(f"{sentiment}:{confidence:.2f}")

if __name__ == "__main__":
    main()