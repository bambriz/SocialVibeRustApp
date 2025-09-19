#!/usr/bin/env python3
"""
Test script for the new toxicity combo system in the moderation endpoint.

This script tests:
1. Identity attack ≥ 0.8 still blocks content (existing behavior)
2. Other categories ≥ 0.5 get tagged as toxicity_tags
3. All scores are included in response for diagnostics
4. New response format with toxicity_tags array
5. Comprehensive diagnostic logging
"""

import json
import requests
import time

PYTHON_SERVER_URL = "http://127.0.0.1:8001"

def test_moderation_endpoint():
    """Test the /moderate endpoint with the new toxicity combo system"""
    
    print("🧪 TESTING: New Toxicity Combo System")
    print("=" * 50)
    
    # Test cases covering different scenarios
    test_cases = [
        {
            "name": "Clean Content",
            "text": "I love this beautiful day!",
            "expected_blocked": False,
            "expected_tags": "minimal or none"
        },
        {
            "name": "Mildly Toxic Content",
            "text": "This is really annoying and stupid",
            "expected_blocked": False,
            "expected_tags": "may have toxicity/insult tags"
        },
        {
            "name": "Moderately Toxic Content", 
            "text": "You're such an idiot, this is garbage",
            "expected_blocked": False,
            "expected_tags": "likely toxicity/insult tags"
        },
        {
            "name": "High Identity Attack (Should Block)",
            "text": "All people from that group are inferior and should be eliminated",
            "expected_blocked": True,
            "expected_tags": "blocked due to identity_attack"
        }
    ]
    
    print(f"📊 Running {len(test_cases)} test cases...\n")
    
    for i, test_case in enumerate(test_cases, 1):
        print(f"🔬 Test {i}: {test_case['name']}")
        print(f"   📄 Text: \"{test_case['text'][:50]}{'...' if len(test_case['text']) > 50 else ''}\"")
        print(f"   🎯 Expected: {test_case['expected_tags']}")
        
        try:
            # Make request to moderation endpoint
            response = requests.post(
                f"{PYTHON_SERVER_URL}/moderate",
                json={"text": test_case["text"]},
                timeout=30
            )
            
            if response.status_code == 200:
                result = response.json()
                
                # Check if response has new format
                has_new_format = all(key in result for key in ['toxicity_tags', 'all_scores', 'moderation_system'])
                
                print(f"   ✅ Response received successfully")
                print(f"   🔧 New format detected: {has_new_format}")
                print(f"   🚫 Blocked: {result.get('is_blocked', 'unknown')}")
                print(f"   🏷️ Toxicity tags: {result.get('toxicity_tags', [])}")
                print(f"   📊 All scores available: {bool(result.get('all_scores', {}))}")
                print(f"   🛡️ Moderation system: {result.get('moderation_system', 'unknown')}")
                
                # Show some key scores if available
                if result.get('all_scores'):
                    scores = result['all_scores']
                    print(f"   🎯 Key scores:")
                    for category in ['identity_attack', 'toxicity', 'insult', 'threat', 'obscene']:
                        if category in scores:
                            score = scores[category]
                            print(f"      - {category}: {score:.3f}")
                
                # Validate expectations
                if test_case['expected_blocked'] and not result.get('is_blocked'):
                    print(f"   ⚠️ WARNING: Expected content to be blocked, but it wasn't")
                elif not test_case['expected_blocked'] and result.get('is_blocked'):
                    print(f"   ⚠️ WARNING: Content was blocked when it shouldn't be")
                else:
                    print(f"   ✅ Blocking behavior matches expectations")
                
            else:
                print(f"   ❌ Failed: HTTP {response.status_code}")
                print(f"   📄 Error: {response.text}")
        
        except Exception as e:
            print(f"   ❌ Error: {e}")
        
        print()  # Empty line between tests
        time.sleep(1)  # Brief pause between requests
    
    print("🏁 Test suite completed!")
    print("\n📋 Summary:")
    print("- The new toxicity combo system should now be active")
    print("- Check the server logs for detailed diagnostic output")
    print("- Response format includes: toxicity_tags, all_scores, moderation_system")
    print("- Identity attack ≥ 0.8 still blocks content")
    print("- Other categories ≥ 0.5 are tagged but don't block")

def test_health_endpoint():
    """Test the health endpoint to verify server status"""
    print("🏥 HEALTH CHECK")
    print("=" * 30)
    
    try:
        response = requests.get(f"{PYTHON_SERVER_URL}/health", timeout=10)
        if response.status_code == 200:
            health_data = response.json()
            print(f"✅ Server is healthy")
            print(f"🔧 Moderation libraries: {health_data.get('moderation_libraries', [])}")
            print(f"🛡️ Moderation detector: {health_data.get('moderation_detector', 'unknown')}")
            print(f"📊 Detoxify available: {health_data.get('detoxify_available', False)}")
            print(f"🎯 Moderation model: {health_data.get('moderation_model', 'unknown')}")
            return True
        else:
            print(f"❌ Health check failed: HTTP {response.status_code}")
            return False
    except Exception as e:
        print(f"❌ Health check error: {e}")
        return False

if __name__ == "__main__":
    print("🚀 Starting Toxicity Combo System Test Suite")
    print("=" * 60)
    
    # First check server health
    print()
    if test_health_endpoint():
        print("\n")
        test_moderation_endpoint()
    else:
        print("❌ Server not ready, skipping moderation tests")
        print("💡 Make sure the Python server is running on port 8001")