#!/usr/bin/env python3
"""
Content Moderation Module for Social Pulse
Handles Detoxify-based toxicity detection with enhanced combo system
"""
import time
from model_cache import save_detoxify_cache_sentinel, check_detoxify_cache_sentinel

class ContentModerator:
    """Handles content moderation using Detoxify AI-based toxicity detection with combo system"""
    
    def __init__(self):
        self.detoxify_classifier = None
        self.detoxify_available = False
        
        # Initialize Detoxify for content moderation
        print("🛡️ Initializing Detoxify classifier for content moderation...")
        self.initialize_detoxify_with_retry()
    
    def initialize_detoxify_with_retry(self, max_retries=3):
        """Initialize Detoxify classifier with sentinel caching and retry logic using 'unbiased' model"""
        
        # Check sentinel cache first (but still need to load model fresh)
        cache_valid = check_detoxify_cache_sentinel()
        if cache_valid:
            print("📋 Sentinel cache indicates previous successful load, proceeding with fresh initialization...")
        
        # Always load Detoxify fresh (no torch model pickling)
        for attempt in range(max_retries):
            try:
                print(f"🔄 Attempt {attempt + 1}/{max_retries}: Loading Detoxify classifier with 'unbiased' model...")
                from detoxify import Detoxify
                self.detoxify_classifier = Detoxify('unbiased')
                
                # Test the classifier to ensure it's working
                test_result = self.detoxify_classifier.predict("This is a test message")
                if test_result and 'identity_attack' in test_result:
                    self.detoxify_available = True
                    print("✅ Detoxify classifier loaded successfully!")
                    
                    # Save sentinel cache for future startups (no model pickle)
                    save_detoxify_cache_sentinel()
                    return True
                    
            except Exception as e:
                print(f"⚠️ Detoxify attempt {attempt + 1} failed: {e}")
                if attempt < max_retries - 1:
                    backoff_time = (2 ** attempt)  # Exponential backoff: 1s, 2s, 4s
                    print(f"⏳ Waiting {backoff_time}s before retry...")
                    time.sleep(backoff_time)
                
        print("❌ Detoxify classifier failed to initialize after all retries")
        return False
    
    def moderate_content(self, text):
        """
        Content moderation using Detoxify AI-based toxicity detection with combo system.
        
        Blocking behavior:
        - identity_attack ≥ 0.8: Blocks content as hate speech (existing behavior)
        
        Toxicity tagging system:
        - toxicity, severe_toxicity, obscene, threat, insult ≥ 0.5: Added as toxicity tags
        
        Response includes:
        - toxicity_tags: Array of categories that passed ≥ 0.5 threshold
        - all_scores: All toxicity scores for diagnostic purposes
        - Comprehensive diagnostic logging
        """
        print(f"🛡️ MODERATION: Incoming content moderation request")
        print(f"   📄 Text: \"{text[:100]}{'...' if len(text) > 100 else ''}\"")
        print(f"   🔍 Processing text (length: {len(text)} chars)")
        
        # Use Detoxify as primary moderation tool
        if self.detoxify_available and self.detoxify_classifier is not None:
            try:
                print(f"   🧠 Calling Detoxify classifier...")
                result = self.detoxify_classifier.predict(text)
                
                if result and 'identity_attack' in result:
                    # Convert all numpy float32 to Python float for JSON serialization
                    all_scores = {}
                    for key in result:
                        all_scores[key] = float(result[key])
                    
                    identity_attack_score = all_scores['identity_attack']
                    
                    # Comprehensive diagnostic logging for all scores
                    print(f"   🎯 Detoxify results (NEW TOXICITY COMBO SYSTEM):")
                    print(f"      🏹 Identity attack: {identity_attack_score:.3f} (BLOCKING THRESHOLD: ≥ 0.8)")
                    
                    # Categories for toxicity tagging (threshold ≥ 0.5)
                    toxicity_categories = ['toxicity', 'severe_toxicity', 'obscene', 'threat', 'insult']
                    toxicity_tags = []
                    toxicity_thresholds = {'toxicity': 0.76, 'severe_toxicity': 0.6, 'obscene': 0.95, 'threat': 0.77, 'insult': 0.78}
                    for category in toxicity_categories:
                        if category in all_scores:
                            score = all_scores[category]
                            is_toxic = score >= toxicity_thresholds[category]
                            status = "TAGGED" if is_toxic else "below threshold"
                            emoji_map = {
                                'toxicity': '😵',
                                'severe_toxicity': '💥', 
                                'obscene': '😡',
                                'threat': '⚡',
                                'insult': '😠'
                            }
                            emoji = emoji_map.get(category, '🔍')
                            print(f"      {emoji} {category.replace('_', ' ').title()}: {score:.3f} ({status})")
                            
                            if is_toxic:
                                toxicity_tags.append(category)
                    
                    # Log toxicity tagging results
                    if toxicity_tags:
                        print(f"   🏷️ TOXICITY TAGS: {len(toxicity_tags)} categories tagged")
                        for tag in toxicity_tags:
                            print(f"      📌 {tag}: {all_scores[tag]:.3f}")
                    else:
                        print(f"   🏷️ TOXICITY TAGS: No categories met threshold")
                    
                    # Check for identity_attack blocking (existing behavior)
                    if identity_attack_score >= 0.8:
                        print(f"   🚨 CONTENT BLOCKED!")
                        print(f"      🛑 Reason: identity_attack ≥ 0.8 threshold")
                        print(f"      ⚖️ Score: {identity_attack_score:.1%}")
                        print(f"      🏷️ Additional toxicity tags: {toxicity_tags}")
                        print(f"      📋 AI-based detection by Detoxify (unbiased model)")
                        print(f"   📤 MODERATION: Content BLOCKED")
                        
                        return {
                            'is_blocked': True,
                            'violation_type': 'identity_attack',
                            'confidence': identity_attack_score,
                            'toxicity_tags': toxicity_tags,
                            'all_scores': all_scores,
                            'details': f'Detoxify detected identity attack with {identity_attack_score:.1%} confidence',
                            'moderation_system': 'toxicity_combo_v1'
                        }
                    else:
                        # Content not blocked but may have toxicity tags
                        print(f"   🟢 CONTENT APPROVED")
                        print(f"      ✅ identity_attack: {identity_attack_score:.3f} (below 0.8 blocking threshold)")
                        if toxicity_tags:
                            print(f"      🏷️ Toxicity tags applied: {toxicity_tags}")
                            print(f"      💡 Content flagged for toxicity but not blocked")
                        else:
                            print(f"      🌟 Clean content: No toxicity detected")
                        print(f"   📤 MODERATION: Content APPROVED with tags")
                        
                        return {
                            'is_blocked': False,
                            'violation_type': None,
                            'confidence': identity_attack_score,
                            'toxicity_tags': toxicity_tags,
                            'all_scores': all_scores,
                            'details': f'Detoxify: identity_attack={identity_attack_score:.1%}, toxicity_tags={len(toxicity_tags)}',
                            'moderation_system': 'toxicity_combo_v1'
                        }
                else:
                    print(f"   ⚠️ Detoxify returned unexpected result format")
                    # Fall through to fallback
                    
            except Exception as e:
                print(f"   ⚠️ Detoxify classifier failed: {e}")
                # Fall through to fallback
        
        # Fallback when Detoxify is not available
        print(f"   ⚠️ Detoxify not available, using minimal fallback")
        print(f"   🟢 MODERATION: Content approved (no AI moderation)")
        print(f"   📤 MODERATION: Content APPROVED")
        
        return {
            'is_blocked': False,
            'violation_type': None,
            'confidence': 0.0,
            'toxicity_tags': [],
            'all_scores': {},
            'details': 'Detoxify unavailable - no moderation applied',
            'moderation_system': 'fallback'
        }
    
    def get_status(self):
        """Get status information about available content moderation libraries"""
        moderation_libraries = []
        if self.detoxify_available:
            moderation_libraries.append("detoxify")
        
        moderation_detector = "detoxify" if self.detoxify_available else "none"
        
        return {
            "moderation_libraries": moderation_libraries,
            "moderation_detector": moderation_detector,
            "detoxify_available": self.detoxify_available,
            "moderation_model": "unbiased",
            "moderation_threshold": 0.8,
            "moderation_focus": "identity_attack_only"
        }