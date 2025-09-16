#!/usr/bin/env python3
"""
Advanced ML-based Sentiment Analysis Service
Uses transformer models for more accurate emotion and sarcasm detection
Fallback to rule-based analysis on errors
"""

import os
import json
import logging
from typing import Dict, Optional, Tuple
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import uvicorn

# Set up logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Configure transformers cache to persist models
os.environ['TRANSFORMERS_CACHE'] = './model_cache'

try:
    from transformers import pipeline
    TRANSFORMERS_AVAILABLE = True
    logger.info("Transformers library loaded successfully")
except ImportError:
    TRANSFORMERS_AVAILABLE = False
    logger.warning("Transformers not available, will use fallback")

app = FastAPI(title="ML Sentiment Analysis Service")

# Global model variables
emotion_classifier = None
sarcasm_classifier = None

class TextRequest(BaseModel):
    text: str

class SentimentResponse(BaseModel):
    sentiment: str
    confidence: float

def load_models():
    """Load transformer models at startup"""
    global emotion_classifier, sarcasm_classifier
    
    if not TRANSFORMERS_AVAILABLE:
        logger.warning("Transformers not available - ML models disabled")
        return
    
    try:
        logger.info("Loading emotion classification model...")
        emotion_classifier = pipeline(
            "text-classification", 
            model="j-hartmann/emotion-english-distilroberta-base",
            top_k=1,
            device=-1  # Use CPU
        )
        logger.info("Emotion model loaded successfully")
        
        logger.info("Loading sarcasm detection model...")
        sarcasm_classifier = pipeline(
            "text-classification",
            model="cardiffnlp/twitter-roberta-base-irony",
            top_k=1,
            device=-1  # Use CPU
        )
        logger.info("Sarcasm model loaded successfully")
        
    except Exception as e:
        logger.error(f"Failed to load models: {e}")
        # Keep models as None to trigger fallback

def map_emotion_to_sentiment(emotion: str, confidence: float, text: str) -> Tuple[str, float]:
    """
    Map transformer emotion outputs to our 7-emotion system
    j-hartmann model outputs: anger, disgust, fear, joy, neutral, sadness, surprise
    """
    emotion_mapping = {
        'joy': 'happy',
        'sadness': 'sad', 
        'anger': 'angry',
        'fear': 'fear',
        'neutral': 'calm',
        'disgust': 'angry'  # Map disgust to angry as fallback
    }
    
    # Special handling for surprise - context-dependent
    if emotion == 'surprise':
        text_lower = text.lower()
        # Check for excitement indicators
        if any(word in text_lower for word in ['!', 'wow', 'amazing', 'awesome', 'great', 'fantastic']):
            return 'excited', confidence
        else:
            return 'confused', confidence
    
    mapped_sentiment = emotion_mapping.get(emotion, 'calm')
    return mapped_sentiment, confidence

def analyze_with_transformers(text: str) -> str:
    """Analyze text using transformer models"""
    if not emotion_classifier or not sarcasm_classifier:
        raise Exception("Models not loaded")
    
    # Get emotion classification
    emotion_result = emotion_classifier(text)[0]
    emotion_label = emotion_result['label'].lower()
    emotion_confidence = emotion_result['score']
    
    # Map to our sentiment system
    base_sentiment, mapped_confidence = map_emotion_to_sentiment(
        emotion_label, emotion_confidence, text
    )
    
    # Get sarcasm classification
    sarcasm_result = sarcasm_classifier(text)[0]
    # cardiffnlp model returns 'IRONY' for sarcastic, 'NOT_IRONY' for non-sarcastic
    is_sarcastic = sarcasm_result['label'] == 'IRONY'
    sarcasm_confidence = sarcasm_result['score']
    
    # Combine results
    if is_sarcastic and sarcasm_confidence > 0.6:  # High confidence sarcasm threshold
        # Combine sarcasm with base emotion
        combined_confidence = min((mapped_confidence + sarcasm_confidence) / 2, 1.0)
        return f"sarcastic+{base_sentiment}:{combined_confidence:.2f}"
    else:
        return f"{base_sentiment}:{mapped_confidence:.2f}"

def analyze_with_fallback(text: str) -> str:
    """Fallback to rule-based analysis"""
    # Import and use the existing rule-based analyzer
    import sys
    import subprocess
    
    try:
        result = subprocess.run([
            sys.executable, 'sentiment_analysis.py', text
        ], capture_output=True, text=True, timeout=5, cwd='/tmp/sentiment_scripts')
        
        if result.returncode == 0:
            return result.stdout.strip()
        else:
            logger.error(f"Rule-based analysis failed: {result.stderr}")
            return "calm:0.5"
    except Exception as e:
        logger.error(f"Fallback analysis failed: {e}")
        return "calm:0.5"

@app.post("/analyze", response_model=SentimentResponse)
async def analyze_sentiment(request: TextRequest):
    """Analyze sentiment using ML models with fallback"""
    try:
        # Validate input
        if not request.text or len(request.text.strip()) == 0:
            raise HTTPException(status_code=400, detail="Text cannot be empty")
        
        if len(request.text) > 1000:  # Limit input size
            raise HTTPException(status_code=400, detail="Text too long (max 1000 chars)")
        
        # Try ML analysis first
        if emotion_classifier and sarcasm_classifier:
            try:
                result = analyze_with_transformers(request.text)
                logger.info(f"ML analysis successful: {result}")
                
                # Parse result to return structured response
                parts = result.split(':')
                sentiment = parts[0]
                confidence = float(parts[1]) if len(parts) > 1 else 0.5
                
                return SentimentResponse(sentiment=sentiment, confidence=confidence)
                
            except Exception as e:
                logger.warning(f"ML analysis failed, using fallback: {e}")
        
        # Fallback to rule-based
        result = analyze_with_fallback(request.text)
        parts = result.split(':')
        sentiment = parts[0]
        confidence = float(parts[1]) if len(parts) > 1 else 0.5
        
        return SentimentResponse(sentiment=sentiment, confidence=confidence)
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Analysis failed: {e}")
        raise HTTPException(status_code=500, detail="Analysis failed")

@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "transformers_available": TRANSFORMERS_AVAILABLE,
        "models_loaded": emotion_classifier is not None and sarcasm_classifier is not None
    }

# Load models on startup
@app.on_event("startup")
async def startup_event():
    logger.info("Starting ML Sentiment Analysis Service...")
    load_models()

if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=8001, log_level="info")