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
    Returns: "blocked:reason" or "allowed"
    """
    text_lower = text.lower()
    
    # Define categorized hate speech patterns
    pattern_categories = {
        'racial_slurs': [
            r'\b(n\*\*ger|n\*gger|nigg[aer]+s?|retards?|retarded)\b',
            r'\b(ch\*nks?|ch[1i]nks?|sp[1i]cs?|k[1i]kes?|ragheads?|towelheads?)\b',
            r'\b(wetbacks?|beaners?|crackers?|honkeys?|whiteys?|gooks?|japs?)\b',
            r'\b(spics?|chinks?|kikes?)\b',
        ],
        'homophobic_slurs': [
            r'\b(f\*ggot|faggots?|fags?|dykes?|trann(y|ies)|homos?|queers?)\b',
        ],
        'hate_speech_terms': [
            r'\b(sand\s*nigg[aer]+s?|mud\s*people|white\s*trash|trailer\s*trash)\b',
            r'\b(illegals?|border\s*jumpers?|anchor\s*babies?)\b',
        ],
        'violent_threats': [
            r'\b(kill\s+all|genocide|lynch|hang\s+the|shoot\s+the)\b.*\b(blacks|jews|muslims|gays|women|men|latinos|hispanics|immigrants)\b',
            r'\b(gas\s+the|exterminate|eliminate)\b.*\b(jews|blacks|muslims|gays|latinos|hispanics)\b',
        ],
        'direct_threats': [
            r'\b(i\s+will\s+kill|gonna\s+kill|will\s+murder|should\s+die)\b',
            r'\b(burn\s+in\s+hell|hope\s+you\s+die|kill\s+yourself|should\s+be\s+shot)\b',
        ],
        'hate_speech_with_context': [
            r'\b(fucking|damn|shit)\b.*\b(jews|blacks|muslims|gays|women|immigrants|latinos|hispanics|beaners|queers)\b',
            r'\b(jews|blacks|muslims|gays|women|immigrants|latinos|hispanics|beaners|queers)\b.*\b(are\s+trash|are\s+scum|should\s+be\s+killed|ruin\s+everything|are\s+disgusting)\b',
        ],
        'derogatory_statements': [
            r'\bi\s+hate\s+(jews|blacks|muslims|gays|women|immigrants|latinos|hispanics|beaners|queers)\b',
            r'\b(disgusting|filthy|dirty)\s+(jews|blacks|muslims|gays|women|immigrants|latinos|hispanics|beaners|queers)\b',
        ]
    }
    
    # Check for each category of hate speech patterns
    for category, patterns in pattern_categories.items():
        for pattern in patterns:
            if re.search(pattern, text_lower):
                return f"blocked:{category}"
    
    # Check for excessive profanity (more than 3 strong profanity words)
    profanity_pattern = r'\b(fuck|shit|bitch|ass|damn|crap|hell|piss)\b'
    profanity_matches = re.findall(profanity_pattern, text_lower)
    if len(profanity_matches) > 3:
        return f"blocked:excessive_profanity:{len(profanity_matches)}_words"
        
    # Skip hatesonar to avoid model download delays - rely on fast pattern matching only
    # try:
    #     from hatesonar import Sonar
    #     sonar = Sonar()
    #     result = sonar.ping(text_lower)
    #     hate_speech_prob = result['classes']['hate_speech']
    #     offensive_prob = result['classes']['offensive_language']
    #     
    #     # More lenient threshold for backup check
    #     if hate_speech_prob > 0.8:  # Only block very high confidence hate speech
    #         return f"blocked:ai_hate_speech_detection:{hate_speech_prob:.2f}_confidence"
    #     elif offensive_prob > 0.9:  # Very high confidence offensive language
    #         return f"blocked:ai_offensive_language:{offensive_prob:.2f}_confidence"
    # except (ImportError, Exception) as e:
    #     # If hatesonar fails, rely on pattern matching only
    #     pass
    
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