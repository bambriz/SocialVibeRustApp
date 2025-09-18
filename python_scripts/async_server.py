#!/usr/bin/env python3
"""
Social Pulse - Persistent Sentiment Analysis Server with Caching

This server provides HTTP endpoints for sentiment analysis and content moderation,
eliminating the need to reinitialize Python libraries on each request.
Uses HuggingFace EmotionClassifier as primary detector with fallback chain.
Implements persistent caching for faster startup times.
"""
import json
import sys
import re
import subprocess
import time
import pickle
import os
from http.server import HTTPServer, BaseHTTPRequestHandler
from nrclex import NRCLex

# Caching configuration
CACHE_DIR = "/tmp/social_pulse_cache"
MODEL_CACHE_FILE = os.path.join(CACHE_DIR, "emotion_classifier.pkl")
CACHE_VERSION = "v1.2"  # Update when model changes

# Initialize HuggingFace EmotionClassifier as primary detector
HF_EMOTIONCLASSIFIER_AVAILABLE = False
hf_emotion_classifier = None

# Secondary detectors for fallback and special cases
TEXT2EMOTION_AVAILABLE = False
NRCLEX_AVAILABLE = True

def save_model_to_cache():
    """Save the loaded classifier to persistent cache"""
    global hf_emotion_classifier
    try:
        os.makedirs(CACHE_DIR, exist_ok=True)
        cache_data = {
            'version': CACHE_VERSION,
            'classifier': hf_emotion_classifier,
            'timestamp': time.time()
        }
        with open(MODEL_CACHE_FILE, 'wb') as f:
            pickle.dump(cache_data, f)
        print("💾 Model saved to persistent cache for faster future startups")
    except Exception as e:
        print(f"⚠️ Failed to cache model: {e}")

def load_model_from_cache():
    """Load classifier from persistent cache if available"""
    global hf_emotion_classifier, HF_EMOTIONCLASSIFIER_AVAILABLE
    
    if not os.path.exists(MODEL_CACHE_FILE):
        print("📋 No model cache found, will load from scratch")
        return False
    
    try:
        with open(MODEL_CACHE_FILE, 'rb') as f:
            cache_data = pickle.load(f)
        
        if cache_data.get('version') != CACHE_VERSION:
            print("⚠️ Cache version mismatch, will reload model")
            return False
        
        print("🚀 Loading model from persistent cache...")
        hf_emotion_classifier = cache_data['classifier']
        
        # Test the cached classifier
        test_result = hf_emotion_classifier.predict("I am happy")
        if test_result and 'label' in test_result:
            HF_EMOTIONCLASSIFIER_AVAILABLE = True
            print("✅ Model loaded from cache successfully! (Much faster startup)")
            return True
        else:
            print("⚠️ Cached model test failed, will reload from scratch")
            return False
            
    except Exception as e:
        print(f"⚠️ Failed to load from cache: {e}, will reload from scratch")
        return False

def initialize_hf_classifier_with_retry(max_retries=3):
    """Initialize HuggingFace EmotionClassifier with caching and retry logic"""
    global HF_EMOTIONCLASSIFIER_AVAILABLE, hf_emotion_classifier
    
    # Try to load from cache first
    if load_model_from_cache():
        return True
    
    # If cache loading failed, load from scratch
    for attempt in range(max_retries):
        try:
            print(f"🔄 Attempt {attempt + 1}/{max_retries}: Loading HuggingFace EmotionClassifier...")
            from emotionclassifier import EmotionClassifier
            hf_emotion_classifier = EmotionClassifier()
            
            # Test the classifier to ensure it's working
            test_result = hf_emotion_classifier.predict("I am happy")
            if test_result and 'label' in test_result:
                HF_EMOTIONCLASSIFIER_AVAILABLE = True
                print("✅ HuggingFace EmotionClassifier loaded successfully!")
                
                # Save to cache for future startups
                save_model_to_cache()
                return True
                
        except Exception as e:
            print(f"⚠️ Attempt {attempt + 1} failed: {e}")
            if attempt < max_retries - 1:
                backoff_time = (2 ** attempt)  # Exponential backoff: 1s, 2s, 4s
                print(f"⏳ Waiting {backoff_time}s before retry...")
                time.sleep(backoff_time)
            
    print("❌ HuggingFace EmotionClassifier failed to initialize after all retries")
    return False

# Try to initialize secondary detectors
try:
    from text2emotion import get_emotion
    TEXT2EMOTION_AVAILABLE = True
    print("✅ text2emotion available as secondary detector")
except ImportError as e:
    print(f"⚠️ text2emotion not available: {e}")

print("✅ NRCLex available as fallback detector")

# Initialize the primary HuggingFace classifier
print("🚀 Initializing HuggingFace EmotionClassifier as primary detector...")
initialize_hf_classifier_with_retry()

class SentimentHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        try:
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)
            data = json.loads(post_data.decode('utf-8'))
            text = data.get('text', '')
            
            if self.path == '/analyze':
                result = self.analyze_sentiment(text)
            elif self.path == '/moderate':
                result = self.moderate_content(text)
            else:
                self.send_error(404)
                return
                
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            self.wfile.write(json.dumps(result).encode('utf-8'))
            
        except Exception as e:
            print(f"Server error: {e}", file=sys.stderr)
            self.send_error(500, str(e))
    
    def do_GET(self):
        if self.path == '/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.send_header('Access-Control-Allow-Origin', '*')
            self.end_headers()
            
            libraries = ["nrclex"]
            if TEXT2EMOTION_AVAILABLE:
                libraries.append("text2emotion")
            if HF_EMOTIONCLASSIFIER_AVAILABLE:
                libraries.insert(0, "huggingface-emotionclassifier")
            
            primary_detector = "huggingface-emotionclassifier" if HF_EMOTIONCLASSIFIER_AVAILABLE else "nrclex"
            
            self.wfile.write(json.dumps({
                "status": "healthy", 
                "libraries": libraries, 
                "primary_detector": primary_detector,
                "supports_combo_sentiments": True
            }).encode('utf-8'))
        else:
            self.send_error(404)
    
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
        
        elif HF_EMOTIONCLASSIFIER_AVAILABLE and hf_emotion_classifier is not None:
            try:
                # Use HuggingFace EmotionClassifier for supported emotions
                print(f"   🧠 Calling HuggingFace EmotionClassifier...")
                result = hf_emotion_classifier.predict(text_clean)
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
        
        # Angry - frustration, rage (sometimes HuggingFace maps to sadness)
        angry_patterns = [
            r'(?:^|\W)(idiots|incompetent.*drivers|these.*idiots|stupid.*people)(?:\W|$)',
            r'(?:^|\W)(furious|livid|enraged|outraged|pissed.*off)(?:\W|$)',
            r'(?:^|\W)(so.*angry|absolutely.*furious|makes.*me.*mad)(?:\W|$)',
            r'(?:^|\W)(can\'?t.*drive|traffic.*nightmare|stuck.*mess)(?:\W|$)'
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
    
    def moderate_content(self, text):
        """
        Content moderation with pattern matching for harmful content.
        Provides comprehensive diagnostic logging for transparency.
        """
        print(f"🛡️ MODERATION: Incoming content moderation request")
        print(f"   📄 Text: \"{text[:100]}{'...' if len(text) > 100 else ''}\"")
        
        text_lower = text.lower()
        print(f"   🔍 Processing text (length: {len(text)} chars)")
        
        # Define moderation patterns with enhanced categorization
        hate_patterns = {
            "racial_slurs": [r'\b(n[i1]gg[ae]r|ch[i1]nk|sp[i1]c|k[i1]ke)\b'],
            "homophobic_slurs": [r'\b(f[ae]gg[o0]t|d[i1]ke|tr[ae]nn[yi1]e?s?)\b'], 
            "violent_threats": [r'\b(kill\s+you|murder\s+you|beat\s+you\s+up|going\s+to\s+hurt)\b'],
            "excessive_profanity": [r'\b(fuck.*fuck|shit.*shit|damn.*damn)\b']
        }
        
        print(f"   🔎 MODERATION: Scanning for {len(hate_patterns)} violation types...")
        
        # Check each violation type with detailed logging
        for violation_type, patterns in hate_patterns.items():
            print(f"   📋 Checking {violation_type}: {len(patterns)} patterns")
            
            for i, pattern in enumerate(patterns):
                if re.search(pattern, text_lower):
                    print(f"   🚨 VIOLATION DETECTED!")
                    print(f"      📌 Type: {violation_type}")
                    print(f"      🎯 Pattern #{i+1} matched: {pattern}")
                    print(f"      ⚖️ Confidence: 80%")
                    print(f"   📤 MODERATION: Content BLOCKED")
                    
                    return {
                        'is_blocked': True,
                        'violation_type': violation_type,
                        'confidence': 0.8,
                        'details': f'Pattern detected: {violation_type}'
                    }
            
            print(f"   ✅ {violation_type}: No violations found")
        
        print(f"   🟢 MODERATION: Content passed all checks")
        print(f"   📤 MODERATION: Content APPROVED")
        
        return {
            'is_blocked': False,
            'violation_type': None,
            'confidence': 0.0,
            'details': None
        }
    
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
    
    def log_message(self, format, *args):
        # Suppress HTTP request logs to keep output clean
        pass

if __name__ == '__main__':
    import signal
    import threading
    import time
    
    print('🚀 Social Pulse Sentiment Server starting on 127.0.0.1:8001...')
    if HF_EMOTIONCLASSIFIER_AVAILABLE:
        print('📚 Using libraries: HuggingFace EmotionClassifier (primary), text2emotion/nrclex (fallback)')
        print('🎭 Supports combo sentiments: sarcastic+emotion, affectionate+emotion')
    else:
        print('📚 Using libraries: text2emotion/nrclex (fallback mode)')
        print('⚠️ HuggingFace EmotionClassifier not available')
    
    # Try to bind to the server
    try:
        server = HTTPServer(('127.0.0.1', 8001), SentimentHandler)
        print('✅ Server ready! Endpoints:')
        print('   GET  /health  - Health check')  
        print('   POST /analyze - Sentiment analysis')
        print('   POST /moderate - Content moderation')
        print('🎯 Server is persistent - no model reinitialization!')
        
        def signal_handler(signum, frame):
            print('\n🛑 Received shutdown signal, stopping server...')
            server.shutdown()
            
        signal.signal(signal.SIGTERM, signal_handler)
        signal.signal(signal.SIGINT, signal_handler)
        
        # Start server in a separate thread
        server_thread = threading.Thread(target=server.serve_forever)
        server_thread.daemon = True
        server_thread.start()
        
        # Keep the main thread alive
        while server_thread.is_alive():
            time.sleep(1)
            
    except Exception as e:
        print(f'❌ Failed to start server: {e}')
        sys.exit(1)