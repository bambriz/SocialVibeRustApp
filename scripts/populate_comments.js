#!/usr/bin/env node

/**
 * Social Pulse Comment Population Script
 * 
 * Creates sample comments from existing users on existing posts.
 * Includes both direct comments and replies to comments.
 */

const API_BASE = 'http://localhost:5000/api';

// Sample comment templates
const commentTemplates = [
    "This is so true! Thanks for sharing.",
    "I completely agree with this perspective.",
    "Love this! Made my day brighter.",
    "Such beautiful thoughts, thank you for posting.",
    "This resonates with me deeply.",
    "Couldn't have said it better myself!",
    "Inspiring words, really needed this today.",
    "What a wonderful way to look at things.",
    "This is exactly what I needed to hear.",
    "Thank you for the positive vibes!",
    "Amazing insight, very thought-provoking.",
    "This brought a smile to my face.",
    "So relatable, thank you for sharing!",
    "Beautiful sentiment, love this post.",
    "This is why I love this community!",
    "Wise words that really hit home.",
    "Such a refreshing perspective!",
    "This made me think about things differently.",
    "Perfectly said, couldn't agree more.",
    "Thank you for the inspiration!"
];

// User credentials for the 12 users we created
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

// Utility functions
function getRandomElement(array) {
    return array[Math.floor(Math.random() * array.length)];
}

function shuffleArray(array) {
    const shuffled = [...array];
    for (let i = shuffled.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
    }
    return shuffled;
}

async function makeRequest(url, options = {}) {
    try {
        const response = await fetch(url, options);
        
        // Check if response is JSON
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

async function getAllPosts() {
    const result = await makeRequest(`${API_BASE}/posts`);
    
    if (result.success) {
        return result.data.posts || result.data;
    } else {
        console.error(`❌ Failed to fetch posts:`, result.error);
        return [];
    }
}

async function getPostComments(postId) {
    const result = await makeRequest(`${API_BASE}/posts/${postId}/comments`);
    
    if (result.success) {
        return result.data || [];
    } else {
        console.error(`❌ Failed to fetch comments for post ${postId}:`, result.error);
        return [];
    }
}

async function createComment(token, postId, content, parentId = null) {
    const commentData = {
        post_id: postId,
        content: content
    };
    
    if (parentId) {
        commentData.parent_id = parentId;
    }
    
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

async function populateComments() {
    console.log('🎯 Social Pulse Comment Population Script');
    console.log('=========================================');
    
    // Get all available posts
    const allPosts = await getAllPosts();
    if (allPosts.length === 0) {
        console.log('❌ No posts available for commenting');
        return;
    }
    
    console.log(`📊 Found ${allPosts.length} posts to comment on`);
    
    let totalComments = 0;
    const shuffledCommentTemplates = shuffleArray(commentTemplates);
    const allCreatedComments = [];
    
    console.log('\n💬 Creating comments...');
    
    for (let i = 0; i < users.length; i++) {
        const user = users[i];
        
        // Login user
        const loginResult = await loginUser(user.email, user.password);
        if (!loginResult) {
            console.log(`❌ Failed to login ${user.email} for commenting`);
            continue;
        }
        
        console.log(`💭 ${loginResult.user.username} adding comments...`);
        
        // Each user makes 4 comments
        const userComments = [];
        for (let j = 0; j < 4; j++) {
            // Pick a random post to comment on
            const post = getRandomElement(allPosts);
            const commentText = shuffledCommentTemplates[(i * 4 + j) % shuffledCommentTemplates.length];
            
            let parentId = null;
            
            // For the last comment, try to reply to an existing comment if available
            if (j === 3 && allCreatedComments.length > 0) {
                const randomComment = getRandomElement(allCreatedComments);
                // Only reply to comments on the same post
                if (randomComment.post_id === post.id) {
                    parentId = randomComment.id;
                    console.log(`  💬 Replying to comment by ${randomComment.author_username || 'unknown'}`);
                }
            }
            
            const comment = await createComment(loginResult.token, post.id, commentText, parentId);
            if (comment) {
                userComments.push(comment);
                allCreatedComments.push(comment);
                totalComments++;
                const commentType = parentId ? 'reply' : 'comment';
                console.log(`  ✅ Added ${commentType} on "${post.title.substring(0, 30)}..."`);
            }
            
            // Small delay between comments
            await new Promise(resolve => setTimeout(resolve, 300));
        }
    }
    
    console.log(`\n🎉 Comment population completed!`);
    console.log(`📊 Summary:`);
    console.log(`   💬 Total comments created: ${totalComments}`);
    console.log(`   📝 Comments per user: ~4`);
    console.log(`   🔗 Includes replies to existing comments`);
    console.log('\n✨ Your Social Pulse feed is now fully populated with engaging conversations!');
}

// Run the script
if (require.main === module) {
    populateComments().catch(console.error);
}

module.exports = {
    populateComments,
    commentTemplates,
    users
};