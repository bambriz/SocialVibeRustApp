# AI Integration Guide for Social Pulse

## Overview

Social Pulse uses AI-powered sentiment analysis and content moderation to create a safe, emotionally-aware social media environment. This guide explains how to configure thresholds, integrate with different AI providers, and customize the system for your needs.

## Current System Architecture

### Sentiment Analysis Pipeline
- **Primary Engine**: HuggingFace EmotionClassifier
- **Secondary Detection**: Pattern-based analysis for unsupported emotions
- **Supported Emotions**: joy, sad, angry, fear, disgust, surprise, confused, neutral, sarcastic, affectionate
- **Output**: Emotion type + confidence score (0.0-1.0)

### Content Moderation Pipeline  
- **Primary Engine**: Detoxify AI (unbiased model)
- **Detection Categories**: toxicity, severe_toxicity, obscene, threat, insult, identity_attack
- **Blocking Threshold**: identity_attack ≥ 0.8
- **Tagging Thresholds**: Configurable per category

## Configuration & Thresholds

### Current Optimized Thresholds

#### Sentiment Analysis
```python
# Joy bias correction - requires higher confidence
JOY_CONFIDENCE_THRESHOLD = 0.75  # Reduced from default to prevent over-detection
JOY_MAX_CONFIDENCE = 0.85        # Capped to allow other emotions

# Neutral detection patterns include:
# - Factual content: "document contains information"
# - Weather reports: "the weather today is"
# - Mundane activities: "going to work"
```

#### Content Moderation
```python
TOXICITY_THRESHOLDS = {
    'toxicity': 0.55,        # General toxicity (lowered from 0.62)
    'severe_toxicity': 0.45, # Serious violations (lowered from 0.5)
    'obscene': 0.7,          # Profanity (lowered from 0.8)
    'threat': 0.5,           # Threats (lowered from 0.6)
    'insult': 0.45           # Insults (lowered from 0.5)
}

BLOCKING_THRESHOLD = 0.8  # identity_attack threshold for content blocking
```

### Customizing Thresholds

#### Sentiment Analysis Adjustments
Location: `python_scripts/sentiment_analyzer.py`

```python
# Make sentiment detection more/less sensitive
if hf_confidence < 0.75:  # Adjust this value (0.5-0.9)
    # Lower = more joy detection, Higher = more neutral detection

# Adjust base confidence ranges
base_confidence = min(0.90, max(0.4, hf_confidence))  # Adjust min/max
```

#### Content Moderation Adjustments
Location: `python_scripts/content_moderator.py`

```python
# Stricter moderation (lower thresholds)
toxicity_thresholds = {
    'toxicity': 0.4,         # More sensitive
    'severe_toxicity': 0.3,
    'obscene': 0.5,
    'threat': 0.3,
    'insult': 0.3
}

# More lenient moderation (higher thresholds)
toxicity_thresholds = {
    'toxicity': 0.7,         # Less sensitive
    'severe_toxicity': 0.6,
    'obscene': 0.85,
    'threat': 0.7,
    'insult': 0.6
}
```

## Integrating Alternative AI Services

### 1. OpenAI ChatGPT Integration

#### Setup Requirements
```bash
pip install openai
```

#### Implementation Example
```python
# Create: python_scripts/openai_sentiment.py
import openai
from typing import Dict, Any

class OpenAISentimentAnalyzer:
    def __init__(self, api_key: str):
        self.client = openai.OpenAI(api_key=api_key)
    
    def analyze_sentiment(self, text: str) -> Dict[str, Any]:
        try:
            response = self.client.chat.completions.create(
                model="gpt-4",
                messages=[
                    {"role": "system", "content": """Analyze the sentiment of the given text. 
                     Return JSON with: sentiment_type (joy/sad/angry/fear/disgust/surprise/confused/neutral/sarcastic/affectionate), 
                     confidence (0.0-1.0), and reasoning."""},
                    {"role": "user", "content": text}
                ],
                response_format={"type": "json_object"}
            )
            
            result = json.loads(response.choices[0].message.content)
            return {
                'sentiment_type': result.get('sentiment_type', 'neutral'),
                'confidence': result.get('confidence', 0.5),
                'reasoning': result.get('reasoning', '')
            }
        except Exception as e:
            print(f"OpenAI API error: {e}")
            return {'sentiment_type': 'neutral', 'confidence': 0.5}

    def moderate_content(self, text: str) -> Dict[str, Any]:
        try:
            response = self.client.moderations.create(input=text)
            result = response.results[0]
            
            return {
                'is_blocked': result.flagged,
                'toxicity_tags': [cat for cat, flagged in result.categories.__dict__.items() if flagged],
                'all_scores': result.category_scores.__dict__,
                'moderation_system': 'openai_moderation'
            }
        except Exception as e:
            print(f"OpenAI Moderation error: {e}")
            return {'is_blocked': False, 'toxicity_tags': []}
```

### 2. Anthropic Claude Integration

#### Setup Requirements
```bash
pip install anthropic
```

#### Implementation Example
```python
# Create: python_scripts/claude_sentiment.py
import anthropic
from typing import Dict, Any
import json

class ClaudeSentimentAnalyzer:
    def __init__(self, api_key: str):
        self.client = anthropic.Anthropic(api_key=api_key)
    
    def analyze_sentiment(self, text: str) -> Dict[str, Any]:
        try:
            message = self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=150,
                messages=[{
                    "role": "user", 
                    "content": f"""Analyze the sentiment of this text: "{text}"
                    
                    Return valid JSON with:
                    - sentiment_type: one of (joy, sad, angry, fear, disgust, surprise, confused, neutral, sarcastic, affectionate)
                    - confidence: number between 0.0 and 1.0
                    - explanation: brief reasoning
                    
                    Only return the JSON, no other text."""
                }]
            )
            
            result = json.loads(message.content[0].text)
            return {
                'sentiment_type': result.get('sentiment_type', 'neutral'),
                'confidence': result.get('confidence', 0.5),
                'explanation': result.get('explanation', '')
            }
        except Exception as e:
            print(f"Claude API error: {e}")
            return {'sentiment_type': 'neutral', 'confidence': 0.5}

    def moderate_content(self, text: str) -> Dict[str, Any]:
        try:
            message = self.client.messages.create(
                model="claude-3-sonnet-20240229",
                max_tokens=200,
                messages=[{
                    "role": "user",
                    "content": f"""Analyze this text for harmful content: "{text}"
                    
                    Return valid JSON with:
                    - is_blocked: boolean (true if content should be blocked)
                    - toxicity_tags: array of applicable tags (toxicity, severe_toxicity, obscene, threat, insult, identity_attack)
                    - confidence_scores: object with scores 0.0-1.0 for each category
                    - reasoning: brief explanation
                    
                    Only return the JSON."""
                }]
            )
            
            result = json.loads(message.content[0].text)
            return {
                'is_blocked': result.get('is_blocked', False),
                'toxicity_tags': result.get('toxicity_tags', []),
                'all_scores': result.get('confidence_scores', {}),
                'reasoning': result.get('reasoning', ''),
                'moderation_system': 'claude_moderation'
            }
        except Exception as e:
            print(f"Claude Moderation error: {e}")
            return {'is_blocked': False, 'toxicity_tags': []}
```

### 3. Azure Cognitive Services Integration

#### Setup Requirements
```bash
pip install azure-cognitiveservices-language-textanalytics
```

#### Implementation Example
```python
# Create: python_scripts/azure_sentiment.py
from azure.cognitiveservices.language.textanalytics import TextAnalyticsClient
from azure.cognitiveservices.language.textanalytics.models import TextAnalyticsError
from msrest.authentication import CognitiveServicesCredentials

class AzureSentimentAnalyzer:
    def __init__(self, subscription_key: str, endpoint: str):
        credentials = CognitiveServicesCredentials(subscription_key)
        self.client = TextAnalyticsClient(endpoint=endpoint, credentials=credentials)
    
    def analyze_sentiment(self, text: str) -> Dict[str, Any]:
        try:
            documents = [{"id": "1", "language": "en", "text": text}]
            response = self.client.sentiment(documents=documents)
            
            if response.documents:
                doc = response.documents[0]
                # Map Azure sentiment to our system
                sentiment_mapping = {
                    'positive': 'joy',
                    'negative': 'sad', 
                    'neutral': 'neutral',
                    'mixed': 'confused'
                }
                
                return {
                    'sentiment_type': sentiment_mapping.get(doc.sentiment, 'neutral'),
                    'confidence': max(doc.confidence_scores.positive, 
                                    doc.confidence_scores.negative,
                                    doc.confidence_scores.neutral)
                }
        except Exception as e:
            print(f"Azure API error: {e}")
            return {'sentiment_type': 'neutral', 'confidence': 0.5}
```

### 4. Configuration System for Multiple Providers

#### Create Provider Configuration
```python
# Create: python_scripts/ai_config.py
import os
from enum import Enum
from typing import Dict, Any, Optional

class AIProvider(Enum):
    HUGGINGFACE = "huggingface"
    OPENAI = "openai"
    CLAUDE = "claude"
    AZURE = "azure"
    DETOXIFY = "detoxify"

class AIProviderConfig:
    def __init__(self):
        self.sentiment_provider = os.getenv('SENTIMENT_PROVIDER', 'huggingface')
        self.moderation_provider = os.getenv('MODERATION_PROVIDER', 'detoxify')
        
        # API Keys from environment
        self.openai_key = os.getenv('OPENAI_API_KEY')
        self.claude_key = os.getenv('ANTHROPIC_API_KEY')
        self.azure_key = os.getenv('AZURE_COGNITIVE_KEY')
        self.azure_endpoint = os.getenv('AZURE_COGNITIVE_ENDPOINT')
    
    def get_sentiment_analyzer(self):
        if self.sentiment_provider == 'openai' and self.openai_key:
            from openai_sentiment import OpenAISentimentAnalyzer
            return OpenAISentimentAnalyzer(self.openai_key)
        elif self.sentiment_provider == 'claude' and self.claude_key:
            from claude_sentiment import ClaudeSentimentAnalyzer
            return ClaudeSentimentAnalyzer(self.claude_key)
        elif self.sentiment_provider == 'azure' and self.azure_key:
            from azure_sentiment import AzureSentimentAnalyzer
            return AzureSentimentAnalyzer(self.azure_key, self.azure_endpoint)
        else:
            # Default to HuggingFace
            from sentiment_analyzer import SentimentAnalyzer
            return SentimentAnalyzer()

    def get_content_moderator(self):
        if self.moderation_provider == 'openai' and self.openai_key:
            from openai_sentiment import OpenAISentimentAnalyzer
            return OpenAISentimentAnalyzer(self.openai_key)
        elif self.moderation_provider == 'claude' and self.claude_key:
            from claude_sentiment import ClaudeSentimentAnalyzer
            return ClaudeSentimentAnalyzer(self.claude_key)
        else:
            # Default to Detoxify
            from content_moderator import ContentModerator
            return ContentModerator()
```

#### Environment Configuration
```bash
# Add to your .env file or environment variables
SENTIMENT_PROVIDER=openai          # or huggingface, claude, azure
MODERATION_PROVIDER=openai         # or detoxify, claude
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_claude_key
AZURE_COGNITIVE_KEY=your_azure_key
AZURE_COGNITIVE_ENDPOINT=your_azure_endpoint
```

### 5. GitHub Copilot / Local Model Integration

For development environments, you can use local models or GitHub Copilot suggestions:

#### Local Model Setup (Ollama)
```python
# Create: python_scripts/local_sentiment.py
import requests
import json

class LocalSentimentAnalyzer:
    def __init__(self, model_name: str = "llama2", endpoint: str = "http://localhost:11434"):
        self.model = model_name
        self.endpoint = endpoint
    
    def analyze_sentiment(self, text: str) -> Dict[str, Any]:
        try:
            prompt = f"""Analyze the sentiment of this text: "{text}"
            
            Return only valid JSON with:
            {{
                "sentiment_type": "one of: joy, sad, angry, fear, disgust, surprise, confused, neutral, sarcastic, affectionate",
                "confidence": 0.7
            }}"""
            
            response = requests.post(f"{self.endpoint}/api/generate", json={
                "model": self.model,
                "prompt": prompt,
                "stream": False
            })
            
            if response.status_code == 200:
                result = response.json()
                # Parse the JSON from the response
                analysis = json.loads(result['response'])
                return analysis
                
        except Exception as e:
            print(f"Local model error: {e}")
            return {'sentiment_type': 'neutral', 'confidence': 0.5}
```

## Best Practices & Recommendations

### 1. Fallback Strategy
Always implement fallback mechanisms:
```python
def analyze_with_fallback(text: str):
    try:
        # Try primary provider (e.g., OpenAI)
        return primary_analyzer.analyze(text)
    except Exception:
        try:
            # Fallback to secondary provider (e.g., HuggingFace)
            return secondary_analyzer.analyze(text)
        except Exception:
            # Final fallback to pattern-based analysis
            return pattern_analyzer.analyze(text)
```

### 2. Rate Limiting & Caching
```python
import time
from functools import lru_cache

class RateLimitedAnalyzer:
    def __init__(self, requests_per_minute: int = 60):
        self.rate_limit = requests_per_minute
        self.last_request = 0
    
    @lru_cache(maxsize=1000)
    def cached_analyze(self, text: str):
        # Rate limiting
        now = time.time()
        if now - self.last_request < 60 / self.rate_limit:
            time.sleep((60 / self.rate_limit) - (now - self.last_request))
        
        self.last_request = time.time()
        return self.provider.analyze(text)
```

### 3. A/B Testing Different Providers
```python
class MultiProviderAnalyzer:
    def __init__(self):
        self.providers = {
            'openai': OpenAIAnalyzer(),
            'claude': ClaudeAnalyzer(), 
            'huggingface': HuggingFaceAnalyzer()
        }
        self.user_assignments = {}  # user_id -> provider
    
    def analyze_for_user(self, user_id: str, text: str):
        provider = self.get_user_provider(user_id)
        return self.providers[provider].analyze(text)
```

### 4. Cost Optimization
- **Free Tier Usage**: Start with HuggingFace for development
- **Production Scale**: Use OpenAI/Claude for accuracy, HuggingFace for volume
- **Hybrid Approach**: AI for complex cases, patterns for simple cases

### 5. Privacy & Security
- **API Key Management**: Use environment variables, never commit keys
- **Data Privacy**: Consider local models for sensitive content
- **Audit Logging**: Track all AI provider responses for debugging

## Performance Monitoring

### Response Time Tracking
```python
import time
import logging

def track_performance(func):
    def wrapper(*args, **kwargs):
        start_time = time.time()
        result = func(*args, **kwargs)
        duration = time.time() - start_time
        
        logging.info(f"AI Analysis took {duration:.2f}s - Provider: {func.__name__}")
        return result
    return wrapper
```

### Accuracy Monitoring
```python
class AccuracyTracker:
    def __init__(self):
        self.predictions = []
        self.ground_truth = []
    
    def log_prediction(self, text: str, predicted: str, actual: str = None):
        self.predictions.append((text, predicted, actual))
        
    def calculate_accuracy(self):
        if not self.ground_truth:
            return None
        correct = sum(1 for p, a in zip(self.predictions, self.ground_truth) if p[1] == a)
        return correct / len(self.ground_truth)
```

## Quick Start Examples

### Switching to OpenAI
```bash
# 1. Install dependency
pip install openai

# 2. Set environment variable
export SENTIMENT_PROVIDER=openai
export OPENAI_API_KEY=your_key_here

# 3. Restart your application
cargo run
```

### Switching to Claude
```bash
# 1. Install dependency  
pip install anthropic

# 2. Set environment variable
export SENTIMENT_PROVIDER=claude
export ANTHROPIC_API_KEY=your_key_here

# 3. Restart your application
cargo run
```

This integration guide provides a foundation for customizing Social Pulse's AI capabilities. Choose the configuration that best fits your accuracy requirements, cost constraints, and privacy needs.