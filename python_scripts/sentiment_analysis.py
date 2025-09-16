#!/usr/bin/env python3
"""
Enhanced Sentiment Analysis Script for Social Media App
Analyzes text and returns sentiment type with confidence score, including sarcasm detection
"""
import sys
import json
import re

def analyze_sentiment(text):
    """
    Enhanced rule-based sentiment analysis with sarcasm detection
    Returns: sentiment_type:confidence or sarcastic+sentiment_type:confidence for combinations
    """
    text_lower = text.lower()
    
    # Enhanced sentiment patterns with much more comprehensive coverage
    patterns = {
        'happy': [
            r'\bhappy\b', r'\bjoy\b', r'\bgreat\b', r'\bawesome\b', r'\blove\b', r'\bwonderful\b',
            r'\bamazing\b', r'\bfantastic\b', r'\bexcellent\b', r'\bperfect\b', r'\bbrilliant\b',
            r'\bdelighted\b', r'\bcheerful\b', r'\bjoyful\b', r'\bglad\b', r'\bpleased\b',
            r'\bgood\b', r'\bnice\b', r'\bpositive\b', r'\bupbeat\b', r'\boptimistic\b',
            r'😊', r'😄', r'🎉', r'😍', r'🥰', r'😎', r'👍'
        ],
        'excited': [
            r'\bexcited\b', r'\bthrilled\b', r'\bpumped\b', r'\bethusiastic\b', r'\beager\b',
            r'\bstoked\b', r'\bfired up\b', r'\becstatic\b', r'\belated\b', r'\bpsyched\b',
            r'\bcan\'t wait\b', r'\bso ready\b', r'\blooking forward\b', r'\bsuper\b',
            r'!!+', r'\bwow\b', r'\bomg\b', r'\byay\b', r'\bwoop\b', r'\byeah\b',
            r'🤩', r'🎊', r'🔥', r'⚡', r'🚀'
        ],
        'sad': [
            r'\bsad\b', r'\bdepressed\b', r'\bunhappy\b', r'\bcry\b', r'\btears\b',
            r'\bmiserable\b', r'\bdowncast\b', r'\bglum\b', r'\bmelancholy\b', r'\bsorrowful\b',
            r'\bhurt\b', r'\bpain\b', r'\bgrief\b', r'\bdisappointed\b', r'\bupset\b',
            r'\bbad\b', r'\bterrible\b', r'\bawful\b', r'\bhopeless\b', r'\blonely\b',
            r'😢', r'😭', r'💔', r'😔', r'😞', r'😿'
        ],
        'angry': [
            r'\bangry\b', r'\bmad\b', r'\bfurious\b', r'\brage\b', r'\bhate\b',
            r'\birritated\b', r'\bannoyed\b', r'\bfrustrated\b', r'\blivid\b', r'\benraged\b',
            r'\boutraged\b', r'\bpissed\b', r'\binfuriated\b', r'\baggravated\b', r'\bcross\b',
            r'\bstupid\b', r'\bidiot\b', r'\bdamn\b', r'\bcrap\b', r'\bsucks\b',
            r'😡', r'🤬', r'👿', r'💢', r'🔥'
        ],
        'confused': [
            r'\bconfused\b', r'\bpuzzled\b', r'\bbewildered\b', r'\bperplexed\b', r'\bmixed up\b',
            r'\bdont understand\b', r'\bdon\'t get\b', r'\bwhat\?\b', r'\bhuh\b', r'\bwait\b',
            r'\bunclear\b', r'\bunsure\b', r'\bbaffled\b', r'\bstumped\b', r'\blost\b',
            r'\bwhat do you mean\b', r'\bi don\'t follow\b', r'\bcan you explain\b',
            r'😕', r'😵', r'🤔', r'🤷', r'❓', r'❔'
        ],
        'fear': [
            r'\bscared\b', r'\bafraid\b', r'\bterrified\b', r'\banxious\b', r'\bworried\b',
            r'\bnervous\b', r'\bfrightened\b', r'\bpanicked\b', r'\bstressed\b', r'\btense\b',
            r'\buneasy\b', r'\bconcerned\b', r'\bapprehensive\b', r'\balarmed\b', r'\bshaken\b',
            r'😨', r'😰', r'😱', r'😬', r'💀'
        ],
        'calm': [
            r'\bcalm\b', r'\bpeaceful\b', r'\brelaxed\b', r'\btranquil\b', r'\bserene\b',
            r'\bzen\b', r'\bmellow\b', r'\bcomposed\b', r'\bchill\b', r'\bstable\b',
            r'\bbalanced\b', r'\bundisturbed\b', r'\bunruffled\b', r'\bsettled\b',
            r'😌', r'🧘', r'☮️', r'🕯️'
        ],
        'affection': [
            r'\blove\b', r'\badore\b', r'\bcare\b', r'\bsweet\b', r'\bdear\b',
            r'\baffection\b', r'\btender\b', r'\bwarm\b', r'\bkind\b', r'\bgentle\b',
            r'\bcompassionate\b', r'\bcaring\b', r'\bloving\b', r'\bfond\b',
            r'❤️', r'💕', r'💖', r'💗', r'💝', r'😘', r'🥰'
        ]
    }
    
    # Advanced sarcasm detection patterns
    sarcasm_patterns = [
        r'\boh really\b', r'\bobviously\b', r'\bsure thing\b', r'\byeah right\b',
        r'\bof course\b.*\bnot\b', r'\bgreat job\b.*\b(fail|wrong|bad)\b',
        r'\bwow\b.*\b(terrible|awful|bad)\b', r'\bso\b.*\b(smart|clever)\b.*\bnot\b',
        r'\bthanks\b.*\b(nothing|lot)\b', r'\breal\b.*\b(genius|brilliant)\b',
        r'\bjust\b.*\b(perfect|great)\b', r'\bexactly\b.*\b(wanted|needed)\b',
        r'\.{3,}', r'\bok then\b', r'\bsure\b\.+$', r'\bfine\b\.+$',
        r'\byep\b\.+$', r'\buh huh\b', r'\bwhatever\b', r'\bgood luck\b.*\bthat\b',
        r'\bnice\b.*\b(try|one)\b', r'\bimpressive\b\.+$'
    ]
    
    # Detect sentiments
    sentiment_scores = {}
    
    for sentiment, pattern_list in patterns.items():
        score = 0
        for pattern in pattern_list:
            matches = len(re.findall(pattern, text_lower))
            score += matches
        
        if score > 0:
            # Normalize by text length but boost sentiment detection
            sentiment_scores[sentiment] = min(score / max(len(text.split()), 5) * 8, 1.0)
    
    # Detect sarcasm
    sarcasm_score = 0
    for pattern in sarcasm_patterns:
        matches = len(re.findall(pattern, text_lower))
        sarcasm_score += matches
    
    # Context-based sarcasm detection
    if sarcasm_score > 0:
        sarcasm_confidence = min(sarcasm_score / max(len(text.split()), 3) * 6, 1.0)
    else:
        sarcasm_confidence = 0
    
    # Determine result
    if not sentiment_scores and sarcasm_confidence == 0:
        return "calm:0.5"  # Default neutral sentiment
    
    # If sarcasm is detected with another sentiment, combine them
    if sarcasm_confidence >= 0.3 and sentiment_scores:
        best_sentiment = max(sentiment_scores.items(), key=lambda x: x[1])
        combined_confidence = min((sarcasm_confidence + best_sentiment[1]) / 2, 1.0)
        return f"sarcastic+{best_sentiment[0]}:{combined_confidence:.2f}"
    
    # If only sarcasm detected
    if sarcasm_confidence >= 0.3:
        return f"sarcastic:{sarcasm_confidence:.2f}"
    
    # Return highest scoring regular sentiment
    if sentiment_scores:
        best_sentiment = max(sentiment_scores.items(), key=lambda x: x[1])
        return f"{best_sentiment[0]}:{best_sentiment[1]:.2f}"
    
    return "calm:0.5"

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 sentiment_analysis.py <text>", file=sys.stderr)
        sys.exit(1)
    
    text = sys.argv[1]
    result = analyze_sentiment(text)
    print(result)

if __name__ == "__main__":
    main()