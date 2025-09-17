#!/usr/bin/env python3
"""
Social Pulse - Persistent Sentiment Analysis Server

This server provides HTTP endpoints for sentiment analysis and content moderation,
eliminating the need to reinitialize Python libraries on each request.
Uses EmotionClassifier as primary detector with NRCLex as fallback.
"""
import json
import sys
import re
import subprocess
from http.server import HTTPServer, BaseHTTPRequestHandler
from nrclex import NRCLex

# Initialize EmotionClassifier as primary detector
EMOTIONCLASSIFIER_AVAILABLE = False
emotion_classifier = None

try:
    from text2emotion import get_emotion
    EMOTIONCLASSIFIER_AVAILABLE = True
    print("✅ EmotionClassifier (text2emotion) loaded successfully")
except ImportError as e:
    print(f"⚠️ EmotionClassifier not available: {e}")
    print("📚 Will use NRCLex as fallback")

# Pre-initialize NRCLex as fallback
nrclex_fallback = None

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
            if EMOTIONCLASSIFIER_AVAILABLE:
                libraries.insert(0, "emotionclassifier")
            self.wfile.write(json.dumps({"status": "healthy", "libraries": libraries, "primary_detector": "emotionclassifier" if EMOTIONCLASSIFIER_AVAILABLE else "nrclex"}).encode('utf-8'))
        else:
            self.send_error(404)
    
    def analyze_sentiment(self, text):
        """
        Analyzes sentiment using EmotionClassifier as primary detector.
        Falls back to NRCLex if EmotionClassifier fails.
        """
        text_clean = text.strip()
        text_lower = text_clean.lower()
        
        # Check for sarcasm patterns first
        sarcasm_patterns = [
            r'\b(oh\s+great|obviously|of\s+course|sure\s+thing|yeah\s+right)\b',
            r'\b(just\s+perfect|just\s+great|how\s+wonderful|absolutely\s+perfect)\b', 
            r'\b(living\s+the\s+dream|perfect\s+timing|magical\s+start)\b',
            r'\b(couldn\'t\s+have\s+asked\s+for|what\s+a\s+day|how\s+perfect)\b',
            r'\b(working\s+flawlessly|could\s+not\s+have\s+asked)\b'
        ]
        
        is_sarcastic = any(re.search(pattern, text_lower) for pattern in sarcasm_patterns)
        
        # Try EmotionClassifier first (primary detector)
        mapped_emotion = 'calm'
        base_confidence = 0.3
        
        if EMOTIONCLASSIFIER_AVAILABLE:
            try:
                emotion_scores = get_emotion(text_clean)
                if emotion_scores:
                    # Find the dominant emotion
                    dominant_emotion = max(emotion_scores, key=emotion_scores.get)
                    emotion_confidence = emotion_scores[dominant_emotion]
                    
                    # Map EmotionClassifier emotions to our system
                    emotion_mapping = {
                        'Happy': 'happy',
                        'Angry': 'angry', 
                        'Surprise': 'surprise',
                        'Sad': 'sad',
                        'Fear': 'fear'
                    }
                    
                    mapped_emotion = emotion_mapping.get(dominant_emotion, 'calm')
                    base_confidence = min(0.9, max(0.4, emotion_confidence))
                    
                    # Special handling for disgust (EmotionClassifier may not have this)
                    if re.search(r'\b(disgusting|gross|revolting|nauseating|yuck|ew|horrible|nasty)\b', text_lower):
                        mapped_emotion = 'disgust'
                        base_confidence = 0.7
                        
            except Exception as e:
                print(f"EmotionClassifier failed: {e}, falling back to NRCLex")
                # Fall through to NRCLex fallback
        
        # Fallback to NRCLex if EmotionClassifier not available or failed
        if mapped_emotion == 'calm' and base_confidence == 0.3:
            try:
                emotions = NRCLex(text_lower)
                # Use correct attribute name for NRCLex
                emotion_scores = emotions.affect_frequencies if hasattr(emotions, 'affect_frequencies') else emotions.affect_list
                
                if emotion_scores and hasattr(emotions, 'affect_frequencies'):
                    # Get the highest scoring emotion
                    if emotion_scores:
                        dominant_emotion = max(emotion_scores, key=emotion_scores.get)
                        emotion_mapping = {
                            'joy': 'joy', 'sadness': 'sad', 'anger': 'angry',
                            'fear': 'fear', 'disgust': 'disgust', 'surprise': 'surprise',
                            'anticipation': 'excited', 'trust': 'affection',
                            'positive': 'happy', 'negative': 'sad'
                        }
                        mapped_emotion = emotion_mapping.get(dominant_emotion, 'calm')
                        base_confidence = min(0.8, max(0.4, emotion_scores[dominant_emotion]))
                        
            except Exception as e:
                print(f"NRCLex also failed: {e}, using pattern-based detection")
        
        # Pattern-based enhancements (always apply)
        if re.search(r'\b(disgusting|gross|revolting|nauseating|yuck|ew|horrible|nasty)\b', text_lower):
            mapped_emotion = 'disgust'
            base_confidence = 0.7
        elif re.search(r'\b(amazing|awesome|fantastic|wonderful|great)\b', text_lower):
            mapped_emotion = 'joy'
            base_confidence = 0.8
        elif re.search(r'\b(hate|angry|furious|mad|pissed)\b', text_lower):
            mapped_emotion = 'angry'
            base_confidence = 0.7
            
        # Handle sarcasm combinations
        if is_sarcastic:
            return {
                'sentiment_type': f'sarcastic+{mapped_emotion}',
                'confidence': 0.8,
                'is_sarcastic': True
            }
        
        return {
            'sentiment_type': mapped_emotion,
            'confidence': base_confidence,
            'is_sarcastic': False
        }
    
    def moderate_content(self, text):
        """
        Content moderation with pattern matching for harmful content.
        """
        text_lower = text.lower()
        
        # Define moderation patterns
        hate_patterns = {
            "racial_slurs": [r'\b(n[i1]gg[ae]r|ch[i1]nk|sp[i1]c|k[i1]ke)\b'],
            "homophobic_slurs": [r'\b(f[ae]gg[o0]t|d[i1]ke|tr[ae]nn[yi1]e?s?)\b'], 
            "violent_threats": [r'\b(kill\s+you|murder\s+you|beat\s+you\s+up|going\s+to\s+hurt)\b'],
            "excessive_profanity": [r'\b(fuck.*fuck|shit.*shit|damn.*damn)\b']
        }
        
        for violation_type, patterns in hate_patterns.items():
            for pattern in patterns:
                if re.search(pattern, text_lower):
                    return {
                        'is_blocked': True,
                        'violation_type': violation_type,
                        'confidence': 0.8,
                        'details': f'Pattern detected: {violation_type}'
                    }
        
        return {
            'is_blocked': False,
            'violation_type': None,
            'confidence': 0.0,
            'details': None
        }
    
    def log_message(self, format, *args):
        # Suppress HTTP request logs to keep output clean
        pass

if __name__ == '__main__':
    import signal
    import threading
    import time
    
    print('🚀 Social Pulse Sentiment Server starting on 127.0.0.1:8001...')
    if EMOTIONCLASSIFIER_AVAILABLE:
        print('📚 Using libraries: emotionclassifier (primary), nrclex (fallback)')
    else:
        print('📚 Using libraries: nrclex (primary)')
    
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