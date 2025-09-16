#!/usr/bin/env python3
"""
Content Moderation Script for Social Media App
Detects hate speech, racist, and homophobic content using pattern matching
"""
import sys
import re

def check_content_moderation(text):
    """
    Check if content should be blocked for hate speech using rule-based patterns
    Returns: "blocked" or "allowed"
    """
    text_lower = text.lower()
    
    # Define hate speech patterns
    hate_patterns = [
        # Racial slurs and hate speech (singular and plural forms)
        r'\b(n\*\*ger|n\*gger|nigg[aer]+s?|f\*ggot|faggots?|retards?|retarded)\b',
        r'\b(ch\*nks?|ch[1i]nks?|sp[1i]cs?|k[1i]kes?|ragheads?|towelheads?)\b',
        r'\b(wetbacks?|beaners?|crackers?|honkeys?|whiteys?|gooks?|japs?)\b',
        r'\b(spics?|chinks?|kikes?)\b',  # Additional common variants
        
        # Homophobic slurs (singular and plural forms)
        r'\b(fags?|dykes?|trann(y|ies)|homos?|queers?)\b',
        
        # Additional hate speech terms
        r'\b(sand\s*nigg[aer]+s?|mud\s*people|white\s*trash|trailer\s*trash)\b',
        r'\b(illegals?|border\s*jumpers?|anchor\s*babies?)\b',
        
        # Violent hate speech
        r'\b(kill\s+all|genocide|lynch|hang\s+the|shoot\s+the)\b.*\b(blacks|jews|muslims|gays|women|men|latinos|hispanics|immigrants)\b',
        r'\b(gas\s+the|exterminate|eliminate)\b.*\b(jews|blacks|muslims|gays|latinos|hispanics)\b',
        
        # Direct threats
        r'\b(i\s+will\s+kill|gonna\s+kill|will\s+murder|should\s+die)\b',
        r'\b(burn\s+in\s+hell|hope\s+you\s+die|kill\s+yourself|should\s+be\s+shot)\b',
        
        # Hate speech with context
        r'\b(fucking|damn|shit)\b.*\b(jews|blacks|muslims|gays|women|immigrants|latinos|hispanics|beaners|queers)\b',
        r'\b(jews|blacks|muslims|gays|women|immigrants|latinos|hispanics|beaners|queers)\b.*\b(are\s+trash|are\s+scum|should\s+be\s+killed|ruin\s+everything|are\s+disgusting)\b',
        
        # Derogatory statements
        r'\bi\s+hate\s+(jews|blacks|muslims|gays|women|immigrants|latinos|hispanics|beaners|queers)\b',
        r'\b(disgusting|filthy|dirty)\s+(jews|blacks|muslims|gays|women|immigrants|latinos|hispanics|beaners|queers)\b',
    ]
    
    # Check for hate speech patterns
    for pattern in hate_patterns:
        if re.search(pattern, text_lower):
            return "blocked"
    
    # Check for excessive profanity (more than 3 strong profanity words)
    profanity_pattern = r'\b(fuck|shit|bitch|ass|damn|crap|hell|piss)\b'
    profanity_count = len(re.findall(profanity_pattern, text_lower))
    if profanity_count > 3:
        return "blocked"
        
    # Try to use hatesonar as backup if available
    try:
        from hatesonar import Sonar
        sonar = Sonar()
        result = sonar.ping(text_lower)
        hate_speech_prob = result['classes']['hate_speech']
        
        # More lenient threshold for backup check
        if hate_speech_prob > 0.8:  # Only block very high confidence hate speech
            return "blocked"
    except (ImportError, Exception):
        # If hatesonar fails, rely on pattern matching only
        pass
    
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