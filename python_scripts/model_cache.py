#!/usr/bin/env python3
"""
Model Cache Management for Social Pulse
Handles persistent caching of sentiment and moderation models for faster startup
"""
import json
import time
import pickle
import os

# Caching configuration
CACHE_DIR = "/tmp/social_pulse_cache"
MODEL_CACHE_FILE = os.path.join(CACHE_DIR, "emotion_classifier.pkl")
DETOXIFY_CACHE_FILE = os.path.join(CACHE_DIR, "detoxify_sentinel.json")
CACHE_VERSION = "v1.3"  # Update when model changes

def save_sentiment_model_to_cache(classifier):
    """Save the loaded sentiment classifier to persistent cache"""
    try:
        os.makedirs(CACHE_DIR, exist_ok=True)
        cache_data = {
            'version': CACHE_VERSION,
            'classifier': classifier,
            'timestamp': time.time()
        }
        with open(MODEL_CACHE_FILE, 'wb') as f:
            pickle.dump(cache_data, f)
        print("💾 Sentiment model saved to persistent cache for faster future startups")
    except Exception as e:
        print(f"⚠️ Failed to cache sentiment model: {e}")

def load_sentiment_model_from_cache():
    """Load sentiment classifier from persistent cache if available"""
    if not os.path.exists(MODEL_CACHE_FILE):
        print("📋 No sentiment model cache found, will load from scratch")
        return None
    
    try:
        with open(MODEL_CACHE_FILE, 'rb') as f:
            cache_data = pickle.load(f)
        
        if cache_data.get('version') != CACHE_VERSION:
            print("⚠️ Sentiment cache version mismatch, will reload model")
            return None
        
        print("🚀 Loading sentiment model from persistent cache...")
        classifier = cache_data['classifier']
        
        # Test the cached classifier
        test_result = classifier.predict("I am happy")
        if test_result and 'label' in test_result:
            print("✅ Sentiment model loaded from cache successfully! (Much faster startup)")
            return classifier
        else:
            print("⚠️ Cached sentiment model test failed, will reload from scratch")
            return None
            
    except Exception as e:
        print(f"⚠️ Failed to load sentiment model from cache: {e}, will reload from scratch")
        return None

def save_detoxify_cache_sentinel():
    """Save Detoxify model metadata to sentinel cache (no pickle of torch models)"""
    try:
        os.makedirs(CACHE_DIR, exist_ok=True)
        cache_data = {
            'version': CACHE_VERSION,
            'model_name': 'unbiased',
            'timestamp': time.time(),
            'detoxify_available': True
        }
        with open(DETOXIFY_CACHE_FILE, 'w') as f:
            json.dump(cache_data, f)
        print("💾 Detoxify sentinel cache saved for faster future startups")
    except Exception as e:
        print(f"⚠️ Failed to cache Detoxify sentinel: {e}")

def check_detoxify_cache_sentinel():
    """Check Detoxify sentinel cache (version + timestamp only, no torch model pickle)"""
    if not os.path.exists(DETOXIFY_CACHE_FILE):
        print("📋 No Detoxify sentinel cache found, will load from scratch")
        return False
    
    try:
        with open(DETOXIFY_CACHE_FILE, 'r') as f:
            cache_data = json.load(f)
        
        if cache_data.get('version') != CACHE_VERSION:
            print("⚠️ Detoxify cache version mismatch, will reload model")
            return False
        
        # Check if cache is recent (within 24 hours)
        cache_age = time.time() - cache_data.get('timestamp', 0)
        if cache_age > 86400:  # 24 hours
            print("⚠️ Detoxify cache too old, will reload model")
            return False
        
        print("🚀 Found valid Detoxify sentinel cache, will attempt fresh load...")
        return True
            
    except Exception as e:
        print(f"⚠️ Failed to load Detoxify sentinel cache: {e}, will reload from scratch")
        return False