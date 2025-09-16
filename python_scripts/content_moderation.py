#!/usr/bin/env python3
"""
Content Moderation Script for Social Media App
Detects hate speech, racist, and homophobic content
"""
import sys
import json
import re
from hatesonar import Sonar

HATE_SPEECH_THRESHOLD = 0.6
HATE_SPEECH_LOWER_THRESHOLD = 0.35

def check_content_moderation(text):
    """
    Check if content should be blocked for hate speech
    Returns: "blocked" or "allowed"
    """
    text_lower = text.lower()
    
    sonar = Sonar()
    result = sonar.ping(text_lower)
    hate_speech_prob = result['classes']['hate_speech']
    offensive_prob = result['classes']['offensive_language']
    neither_prob = result['classes']['neither']
    top_class = result['top_class']
    if top_class == 'hate_speech' and hate_speech_prob > HATE_SPEECH_THRESHOLD:
        return "blocked"
    if top_class == 'offensive_language' and hate_speech > HATE_SPEECH_LOWER_THRESHOLD:
        return "blocked"
    if hate_speech > HATE_SPEECH_LOWER_THRESHOLD:
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