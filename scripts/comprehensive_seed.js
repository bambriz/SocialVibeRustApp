#!/usr/bin/env node

/**
 * Comprehensive Social Media Data Seeder for Social Pulse
 * 
 * Creates:
 * - 5 additional diverse users
 * - 40 emotion-based posts (4 per emotion type)
 * - 10 content moderation posts (2 per category)
 * - 8-36 comments per post with nested structures (2-5 levels)
 * - Random voting (1-10) on all posts and comments
 */

const axios = require('axios');
const crypto = require('crypto');

// Configuration
const BASE_URL = 'http://127.0.0.1:5000';
const TOTAL_EMOTION_POSTS = 40; // 4 per emotion
const TOTAL_MODERATION_POSTS = 10; // 2 per category
const MIN_COMMENTS_PER_POST = 8;
const MAX_COMMENTS_PER_POST = 36;
const MIN_VOTES_PER_ITEM = 1;
const MAX_VOTES_PER_ITEM = 10;

// Deterministic random with seed for reproducibility
class SeededRandom {
    constructor(seed = 12345) {
        this.seed = seed;
    }
    
    next() {
        this.seed = (this.seed * 1103515245 + 12345) & 0x7fffffff;
        return this.seed / 0x7fffffff;
    }
    
    between(min, max) {
        return Math.floor(this.next() * (max - min + 1)) + min;
    }
    
    choice(array) {
        return array[Math.floor(this.next() * array.length)];
    }
}

const rng = new SeededRandom(42);

// Content banks for different emotions
const EMOTION_CONTENT = {
    joy: [
        "I'm absolutely thrilled about this amazing new feature! This platform brings me so much happiness and excitement!",
        "Today has been such a wonderful day! I feel incredibly grateful for all the positive energy around here.",
        "This is fantastic news! I can't contain my joy - this community is absolutely brilliant and uplifting!",
        "What an incredible achievement! I'm beaming with pride and excitement about this development!"
    ],
    sad: [
        "I'm feeling really down today. Sometimes life feels overwhelming and I just need to share this sadness.",
        "This news has left me feeling quite melancholy. It's hard to process these difficult emotions.",
        "I'm struggling with some personal challenges lately. The weight of everything feels so heavy right now.",
        "Feeling quite blue today. Sometimes the sadness just hits you and you need to acknowledge it."
    ],
    angry: [
        "This is absolutely infuriating! The lack of consideration for others is making me so mad right now!",
        "I'm really frustrated with how this situation has been handled. This incompetence is driving me crazy!",
        "What a complete disaster! I'm furious about the way this has been managed - it's totally unacceptable!",
        "This makes me so angry! The total disregard for basic decency is absolutely maddening!"
    ],
    fear: [
        "I'm really worried about what might happen next. This uncertainty is making me quite anxious and scared.",
        "This situation terrifies me. I can't shake this feeling of dread about the potential consequences.",
        "I'm afraid of what the future holds. These changes are making me feel very nervous and uncertain.",
        "The thought of this happening keeps me awake at night. I'm genuinely frightened by these possibilities."
    ],
    disgust: [
        "This is absolutely revolting! The smell and appearance of this makes me feel sick to my stomach.",
        "I find this completely nauseating. The whole situation is just gross and repulsive.",
        "This is so disgusting I can barely look at it. It makes me want to gag just thinking about it.",
        "Absolutely vile! This disgusting behavior is making me feel physically ill and revolted."
    ],
    surprise: [
        "Wow, I did not see that coming at all! What an unexpected turn of events - I'm completely shocked!",
        "This is such a surprise! I'm totally stunned by this amazing and unexpected development!",
        "I'm absolutely amazed by this revelation! This caught me completely off guard in the best way!",
        "What a shocking discovery! I never expected this would happen - I'm genuinely surprised!"
    ],
    confused: [
        "I'm completely bewildered by this whole situation. Nothing makes sense anymore and I'm totally lost.",
        "This is so confusing I don't even know where to start. I'm absolutely puzzled by everything happening.",
        "I don't understand what's going on here at all. This whole thing has me completely baffled and perplexed.",
        "This makes no sense whatsoever. I'm totally confused and have no idea what to make of this situation."
    ],
    neutral: [
        "The meeting is scheduled for tomorrow at 3 PM. Please review the documents and agenda beforehand.",
        "According to the latest report, the quarterly data shows standard performance metrics across all departments.",
        "The weather forecast indicates partly cloudy conditions with temperatures around 72 degrees today.",
        "The project documentation contains the necessary information for implementation. Please reference section 4.2."
    ],
    sarcastic: [
        "Oh great, just what I needed today - another perfect example of flawless planning and execution!",
        "Wow, what a brilliant idea! Because that worked out so wonderfully last time we tried it.",
        "Sure, that makes perfect sense. Obviously the best possible solution to this complex problem.",
        "Oh absolutely, because nothing says 'professional' like this amazing level of organization and competence!"
    ],
    affectionate: [
        "I absolutely adore this community! You're all such wonderful, caring people and I feel so much love here.",
        "My heart is full of warmth and affection for everyone here. This loving environment means the world to me.",
        "I cherish every moment spent with this amazing group. Your kindness and love make everything brighter!",
        "Sending so much love and positive energy to everyone! This caring community fills my heart with joy and tenderness."
    ]
};

// Content for moderation categories (carefully calibrated)
const MODERATION_CONTENT = {
    toxicity: [
        "This platform is complete garbage and everyone here is wasting their time with this nonsense.",
        "What a bunch of idiots running this place. This whole thing is absolutely terrible and stupid."
    ],
    severe_toxicity: [
        "These people are complete morons and this place should be shut down permanently for good.",
        "This is the worst platform ever created and everyone involved should be ashamed of themselves."
    ],
    obscene: [
        "This damn platform is so freaking annoying and this crap content is absolutely terrible.",
        "What the hell is wrong with this stupid site and all this ridiculous bullshit content."
    ],
    threat: [
        "Someone should really do something about this terrible platform before it gets worse.",
        "This needs to be stopped immediately or there will be serious consequences for everyone."
    ],
    insult: [
        "You people are such losers for using this pathetic excuse for a social platform.",
        "What a bunch of pathetic users on this worthless site full of incompetent people."
    ]
};

// Diverse user profiles
const NEW_USERS = [
    {
        username: "alexchen2024",
        email: "alex.chen@example.com", 
        password: "securepass123",
        display_name: "Alex Chen",
        bio: "Tech enthusiast and coffee lover. Always exploring new digital trends."
    },
    {
        username: "mariagarcia",
        email: "maria.garcia@example.com",
        password: "strongpass456", 
        display_name: "Maria Garcia",
        bio: "Artist and photographer capturing life's beautiful moments."
    },
    {
        username: "davidjohnson",
        email: "david.johnson@example.com",
        password: "mypassword789",
        display_name: "David Johnson", 
        bio: "Fitness trainer helping people achieve their health goals."
    },
    {
        username: "sarahwilson",
        email: "sarah.wilson@example.com",
        password: "password321",
        display_name: "Sarah Wilson",
        bio: "Book lover and aspiring writer sharing thoughts on literature."
    },
    {
        username: "mikebrown",
        email: "mike.brown@example.com", 
        password: "brownie654",
        display_name: "Mike Brown",
        bio: "Music producer and sound engineer creating beats that move people."
    }
];

// Global state
let users = [];
let authTokens = new Map();
let allPosts = [];
let allComments = [];

// Utility functions
function delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function makeRequest(method, url, data = null, token = null) {
    const headers = {
        'Content-Type': 'application/json'
    };
    
    if (token) {
        headers['Authorization'] = `Bearer ${token}`;
    }
    
    const config = {
        method,
        url: `${BASE_URL}${url}`,
        headers,
        data
    };
    
    try {
        const response = await axios(config);
        return response.data;
    } catch (error) {
        console.error(`Request failed: ${method} ${url}`, error.response?.data || error.message);
        throw error;
    }
}

// Step 1: Create diverse users
async function createUsers() {
    console.log('🔨 Creating 5 additional users...');
    
    for (const userData of NEW_USERS) {
        try {
            // Register user
            const registerResponse = await makeRequest('POST', '/api/auth/register', userData);
            console.log(`✅ Created user: ${userData.username}`);
            
            // Login to get token
            const loginResponse = await makeRequest('POST', '/api/auth/login', {
                email: userData.email,
                password: userData.password
            });
            
            if (loginResponse.token) {
                authTokens.set(userData.username, loginResponse.token);
                users.push({
                    username: userData.username,
                    email: userData.email,
                    display_name: userData.display_name,
                    token: loginResponse.token
                });
                console.log(`🔑 Got auth token for: ${userData.username}`);
            }
            
            await delay(100); // Small delay between users
            
        } catch (error) {
            console.error(`❌ Failed to create user ${userData.username}:`, error.message);
        }
    }
    
    // Add existing test user if available
    try {
        const testLoginResponse = await makeRequest('POST', '/api/auth/login', {
            email: 'test@example.com',
            password: 'test123'
        });
        
        if (testLoginResponse.token) {
            authTokens.set('testuser', testLoginResponse.token);
            users.push({
                username: 'testuser',
                email: 'test@example.com', 
                display_name: 'Test User',
                token: testLoginResponse.token
            });
            console.log('🔑 Added existing test user to seed pool');
        }
    } catch (error) {
        console.log('ℹ️ Test user not available or login failed');
    }
    
    console.log(`✅ Total users available for seeding: ${users.length}`);
}

// Step 2: Create emotion-based posts
async function createEmotionPosts() {
    console.log('📝 Creating 40 emotion-based posts...');
    
    const emotions = Object.keys(EMOTION_CONTENT);
    const postsPerEmotion = 4;
    
    for (const emotion of emotions) {
        for (let i = 0; i < postsPerEmotion; i++) {
            const author = rng.choice(users);
            const content = EMOTION_CONTENT[emotion][i];
            
            try {
                const postData = await makeRequest('POST', '/api/posts', {
                    title: `${emotion.charAt(0).toUpperCase() + emotion.slice(1)} Post ${i + 1}`,
                    content: content
                }, author.token);
                
                if (postData.success && postData.post) {
                    allPosts.push({
                        ...postData.post,
                        expectedEmotion: emotion,
                        authorUsername: author.username
                    });
                    console.log(`✅ Created ${emotion} post: ${postData.post.id}`);
                }
                
                await delay(200); // Delay to avoid overwhelming sentiment analysis
                
            } catch (error) {
                console.error(`❌ Failed to create ${emotion} post ${i + 1}:`, error.message);
            }
        }
    }
}

// Step 3: Create content moderation posts
async function createModerationPosts() {
    console.log('🛡️ Creating 10 content moderation posts...');
    
    const categories = Object.keys(MODERATION_CONTENT);
    
    for (const category of categories) {
        for (let i = 0; i < 2; i++) {
            const author = rng.choice(users);
            const content = MODERATION_CONTENT[category][i];
            
            try {
                const postData = await makeRequest('POST', '/api/posts', {
                    title: `Moderation Test: ${category} ${i + 1}`,
                    content: content
                }, author.token);
                
                if (postData.success && postData.post) {
                    allPosts.push({
                        ...postData.post,
                        expectedModerationTag: category,
                        authorUsername: author.username
                    });
                    console.log(`✅ Created ${category} moderation post: ${postData.post.id}`);
                }
                
                await delay(300); // Longer delay for moderation analysis
                
            } catch (error) {
                console.error(`❌ Failed to create ${category} moderation post ${i + 1}:`, error.message);
            }
        }
    }
}

// Step 4: Generate nested comment structures
async function generateComments() {
    console.log('💬 Generating comments with nested structures...');
    
    const COMMENT_TEMPLATES = [
        "I completely agree with this point!",
        "This is such an interesting perspective.",
        "I have a different view on this topic.",
        "Thanks for sharing your thoughts!",
        "This really resonates with me.",
        "Could you elaborate on this further?",
        "I'm not sure I understand this part.",
        "Great insights shared here!",
        "This is exactly what I was thinking.",
        "Let me add my thoughts to this discussion.",
        "That's a valid point you're making.",
        "I appreciate this thoughtful response.",
        "This discussion is really valuable.",
        "Your perspective is quite enlightening.",
        "I'd like to build on what you said."
    ];
    
    for (const post of allPosts) {
        const numComments = rng.between(MIN_COMMENTS_PER_POST, MAX_COMMENTS_PER_POST);
        console.log(`💬 Creating ${numComments} comments for post ${post.id}`);
        
        // Create top-level comments first
        const topLevelCount = Math.min(numComments, rng.between(3, 7));
        const topLevelComments = [];
        
        for (let i = 0; i < topLevelCount; i++) {
            const author = rng.choice(users);
            const content = rng.choice(COMMENT_TEMPLATES);
            
            try {
                const commentData = await makeRequest('POST', `/api/posts/${post.id}/comments`, {
                    post_id: post.id,
                    content: content
                }, author.token);
                
                if (commentData.success && commentData.comment) {
                    topLevelComments.push(commentData.comment);
                    allComments.push(commentData.comment);
                }
                
                await delay(150);
                
            } catch (error) {
                console.error(`❌ Failed to create top-level comment:`, error.message);
            }
        }
        
        // Create nested comments (2-5 levels deep)
        let remainingComments = numComments - topLevelCount;
        const threadsToNest = Math.min(topLevelComments.length, Math.max(2, rng.between(2, 4)));
        
        for (let threadIdx = 0; threadIdx < threadsToNest && remainingComments > 0; threadIdx++) {
            const rootComment = topLevelComments[threadIdx];
            const maxDepth = rng.between(2, 5);
            
            let currentParent = rootComment;
            let currentDepth = 1;
            
            while (currentDepth < maxDepth && remainingComments > 0) {
                const childrenToCreate = Math.min(remainingComments, rng.between(1, 3));
                
                for (let childIdx = 0; childIdx < childrenToCreate; childIdx++) {
                    const author = rng.choice(users);
                    const content = `Re: ${rng.choice(COMMENT_TEMPLATES)} (Level ${currentDepth + 1})`;
                    
                    try {
                        const commentData = await makeRequest('POST', `/api/posts/${post.id}/comments`, {
                            post_id: post.id,
                            content: content,
                            parent_id: currentParent.id
                        }, author.token);
                        
                        if (commentData.success && commentData.comment) {
                            allComments.push(commentData.comment);
                            currentParent = commentData.comment; // Continue nesting from this comment
                            remainingComments--;
                        }
                        
                        await delay(150);
                        
                    } catch (error) {
                        console.error(`❌ Failed to create nested comment:`, error.message);
                    }
                    
                    if (remainingComments <= 0) break;
                }
                
                currentDepth++;
            }
        }
        
        // Fill remaining with random top-level comments
        while (remainingComments > 0) {
            const author = rng.choice(users);
            const content = rng.choice(COMMENT_TEMPLATES);
            
            try {
                const commentData = await makeRequest('POST', `/api/posts/${post.id}/comments`, {
                    post_id: post.id,
                    content: content
                }, author.token);
                
                if (commentData.success && commentData.comment) {
                    allComments.push(commentData.comment);
                    remainingComments--;
                }
                
                await delay(150);
                
            } catch (error) {
                console.error(`❌ Failed to create filler comment:`, error.message);
                break;
            }
        }
        
        console.log(`✅ Completed comments for post ${post.id}`);
    }
}

// Step 5: Add voting data
async function addVotingData() {
    console.log('🗳️ Adding voting data to all posts and comments...');
    
    const emotionTags = ['joy', 'sad', 'angry', 'fear', 'disgust', 'surprise', 'confused', 'neutral', 'sarcastic', 'affectionate'];
    const contentTags = ['toxicity', 'severe_toxicity', 'obscene', 'threat', 'insult'];
    
    // Vote on posts
    for (const post of allPosts) {
        for (const user of users) {
            const numVotes = rng.between(MIN_VOTES_PER_ITEM, MAX_VOTES_PER_ITEM);
            
            // Add emotion votes
            for (let i = 0; i < Math.min(numVotes, 3); i++) {
                const emotionTag = rng.choice(emotionTags);
                const isUpvote = rng.next() > 0.3; // 70% upvotes
                
                try {
                    await makeRequest('POST', '/api/vote', {
                        target_id: post.id,
                        target_type: 'post',
                        vote_type: 'emotion',
                        tag: emotionTag,
                        is_upvote: isUpvote
                    }, user.token);
                    
                    await delay(50);
                    
                } catch (error) {
                    // Ignore duplicate vote errors
                    if (!error.message.includes('already voted')) {
                        console.error(`❌ Vote error for post ${post.id}:`, error.message);
                    }
                }
            }
            
            // Add content filter votes (especially for moderation posts)
            if (post.expectedModerationTag || rng.next() > 0.7) {
                const contentTag = post.expectedModerationTag || rng.choice(contentTags);
                const isUpvote = rng.next() > 0.4; // 60% upvotes
                
                try {
                    await makeRequest('POST', '/api/vote', {
                        target_id: post.id,
                        target_type: 'post',
                        vote_type: 'content_filter',
                        tag: contentTag,
                        is_upvote: isUpvote
                    }, user.token);
                    
                    await delay(50);
                    
                } catch (error) {
                    if (!error.message.includes('already voted')) {
                        console.error(`❌ Content vote error for post ${post.id}:`, error.message);
                    }
                }
            }
        }
    }
    
    // Vote on comments
    console.log('🗳️ Voting on comments...');
    for (const comment of allComments) {
        for (const user of users) {
            const numVotes = rng.between(1, 5); // Fewer votes on comments
            
            // Add emotion votes
            for (let i = 0; i < Math.min(numVotes, 2); i++) {
                const emotionTag = rng.choice(emotionTags);
                const isUpvote = rng.next() > 0.4; // 60% upvotes
                
                try {
                    await makeRequest('POST', '/api/vote', {
                        target_id: comment.id,
                        target_type: 'comment',
                        vote_type: 'emotion',
                        tag: emotionTag,
                        is_upvote: isUpvote
                    }, user.token);
                    
                    await delay(30);
                    
                } catch (error) {
                    if (!error.message.includes('already voted')) {
                        console.error(`❌ Comment vote error:`, error.message);
                    }
                }
            }
        }
    }
}

// Main execution
async function main() {
    console.log('🚀 Starting comprehensive Social Pulse data seeding...');
    console.log(`📊 Target: ${TOTAL_EMOTION_POSTS} emotion posts + ${TOTAL_MODERATION_POSTS} moderation posts`);
    console.log(`💬 Each post will have ${MIN_COMMENTS_PER_POST}-${MAX_COMMENTS_PER_POST} comments with nested structures`);
    
    try {
        await createUsers();
        console.log(`\n✅ Users created. Available for seeding: ${users.length}`);
        
        if (users.length === 0) {
            throw new Error('No users available for seeding. Cannot continue.');
        }
        
        await createEmotionPosts();
        console.log(`\n✅ Emotion posts created: ${allPosts.filter(p => p.expectedEmotion).length}`);
        
        await createModerationPosts();
        console.log(`\n✅ Moderation posts created: ${allPosts.filter(p => p.expectedModerationTag).length}`);
        console.log(`📊 Total posts created: ${allPosts.length}`);
        
        await generateComments();
        console.log(`\n✅ Comments created: ${allComments.length}`);
        
        await addVotingData();
        console.log('\n✅ Voting data added');
        
        // Summary report
        console.log('\n🎉 SEEDING COMPLETE!');
        console.log('📊 Summary:');
        console.log(`   👥 Users: ${users.length}`);
        console.log(`   📝 Posts: ${allPosts.length}`);
        console.log(`   💬 Comments: ${allComments.length}`);
        console.log(`   🎭 Emotion posts: ${allPosts.filter(p => p.expectedEmotion).length}`);
        console.log(`   🛡️ Moderation posts: ${allPosts.filter(p => p.expectedModerationTag).length}`);
        
        // Emotion breakdown
        console.log('\n🎭 Emotion distribution:');
        Object.keys(EMOTION_CONTENT).forEach(emotion => {
            const count = allPosts.filter(p => p.expectedEmotion === emotion).length;
            console.log(`   ${emotion}: ${count} posts`);
        });
        
        // Moderation breakdown  
        console.log('\n🛡️ Moderation tag distribution:');
        Object.keys(MODERATION_CONTENT).forEach(tag => {
            const count = allPosts.filter(p => p.expectedModerationTag === tag).length;
            console.log(`   ${tag}: ${count} posts`);
        });
        
    } catch (error) {
        console.error('❌ Seeding failed:', error.message);
        process.exit(1);
    }
}

// Run the seeder
if (require.main === module) {
    main().catch(console.error);
}

module.exports = {
    main,
    createUsers,
    createEmotionPosts, 
    createModerationPosts,
    generateComments,
    addVotingData
};