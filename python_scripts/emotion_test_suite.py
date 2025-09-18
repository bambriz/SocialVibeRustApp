#!/usr/bin/env python3
"""
Comprehensive Emotion Testing Suite

Tests all 33 emotion combinations (11 basic + 11 sarcastic + 11 affectionate) 
with 2 posts each for a total of 66 test posts.

Validates:
- sentiment_type matches expected emotion
- sentiment_colors match expected color mappings
- popularity_scores are in expected ranges
- UI visual verification (with --ui-verify flag)
- Provides clear pass/fail reporting

Usage:
    python3 python_scripts/emotion_test_suite.py [--ui-verify]
"""
import asyncio
import aiohttp
import json
import sys
import time
import argparse
import os
import re
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
from datetime import datetime

# UI verification imports
try:
    from selenium import webdriver
    from selenium.webdriver.common.by import By
    from selenium.webdriver.support.ui import WebDriverWait
    from selenium.webdriver.support import expected_conditions as EC
    from selenium.webdriver.chrome.options import Options as ChromeOptions
    from selenium.webdriver.firefox.options import Options as FirefoxOptions
    from selenium.webdriver.chrome.service import Service as ChromeService
    from selenium.webdriver.firefox.service import Service as FirefoxService
    from webdriver_manager.chrome import ChromeDriverManager
    from webdriver_manager.firefox import GeckoDriverManager
    from PIL import Image
    import numpy as np
    import cv2
    from sklearn.cluster import KMeans
    UI_VERIFICATION_AVAILABLE = True
except ImportError as e:
    print(f"Warning: UI verification dependencies not available: {e}")
    UI_VERIFICATION_AVAILABLE = False
    # Define dummy classes to prevent unbound errors
    ChromeOptions = None
    FirefoxOptions = None
    ChromeService = None
    FirefoxService = None
    ChromeDriverManager = None
    GeckoDriverManager = None
    WebDriverWait = None
    EC = None
    By = None
    np = None
    cv2 = None
    KMeans = None

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
    ui_verification_passed: Optional[bool] = None
    ui_verification_errors: Optional[List[str]] = None
    screenshot_path: Optional[str] = None
    
    def __post_init__(self):
        if self.ui_verification_errors is None:
            self.ui_verification_errors = []

class EmotionTestSuite:
    def __init__(self, ui_verify=False):
        self.session = None
        self.auth_token = None
        self.test_user_id = None
        self.created_posts = []
        self.results = []
        self.ui_verify = ui_verify
        self.driver = None
        self.screenshot_dir = "screenshots"
        
        # Create screenshots directory if UI verification is enabled
        if self.ui_verify:
            os.makedirs(self.screenshot_dir, exist_ok=True)
        
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
        
        # Select key test cases for UI verification (reduced set for speed)
        self.ui_test_cases = self._select_ui_test_cases() if ui_verify else []
    
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
    
    def _select_ui_test_cases(self) -> List[TestCase]:
        """Select a subset of key test cases for UI verification"""
        # Choose representative cases covering all major emotion types
        ui_cases = []
        
        # Basic emotions - 1 case each for key emotions
        basic_selections = {
            "angry": 0,    # First angry test case
            "joy": 0,      # First joy test case  
            "sad": 0,      # First sad test case
            "sarcastic": 0, # First sarcastic test case
            "happy": 0,    # First happy test case
            "affection": 0 # First affection test case
        }
        
        # Combo emotions - key combinations
        combo_selections = {
            "sarcastic+joy": 0,        # Sarcastic combo
            "sarcastic+angry": 0,      # Another sarcastic combo
            "affectionate+joy": 0,     # Affectionate combo
            "affectionate+sad": 0      # Another affectionate combo
        }
        
        # Extract selected test cases
        for test_case in self.test_cases:
            emotion_type = test_case.emotion_type
            
            # Check basic emotions
            for basic_emotion, selected_index in basic_selections.items():
                if emotion_type.startswith(f"basic_{basic_emotion}"):
                    case_index = int(emotion_type.split('_')[-1]) - 1  # Extract case number
                    if case_index == selected_index:
                        ui_cases.append(test_case)
                        break
            
            # Check combo emotions
            for combo_emotion, selected_index in combo_selections.items():
                if combo_emotion in emotion_type:
                    case_index = int(emotion_type.split('_')[-1]) - 1  # Extract case number  
                    if case_index == selected_index:
                        ui_cases.append(test_case)
                        break
        
        print(f"📱 Selected {len(ui_cases)} test cases for UI verification")
        return ui_cases
    
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
                    if "post" in data:
                        post = data["post"]
                    else:
                        post = data
                    
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
        
        # Run UI verification if enabled
        if self.ui_verify:
            await self.run_ui_verification()
        
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
    
    def setup_ui_verification(self):
        """Setup Selenium webdriver for UI verification"""
        if not UI_VERIFICATION_AVAILABLE:
            raise Exception("UI verification dependencies not available. Please install selenium, pillow, opencv-python, and webdriver-manager.")
        
        print("🌐 Setting up browser for UI verification...")
        
        # Try Chrome first, fallback to Firefox
        try:
            chrome_options = ChromeOptions()
            chrome_options.add_argument('--headless')
            chrome_options.add_argument('--no-sandbox')
            chrome_options.add_argument('--disable-dev-shm-usage')
            chrome_options.add_argument('--window-size=1920,1080')
            
            # Use webdriver manager to handle driver installation
            service = ChromeService(ChromeDriverManager().install())
            self.driver = webdriver.Chrome(service=service, options=chrome_options)
            print("✅ Chrome driver initialized")
            
        except Exception as e:
            print(f"Chrome driver failed: {e}")
            print("Trying Firefox...")
            
            try:
                firefox_options = FirefoxOptions()
                firefox_options.add_argument('--headless')
                firefox_options.add_argument('--width=1920')
                firefox_options.add_argument('--height=1080')
                
                service = FirefoxService(GeckoDriverManager().install())
                self.driver = webdriver.Firefox(service=service, options=firefox_options)
                print("✅ Firefox driver initialized")
                
            except Exception as e:
                raise Exception(f"Failed to initialize both Chrome and Firefox drivers: {e}")
    
    def teardown_ui_verification(self):
        """Clean up UI verification resources"""
        if self.driver:
            self.driver.quit()
            self.driver = None
            print("🔌 Browser driver closed")
    
    def take_screenshot(self, test_case: TestCase, post_id: str) -> str:
        """Take a screenshot of the frontend with the test post visible"""
        if not self.driver:
            raise Exception("UI verification not initialized")
        
        # Navigate to the main page
        self.driver.get(BASE_URL)
        
        # Wait for page to load
        wait = WebDriverWait(self.driver, 10)
        
        try:
            # Wait for posts to load
            wait.until(EC.presence_of_element_located((By.ID, "postsList")))
            
            # Give extra time for all posts to render
            time.sleep(2)
            
            # Take screenshot
            timestamp = int(time.time())
            emotion_type_clean = re.sub(r'[^a-zA-Z0-9_+]', '_', test_case.emotion_type)
            screenshot_filename = f"{emotion_type_clean}_{timestamp}.png"
            screenshot_path = os.path.join(self.screenshot_dir, screenshot_filename)
            
            self.driver.save_screenshot(screenshot_path)
            print(f"📸 Screenshot saved: {screenshot_path}")
            
            return screenshot_path
            
        except Exception as e:
            raise Exception(f"Failed to take screenshot: {e}")
    
    def analyze_post_colors(self, screenshot_path: str, test_case: TestCase) -> Tuple[bool, List[str]]:
        """Analyze screenshot to verify post colors match expectations"""
        try:
            # Load screenshot
            image = cv2.imread(screenshot_path)
            if image is None:
                return False, ["Could not load screenshot image"]
            
            # Convert BGR to RGB
            image_rgb = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
            
            # Get expected colors from test case
            expected_colors = [color.lower() for color in test_case.expected_colors]
            
            # Analyze colors in the image
            detected_colors = self._extract_sentiment_colors_from_image(image_rgb)
            
            errors = []
            
            # Check if expected colors are present
            for expected_color in expected_colors:
                if not self._is_color_present_in_image(expected_color, detected_colors, image_rgb):
                    errors.append(f"Expected color {expected_color} not found in UI")
            
            # For combo emotions, verify gradient patterns
            if len(expected_colors) > 1:
                gradient_valid = self._verify_gradient_pattern(image_rgb, expected_colors)
                if not gradient_valid:
                    errors.append("Gradient pattern does not match expected combo emotion colors")
            
            return len(errors) == 0, errors
            
        except Exception as e:
            return False, [f"Error analyzing screenshot: {str(e)}"]
    
    def _extract_sentiment_colors_from_image(self, image_rgb: np.ndarray) -> List[str]:
        """Extract prominent colors from image that might be sentiment colors"""
        # Reshape image to list of pixels
        pixels = image_rgb.reshape(-1, 3)
        
        # Use kmeans clustering to find dominant colors
        if KMeans is None:
            return []
        
        # Cluster into 8 dominant colors
        kmeans = KMeans(n_clusters=8, random_state=42, n_init='auto')
        kmeans.fit(pixels)
        
        # Get cluster centers (dominant colors)
        dominant_colors = kmeans.cluster_centers_.astype(int)
        
        # Convert to hex colors
        hex_colors = []
        for color in dominant_colors:
            hex_color = f"#{color[0]:02x}{color[1]:02x}{color[2]:02x}"
            hex_colors.append(hex_color)
        
        return hex_colors
    
    def _is_color_present_in_image(self, expected_color: str, detected_colors: List[str], image_rgb: np.ndarray) -> bool:
        """Check if expected color is present in the image with tolerance"""
        # Convert expected color to RGB
        expected_rgb = self._hex_to_rgb(expected_color)
        
        # Check against detected dominant colors first
        for detected_color in detected_colors:
            detected_rgb = self._hex_to_rgb(detected_color)
            if self._colors_similar(expected_rgb, detected_rgb, tolerance=30):
                return True
        
        # Also check by scanning the image for similar colors
        # Sample pixels from different regions
        h, w = image_rgb.shape[:2]
        sample_points = [
            (h//4, w//4), (h//4, 3*w//4),
            (h//2, w//4), (h//2, w//2), (h//2, 3*w//4),
            (3*h//4, w//4), (3*h//4, 3*w//4)
        ]
        
        for y, x in sample_points:
            pixel_color = image_rgb[y, x]
            if self._colors_similar(expected_rgb, pixel_color, tolerance=40):
                return True
        
        return False
    
    def _verify_gradient_pattern(self, image_rgb: np.ndarray, expected_colors: List[str]) -> bool:
        """Verify that a gradient pattern exists for combo emotions"""
        # For combo emotions, we expect to see both colors in close proximity
        # This is a simplified check - in a real implementation you might want more sophisticated gradient detection
        
        color1_rgb = self._hex_to_rgb(expected_colors[0])
        color2_rgb = self._hex_to_rgb(expected_colors[1])
        
        color1_found = False
        color2_found = False
        
        # Sample multiple regions of the image
        h, w = image_rgb.shape[:2]
        for y in range(0, h, h//10):
            for x in range(0, w, w//10):
                if y < h and x < w:
                    pixel = image_rgb[y, x]
                    if self._colors_similar(color1_rgb, pixel, tolerance=50):
                        color1_found = True
                    if self._colors_similar(color2_rgb, pixel, tolerance=50):
                        color2_found = True
        
        return color1_found and color2_found
    
    def _hex_to_rgb(self, hex_color: str) -> Tuple[int, int, int]:
        """Convert hex color to RGB tuple"""
        hex_color = hex_color.lstrip('#')
        rgb_values = [int(hex_color[i:i+2], 16) for i in (0, 2, 4)]
        return (rgb_values[0], rgb_values[1], rgb_values[2])
    
    def _colors_similar(self, color1: Tuple[int, int, int], color2: Tuple[int, int, int], tolerance: int = 30) -> bool:
        """Check if two RGB colors are similar within tolerance"""
        return all(abs(c1 - c2) <= tolerance for c1, c2 in zip(color1, color2))

    async def run_ui_verification(self):
        """Run UI verification tests on selected test cases"""
        if not self.ui_verify or not self.ui_test_cases:
            return
        
        print(f"\n🎨 Starting UI verification for {len(self.ui_test_cases)} test cases...")
        
        try:
            self.setup_ui_verification()
            
            # Find the test results for our UI test cases
            ui_results = []
            for ui_test_case in self.ui_test_cases:
                # Find the corresponding result from the main test run
                matching_result = next(
                    (r for r in self.results if r.test_case.emotion_type == ui_test_case.emotion_type),
                    None
                )
                
                if matching_result and matching_result.post_id:
                    ui_results.append(matching_result)
                else:
                    print(f"⚠️ No result found for UI test case: {ui_test_case.emotion_type}")
            
            print(f"📱 Running UI verification on {len(ui_results)} posts...")
            
            # Verify each post's UI display
            for result in ui_results:
                print(f"🔍 UI verification: {result.test_case.emotion_type}")
                
                try:
                    # Take screenshot
                    screenshot_path = self.take_screenshot(result.test_case, result.post_id)
                    result.screenshot_path = screenshot_path
                    
                    # Analyze colors
                    ui_passed, ui_errors = self.analyze_post_colors(screenshot_path, result.test_case)
                    result.ui_verification_passed = ui_passed
                    result.ui_verification_errors = ui_errors
                    
                    if ui_passed:
                        print(f"   ✅ UI PASS: Colors match expectations")
                    else:
                        print(f"   ❌ UI FAIL: {ui_errors}")
                    
                except Exception as e:
                    result.ui_verification_passed = False
                    result.ui_verification_errors = [f"UI verification error: {str(e)}"]
                    print(f"   ❌ UI ERROR: {str(e)}")
                
                # Brief pause between screenshots
                time.sleep(1)
            
        finally:
            self.teardown_ui_verification()
        
        # Update overall results
        ui_passed = sum(1 for r in ui_results if r.ui_verification_passed)
        ui_failed = len(ui_results) - ui_passed
        
        print(f"\n🎨 UI Verification Summary:")
        print(f"UI Tests Passed: {ui_passed}/{len(ui_results)}")
        print(f"UI Tests Failed: {ui_failed}/{len(ui_results)}")
        if len(ui_results) > 0:
            print(f"UI Success Rate: {(ui_passed/len(ui_results)*100):.1f}%")

async def main(args):
    """Main test runner"""
    suite = EmotionTestSuite(ui_verify=args.ui_verify)
    
    try:
        success = await suite.run_all_tests()
        await suite.cleanup()
        
        return success
        
    except Exception as e:
        print(f"❌ Test suite failed: {e}")
        await suite.cleanup()
        raise e

if __name__ == "__main__":
    # Parse command line arguments
    parser = argparse.ArgumentParser(description='Emotion Testing Suite')
    parser.add_argument('--ui-verify', action='store_true', 
                       help='Enable UI verification with screenshots')
    args = parser.parse_args()
    
    print("🎯 Emotion Testing Suite - Comprehensive Regression Test")
    print("Testing 33 emotion combinations × 2 posts each = 66 total tests")
    print("Validating sentiment_type, sentiment_colors, and popularity_score")
    
    if args.ui_verify:
        print("🎨 UI Verification ENABLED - will take screenshots and verify colors")
        if not UI_VERIFICATION_AVAILABLE:
            print("❌ ERROR: UI verification dependencies not available")
            print("Please install: selenium, pillow, opencv-python, webdriver-manager, scikit-learn")
            sys.exit(1)
    
    print("-" * 80)
    
    # Run the main function and handle the result
    try:
        success = asyncio.run(main(args))
        sys.exit(0 if success else 1)
    except Exception:
        sys.exit(1)