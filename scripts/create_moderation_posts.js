#!/usr/bin/env node

const baseUrl = 'http://127.0.0.1:5000';

// Strong content designed to trigger different moderation categories
const MODERATION_TEST_POSTS = [
    {
        title: "Toxicity Test 1",
        content: "You people are all idiots and this stupid platform is complete garbage trash that nobody should ever use because it's run by morons.",
        expected_tags: ["toxicity", "insult"]
    },
    {
        title: "Toxicity Test 2", 
        content: "This website is absolute shit and everyone who uses it is a complete waste of space and oxygen.",
        expected_tags: ["toxicity", "insult", "obscene"]
    },
    {
        title: "Severe Toxicity Test 1",
        content: "All of you losers should just disappear and never come back to this pathetic excuse for a platform.",
        expected_tags: ["severe_toxicity", "insult"]
    },
    {
        title: "Severe Toxicity Test 2",
        content: "This place is filled with the most worthless human beings on the planet who contribute nothing to society.",
        expected_tags: ["severe_toxicity", "insult"]
    },
    {
        title: "Obscene Test 1",
        content: "What the hell is wrong with all these damn idiots posting this bullshit crap all over this fucking terrible site.",
        expected_tags: ["obscene", "toxicity"]
    },
    {
        title: "Obscene Test 2",
        content: "This damn platform is so fucking stupid and all this shit content makes me want to puke.",
        expected_tags: ["obscene", "toxicity"]
    },
    {
        title: "Threat Test 1",
        content: "Someone needs to shut this place down before something bad happens to everyone involved.",
        expected_tags: ["threat"]
    },
    {
        title: "Threat Test 2",
        content: "This platform needs to be destroyed and everyone responsible should pay the consequences.",
        expected_tags: ["threat"]
    },
    {
        title: "Insult Test 1",
        content: "You are all pathetic losers who have no life and spend all day on this worthless piece of junk platform.",
        expected_tags: ["insult"]
    },
    {
        title: "Insult Test 2",
        content: "The users here are complete morons and the developers are incompetent fools who built this terrible system.",
        expected_tags: ["insult"]
    }
];

// Test users (these should already exist from the previous seeding)
const TEST_USERS = [
    { username: 'alexchen2024', email: 'alex.chen@example.com', password: 'securepass123' },
    { username: 'mariagarcia', email: 'maria.garcia@example.com', password: 'strongpass456' },
    { username: 'davidjohnson', email: 'david.johnson@example.com', password: 'mypassword789' },
    { username: 'sarahwilson', email: 'sarah.wilson@example.com', password: 'password321' },
    { username: 'mikebrown', email: 'mike.brown@example.com', password: 'brownie654' }
];

async function makeRequest(method, endpoint, data = null, token = null) {
    const url = `${baseUrl}${endpoint}`;
    const options = {
        method,
        headers: {
            'Content-Type': 'application/json',
        },
    };

    if (token) {
        options.headers['Authorization'] = `Bearer ${token}`;
    }

    if (data) {
        options.body = JSON.stringify(data);
    }

    const response = await fetch(url, options);
    
    if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Request failed: ${method} ${endpoint} { error: '${errorText}', status: ${response.status} }`);
    }

    return response.json();
}

async function getUserTokens() {
    console.log('🔑 Getting user tokens...');
    const tokens = new Map();
    
    for (const userData of TEST_USERS) {
        try {
            const loginResponse = await makeRequest('POST', '/api/auth/login', {
                email: userData.email,
                password: userData.password
            });
            
            if (loginResponse.token) {
                tokens.set(userData.username, loginResponse.token);
                console.log(`✅ Got token for: ${userData.username}`);
            }
        } catch (error) {
            console.error(`❌ Failed to get token for ${userData.username}:`, error.message);
        }
    }
    
    console.log(`✅ Total tokens available: ${tokens.size}`);
    return tokens;
}

async function createModerationPosts() {
    console.log('🛡️ Creating content moderation test posts...');
    
    const tokens = await getUserTokens();
    const usernames = Array.from(tokens.keys());
    
    if (usernames.length === 0) {
        console.error('❌ No valid user tokens available. Exiting.');
        return;
    }
    
    let createdPosts = 0;
    let postsWithTags = 0;
    
    for (let i = 0; i < MODERATION_TEST_POSTS.length; i++) {
        const postData = MODERATION_TEST_POSTS[i];
        const username = usernames[i % usernames.length];
        const token = tokens.get(username);
        
        try {
            console.log(`\n📝 Creating post: "${postData.title}" by ${username}`);
            console.log(`   📄 Content: "${postData.content.substring(0, 60)}..."`);
            console.log(`   🎯 Expected tags: [${postData.expected_tags.join(', ')}]`);
            
            const response = await makeRequest('POST', '/api/posts', {
                title: postData.title,
                content: postData.content
            }, token);
            
            createdPosts++;
            console.log(`✅ Post created successfully`);
            console.log(`   🆔 ID: ${response.id}`);
            console.log(`   🎭 Sentiment: ${response.sentiment_type}`);
            
            if (response.toxicity_tags && response.toxicity_tags.length > 0) {
                postsWithTags++;
                console.log(`   🏷️ Toxicity tags: [${response.toxicity_tags.join(', ')}]`);
                
                // Check if we got the expected tags
                const expectedTags = postData.expected_tags;
                const actualTags = response.toxicity_tags;
                const matchedTags = expectedTags.filter(tag => actualTags.includes(tag));
                
                if (matchedTags.length > 0) {
                    console.log(`   ✅ Matched expected tags: [${matchedTags.join(', ')}]`);
                } else {
                    console.log(`   ⚠️ No expected tags matched. Expected: [${expectedTags.join(', ')}], Got: [${actualTags.join(', ')}]`);
                }
            } else {
                console.log(`   ⚠️ No toxicity tags applied (content may be below thresholds)`);
            }
            
            // Add a small delay to avoid overwhelming the server
            await new Promise(resolve => setTimeout(resolve, 500));
            
        } catch (error) {
            console.error(`❌ Failed to create post "${postData.title}":`, error.message);
        }
    }
    
    console.log(`\n🎉 MODERATION TESTING COMPLETE!`);
    console.log(`📊 Results:`);
    console.log(`   📝 Posts created: ${createdPosts}/${MODERATION_TEST_POSTS.length}`);
    console.log(`   🏷️ Posts with toxicity tags: ${postsWithTags}`);
    console.log(`   📈 Tag success rate: ${postsWithTags > 0 ? ((postsWithTags / createdPosts) * 100).toFixed(1) : 0}%`);
    
    if (postsWithTags === 0) {
        console.log(`\n⚠️ WARNING: No posts triggered toxicity tags!`);
        console.log(`   💡 This may indicate:`);
        console.log(`      • Content is not strong enough for current thresholds`);
        console.log(`      • Moderation system may need threshold adjustment`);
        console.log(`      • AI model may be more conservative than expected`);
    } else {
        console.log(`\n✅ SUCCESS: Moderation system is working and detecting toxic content!`);
    }
}

// Run the script
createModerationPosts().catch(console.error);