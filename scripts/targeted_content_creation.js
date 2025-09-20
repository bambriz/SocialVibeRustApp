#!/usr/bin/env node

/**
 * Targeted Content Creation for Social Pulse
 * 
 * Creates specific posts and comments to ensure proper representation of:
 * - All emotion categories (at least 2 posts each)
 * - All content tags (at least 1 post each)
 * - Comments with the same distribution
 */

const API_BASE = 'http://localhost:5000/api';

// Existing users (from previous population)
const users = [
    { email: 'tech@example.com', password: 'test123' },
    { email: 'art@example.com', password: 'test123' },
    { email: 'music@example.com', password: 'test123' },
    { email: 'nature@example.com', password: 'test123' },
    { email: 'book@example.com', password: 'test123' },
    { email: 'food@example.com', password: 'test123' },
    { email: 'travel@example.com', password: 'test123' },
    { email: 'gaming@example.com', password: 'test123' },
    { email: 'fitness@example.com', password: 'test123' },
    { email: 'movies@example.com', password: 'test123' },
    { email: 'coding@example.com', password: 'test123' },
    { email: 'pets@example.com', password: 'test123' }
];

// Content specifically designed to trigger different emotions
const emotionContent = {
    sad: [
        { title: "Missing my grandmother", content: "Today marks one year since my grandmother passed away. I miss her wisdom and warm hugs so much." },
        { title: "Lost job today", content: "After five years at my company, they let me go due to budget cuts. Feeling lost and uncertain about the future." }
    ],
    angry: [
        { title: "Traffic makes me furious", content: "Stuck in traffic for two hours because of poor city planning. This is absolutely infuriating and unacceptable!" },
        { title: "Broken promises again", content: "Politicians keep making promises they never intend to keep. This dishonesty makes me so angry and frustrated." }
    ],
    fear: [
        { title: "Worried about climate change", content: "The latest climate reports are terrifying. I'm genuinely scared about what kind of world we're leaving for future generations." },
        { title: "Health anxiety", content: "Been having chest pains and I'm terrified it might be something serious. Medical appointments make me so anxious." }
    ],
    disgust: [
        { title: "Revolting behavior", content: "Witnessed someone littering right in front of a trash can today. Such disgusting and disrespectful behavior towards our environment." },
        { title: "Food poisoning nightmare", content: "That restaurant served me spoiled seafood. The smell and taste were absolutely revolting and made me sick for days." }
    ],
    surprise: [
        { title: "Unexpected good news", content: "Just found out I got the scholarship I applied for! I'm completely shocked and amazed - never expected this!" },
        { title: "Surprise birthday party", content: "My friends threw me the most incredible surprise party. I was absolutely stunned when I walked in!" }
    ],
    confused: [
        { title: "Technology confuses me", content: "Trying to understand cryptocurrency and blockchain technology. The more I read, the more bewildered and perplexed I become." },
        { title: "Mixed signals everywhere", content: "Getting conflicting advice from different experts about the same topic. I'm completely baffled and don't know what to believe." }
    ],
    sarcastic: [
        { title: "Oh wonderful weather", content: "Oh great, another rainy day. Just perfect weather for my outdoor wedding plans. Absolutely fantastic timing, Mother Nature." },
        { title: "Amazing customer service", content: "Waited on hold for three hours only to be disconnected. What outstanding customer service! Truly exceptional experience." }
    ]
};

// Content designed to trigger specific toxicity tags (mild enough to be tagged but not blocked)
const toxicityContent = {
    toxicity: [
        { title: "Criticism of poor service", content: "This company's service is absolutely terrible and their staff are completely incompetent at their jobs." }
    ],
    insult: [
        { title: "Frustrated with behavior", content: "Some people are just thoughtless and inconsiderate. Their behavior is really stupid and selfish." }
    ],
    crude: [
        { title: "Crude humor attempt", content: "That comedian's jokes were pretty crude and inappropriate for a family show. Not my cup of tea." }
    ],
    threat: [
        { title: "Competitive gaming", content: "I'm going to absolutely destroy you in this video game match. Prepare to get completely wrecked!" }
    ],
    obscene: [
        { title: "Mildly inappropriate", content: "That movie had some pretty obscene scenes that weren't necessary for the plot. Bit too much for my taste." }
    ]
};

// Comment templates for different emotions/content
const emotionComments = {
    sad: [
        "I'm so sorry for your loss. Sending you love and support.",
        "This makes me feel really sad too. Hope things get better soon."
    ],
    angry: [
        "This situation is absolutely infuriating! I totally understand your anger.",
        "So frustrated reading this. Something needs to change!"
    ],
    fear: [
        "This is genuinely scary. I worry about the same things.",
        "Your concerns are completely valid. These are frightening times."
    ],
    disgust: [
        "That behavior is absolutely revolting and unacceptable.",
        "How disgusting! People like that make me sick."
    ],
    surprise: [
        "Wow, what an amazing surprise! So happy for you!",
        "That's incredible news! I'm completely shocked and thrilled!"
    ],
    confused: [
        "I'm just as baffled about this as you are. Very confusing topic.",
        "This whole situation is completely perplexing to me too."
    ],
    sarcastic: [
        "Oh sure, that sounds absolutely delightful. What a treat.",
        "Yeah right, like that's ever going to work. Brilliant plan."
    ],
    joy: [
        "This makes me so incredibly happy! What wonderful news!",
        "Reading this brought such joy to my day. Thank you for sharing!"
    ]
};

// Utility functions
function getRandomElement(array) {
    return array[Math.floor(Math.random() * array.length)];
}

async function makeRequest(url, options = {}) {
    try {
        const response = await fetch(url, options);
        
        const contentType = response.headers.get('content-type');
        if (!contentType || !contentType.includes('application/json')) {
            console.error(`❌ Non-JSON response: ${url} - ${response.status} ${response.statusText}`);
            const text = await response.text();
            return { success: false, error: `Non-JSON response: ${text.substring(0, 100)}` };
        }
        
        const data = await response.json();
        
        if (!response.ok) {
            console.error(`❌ Request failed: ${url}`, data);
            return { success: false, error: data };
        }
        
        return { success: true, data };
    } catch (error) {
        console.error(`❌ Network error: ${url}`, error.message);
        return { success: false, error: error.message };
    }
}

async function loginUser(email, password) {
    const result = await makeRequest(`${API_BASE}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password })
    });
    
    if (result.success) {
        return { token: result.data.token, user: result.data.user };
    } else {
        console.error(`❌ Login failed for ${email}:`, result.error);
        return null;
    }
}

async function createPost(token, post) {
    const result = await makeRequest(`${API_BASE}/posts`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify(post)
    });
    
    if (result.success) {
        return result.data.post || result.data;
    } else {
        console.error(`❌ Post creation failed:`, result.error);
        return null;
    }
}

async function getAllPosts() {
    const result = await makeRequest(`${API_BASE}/posts`);
    
    if (result.success) {
        return result.data.posts || result.data;
    } else {
        console.error(`❌ Failed to fetch posts:`, result.error);
        return [];
    }
}

async function createComment(token, postId, content) {
    const commentData = {
        post_id: postId,
        content: content
    };
    
    const result = await makeRequest(`${API_BASE}/posts/${postId}/comments`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify(commentData)
    });
    
    if (result.success) {
        return result.data.comment || result.data;
    } else {
        console.error(`❌ Comment creation failed:`, result.error);
        return null;
    }
}

async function createTargetedPosts() {
    console.log('\n📝 Creating posts for missing emotions...');
    
    let userIndex = 0;
    const createdPosts = [];
    
    // Create posts for each missing emotion (2 posts per emotion)
    for (const [emotion, posts] of Object.entries(emotionContent)) {
        console.log(`\n🎭 Creating posts for emotion: ${emotion}`);
        
        for (const postContent of posts) {
            const user = users[userIndex % users.length];
            userIndex++;
            
            const loginResult = await loginUser(user.email, user.password);
            if (!loginResult) {
                console.log(`❌ Failed to login ${user.email}`);
                continue;
            }
            
            const post = await createPost(loginResult.token, postContent);
            if (post) {
                createdPosts.push({ ...post, emotion, author: loginResult.user.username });
                console.log(`  ✅ Created ${emotion} post: "${postContent.title}"`);
            }
            
            await new Promise(resolve => setTimeout(resolve, 200));
        }
    }
    
    // Create posts for toxicity content
    console.log('\n🏷️ Creating posts for content tags...');
    
    for (const [tag, posts] of Object.entries(toxicityContent)) {
        for (const postContent of posts) {
            const user = users[userIndex % users.length];
            userIndex++;
            
            const loginResult = await loginUser(user.email, user.password);
            if (!loginResult) {
                console.log(`❌ Failed to login ${user.email}`);
                continue;
            }
            
            const post = await createPost(loginResult.token, postContent);
            if (post) {
                createdPosts.push({ ...post, contentTag: tag, author: loginResult.user.username });
                console.log(`  ✅ Created ${tag} tagged post: "${postContent.title}"`);
            }
            
            await new Promise(resolve => setTimeout(resolve, 200));
        }
    }
    
    return createdPosts;
}

async function createTargetedComments() {
    console.log('\n💬 Creating comments for all emotions...');
    
    const allPosts = await getAllPosts();
    if (allPosts.length === 0) {
        console.log('❌ No posts available for commenting');
        return;
    }
    
    let userIndex = 0;
    let totalComments = 0;
    
    // Create comments for each emotion type (2 comments per emotion)
    for (const [emotion, commentTexts] of Object.entries(emotionComments)) {
        console.log(`\n💭 Creating ${emotion} comments...`);
        
        for (const commentText of commentTexts) {
            const user = users[userIndex % users.length];
            userIndex++;
            
            const loginResult = await loginUser(user.email, user.password);
            if (!loginResult) {
                console.log(`❌ Failed to login ${user.email}`);
                continue;
            }
            
            const randomPost = getRandomElement(allPosts);
            const comment = await createComment(loginResult.token, randomPost.id, commentText);
            
            if (comment) {
                totalComments++;
                console.log(`  ✅ Created ${emotion} comment by ${loginResult.user.username}`);
            }
            
            await new Promise(resolve => setTimeout(resolve, 300));
        }
    }
    
    return totalComments;
}

async function main() {
    console.log('🎯 Targeted Content Creation for Social Pulse');
    console.log('===========================================');
    
    try {
        // Create targeted posts for missing emotions and content tags
        const createdPosts = await createTargetedPosts();
        
        // Create targeted comments for all emotions
        const commentCount = await createTargetedComments();
        
        console.log('\n🎉 Targeted content creation completed!');
        console.log(`📊 Summary:`);
        console.log(`   📝 New posts created: ${createdPosts.length}`);
        console.log(`   💬 Comments created: ${commentCount}`);
        console.log(`   🎭 Emotions covered: sad, angry, fear, disgust, surprise, confused, sarcastic`);
        console.log(`   🏷️ Content tags covered: toxicity, insult, crude, threat, obscene`);
        console.log('\n✨ Your Social Pulse feed now has comprehensive emotion and content representation!');
        
    } catch (error) {
        console.error('❌ Fatal error during targeted content creation:', error);
        process.exit(1);
    }
}

// Run the script
if (require.main === module) {
    main().catch(console.error);
}

module.exports = {
    emotionContent,
    toxicityContent,
    emotionComments,
    createTargetedPosts,
    createTargetedComments,
    main
};