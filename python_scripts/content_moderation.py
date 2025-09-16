#!/usr/bin/env python3
"""
Content Moderation Script for Social Media App
Detects hate speech, racist, and homophobic content
"""
import sys
import json
import re

def check_content_moderation(text):
    """
    Check if content should be blocked for hate speech
    Returns: "blocked" or "allowed"
    """
    text_lower = text.lower()
    
    # Define offensive patterns (this is a basic implementation)
    # In production, use more sophisticated ML models
    hate_speech_patterns = [
        # Racial slurs and discriminatory language (censored examples)
        r'\b(racist?|bigot|discrimination|prejudice)\b',
        # Homophobic language (censored examples)  
        r'\b(homophobic?|transphobic?)\b',
        # General hate patterns
        r'\bhate\s+(you|them|him|her)\b',
        r'\bkill\s+(yourself|themselves)\b',
        r'\b(die|death)\s+(to|for)\s+',
        r'\b(stupid|dumb|idiot)\s+(people|person)\b',
        # Threat patterns
        r'\bthreat(en)?(s|ed)?\b',
        r'\bviolent?|violence\b',
    ]
    
    # Check for hate speech patterns
    for pattern in hate_speech_patterns:
        if re.search(pattern, text_lower):
            return "blocked"
    
    # Check for excessive profanity (basic implementation)
    profanity_count = 0
    profanity_patterns = [
        r'\bf\*+k\b', r'\bs\*+t\b', r'\bd\*+n\b',  # Censored profanity
        r'\bbitch\b', r'\basshole\b'
    ]
    
    for pattern in profanity_patterns:
        matches = len(re.findall(pattern, text_lower))
        profanity_count += matches
    
    # Block if too much profanity
    if profanity_count > 2:
        return "blocked"
    
    return "allowed"

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 content_moderation.py <text>", file=sys.stderr)
        sys.exit(1)
    
    text = sys.argv[1]
    result = check_content_moderation(text)
    print(result)

if __name__ == "__main__":
    main()