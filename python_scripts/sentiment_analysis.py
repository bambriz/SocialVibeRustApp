#!/usr/bin/env python3
"""
Sentiment Analysis Script for Social Media App
Analyzes text and returns sentiment type with confidence score
"""
import sys
import json
import re

# Simple sentiment analysis implementation (can be replaced with ML models)
def analyze_sentiment(text):
    """
    Simple rule-based sentiment analysis
    Returns: sentiment_type:confidence
    """
    text_lower = text.lower()
    
    # Define sentiment patterns
    patterns = {
        'happy': [r'\bhappy\b', r'\bjoy\b', r'\bgreat\b', r'\bawesome\b', r'\blove\b', r'😊', r'😄', r'🎉'],
        'sad': [r'\bsad\b', r'\bdepressed\b', r'\bunhappy\b', r'\bcry\b', r'\btears\b', r'😢', r'😭'],
        'angry': [r'\bangry\b', r'\bmad\b', r'\bfurious\b', r'\brage\b', r'\bhate\b', r'😡', r'🤬'],
        'fear': [r'\bscared\b', r'\bafraid\b', r'\bterrified\b', r'\banxious\b', r'\bworried\b', r'😨', r'😰'],
        'calm': [r'\bcalm\b', r'\bpeaceful\b', r'\brelaxed\b', r'\btranquil\b', r'\bserene\b'],
        'affection': [r'\blove\b', r'\badore\b', r'\bcare\b', r'\bsweet\b', r'\bdear\b', r'❤️', r'💕'],
        'sarcastic': [r'\bsarcastic\b', r'\boh really\b', r'\bobviously\b', r'\bsure thing\b', r'\byeah right\b']
    }
    
    sentiment_scores = {}
    
    for sentiment, pattern_list in patterns.items():
        score = 0
        for pattern in pattern_list:
            matches = len(re.findall(pattern, text_lower))
            score += matches
        
        if score > 0:
            sentiment_scores[sentiment] = min(score / len(text.split()) * 10, 1.0)  # Normalize
    
    if not sentiment_scores:
        return "calm:0.5"  # Default neutral sentiment
    
    # Return highest scoring sentiment
    best_sentiment = max(sentiment_scores.items(), key=lambda x: x[1])
    return f"{best_sentiment[0]}:{best_sentiment[1]:.2f}"

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 sentiment_analysis.py <text>", file=sys.stderr)
        sys.exit(1)
    
    text = sys.argv[1]
    result = analyze_sentiment(text)
    print(result)

if __name__ == "__main__":
    main()