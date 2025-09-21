#!/usr/bin/env python3
"""
AI Provider Configuration System for Social Pulse
Allows easy switching between different AI providers for sentiment analysis and content moderation
"""
import os
from enum import Enum
from typing import Dict, Any, Optional, Union

class AIProvider(Enum):
    """Supported AI providers for sentiment analysis and content moderation"""
    HUGGINGFACE = "huggingface"
    OPENAI = "openai"
    CLAUDE = "claude"
    AZURE = "azure"
    DETOXIFY = "detoxify"
    LOCAL = "local"

class AIProviderConfig:
    """Configuration manager for AI providers with fallback strategies"""
    
    def __init__(self):
        # Provider selection from environment variables
        self.sentiment_provider = os.getenv('SENTIMENT_PROVIDER', 'huggingface').lower()
        self.moderation_provider = os.getenv('MODERATION_PROVIDER', 'detoxify').lower()
        
        # API Keys and endpoints from environment
        self.openai_key = os.getenv('OPENAI_API_KEY')
        self.claude_key = os.getenv('ANTHROPIC_API_KEY') 
        self.azure_key = os.getenv('AZURE_COGNITIVE_KEY')
        self.azure_endpoint = os.getenv('AZURE_COGNITIVE_ENDPOINT')
        self.local_endpoint = os.getenv('LOCAL_MODEL_ENDPOINT', 'http://localhost:11434')
        self.local_model = os.getenv('LOCAL_MODEL_NAME', 'llama2')
        
        # Thresholds and configuration
        self.confidence_threshold = float(os.getenv('AI_CONFIDENCE_THRESHOLD', '0.5'))
        self.enable_fallback = os.getenv('AI_ENABLE_FALLBACK', 'true').lower() == 'true'
        self.cache_responses = os.getenv('AI_CACHE_RESPONSES', 'true').lower() == 'true'
        
        # Rate limiting (requests per minute)
        self.openai_rate_limit = int(os.getenv('OPENAI_RATE_LIMIT', '60'))
        self.claude_rate_limit = int(os.getenv('CLAUDE_RATE_LIMIT', '60'))
        self.azure_rate_limit = int(os.getenv('AZURE_RATE_LIMIT', '300'))
        
        print(f"🔧 AI Config: Sentiment={self.sentiment_provider}, Moderation={self.moderation_provider}")
        print(f"🔧 Fallback enabled: {self.enable_fallback}, Cache enabled: {self.cache_responses}")
    
    def get_sentiment_analyzer(self):
        """Get the configured sentiment analyzer with fallback"""
        analyzer = None
        
        try:
            if self.sentiment_provider == 'openai' and self.openai_key:
                print("🤖 Loading OpenAI sentiment analyzer...")
                analyzer = self._create_openai_analyzer()
            elif self.sentiment_provider == 'claude' and self.claude_key:
                print("🤖 Loading Claude sentiment analyzer...")
                analyzer = self._create_claude_analyzer()
            elif self.sentiment_provider == 'azure' and self.azure_key and self.azure_endpoint:
                print("🤖 Loading Azure sentiment analyzer...")
                analyzer = self._create_azure_analyzer()
            elif self.sentiment_provider == 'local':
                print("🤖 Loading local model sentiment analyzer...")
                analyzer = self._create_local_analyzer()
            else:
                print("🤖 Loading HuggingFace sentiment analyzer (default)...")
                analyzer = self._create_huggingface_analyzer()
                
        except Exception as e:
            print(f"⚠️ Failed to load {self.sentiment_provider} analyzer: {e}")
            if self.enable_fallback:
                print("🔄 Falling back to HuggingFace analyzer...")
                analyzer = self._create_huggingface_analyzer()
        
        return analyzer or self._create_huggingface_analyzer()
    
    def get_content_moderator(self):
        """Get the configured content moderator with fallback"""
        moderator = None
        
        try:
            if self.moderation_provider == 'openai' and self.openai_key:
                print("🛡️ Loading OpenAI content moderator...")
                moderator = self._create_openai_moderator()
            elif self.moderation_provider == 'claude' and self.claude_key:
                print("🛡️ Loading Claude content moderator...")
                moderator = self._create_claude_moderator()
            else:
                print("🛡️ Loading Detoxify content moderator (default)...")
                moderator = self._create_detoxify_moderator()
                
        except Exception as e:
            print(f"⚠️ Failed to load {self.moderation_provider} moderator: {e}")
            if self.enable_fallback:
                print("🔄 Falling back to Detoxify moderator...")
                moderator = self._create_detoxify_moderator()
        
        return moderator or self._create_detoxify_moderator()
    
    def _create_openai_analyzer(self):
        """Create OpenAI analyzer (requires separate implementation)"""
        try:
            from openai_sentiment import OpenAISentimentAnalyzer
            return OpenAISentimentAnalyzer(
                api_key=self.openai_key,
                rate_limit=self.openai_rate_limit,
                confidence_threshold=self.confidence_threshold
            )
        except ImportError:
            raise Exception("openai_sentiment module not found. Please implement OpenAI integration.")
    
    def _create_claude_analyzer(self):
        """Create Claude analyzer (requires separate implementation)"""
        try:
            from claude_sentiment import ClaudeSentimentAnalyzer
            return ClaudeSentimentAnalyzer(
                api_key=self.claude_key,
                rate_limit=self.claude_rate_limit,
                confidence_threshold=self.confidence_threshold
            )
        except ImportError:
            raise Exception("claude_sentiment module not found. Please implement Claude integration.")
    
    def _create_azure_analyzer(self):
        """Create Azure analyzer (requires separate implementation)"""
        try:
            from azure_sentiment import AzureSentimentAnalyzer
            return AzureSentimentAnalyzer(
                subscription_key=self.azure_key,
                endpoint=self.azure_endpoint,
                rate_limit=self.azure_rate_limit
            )
        except ImportError:
            raise Exception("azure_sentiment module not found. Please implement Azure integration.")
    
    def _create_local_analyzer(self):
        """Create local model analyzer (requires separate implementation)"""
        try:
            from local_sentiment import LocalSentimentAnalyzer
            return LocalSentimentAnalyzer(
                model_name=self.local_model,
                endpoint=self.local_endpoint
            )
        except ImportError:
            raise Exception("local_sentiment module not found. Please implement local model integration.")
    
    def _create_huggingface_analyzer(self):
        """Create HuggingFace analyzer (current default implementation)"""
        from sentiment_analyzer import SentimentAnalyzer
        return SentimentAnalyzer()
    
    def _create_openai_moderator(self):
        """Create OpenAI moderator (requires separate implementation)"""
        try:
            from openai_moderation import OpenAIContentModerator
            return OpenAIContentModerator(
                api_key=self.openai_key,
                rate_limit=self.openai_rate_limit
            )
        except ImportError:
            raise Exception("openai_moderation module not found. Please implement OpenAI moderation.")
    
    def _create_claude_moderator(self):
        """Create Claude moderator (requires separate implementation)"""
        try:
            from claude_moderation import ClaudeContentModerator
            return ClaudeContentModerator(
                api_key=self.claude_key,
                rate_limit=self.claude_rate_limit
            )
        except ImportError:
            raise Exception("claude_moderation module not found. Please implement Claude moderation.")
    
    def _create_detoxify_moderator(self):
        """Create Detoxify moderator (current default implementation)"""
        from content_moderator import ContentModerator
        return ContentModerator()
    
    def get_config_summary(self) -> Dict[str, Any]:
        """Get a summary of the current configuration"""
        return {
            "sentiment_provider": self.sentiment_provider,
            "moderation_provider": self.moderation_provider,
            "confidence_threshold": self.confidence_threshold,
            "fallback_enabled": self.enable_fallback,
            "cache_enabled": self.cache_responses,
            "api_keys_configured": {
                "openai": bool(self.openai_key),
                "claude": bool(self.claude_key),
                "azure": bool(self.azure_key and self.azure_endpoint)
            },
            "rate_limits": {
                "openai": self.openai_rate_limit,
                "claude": self.claude_rate_limit,
                "azure": self.azure_rate_limit
            }
        }

# Global configuration instance
config = AIProviderConfig()

def get_sentiment_analyzer():
    """Global function to get configured sentiment analyzer"""
    return config.get_sentiment_analyzer()

def get_content_moderator():
    """Global function to get configured content moderator"""
    return config.get_content_moderator()

def get_ai_config():
    """Global function to get AI configuration"""
    return config

if __name__ == "__main__":
    # Test the configuration
    print("🧪 Testing AI Configuration...")
    print("Configuration Summary:")
    import json
    print(json.dumps(config.get_config_summary(), indent=2))
    
    print("\n🤖 Testing Sentiment Analyzer...")
    analyzer = get_sentiment_analyzer()
    print(f"Loaded analyzer: {type(analyzer).__name__}")
    
    print("\n🛡️ Testing Content Moderator...")
    moderator = get_content_moderator()
    print(f"Loaded moderator: {type(moderator).__name__}")