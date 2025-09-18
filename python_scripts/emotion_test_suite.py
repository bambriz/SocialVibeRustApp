#!/usr/bin/env python3
"""
Comprehensive Emotion Testing Suite

Tests all 33 emotion combinations (11 basic + 11 sarcastic + 11 affectionate) 
with 2 posts each for a total of 66 test posts.

Validates:
- sentiment_type matches expected emotion
- sentiment_colors match expected color mappings
- popularity_scores are in expected ranges
- Provides clear pass/fail reporting

Usage:
    python3 python_scripts/emotion_test_suite.py
"""
import asyncio
import aiohttp
import json
import sys
import time
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from datetime import datetime

# Base server URL
BASE_URL = "http://127.0.0.1:5000"
API_BASE = f"{BASE_URL}/api"

@dataclass
class TestCase:
    emotion_type: str
    title: str
    content: str
    expected_sentiment_type: str
    expected_colors: List[str]

@dataclass
class TestResult:
    test_case: TestCase
    post_id: Optional[str]
    actual_sentiment_type: Optional[str]
    actual_colors: List[str]
    popularity_score: Optional[float]
    passed: bool
    errors: List[str]

class EmotionTestSuite:
    def __init__(self):
        self.session = None
        self.auth_token = None
        self.test_user_id = None
        self.created_posts = []
        self.results = []
        
        # Expected color mappings based on actual code implementation
        self.color_mappings = {
            # Basic emotions - single colors
            "sad": ["#1e3a8a"],
            "angry": ["#dc2626"],
            "sarcastic": ["#7c3aed"],
            "happy": ["#fbbf24"],
            "joy": ["#22d3ee"],
            "excited": ["#f59e0b"],
            "confused": ["#8b5cf6"],
            "affection": ["#ec4899"],
            "calm": ["#059669"],
            "fear": ["#374151"],
            "disgust": ["#84cc16"],
            "surprise": ["#f97316"],
            
            # Sarcastic combinations - purple + base emotion color
            "sarcastic+sad": ["#7c3aed", "#1e3a8a"],
            "sarcastic+angry": ["#7c3aed", "#dc2626"],
            "sarcastic+happy": ["#7c3aed", "#fbbf24"],
            "sarcastic+joy": ["#7c3aed", "#22d3ee"],
            "sarcastic+excited": ["#7c3aed", "#f59e0b"],
            "sarcastic+confused": ["#7c3aed", "#8b5cf6"],
            "sarcastic+affection": ["#7c3aed", "#ec4899"],
            "sarcastic+calm": ["#7c3aed", "#059669"],
            "sarcastic+fear": ["#7c3aed", "#374151"],
            "sarcastic+disgust": ["#7c3aed", "#84cc16"],
            "sarcastic+surprise": ["#7c3aed", "#f97316"],
            
            # Affectionate combinations - pink + base emotion color  
            "affectionate+sad": ["#ec4899", "#1e3a8a"],
            "affectionate+angry": ["#ec4899", "#dc2626"],
            "affectionate+happy": ["#ec4899", "#fbbf24"],
            "affectionate+joy": ["#ec4899", "#22d3ee"],
            "affectionate+excited": ["#ec4899", "#f59e0b"],
            "affectionate+confused": ["#ec4899", "#8b5cf6"],
            "affectionate+affection": ["#ec4899", "#ec4899"],
            "affectionate+calm": ["#ec4899", "#059669"],
            "affectionate+fear": ["#ec4899", "#374151"],
            "affectionate+disgust": ["#ec4899", "#84cc16"],
            "affectionate+surprise": ["#ec4899", "#f97316"],
        }
        
        self.test_cases = self._generate_test_cases()
    
    def _generate_test_cases(self) -> List[TestCase]:
        """Generate all 66 test cases (33 emotion types × 2 posts each)"""
        test_cases = []
        
        # Basic emotions (11 × 2 = 22 posts)
        basic_emotions = [
            ("sad", [
                ("Another rainy Monday", "I can't shake this feeling of emptiness. Everything feels pointless and gray today."),
                ("Lost and defeated", "I failed the interview again. Nothing ever goes right for me. I feel so hopeless.")
            ]),
            ("angry", [
                ("Traffic nightmare again!", "These idiots can't drive! I'm stuck in this mess for 2 hours because of incompetent drivers!"),
                ("Customer service disaster", "They hung up on me THREE times! This company treats customers like garbage. I'm furious!")
            ]),
            ("sarcastic", [
                ("Oh great, another Monday", "Just perfect timing for the elevator to break. Living the dream here."),
                ("Working flawlessly as usual", "The system crashed right before the presentation. Obviously this would happen today.")
            ]),
            ("happy", [
                ("Beautiful sunny day!", "The weather is gorgeous and I'm feeling wonderful! Life is good today."),
                ("Great news everyone!", "I got the promotion! I'm so pleased and content with how things turned out.")
            ]),
            ("joy", [
                ("I'm over the moon!", "I can't contain my excitement! Everything is absolutely amazing right now!"),
                ("Pure bliss today!", "This is the best day ever! I'm bursting with happiness and energy!")
            ]),
            ("excited", [
                ("Can't wait for tomorrow!", "The concert is tomorrow and I'm bouncing off the walls with anticipation!"),
                ("Adventure time!", "We're going skydiving next week! I'm so pumped and ready for this adrenaline rush!")
            ]),
            ("confused", [
                ("What just happened?", "I have no idea what's going on here. This makes absolutely no sense to me."),
                ("Lost in translation", "The instructions are completely unclear. I'm totally bewildered by this whole situation.")
            ]),
            ("affection", [
                ("Love you all", "I cherish every moment with my family. You mean the world to me, darling."),
                ("Warm feelings", "My heart is full of love and tenderness for everyone I care about today.")
            ]),
            ("calm", [
                ("Peaceful evening", "Everything is serene and tranquil. I feel completely at peace with the world."),
                ("Zen moment", "Taking deep breaths and enjoying this quiet, relaxed atmosphere. All is well.")
            ]),
            ("fear", [
                ("Something's wrong", "I hear strange noises downstairs and I'm terrified to check what it is."),
                ("Nightmare scenario", "The plane is shaking violently and I'm absolutely terrified we're going to crash.")
            ]),
            ("disgust", [
                ("That's revolting", "The smell from the garbage is making me sick. This is absolutely nauseating."),
                ("Gross situation", "I found moldy food in the fridge. This is disgusting and repulsive.")
            ]),
            ("surprise", [
                ("Didn't see that coming!", "I can't believe they threw me a surprise party! I'm completely shocked!"),
                ("Plot twist!", "The movie ending was incredible! I never expected that amazing twist!")
            ])
        ]
        
        # Sarcastic combinations (11 × 2 = 22 posts)
        sarcastic_emotions = [
            ("sarcastic+sad", [
                ("Oh wonderful, more sadness", "Great, another crying session. Just what I needed to make this day perfect."),
                ("Obviously depressing", "Sure, let's add more misery to this already fantastic situation. Living the dream.")
            ]),
            ("sarcastic+angry", [
                ("Oh fantastic, I'm livid", "Great, now I'm furious too. Obviously this anger is exactly what this situation needed."),
                ("Perfect rage timing", "Yeah right, getting mad will definitely solve everything. How absolutely brilliant.")
            ]),
            ("sarcastic+happy", [
                ("Yeah sure, so thrilled", "Oh obviously I'm just ecstatic about this wonderful development. Living the dream."),
                ("Absolutely delighted", "Yeah right, I'm just overjoyed by this fantastic news. Couldn't be happier.")
            ]),
            ("sarcastic+joy", [
                ("Pure bliss, obviously", "Oh great, such overwhelming joy. Just what I needed - more excitement in my life."),
                ("Totally ecstatic", "Yeah sure, I'm bursting with happiness about this amazing situation. Absolutely perfect.")
            ]),
            ("sarcastic+excited", [
                ("So pumped for this", "Oh obviously I'm just thrilled about waiting in line for 3 hours. Can barely contain my excitement."),
                ("Absolutely buzzing", "Yeah right, getting up at 5am is exactly what I hoped for. Living the dream here.")
            ]),
            ("sarcastic+confused", [
                ("Obviously crystal clear", "Yeah sure, this makes perfect sense. Totally not bewildered by this brilliant explanation."),
                ("Completely understand", "Oh great, more confusing instructions. Obviously I know exactly what to do now.")
            ]),
            ("sarcastic+affection", [
                ("Love this so much", "Oh obviously I just adore dealing with this mess. Such tender feelings for this situation."),
                ("Absolutely cherish this", "Yeah right, I'm filled with warm loving feelings about this disaster. How sweet.")
            ]),
            ("sarcastic+calm", [
                ("So peaceful and serene", "Oh great, such tranquility while everything falls apart. Obviously feeling zen about chaos."),
                ("Totally relaxed", "Yeah sure, completely calm while the world burns. Such inner peace and serenity.")
            ]),
            ("sarcastic+fear", [
                ("Absolutely terrified", "Oh obviously I'm just trembling with fear about this minor inconvenience. So scary."),
                ("Living in terror", "Yeah right, I'm completely petrified by this devastating problem. How frightening.")
            ]),
            ("sarcastic+disgust", [
                ("So absolutely repulsive", "Oh great, this is just nauseating. Obviously exactly what I wanted to smell today."),
                ("Totally disgusting", "Yeah sure, this revolting mess is just what this day needed. How delightful.")
            ]),
            ("sarcastic+surprise", [
                ("What a shocking twist", "Oh obviously, didn't see that coming at all. Such an amazing and unexpected development."),
                ("Absolutely stunned", "Yeah right, I'm completely shocked by this predictable outcome. How surprising.")
            ])
        ]
        
        # Affectionate combinations (11 × 2 = 22 posts)
        affectionate_emotions = [
            ("affectionate+sad", [
                ("Missing you sweetly", "My darling, I'm feeling blue without you here. My heart aches with loving sadness."),
                ("Bittersweet love", "Honey, even in my sorrow, I cherish our memories. Love fills my melancholy heart.")
            ]),
            ("affectionate+angry", [
                ("Mad but loving you", "Baby, I'm furious but I still adore you completely. My anger comes from caring so much."),
                ("Protective rage, my love", "Sweetheart, I'm angry because I love you. My heart burns with fierce protective feelings.")
            ]),
            ("affectionate+happy", [
                ("Joyful with my beloved", "My love, I'm so happy when I'm with you. Your presence fills my heart with warmth."),
                ("Blissful love", "Darling, being near you makes me glow with happiness. I treasure every loving moment.")
            ]),
            ("affectionate+joy", [
                ("Overjoyed with love", "My dear heart, I'm bursting with joy because of your love. You make me feel alive!"),
                ("Ecstatic devotion", "Beloved, my heart soars with pure loving joy. Every moment with you is magical!")
            ]),
            ("affectionate+excited", [
                ("Thrilled for us, love", "Honey, I'm so excited about our future together! My heart races with loving anticipation."),
                ("Buzzing with affection", "Sweetheart, I can barely contain my excitement to see you! My love for you energizes me.")
            ]),
            ("affectionate+confused", [
                ("Puzzled but loving", "My dear, I don't understand what's happening, but I love you through this confusion."),
                ("Bewildered with care", "Darling, I'm confused but my heart still overflows with tender feelings for you.")
            ]),
            ("affectionate+affection", [
                ("Pure loving devotion", "My beloved, I love you with every fiber of my being. You are my heart and soul."),
                ("Overflowing with love", "Sweetheart, my affection for you knows no bounds. I cherish you completely and deeply.")
            ]),
            ("affectionate+calm", [
                ("Peaceful love", "My dear, your presence brings such serene love to my heart. I feel so tranquil with you."),
                ("Serene devotion", "Beloved, I'm calm and content in your loving embrace. You bring peace to my soul.")
            ]),
            ("affectionate+fear", [
                ("Scared for you, love", "My darling, I'm terrified something might happen to you. My fear comes from loving you so much."),
                ("Protective worry", "Sweetheart, I'm afraid but I'll protect you always. My love makes me brave despite the fear.")
            ]),
            ("affectionate+disgust", [
                ("Revolted but loving", "My dear, this situation disgusts me, but my love for you remains pure and strong."),
                ("Nauseated with care", "Honey, I feel sick about this mess, but my tender feelings for you never waver.")
            ]),
            ("affectionate+surprise", [
                ("Amazed by love", "My beloved, I'm shocked by how much I love you! What a wonderful surprising feeling!"),
                ("Stunned with affection", "Darling, I'm surprised by these overwhelming loving feelings. You amaze my heart!")
            ])
        ]
        
        # Generate test cases for each emotion type
        for emotion, posts in basic_emotions:
            for i, (title, content) in enumerate(posts):
                test_cases.append(TestCase(
                    emotion_type=f"basic_{emotion}_{i+1}",
                    title=title,
                    content=content,
                    expected_sentiment_type=emotion,
                    expected_colors=self.color_mappings[emotion]
                ))
        
        for emotion, posts in sarcastic_emotions:
            for i, (title, content) in enumerate(posts):
                test_cases.append(TestCase(
                    emotion_type=f"sarcastic_{emotion}_{i+1}",
                    title=title,
                    content=content,
                    expected_sentiment_type=emotion,
                    expected_colors=self.color_mappings[emotion]
                ))
        
        for emotion, posts in affectionate_emotions:
            for i, (title, content) in enumerate(posts):
                test_cases.append(TestCase(
                    emotion_type=f"affectionate_{emotion}_{i+1}",
                    title=title,
                    content=content,
                    expected_sentiment_type=emotion,
                    expected_colors=self.color_mappings[emotion]
                ))
        
        return test_cases
    
    async def setup_session(self):
        """Initialize HTTP session and authenticate test user"""
        print("🔧 Setting up test session...")
        self.session = aiohttp.ClientSession()
        
        # Try to login with existing test user first
        try:
            await self._login_existing_test_user()
            print("✅ Logged in with existing test user")
        except Exception as e:
            print(f"   Existing user login failed: {e}")
            # Create new test user if login fails
            print("📝 Creating new test user...")
            test_user_data = await self._create_test_user()
            await self._login_new_test_user(test_user_data)
            print("✅ Created and logged in with new test user")
    
    async def _create_test_user(self):
        """Create a test user for the testing suite"""
        test_user_data = {
            "username": f"emotion_test_user_{int(time.time())}",
            "email": f"emotion_test_{int(time.time())}@test.com",
            "password": "test123456"
        }
        
        async with self.session.post(f"{API_BASE}/auth/register", json=test_user_data) as response:
            if response.status == 200:
                data = await response.json()
                self.test_user_id = data["user"]["id"]
                self.auth_token = data["token"]  # Use token from registration
                print(f"   Created user: {test_user_data['username']}")
                return test_user_data
            else:
                error_text = await response.text()
                raise Exception(f"Failed to create test user: {error_text}")
    
    async def _login_existing_test_user(self):
        """Login with existing test credentials"""
        login_data = {
            "email": "frontend@test.com",
            "password": "test123"
        }
        
        async with self.session.post(f"{API_BASE}/auth/login", json=login_data) as response:
            if response.status == 200:
                data = await response.json()
                self.auth_token = data["token"]
                self.test_user_id = data["user"]["id"]
            else:
                error_text = await response.text()
                raise Exception(f"Failed to login with existing user: {error_text}")
    
    async def _login_new_test_user(self, test_user_data):
        """Login with newly created test user"""
        login_data = {
            "email": test_user_data["email"],
            "password": test_user_data["password"]
        }
        
        async with self.session.post(f"{API_BASE}/auth/login", json=login_data) as response:
            if response.status == 200:
                data = await response.json()
                self.auth_token = data["token"]
                self.test_user_id = data["user"]["id"]
            else:
                error_text = await response.text()
                raise Exception(f"Failed to login with new user: {error_text}")
    
    async def create_test_post(self, test_case: TestCase) -> TestResult:
        """Create a single test post and validate the results"""
        print(f"📝 Testing {test_case.emotion_type}: {test_case.title[:50]}...")
        
        result = TestResult(
            test_case=test_case,
            post_id=None,
            actual_sentiment_type=None,
            actual_colors=[],
            popularity_score=None,
            passed=False,
            errors=[]
        )
        
        try:
            # Create the post
            post_data = {
                "title": test_case.title,
                "content": test_case.content
            }
            
            headers = {
                "Authorization": f"Bearer {self.auth_token}",
                "Content-Type": "application/json"
            }
            
            async with self.session.post(f"{API_BASE}/posts", json=post_data, headers=headers) as response:
                if response.status == 200:
                    data = await response.json()
                    post = data["post"]
                    
                    result.post_id = post["id"]
                    result.actual_sentiment_type = post.get("sentiment_type")
                    result.actual_colors = post.get("sentiment_colors", [])
                    result.popularity_score = post.get("popularity_score")
                    
                    self.created_posts.append(result.post_id)
                    
                    # Validate results
                    await self._validate_result(result)
                    
                else:
                    error_text = await response.text()
                    result.errors.append(f"Failed to create post: {error_text}")
                    
        except Exception as e:
            result.errors.append(f"Exception during post creation: {str(e)}")
        
        return result
    
    async def _validate_result(self, result: TestResult):
        """Validate sentiment analysis results against expectations"""
        errors = []
        
        # Validate sentiment type
        if result.actual_sentiment_type != result.test_case.expected_sentiment_type:
            errors.append(
                f"Sentiment type mismatch: expected '{result.test_case.expected_sentiment_type}', "
                f"got '{result.actual_sentiment_type}'"
            )
        
        # Validate colors (allow for some flexibility in gradient formats)
        expected_colors = set(result.test_case.expected_colors)
        actual_colors = set(result.actual_colors)
        
        if not expected_colors.issubset(actual_colors):
            missing_colors = expected_colors - actual_colors
            errors.append(
                f"Color mismatch: missing expected colors {list(missing_colors)}. "
                f"Expected: {result.test_case.expected_colors}, Got: {result.actual_colors}"
            )
        
        # Validate popularity score range (should be reasonable)
        if result.popularity_score is not None:
            if not (0.0 <= result.popularity_score <= 100.0):
                errors.append(f"Popularity score out of range: {result.popularity_score}")
        else:
            errors.append("Missing popularity score")
        
        result.errors.extend(errors)
        result.passed = len(errors) == 0
        
        if result.passed:
            print(f"   ✅ PASS: {result.test_case.emotion_type}")
        else:
            print(f"   ❌ FAIL: {result.test_case.emotion_type}")
            for error in errors:
                print(f"      {error}")
    
    async def run_all_tests(self):
        """Run all 66 test cases"""
        print(f"🚀 Starting emotion test suite with {len(self.test_cases)} test cases...")
        
        await self.setup_session()
        
        # Run tests in batches to avoid overwhelming the server
        batch_size = 5
        for i in range(0, len(self.test_cases), batch_size):
            batch = self.test_cases[i:i + batch_size]
            print(f"\n📦 Processing batch {i//batch_size + 1}/{(len(self.test_cases) + batch_size - 1)//batch_size}")
            
            # Run batch concurrently
            batch_results = await asyncio.gather(*[
                self.create_test_post(test_case) for test_case in batch
            ])
            
            self.results.extend(batch_results)
            
            # Brief pause between batches
            await asyncio.sleep(1)
        
        await self.generate_report()
    
    async def generate_report(self):
        """Generate comprehensive test report"""
        print("\n" + "="*80)
        print("📊 EMOTION TESTING SUITE RESULTS")
        print("="*80)
        
        passed = sum(1 for r in self.results if r.passed)
        failed = len(self.results) - passed
        
        print(f"Total Tests: {len(self.results)}")
        print(f"Passed: {passed} ✅")
        print(f"Failed: {failed} ❌")
        print(f"Success Rate: {(passed/len(self.results)*100):.1f}%")
        
        # Group results by emotion type
        basic_results = [r for r in self.results if r.test_case.emotion_type.startswith('basic_')]
        sarcastic_results = [r for r in self.results if r.test_case.emotion_type.startswith('sarcastic_')]
        affectionate_results = [r for r in self.results if r.test_case.emotion_type.startswith('affectionate_')]
        
        print(f"\n📈 Breakdown by category:")
        print(f"Basic emotions: {sum(1 for r in basic_results if r.passed)}/{len(basic_results)} passed")
        print(f"Sarcastic combos: {sum(1 for r in sarcastic_results if r.passed)}/{len(sarcastic_results)} passed")
        print(f"Affectionate combos: {sum(1 for r in affectionate_results if r.passed)}/{len(affectionate_results)} passed")
        
        # Show failed tests
        failed_results = [r for r in self.results if not r.passed]
        if failed_results:
            print(f"\n❌ Failed Tests ({len(failed_results)}):")
            for result in failed_results:
                print(f"\n{result.test_case.emotion_type}:")
                print(f"  Title: {result.test_case.title}")
                print(f"  Expected: {result.test_case.expected_sentiment_type}")
                print(f"  Actual: {result.actual_sentiment_type}")
                for error in result.errors:
                    print(f"  Error: {error}")
        
        # Summary statistics
        print(f"\n📊 Statistics:")
        sentiment_types = {}
        for result in self.results:
            if result.actual_sentiment_type:
                sentiment_types[result.actual_sentiment_type] = sentiment_types.get(result.actual_sentiment_type, 0) + 1
        
        print("Detected emotion distribution:")
        for emotion, count in sorted(sentiment_types.items()):
            print(f"  {emotion}: {count}")
        
        print("\n🏁 Test suite completed!")
        
        return passed == len(self.results)
    
    async def cleanup(self):
        """Clean up test data and close session"""
        print(f"\n🧹 Cleaning up {len(self.created_posts)} test posts...")
        
        # Note: In a real implementation, you might want to delete the test posts
        # For now, we'll just close the session
        
        if self.session:
            await self.session.close()
        
        print("✅ Cleanup completed")

async def main():
    """Main test runner"""
    suite = EmotionTestSuite()
    
    try:
        success = await suite.run_all_tests()
        await suite.cleanup()
        
        sys.exit(0 if success else 1)
        
    except Exception as e:
        print(f"❌ Test suite failed: {e}")
        await suite.cleanup()
        sys.exit(1)

if __name__ == "__main__":
    print("🎯 Emotion Testing Suite - Comprehensive Regression Test")
    print("Testing 33 emotion combinations × 2 posts each = 66 total tests")
    print("Validating sentiment_type, sentiment_colors, and popularity_score")
    print("-" * 80)
    
    asyncio.run(main())