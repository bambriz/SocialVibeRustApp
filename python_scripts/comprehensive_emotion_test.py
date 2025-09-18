#!/usr/bin/env python3
"""
Comprehensive Emotion Testing Suite

This script systematically tests all emotions with dual emoji combinations:
- All 11+ base emotions 
- All sarcastic combinations
- All affectionate combinations
- Backend-frontend consistency verification
- Visual verification support
"""

import requests
import json
import time
import sys
from datetime import datetime

# Test configuration
BASE_URL = "http://localhost:5000"
API_BASE = f"{BASE_URL}/api"

class EmotionTester:
    def __init__(self):
        self.auth_token = None
        self.test_results = {}
        self.failed_tests = []
        self.passed_tests = []
        
    def login_test_user(self):
        """Login with test user account"""
        print("🔑 Logging in test user...")
        
        login_data = {
            "email": "frontend@test.com", 
            "password": "test123"
        }
        
        try:
            response = requests.post(f"{API_BASE}/auth/login", json=login_data)
            if response.status_code == 200:
                data = response.json()
                self.auth_token = data['token']
                print(f"✅ Logged in successfully as {data['user']['username']}")
                return True
            else:
                print(f"❌ Login failed: {response.status_code} - {response.text}")
                return False
        except Exception as e:
            print(f"❌ Login error: {e}")
            return False
    
    def create_test_post(self, title, content, expected_emotion):
        """Create a test post and verify the sentiment analysis"""
        print(f"\n🧪 Testing emotion: {expected_emotion}")
        print(f"   📝 Text: '{content[:50]}...'")
        
        post_data = {
            "title": title,
            "content": content
        }
        
        headers = {
            "Authorization": f"Bearer {self.auth_token}",
            "Content-Type": "application/json"
        }
        
        try:
            response = requests.post(f"{API_BASE}/posts", json=post_data, headers=headers)
            if response.status_code == 200:
                post_data = response.json()
                print(f"   ✅ Post created successfully")
                
                # Verify sentiment data
                self.verify_sentiment(post_data, expected_emotion)
                return post_data
            else:
                print(f"   ❌ Post creation failed: {response.status_code} - {response.text}")
                self.failed_tests.append(f"{expected_emotion}: Post creation failed")
                return None
        except Exception as e:
            print(f"   ❌ Post creation error: {e}")
            self.failed_tests.append(f"{expected_emotion}: Post creation error - {e}")
            return None
    
    def verify_sentiment(self, post_data, expected_emotion):
        """Verify the sentiment analysis results"""
        # Extract data from the 'post' object in the response
        post_obj = post_data.get('post', {})
        sentiment_type = post_obj.get('sentiment_type', '')
        sentiment_colors = post_obj.get('sentiment_colors', [])
        # Note: confidence is not returned in the API response, using popularity_score as indicator
        confidence = post_obj.get('popularity_score', 1.0)
        
        print(f"   🎯 Detected: {sentiment_type} (confidence: {confidence})")
        print(f"   🎨 Colors: {sentiment_colors}")
        
        # Check if expected emotion matches detected
        is_correct = False
        test_result = {
            'expected': expected_emotion,
            'detected': sentiment_type,
            'colors': sentiment_colors,
            'confidence': confidence,
            'status': 'FAIL'
        }
        
        if '+' in expected_emotion:
            # Combo emotion test
            if sentiment_type == expected_emotion:
                is_correct = True
                # Verify dual colors for combos
                if len(sentiment_colors) >= 2:
                    print(f"   ✅ Combo emotion correctly detected with dual colors")
                else:
                    print(f"   ⚠️  Combo detected but missing dual colors")
                    test_result['warning'] = 'Missing dual colors for combo'
            else:
                print(f"   ❌ Expected '{expected_emotion}' but got '{sentiment_type}'")
        else:
            # Base emotion test
            if sentiment_type == expected_emotion or sentiment_type.endswith(f'+{expected_emotion}'):
                is_correct = True
            else:
                print(f"   ❌ Expected '{expected_emotion}' but got '{sentiment_type}'")
        
        if is_correct:
            test_result['status'] = 'PASS'
            self.passed_tests.append(expected_emotion)
            print(f"   ✅ Sentiment verification PASSED")
        else:
            self.failed_tests.append(expected_emotion)
            print(f"   ❌ Sentiment verification FAILED")
        
        self.test_results[expected_emotion] = test_result
    
    def test_base_emotions(self):
        """Test all base emotions"""
        print("\n" + "="*60)
        print("🎭 TESTING BASE EMOTIONS")
        print("="*60)
        
        base_emotions = {
            'angry': "I am so angry and furious about this situation! This makes me mad and rage-filled!",
            'sad': "I feel so sad and depressed today. This makes me cry and feel terrible.",
            'joy': "I feel pure joy and happiness! This brings me such delight and bliss!",
            'happy': "I am so happy and cheerful! This makes me smile and feel great!",
            'excited': "I am so excited and pumped up! Can't wait, this is thrilling and exhilarating!",
            'confused': "I am so confused and bewildered by this. This makes no sense to me at all.",
            'calm': "I feel calm and peaceful. This brings me tranquility and serenity.",
            'fear': "I am scared and afraid of this. This terrifies me and makes me anxious.",
            'disgust': "This is disgusting and revolting. This makes me feel sick and nauseated.",
            'surprise': "Wow, this is such a surprise! I am shocked and amazed by this!",
            'affection': "I love you so much, my darling. You are precious and dear to me.",
            'sarcastic': "Oh great, just perfect. Obviously this is working flawlessly."
        }
        
        for emotion, text in base_emotions.items():
            self.create_test_post(f"Test {emotion.title()}", text, emotion)
            time.sleep(0.5)  # Small delay between tests
    
    def test_sarcastic_combinations(self):
        """Test sarcastic combination emotions"""
        print("\n" + "="*60)
        print("😏 TESTING SARCASTIC COMBINATIONS")
        print("="*60)
        
        sarcastic_combos = {
            'sarcastic+joy': "Oh great, I'm just so happy and joyful about this obviously wonderful situation.",
            'sarcastic+angry': "Yeah, I'm totally not angry about this. Just perfect, makes me so mad.",
            'sarcastic+sad': "Oh sure, this doesn't make me sad at all. Obviously brings me such joy.",
            'sarcastic+excited': "Can't wait for this exciting development. So pumped about it, obviously.",
            'sarcastic+confused': "This makes perfect sense to me. Obviously crystal clear what's happening.",
            'sarcastic+fear': "Oh I'm totally not scared of this at all. Obviously nothing to fear here.",
            'sarcastic+surprise': "What a shocking surprise this is. Never saw this coming, obviously.",
            'sarcastic+calm': "I'm just so calm about this obviously peaceful situation. How zen.",
            'sarcastic+affection': "Oh I just love this so much, darling. Obviously brings me such affection."
        }
        
        for combo, text in sarcastic_combos.items():
            self.create_test_post(f"Test {combo}", text, combo)
            time.sleep(0.5)
    
    def test_affectionate_combinations(self):
        """Test affectionate combination emotions"""
        print("\n" + "="*60)
        print("💕 TESTING AFFECTIONATE COMBINATIONS") 
        print("="*60)
        
        affectionate_combos = {
            'affectionate+joy': "My darling sweetheart, I love and adore you so much! This makes me incredibly happy and joyful!",
            'affectionate+sad': "My beloved treasure, I love you deeply even though this situation makes me so sad and heartbroken.",
            'affectionate+angry': "My dear love, I adore you but this situation makes me angry and frustrated, honey.",
            'affectionate+excited': "My sweet darling, I love you so much and I'm excited and thrilled about this!",
            'affectionate+confused': "My precious love, I adore you but I'm so confused and bewildered by this situation.",
            'affectionate+fear': "My beloved dear, I love you deeply but this terrifies and scares me, sweetheart.",
            'affectionate+surprise': "My darling treasure, I love you and I'm so surprised and shocked by this!",
            'affectionate+calm': "My sweet love, I adore you and feel so calm and peaceful with you.",
            'affectionate+affection': "My beloved darling, I love and cherish you with all my heart. You are my treasure."
        }
        
        for combo, text in affectionate_combos.items():
            self.create_test_post(f"Test {combo}", text, combo)
            time.sleep(0.5)
    
    def verify_color_consistency(self):
        """Verify color mappings are consistent between backend and frontend"""
        print("\n" + "="*60)
        print("🎨 VERIFYING COLOR CONSISTENCY")
        print("="*60)
        
        expected_colors = {
            'angry': '#dc2626',      # Red
            'sad': '#1e3a8a',        # Dark blue
            'joy': '#22d3ee',        # Bright cyan
            'happy': '#fbbf24',      # Bright yellow/gold
            'excited': '#f59e0b',    # Bright orange
            'confused': '#8b5cf6',   # Light purple
            'calm': '#059669',       # Green
            'fear': '#374151',       # Dark grey
            'disgust': '#84cc16',    # Lime green
            'surprise': '#f97316',   # Orange
            'affection': '#ec4899',  # Pink
            'sarcastic': '#7c3aed'   # Purple
        }
        
        color_consistency_passed = 0
        color_consistency_total = 0
        
        for emotion, expected_color in expected_colors.items():
            if emotion in self.test_results:
                result = self.test_results[emotion]
                colors = result['colors']
                
                color_consistency_total += 1
                if colors and expected_color in colors:
                    color_consistency_passed += 1
                    print(f"   ✅ {emotion}: {expected_color} ✓")
                else:
                    print(f"   ❌ {emotion}: Expected {expected_color}, got {colors}")
        
        print(f"\n🎨 Color consistency: {color_consistency_passed}/{color_consistency_total} passed")
        return color_consistency_passed == color_consistency_total
    
    def generate_test_report(self):
        """Generate comprehensive test report"""
        print("\n" + "="*80)
        print("📊 COMPREHENSIVE EMOTION TEST REPORT")
        print("="*80)
        
        total_tests = len(self.test_results)
        passed_count = len(self.passed_tests)
        failed_count = len(self.failed_tests)
        
        print(f"\n📈 SUMMARY:")
        print(f"   Total tests: {total_tests}")
        print(f"   Passed: {passed_count}")
        print(f"   Failed: {failed_count}")
        print(f"   Success rate: {(passed_count/total_tests*100):.1f}%")
        
        if self.passed_tests:
            print(f"\n✅ PASSED EMOTIONS ({len(self.passed_tests)}):")
            for emotion in self.passed_tests:
                result = self.test_results[emotion]
                print(f"   • {emotion}: {result['detected']} (conf: {result['confidence']:.2f})")
        
        if self.failed_tests:
            print(f"\n❌ FAILED EMOTIONS ({len(self.failed_tests)}):")
            for emotion in self.failed_tests:
                if emotion in self.test_results:
                    result = self.test_results[emotion]
                    print(f"   • {emotion}: Expected '{result['expected']}', got '{result['detected']}'")
                else:
                    print(f"   • {emotion}: Test execution failed")
        
        # Detailed results
        print(f"\n📋 DETAILED RESULTS:")
        for emotion, result in self.test_results.items():
            status_icon = "✅" if result['status'] == 'PASS' else "❌"
            print(f"   {status_icon} {emotion:20} | {result['detected']:25} | {len(result['colors'])} colors | {result['confidence']:.2f}")
        
        # Dual emoji combo verification
        combo_tests = [k for k in self.test_results.keys() if '+' in k]
        dual_emoji_success = 0
        
        print(f"\n🎭 DUAL EMOJI COMBO VERIFICATION:")
        for combo in combo_tests:
            result = self.test_results[combo]
            if len(result['colors']) >= 2:
                dual_emoji_success += 1
                print(f"   ✅ {combo}: {len(result['colors'])} colors (dual emoji capable)")
            else:
                print(f"   ❌ {combo}: {len(result['colors'])} colors (missing dual emoji)")
        
        print(f"\n🎨 Dual emoji combos working: {dual_emoji_success}/{len(combo_tests)}")
        
        # Overall success criteria
        print(f"\n🎯 SUCCESS CRITERIA EVALUATION:")
        criteria_passed = 0
        criteria_total = 4
        
        # 1. All base emotions working
        base_emotions = [k for k in self.test_results.keys() if '+' not in k]
        base_passed = sum(1 for e in base_emotions if e in self.passed_tests)
        if base_passed == len(base_emotions):
            print(f"   ✅ All base emotions working ({base_passed}/{len(base_emotions)})")
            criteria_passed += 1
        else:
            print(f"   ❌ Base emotions incomplete ({base_passed}/{len(base_emotions)})")
        
        # 2. All combo emotions showing dual emojis
        if dual_emoji_success == len(combo_tests):
            print(f"   ✅ All combo emotions have dual emojis ({dual_emoji_success}/{len(combo_tests)})")
            criteria_passed += 1
        else:
            print(f"   ❌ Combo emotions missing dual emojis ({dual_emoji_success}/{len(combo_tests)})")
        
        # 3. High success rate
        success_rate = passed_count / total_tests * 100
        if success_rate >= 90:
            print(f"   ✅ High success rate ({success_rate:.1f}% >= 90%)")
            criteria_passed += 1
        else:
            print(f"   ❌ Low success rate ({success_rate:.1f}% < 90%)")
        
        # 4. Color consistency
        color_consistent = self.verify_color_consistency()
        if color_consistent:
            print(f"   ✅ Color mapping consistency verified")
            criteria_passed += 1
        else:
            print(f"   ❌ Color mapping inconsistencies found")
        
        print(f"\n🎯 OVERALL RESULT: {criteria_passed}/{criteria_total} criteria passed")
        
        if criteria_passed == criteria_total:
            print("🎉 ALL TESTS PASSED! Emotion system is working perfectly!")
        elif criteria_passed >= 3:
            print("⚠️  Most tests passed, minor issues need attention")
        else:
            print("❌ Major issues found, system needs debugging")
        
        return {
            'total_tests': total_tests,
            'passed': passed_count,
            'failed': failed_count,
            'success_rate': success_rate,
            'criteria_passed': criteria_passed,
            'criteria_total': criteria_total,
            'dual_emoji_success': dual_emoji_success,
            'dual_emoji_total': len(combo_tests)
        }

def main():
    """Run comprehensive emotion testing"""
    print("🚀 Starting Comprehensive Emotion Testing Suite")
    print("=" * 80)
    
    tester = EmotionTester()
    
    # Step 1: Login
    if not tester.login_test_user():
        print("❌ Cannot proceed without authentication")
        sys.exit(1)
    
    # Step 2: Test all base emotions
    tester.test_base_emotions()
    
    # Step 3: Test sarcastic combinations
    tester.test_sarcastic_combinations()
    
    # Step 4: Test affectionate combinations  
    tester.test_affectionate_combinations()
    
    # Step 5: Generate comprehensive report
    final_report = tester.generate_test_report()
    
    # Step 6: Save results to file
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    report_file = f"emotion_test_report_{timestamp}.json"
    
    with open(report_file, 'w') as f:
        json.dump({
            'timestamp': timestamp,
            'summary': final_report,
            'detailed_results': tester.test_results,
            'passed_tests': tester.passed_tests,
            'failed_tests': tester.failed_tests
        }, f, indent=2)
    
    print(f"\n📄 Full report saved to: {report_file}")
    
    return final_report['criteria_passed'] == final_report['criteria_total']

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)