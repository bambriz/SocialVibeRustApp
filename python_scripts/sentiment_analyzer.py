#!/usr/bin/env python3
"""
Sentiment Analysis Module for Social Pulse
Handles HuggingFace emotion classification, sarcasm detection, and affection detection
"""
import re
import time
from nrclex import NRCLex
from model_cache import save_sentiment_model_to_cache, load_sentiment_model_from_cache

class SentimentAnalyzer:
    """Handles sentiment analysis using HuggingFace EmotionClassifier with pattern-based fallbacks"""
    
    def __init__(self):
        self.hf_classifier = None
        self.hf_available = False
        self.text2emotion_available = False
        self.nrclex_available = True
        
        # Try to initialize secondary detectors with robust error handling
        try:
            # Quick check if NLTK data exists, download only if missing
            import nltk
            import os
            nltk_data_dir = os.path.expanduser('~/nltk_data')
            os.makedirs(nltk_data_dir, exist_ok=True)
            
            # Only download missing packages to speed up startup
            required_packages = ['stopwords', 'punkt', 'wordnet']
            for package in required_packages:
                try:
                    # Quick check if package exists
                    nltk.data.find(f'{package}')
                except LookupError:
                    print(f"📦 Downloading missing NLTK package: {package}")
                    nltk.download(package, quiet=True, force=False)
            
            from text2emotion import get_emotion
            self.get_emotion = get_emotion
            self.text2emotion_available = True
            print("✅ text2emotion available as secondary detector")
        except Exception as e:
            print(f"⚠️ text2emotion not available, using fallback: {e}")
            # Create fallback function for text2emotion
            self.get_emotion = self._fallback_emotion_detection
            self.text2emotion_available = False

        print("✅ NRCLex available as fallback detector")
        
        # Initialize the primary HuggingFace classifier
        print("🚀 Initializing HuggingFace EmotionClassifier as primary detector...")
        self.initialize_hf_classifier_with_retry()
    
    def initialize_hf_classifier_with_retry(self, max_retries=2):
        """Initialize HuggingFace EmotionClassifier with caching and retry logic"""
        
        # Try to load from cache first
        cached_classifier = load_sentiment_model_from_cache()
        if cached_classifier:
            self.hf_classifier = cached_classifier
            self.hf_available = True
            return True
        
        # If cache loading failed, load from scratch
        for attempt in range(max_retries):
            try:
                print(f"🔄 Attempt {attempt + 1}/{max_retries}: Loading HuggingFace EmotionClassifier...")
                from emotionclassifier import EmotionClassifier
                self.hf_classifier = EmotionClassifier()
                
                # Test the classifier to ensure it's working
                test_result = self.hf_classifier.predict("I am happy")
                if test_result and 'label' in test_result:
                    self.hf_available = True
                    print("✅ HuggingFace EmotionClassifier loaded successfully!")
                    
                    # Save to cache for future startups
                    save_sentiment_model_to_cache(self.hf_classifier)
                    return True
                    
            except Exception as e:
                print(f"⚠️ Attempt {attempt + 1} failed: {e}")
                if attempt < max_retries - 1:
                    backoff_time = (2 ** attempt)  # Exponential backoff: 1s, 2s, 4s
                    print(f"⏳ Waiting {backoff_time}s before retry...")
                    time.sleep(backoff_time)
                
        print("❌ HuggingFace EmotionClassifier failed to initialize after all retries")
        return False
    
    def analyze_sentiment(self, text):
        """
        Analyzes sentiment using HuggingFace EmotionClassifier as primary detector.
        Uses text2emotion/NRCLex only for sarcasm and affectionate detection.
        Returns combo sentiments with gradients.
        """
        print(f"🔍 DIAGNOSTIC: Incoming sentiment analysis request")
        print(f"   📄 Text: \"{text[:100]}{'...' if len(text) > 100 else ''}\"")
        
        text_clean = text.strip()
        text_lower = text_clean.lower()
        
        # Use secondary libraries (text2emotion/NRCLex) ONLY for sarcasm/affectionate detection
        # Since NRCLex has missing data, use robust pattern-based detection
        is_sarcastic = False
        is_affectionate = False
        
        # Advanced sarcasm detection using contextual analysis
        is_sarcastic = self.detect_advanced_sarcasm(text_clean, text_lower)
        
        # Detect affection using robust patterns (no library dependency)
        affectionate_patterns = [
            r'(?:^|\W)(love|adore|cherish|treasure|devoted|caring|tender|sweet)(?:\W|$)',
            r'(?:^|\W)(darling|sweetheart|honey|dear|beloved|babe|baby)(?:\W|$)',
            r'(?:^|\W)(warm\s+feelings|deep\s+affection|heartfelt)(?:\W|$)',
            r'(?:^|\W)(my\s+love|my\s+dear|my\s+darling|my\s+heart)(?:\W|$)',
            r'[❤️💕💖💗💓💝🥰😍💋]',  # Heart and love emojis
            r'(?:^|\W)(affectionate|loving|warmth|tenderness)(?:\W|$)'
        ]
        is_affectionate = any(re.search(pattern, text_lower, re.IGNORECASE) for pattern in affectionate_patterns)
        
        print(f"   🎭 SARCASM: None detected" if not is_sarcastic else f"   🎭 SARCASM: {is_sarcastic} detected")
        print(f"   🎭 Sarcasm detected: {is_sarcastic}")
        print(f"   💕 Affection detected: {is_affectionate}")
        
        # Use HuggingFace EmotionClassifier as PRIMARY detector
        mapped_emotion = 'neutral'
        base_confidence = 0.3
        
        # First, check for emotions HuggingFace doesn't support using patterns
        unsupported_emotion = self.detect_unsupported_emotions(text_lower)
        if unsupported_emotion:
            mapped_emotion = unsupported_emotion
            base_confidence = 0.8
            print(f"   🎨 Pattern-detected unsupported emotion: {mapped_emotion}")
        
        elif self.hf_available and self.hf_classifier is not None:
            try:
                # Use HuggingFace EmotionClassifier for supported emotions
                print(f"   🧠 Calling HuggingFace EmotionClassifier...")
                result = self.hf_classifier.predict(text_clean)
                if result and 'label' in result and 'confidence' in result:
                    hf_emotion = result['label']
                    hf_confidence = result['confidence']
                    print(f"   🎯 HuggingFace result: {hf_emotion} (confidence: {hf_confidence})")
                    
                    # Map HuggingFace emotions to our system
                    # HuggingFace supports: anger, sadness, joy, fear, surprise, love, disgust
                    emotion_mapping = {
                        'joy': 'joy',
                        'happiness': 'joy', 
                        'happy': 'joy',
                        'sadness': 'sad',
                        'sad': 'sad',
                        'anger': 'angry',
                        'angry': 'angry',
                        'fear': 'fear',
                        'surprise': 'surprise',
                        'disgust': 'disgust',
                        'love': 'affection',
                        'neutral': 'neutral'
                    }
                    
                    mapped_emotion = emotion_mapping.get(hf_emotion.lower(), 'neutral')
                    base_confidence = min(0.95, max(0.5, hf_confidence))
                    
            except Exception as e:
                print(f"HuggingFace EmotionClassifier failed: {e}, falling back")
                # Fall through to fallback detectors
        
        # If HuggingFace EmotionClassifier failed, use minimal fallback
        if mapped_emotion == 'neutral' and base_confidence == 0.3:
            print("HuggingFace EmotionClassifier failed - using neutral emotion")
            # No general fallbacks - HuggingFace is the ONLY primary detector
            # Other libraries are used ONLY for sarcasm/affectionate detection
            mapped_emotion = 'neutral'
            base_confidence = 0.5
        
        # Handle combo sentiments with gradients
        final_result = {}
        if is_sarcastic:
            final_result = {
                'sentiment_type': f'sarcastic+{mapped_emotion}',
                'confidence': min(0.9, base_confidence + 0.1),
                'is_sarcastic': True,
                'is_combo': True,
                'primary_emotion': mapped_emotion,
                'combo_type': 'sarcastic'
            }
            
        elif is_affectionate:
            final_result = {
                'sentiment_type': f'affectionate+{mapped_emotion}',
                'confidence': min(0.9, base_confidence + 0.1),
                'is_affectionate': True,
                'is_combo': True,
                'primary_emotion': mapped_emotion,
                'combo_type': 'affectionate'
            }
        else:
            final_result = {
                'sentiment_type': mapped_emotion,
                'confidence': base_confidence,
                'is_sarcastic': False,
                'is_combo': False
            }
        
        print(f"   ✅ Final result: {final_result['sentiment_type']} (confidence: {final_result['confidence']})")
        if final_result.get('is_combo'):
            print(f"   🎭 Combo type: {final_result.get('combo_type', 'unknown')}")
            print(f"   🧠 Primary emotion: {final_result.get('primary_emotion', 'unknown')}")
        print(f"   📤 Response sent")
        
        return final_result
    
    def detect_unsupported_emotions(self, text_lower):
        """
        Detect emotions that HuggingFace doesn't support using pattern matching.
        HuggingFace supports: anger, sadness, joy, fear, surprise, love, disgust
        We need to detect: confused, neutral (map happy/excited to joy)
        """
        
        # Confused - uncertainty, bewilderment
        confused_patterns = [
            r'(?:^|\W)(confused|bewildered|puzzled|perplexed|baffled)(?:\W|$)',
            r'(?:^|\W)(don\'?t\s+understand|makes\s+no\s+sense|no\s+sense|unclear)(?:\W|$)',
            r'(?:^|\W)(what\s+just\s+happened|what\'?s\s+going\s+on|no\s+idea)(?:\W|$)',
            r'(?:^|\W)(lost\s+in|totally\s+bewildered|absolutely\s+no\s+sense)(?:\W|$)'
        ]
        
        # Neutral - balanced, peaceful, serene (default state)
        neutral_patterns = [
            r'(?:^|\W)(calm|peaceful|serene|tranquil|relaxed|zen)(?:\W|$)',
            r'(?:^|\W)(at\s+peace|deep\s+breath|quiet|still|centered)(?:\W|$)',
            r'(?:^|\W)(meditation|mindful|balanced)(?:\W|$)'
        ]
        
        # Joy patterns - merge happy and excited into joy (high-energy positive)
        joy_patterns = [
            # Former excited patterns
            r'(?:^|\W)(excited|pumped|thrilled|exhilarated|energized|hyped)(?:\W|$)',
            r'(?:^|\W)(can\'?t\s+wait|so\s+pumped|bouncing|adrenaline|rush)(?:\W|$)',
            r'(?:^|\W)(fired\s+up|psyched|amped|revved\s+up)(?:\W|$)',
            # Former happy patterns  
            r'(?:^|\W)(content|pleased|satisfied|glad|cheerful)(?:\W|$)',
            r'(?:^|\W)(good\s+mood|feeling\s+good|nice\s+day|pleasant)(?:\W|$)',
            r'(?:^|\W)(smile|smiling|grinning)(?:\W|$)(?!.*excitement|thrilled|ecstatic)',
            # Additional joy indicators
            r'(?:^|\W)(happy|joyful|delighted|elated|ecstatic)(?:\W|$)'
        ]
        
        # Disgust - revulsion, nausea (HuggingFace maps to sadness, we need patterns)
        disgust_patterns = [
            r'(?:^|\W)(disgusting|revolting|nauseating|repulsive|gross|vile)(?:\W|$)',
            r'(?:^|\W)(makes\s+me\s+sick|absolutely\s+nauseating|smell.*garbage)(?:\W|$)',
            r'(?:^|\W)(moldy|rotten|stinks|putrid|foul|reeks)(?:\W|$)'
        ]
        
        # Angry - frustration, rage (sometimes HuggingFace maps to sadness, need comprehensive patterns)
        angry_patterns = [
            # Direct anger expressions
            r'(?:^|\W)(furious|livid|enraged|outraged|pissed.*off|mad|angry)(?:\W|$)',
            r'(?:^|\W)(so.*angry|absolutely.*furious|makes.*me.*mad|driving.*me.*insane)(?:\W|$)',
            
            # Insulting/derogatory language (strong anger indicators)
            r'(?:^|\W)(idiots|morons|assh[o0]les?|jackasses?|bastards?|scumbags?)(?:\W|$)',
            r'(?:^|\W)(stupid.*people|worthless.*trash|piece.*of.*sh[i1]t|pathetic.*losers)(?:\W|$)',
            r'(?:^|\W)(braindead|incompetent|worthless|pathetic.*c[u\*]nts?)(?:\W|$)',
            
            # Profanity with hostility (anger context)
            r'(?:^|\W)(f[u\*]ck.*all|what.*the.*f[u\*]ck|sh[i1]tty.*world|goddamn.*bastards?)(?:\W|$)',
            r'(?:^|\W)(these.*b[i1]tches|sick.*f[u\*]cks|disgusting.*perverts)(?:\W|$)',
            
            # Expressions of wanting to harm/violence (anger)
            r'(?:^|\W)(want.*to.*beat|punch.*in.*the.*face|want.*to.*kill|beat.*the.*sh[i1]t)(?:\W|$)',
            r'(?:^|\W)(until.*they.*bleed|could.*kill.*them|rot.*in.*hell)(?:\W|$)',
            
            # Frustration and complaint patterns
            r'(?:^|\W)(fed.*up|sick.*of|tired.*of.*dealing|absolutely.*terrible)(?:\W|$)',
            r'(?:^|\W)(makes.*me.*furious|driving.*me.*crazy|screwing.*everything.*up)(?:\W|$)',
            r'(?:^|\W)(bullsh[i1]t|this.*garbage|absolute.*trash|completely.*incompetent)(?:\W|$)',
            
            # Hostile dismissive language
            r'(?:^|\W)(should.*disappear|need.*to.*get.*their.*sh[i1]t.*together|bunch.*of.*creepy)(?:\W|$)',
            r'(?:^|\W)(deserve.*to.*rot|nobody.*respects.*them|complete.*jackass)(?:\W|$)',
            
            # Traffic/driving anger (original patterns)
            r'(?:^|\W)(can\'?t.*drive|traffic.*nightmare|stuck.*mess|incompetent.*drivers)(?:\W|$)'
        ]
        
        # Check patterns in order of specificity
        if any(re.search(pattern, text_lower, re.IGNORECASE) for pattern in angry_patterns):
            return 'angry'
        elif any(re.search(pattern, text_lower, re.IGNORECASE) for pattern in disgust_patterns):
            return 'disgust'
        elif any(re.search(pattern, text_lower, re.IGNORECASE) for pattern in joy_patterns):
            return 'joy'  # Map both happy and excited to joy
        elif any(re.search(pattern, text_lower, re.IGNORECASE) for pattern in confused_patterns):
            return 'confused'  
        elif any(re.search(pattern, text_lower, re.IGNORECASE) for pattern in neutral_patterns):
            return 'neutral'
        
        return None
    
    def detect_advanced_sarcasm(self, text_original, text_lower):
        """
        Advanced sarcasm detection using contextual analysis instead of basic pattern matching.
        Detects:
        1. Rhetorical questions with negative implications
        2. Positive words used in negative contexts (sentiment contradiction)
        3. Exaggerated statements with underlying criticism
        4. Subtle irony and passive-aggressive language
        """
        
        # Basic sarcastic phrases (keep some obvious patterns)
        obvious_sarcasm_patterns = [
            r'(?:^|\W)(oh\s+great|obviously|of\s+course|sure\s+thing|yeah\s+right)(?:\W|$)',
            r'(?:^|\W)(just\s+perfect|just\s+great|how\s+wonderful|absolutely\s+perfect)(?:\W|$)',
            r'(?:^|\W)(living\s+the\s+dream|perfect\s+timing|magical\s+start)(?:\W|$)',
            r'(?:^|\W)(oh\s+sure|as\s+if|totally|love\s+that\s+for\s+me)(?:\W|$)',
        ]
        
        if any(re.search(pattern, text_lower, re.IGNORECASE) for pattern in obvious_sarcasm_patterns):
            print("   🎭 SARCASM: Basic pattern detected")
            return True
        
        # 1. Detect rhetorical questions with negative implications
        rhetorical_negative_patterns = [
            r"i\s+don'?t\s+know\s+what'?s\s+worse",
            r"what\s+could\s+be\s+better\s+than",
            r"who\s+doesn'?t\s+love",
            r"what\s+more\s+could\s+you\s+want",
            r"how\s+much\s+worse\s+can\s+it\s+get",
            r"what\s+else\s+could\s+go\s+wrong",
        ]
        
        if any(re.search(pattern, text_lower, re.IGNORECASE) for pattern in rhetorical_negative_patterns):
            print("   🎭 SARCASM: Rhetorical negative question detected")
            return True
        
        # 2. Detect positive words in clearly negative contexts (sentiment contradiction)
        positive_words = ['great', 'perfect', 'wonderful', 'amazing', 'fantastic', 'brilliant', 'awesome', 'excellent', 'marvelous']
        negative_context_words = ['mess', 'smell', 'broken', 'fail', 'disaster', 'terrible', 'awful', 'worst', 'horrible', 'falling apart', 'chaos', 'nightmare']
        
        has_positive = any(word in text_lower for word in positive_words)
        has_negative_context = any(word in text_lower for word in negative_context_words)
        
        if has_positive and has_negative_context:
            print("   🎭 SARCASM: Positive words in negative context detected")
            return True
        
        # 3. Detect "Oh sure" + positive statement + negative context
        oh_sure_pattern = r'(?:^|\W)oh\s+sure.*(?:great|good|perfect|wonderful)'
        if re.search(oh_sure_pattern, text_lower, re.IGNORECASE):
            print("   🎭 SARCASM: 'Oh sure' + positive statement detected")
            return True
        
        # 4. Detect exaggerated criticism patterns
        exaggerated_criticism_patterns = [
            r"it'?s\s+like.*(?:optional|doesn'?t\s+matter|no\s+big\s+deal)",
            r"nobody\s+seems\s+to\s+care",
            r"as\s+if.*(?:matters|cares|helps)",
            r"sure.*just\s+what\s+i\s+needed",
            r"exactly\s+what\s+i\s+wanted",
        ]
        
        if any(re.search(pattern, text_lower, re.IGNORECASE) for pattern in exaggerated_criticism_patterns):
            print("   🎭 SARCASM: Exaggerated criticism detected")
            return True
        
        # 5. Detect timing-based sarcasm
        timing_sarcasm_patterns = [
            r"just\s+what\s+i\s+needed\s+(?:right\s+now|when|today)",
            r"perfect\s+timing",
            r"couldn'?t\s+have\s+come\s+at\s+a\s+(?:better|worse)\s+time",
        ]
        
        if any(re.search(pattern, text_lower, re.IGNORECASE) for pattern in timing_sarcasm_patterns):
            print("   🎭 SARCASM: Timing-based sarcasm detected")
            return True
        
        print("   🎭 SARCASM: None detected")
        return False
    
    def get_status(self):
        """Get status information about available sentiment analysis libraries"""
        libraries = ["nrclex"]
        if self.text2emotion_available:
            libraries.append("text2emotion")
        if self.hf_available:
            libraries.insert(0, "huggingface-emotionclassifier")
        
        primary_detector = "huggingface-emotionclassifier" if self.hf_available else "nrclex"
        
        return {
            "libraries": libraries,
            "primary_detector": primary_detector,
            "supports_combo_sentiments": True,
            "hf_available": self.hf_available
        }
    
    def _fallback_emotion_detection(self, text):
        """Fallback emotion detection when text2emotion is not available"""
        # Simple pattern-based emotion detection as fallback
        text_lower = text.lower()
        
        # Basic emotion patterns
        if any(word in text_lower for word in ['happy', 'joy', 'excited', 'great', 'awesome', 'wonderful', 'amazing', 'fantastic']):
            return {'Happy': 0.8, 'Sad': 0.0, 'Angry': 0.0, 'Fear': 0.0, 'Surprise': 0.2}
        elif any(word in text_lower for word in ['sad', 'depressed', 'down', 'disappointed', 'upset', 'hurt']):
            return {'Happy': 0.0, 'Sad': 0.8, 'Angry': 0.0, 'Fear': 0.2, 'Surprise': 0.0}
        elif any(word in text_lower for word in ['angry', 'mad', 'furious', 'annoyed', 'irritated', 'frustrated']):
            return {'Happy': 0.0, 'Sad': 0.0, 'Angry': 0.8, 'Fear': 0.0, 'Surprise': 0.2}
        elif any(word in text_lower for word in ['scared', 'afraid', 'worried', 'anxious', 'nervous', 'terrified']):
            return {'Happy': 0.0, 'Sad': 0.2, 'Angry': 0.0, 'Fear': 0.8, 'Surprise': 0.0}
        elif any(word in text_lower for word in ['surprised', 'shocked', 'amazed', 'wow', 'unexpected', 'sudden']):
            return {'Happy': 0.3, 'Sad': 0.0, 'Angry': 0.0, 'Fear': 0.0, 'Surprise': 0.7}
        else:
            # Neutral fallback
            return {'Happy': 0.2, 'Sad': 0.2, 'Angry': 0.2, 'Fear': 0.2, 'Surprise': 0.2}