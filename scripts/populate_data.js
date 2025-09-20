#!/usr/bin/env node

/**
 * Social Pulse Data Population Script
 * 
 * Creates sample users, posts, and comments for testing and demonstration.
 * Saves all data to the database via the API endpoints.
 */

const API_BASE = 'http://localhost:5000/api';

// Sample user data
const users = [
    { username: 'techguru', email: 'tech@example.com', password: 'test123' },
    { username: 'artlover', email: 'art@example.com', password: 'test123' },
    { username: 'musicfan', email: 'music@example.com', password: 'test123' },
    { username: 'naturewalk', email: 'nature@example.com', password: 'test123' },
    { username: 'bookworm', email: 'book@example.com', password: 'test123' },
    { username: 'foodie123', email: 'food@example.com', password: 'test123' },
    { username: 'traveler', email: 'travel@example.com', password: 'test123' },
    { username: 'gamer_pro', email: 'gaming@example.com', password: 'test123' },
    { username: 'fitnessguru', email: 'fitness@example.com', password: 'test123' },
    { username: 'moviebuff', email: 'movies@example.com', password: 'test123' },
    { username: 'coder_life', email: 'coding@example.com', password: 'test123' },
    { username: 'pet_parent', email: 'pets@example.com', password: 'test123' }
];

// Sample post templates
const postTemplates = [
    { title: "Amazing sunset today", content: "Watched the most beautiful sunset from my window. Nature never fails to amaze me!" },
    { title: "Coffee thoughts", content: "There's something magical about the first cup of coffee in the morning. It sets the tone for the entire day." },
    { title: "Weekend vibes", content: "Finally time to relax and unwind. Planning to read a good book and maybe take a long walk." },
    { title: "New project excitement", content: "Just started working on something incredible! Can't wait to share more details soon." },
    { title: "Rainy day reflections", content: "Rain always makes me contemplative. There's beauty in quiet moments like these." },
    { title: "Friendship appreciation", content: "Grateful for the amazing people in my life. Good friends make everything better!" },
    { title: "Learning something new", content: "Picked up a new skill today and feeling accomplished. It's never too late to learn!" },
    { title: "Music discovery", content: "Found an incredible new song that's been on repeat all day. Music truly feeds the soul." },
    { title: "Food adventure", content: "Tried a new recipe today and it turned out amazing! Cooking is such a therapeutic activity." },
    { title: "Technology thoughts", content: "The pace of technological advancement is mind-blowing. Excited to see what the future holds!" },
    { title: "Morning motivation", content: "Starting the day with positive energy and clear goals. Ready to make things happen!" },
    { title: "Nature walk discovery", content: "Found a hidden trail today with the most peaceful scenery. Sometimes the best things are unexpected." },
    { title: "Creative inspiration", content: "Feeling incredibly inspired today. Time to channel this energy into something meaningful!" },
    { title: "Seasonal changes", content: "Love how each season brings its own unique beauty and energy. Change can be refreshing." },
    { title: "Community spirit", content: "Witnessed an amazing act of kindness today. Humanity at its finest!" },
    { title: "Personal growth", content: "Reflecting on how much I've grown this year. Every challenge was a stepping stone." },
    { title: "Simple pleasures", content: "Sometimes it's the small things that bring the most joy. Appreciating life's simple pleasures." },
    { title: "Dream big thoughts", content: "Been thinking about future goals and dreams. The possibilities are endless!" },
    { title: "Gratitude moment", content: "Taking a moment to appreciate all the good things in life. Gratitude changes everything." },
    { title: "Adventure calling", content: "Planning my next adventure! Life is too short not to explore and experience new things." }
];

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

async function registerUser(user) {
    console.log(`📝 Registering user: ${user.username}`);
    
    const result = await makeRequest(`${API_BASE}/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(user)
    });
    
    if (result.success) {
        console.log(`✅ Registered: ${user.username}`);
        return result.data;
    } else {
        console.log(`⚠️  Registration failed for ${user.username}: ${result.error.message || 'Unknown error'}`);
        return null;
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

async function populateUsers() {
    console.log('\n🚀 Starting user registration...');
    const registeredUsers = [];
    
    for (const user of users) {
        const registered = await registerUser(user);
        if (registered) {
            registeredUsers.push({ ...user, registered });
        }
        // Small delay to avoid overwhelming the server
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    console.log(`✅ Registered ${registeredUsers.length} users`);
    return registeredUsers;
}

async function populatePosts(registeredUsers) {
    console.log('\n📝 Creating posts for each user...');
    const createdPosts = [];
    const shuffledTemplates = shuffleArray(postTemplates);
    
    for (let i = 0; i < registeredUsers.length; i++) {
        const user = registeredUsers[i];
        
        // Login user
        const loginResult = await loginUser(user.email, user.password);
        if (!loginResult) {
            console.log(`❌ Failed to login ${user.username}, skipping posts`);
            continue;
        }
        
        console.log(`👤 Creating posts for ${user.username}...`);
        
        // Create 3 posts per user
        for (let j = 0; j < 3; j++) {
            const templateIndex = (i * 3 + j) % shuffledTemplates.length;
            const template = shuffledTemplates[templateIndex];
            
            const post = await createPost(loginResult.token, template);
            if (post) {
                createdPosts.push({
                    ...post,
                    author: user.username,
                    authorToken: loginResult.token,
                    authorUser: loginResult.user
                });
                console.log(`  ✅ Created: "${template.title}"`);
            }
            
            // Small delay between posts
            await new Promise(resolve => setTimeout(resolve, 200));
        }
    }
    
    console.log(`✅ Created ${createdPosts.length} posts`);
    return createdPosts;
}

async function populateComments(registeredUsers) {
    console.log('\n💬 Creating comments...');
    
    // Get all available posts
    const allPosts = await getAllPosts();
    if (allPosts.length === 0) {
        console.log('❌ No posts available for commenting');
        return;
    }
    
    console.log(`📊 Found ${allPosts.length} posts to comment on`);
    
    let totalComments = 0;
    const shuffledCommentTemplates = shuffleArray(commentTemplates);
    
    for (let i = 0; i < registeredUsers.length; i++) {
        const user = registeredUsers[i];
        
        // Login user
        const loginResult = await loginUser(user.email, user.password);
        if (!loginResult) {
            console.log(`❌ Failed to login ${user.username} for commenting`);
            continue;
        }
        
        console.log(`💭 ${user.username} adding comments...`);
        
        // Each user makes 4 comments
        const userComments = [];
        for (let j = 0; j < 4; j++) {
            // Pick a random post to comment on
            const post = getRandomElement(allPosts);
            const commentText = shuffledCommentTemplates[(i * 4 + j) % shuffledCommentTemplates.length];
            
            let parentId = null;
            
            // For at least one comment, try to reply to an existing comment
            if (j === 0 && userComments.length === 0) {
                const existingComments = await getPostComments(post.id);
                if (existingComments.length > 0) {
                    const randomComment = getRandomElement(existingComments);
                    parentId = randomComment.comment ? randomComment.comment.id : randomComment.id;
                    console.log(`  💬 Replying to existing comment`);
                }
            }
            
            const comment = await createComment(loginResult.token, post.id, commentText, parentId);
            if (comment) {
                userComments.push(comment);
                totalComments++;
                const commentType = parentId ? 'reply' : 'comment';
                console.log(`  ✅ Added ${commentType} on "${post.title.substring(0, 30)}..."`);
            }
            
            // Small delay between comments
            await new Promise(resolve => setTimeout(resolve, 150));
        }
    }
    
    console.log(`✅ Created ${totalComments} comments total`);
}

async function main() {
    console.log('🎯 Social Pulse Data Population Script');
    console.log('=====================================');
    
    try {
        // Step 1: Register users
        const registeredUsers = await populateUsers();
        
        if (registeredUsers.length === 0) {
            console.log('❌ No users registered, stopping script');
            return;
        }
        
        // Step 2: Create posts
        const createdPosts = await populatePosts(registeredUsers);
        
        // Step 3: Create comments
        await populateComments(registeredUsers);
        
        console.log('\n🎉 Data population completed!');
        console.log(`📊 Summary:`);
        console.log(`   👥 Users: ${registeredUsers.length}`);
        console.log(`   📝 Posts: ${createdPosts.length}`);
        console.log(`   💬 Comments: Check database for final count`);
        console.log('\n✨ Your Social Pulse feed is now populated with engaging content!');
        
    } catch (error) {
        console.error('❌ Fatal error during population:', error);
        process.exit(1);
    }
}

// Run the script
if (require.main === module) {
    main().catch(console.error);
}

module.exports = {
    users,
    postTemplates,
    commentTemplates,
    populateUsers,
    populatePosts,
    populateComments,
    main
};