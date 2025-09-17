// Global state
let currentUser = null;
let authToken = localStorage.getItem('authToken');
let posts = [];

// API Configuration
const API_BASE = '/api';

// Initialize app
document.addEventListener('DOMContentLoaded', function() {
    initializeApp();
    setupEventListeners();
    
    // Debug function accessible from console
    window.debugAuth = function() {
        console.log('=== AUTH DEBUG INFO ===');
        console.log('Auth token:', authToken);
        console.log('Current user:', currentUser);
        console.log('Token in localStorage:', localStorage.getItem('authToken'));
        if (authToken) {
            try {
                const payload = JSON.parse(atob(authToken.split('.')[1]));
                console.log('Token payload:', payload);
                console.log('Token expires:', new Date(payload.exp * 1000));
            } catch (e) {
                console.error('Failed to parse token:', e);
            }
        }
        console.log('Post form visible:', !document.getElementById('postCreator').classList.contains('hidden'));
        console.log('========================');
    };
    
    // Auto-setup function that clears everything and logs in
    window.autoSetup = async function() {
        console.log('🔧 Starting auto setup...');
        
        // Clear everything
        localStorage.clear();
        authToken = null;
        currentUser = null;
        
        // Close any open modals
        document.querySelectorAll('.modal').forEach(modal => {
            modal.classList.add('hidden');
        });
        
        console.log('✅ Cleared browser data');
        
        try {
            // Auto login with test account
            const response = await fetch('/api/auth/login', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    email: 'frontend@test.com',
                    password: 'test123'
                })
            });
            
            const data = await response.json();
            
            if (response.ok) {
                authToken = data.token;
                localStorage.setItem('authToken', authToken);
                currentUser = data.user;
                showUserInterface();
                loadPosts();
                console.log('✅ Auto-logged in successfully!');
                console.log('🎉 Ready to post! The "Share Your Thoughts" form should now be visible.');
                return true;
            } else {
                console.error('❌ Login failed:', data.message);
                return false;
            }
        } catch (error) {
            console.error('❌ Auto setup failed:', error);
            return false;
        }
    };
});

function initializeApp() {
    // Check if user is logged in
    if (authToken) {
        try {
            const payload = JSON.parse(atob(authToken.split('.')[1]));
            currentUser = { username: payload.username, id: payload.user_id };
            showUserInterface();
            loadPosts();
        } catch (e) {
            console.error('Invalid token:', e);
            logout();
        }
    } else {
        showGuestInterface();
        loadPosts();
    }
}

function setupEventListeners() {
    // Auth forms
    document.getElementById('loginForm').addEventListener('submit', handleLogin);
    document.getElementById('registerForm').addEventListener('submit', handleRegister);
    document.getElementById('postForm').addEventListener('submit', handleCreatePost);
    
    // Modal close on background click
    document.querySelectorAll('.modal').forEach(modal => {
        modal.addEventListener('click', function(e) {
            if (e.target === this) {
                closeModal(this.id);
            }
        });
    });
    
    // Auto-preview sentiment while typing
    const titleInput = document.getElementById('postTitle');
    const contentInput = document.getElementById('postContent');
    
    titleInput?.addEventListener('input', previewSentiment);
    contentInput?.addEventListener('input', previewSentiment);
}

// Authentication functions
function showLogin() {
    document.getElementById('loginModal').classList.remove('hidden');
}

function showRegister() {
    document.getElementById('registerModal').classList.remove('hidden');
}

function closeModal(modalId) {
    document.getElementById(modalId).classList.add('hidden');
}

async function handleLogin(e) {
    e.preventDefault();
    
    const email = document.getElementById('loginEmail').value;
    const password = document.getElementById('loginPassword').value;
    
    try {
        const response = await fetch(`${API_BASE}/auth/login`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ email, password })
        });
        
        const data = await response.json();
        
        if (response.ok) {
            authToken = data.token;
            localStorage.setItem('authToken', authToken);
            currentUser = data.user;
            
            showUserInterface();
            closeModal('loginModal');
            showToast('Welcome back!', 'success');
            loadPosts();
        } else {
            showToast(data.message || 'Login failed', 'error');
        }
    } catch (error) {
        console.error('Login error:', error);
        showToast('Login failed. Please try again.', 'error');
    }
}

async function handleRegister(e) {
    e.preventDefault();
    
    const username = document.getElementById('registerUsername').value;
    const email = document.getElementById('registerEmail').value;
    const password = document.getElementById('registerPassword').value;
    
    try {
        const response = await fetch(`${API_BASE}/auth/register`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ username, email, password })
        });
        
        const data = await response.json();
        
        if (response.ok) {
            authToken = data.token;
            localStorage.setItem('authToken', authToken);
            currentUser = data.user;
            
            showUserInterface();
            closeModal('registerModal');
            showToast('Account created successfully!', 'success');
            loadPosts();
        } else {
            showToast(data.message || 'Registration failed', 'error');
        }
    } catch (error) {
        console.error('Registration error:', error);
        showToast('Registration failed. Please try again.', 'error');
    }
}

function logout() {
    authToken = null;
    currentUser = null;
    localStorage.removeItem('authToken');
    showGuestInterface();
    showToast('Logged out successfully', 'info');
}

function showUserInterface() {
    document.getElementById('navActions').classList.add('hidden');
    document.getElementById('navUser').classList.remove('hidden');
    document.getElementById('navUsername').textContent = `Hello, ${currentUser.username}!`;
    document.getElementById('postCreator').classList.remove('hidden');
}

function showGuestInterface() {
    document.getElementById('navActions').classList.remove('hidden');
    document.getElementById('navUser').classList.add('hidden');
    document.getElementById('postCreator').classList.add('hidden');
}

// Post functions
async function handleCreatePost(e) {
    e.preventDefault();
    
    const title = document.getElementById('postTitle').value;
    const content = document.getElementById('postContent').value;
    
    if (!authToken) {
        showToast('Please login to create posts', 'error');
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/posts`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({ title, content })
        });
        
        const data = await response.json();
        
        if (response.ok) {
            document.getElementById('postForm').reset();
            document.getElementById('sentimentPreview').textContent = '';
            showToast('Post created successfully!', 'success');
            loadPosts(); // Refresh posts
        } else {
            showToast(data.message || 'Failed to create post', 'error');
        }
    } catch (error) {
        console.error('Create post error:', error);
        console.error('Error details:', error.message);
        console.error('Auth token present:', !!authToken);
        showToast('Failed to create post. Please try again.', 'error');
    }
}

async function loadPosts() {
    const container = document.getElementById('postsContainer');
    const spinner = document.getElementById('loadingSpinner');
    
    if (spinner) {
        spinner.classList.remove('hidden');
    }
    
    try {
        const response = await fetch(`${API_BASE}/posts`);
        const data = await response.json();
        
        if (response.ok) {
            posts = Array.isArray(data) ? data : data.posts || [];
            renderPosts(posts);
        } else {
            showToast('Failed to load posts', 'error');
            renderEmptyState();
        }
    } catch (error) {
        console.error('Load posts error:', error);
        showToast('Failed to load posts', 'error');
        renderEmptyState();
    } finally {
        if (spinner) {
            spinner.classList.add('hidden');
        }
    }
}

function renderPosts(postsToRender) {
    const container = document.getElementById('postsList');
    
    if (postsToRender.length === 0) {
        renderEmptyState();
        return;
    }
    
    const postsHTML = postsToRender.map(post => {
        const sentimentClass = getSentimentClass(post);
        const sentimentLabel = getSentimentLabel(post);
        const backgroundStyle = getSentimentBackground(post);
        const timeAgo = formatTimeAgo(post.created_at);
        
        return `
            <article class="post-card" style="${backgroundStyle}">
                <div class="post-header">
                    <div>
                        <div class="post-author">${escapeHtml(post.author_username)}</div>
                        <div class="post-time">${timeAgo}</div>
                    </div>
                    ${sentimentLabel ? `<div class="sentiment-badge ${sentimentClass}">${sentimentLabel}</div>` : ''}
                </div>
                <h3 class="post-title">${escapeHtml(post.title)}</h3>
                <p class="post-content">${escapeHtml(post.content)}</p>
                <div class="post-footer">
                    <div>Popularity: ${(post.popularity_score || 1.0).toFixed(1)}</div>
                    <div>${post.comment_count || 0} comments</div>
                </div>
            </article>
        `;
    }).join('');
    
    container.innerHTML = postsHTML;
}

function renderEmptyState() {
    const container = document.getElementById('postsList');
    container.innerHTML = `
        <div style="text-align: center; padding: 3rem; color: #6b7280;">
            <h3>No posts yet</h3>
            <p>Be the first to share something!</p>
        </div>
    `;
}

function getSentimentClass(post) {
    if (!post.sentiment_colors || post.sentiment_colors.length === 0) {
        return 'sentiment-calm';
    }
    
    // Handle combo sentiments (multiple colors)
    if (post.sentiment_colors.length > 1) {
        // Check if first color indicates sarcastic or affectionate combo
        const firstColor = post.sentiment_colors[0];
        if (firstColor === '#7c3aed') {
            return 'sentiment-sarcastic-combo';
        } else if (firstColor === '#ec4899') {
            return 'sentiment-affectionate-combo';
        } else {
            return 'sentiment-combo'; // Generic combo
        }
    }
    
    // Use the first sentiment color to determine class
    const primaryColor = post.sentiment_colors[0];
    
    // Map colors to sentiment classes (updated with new backend colors)
    const colorToClass = {
        '#fbbf24': 'sentiment-happy',      // Bright yellow/gold - Happy
        '#22d3ee': 'sentiment-joy',        // Bright cyan - Joy 😊
        '#f59e0b': 'sentiment-excited',    // Bright orange - Excited  
        '#1e3a8a': 'sentiment-sad',        // Dark blue - Sad
        '#dc2626': 'sentiment-angry',      // Red - Angry
        '#8b5cf6': 'sentiment-confused',   // Light purple - Confused
        '#84cc16': 'sentiment-disgust',    // Lime green - Disgust 🤢
        '#f97316': 'sentiment-surprise',   // Orange - Surprise 😲
        '#374151': 'sentiment-fear',       // Dark grey - Fear
        '#059669': 'sentiment-calm',       // Green - Calm
        '#ec4899': 'sentiment-affection',  // Pink - Affection
        '#7c3aed': 'sentiment-sarcastic'   // Purple - Sarcastic
    };
    
    return colorToClass[primaryColor] || 'sentiment-calm';
}

function getSentimentLabel(post) {
    // Use the actual sentiment detected by our enhanced analysis system
    if (post.sentiment_colors && post.sentiment_colors.length > 0) {
        const sentimentClass = getSentimentClass(post);
        const sentimentType = getSentimentTypeFromClass(sentimentClass);
        
        // For combo sentiments, show special labels with dual emojis
        if (post.sentiment_colors.length > 1) {
            const firstColor = post.sentiment_colors[0];
            if (firstColor === '#7c3aed') {
                return `😏${sentimentType} Sarcastic Combo`;
            } else if (firstColor === '#ec4899') {
                return `💕${sentimentType} Affectionate Combo`;
            } else {
                return `${sentimentType} Combo`;
            }
        }
        
        // Show the actual detected emotion without percentage
        return sentimentType;
    }
    
    // If no sentiment data, show calm instead of neutral
    return '😌 Calm';
}

// New function to handle gradient backgrounds for sarcasm combinations
function getSentimentBackground(post) {
    if (!post.sentiment_colors || post.sentiment_colors.length === 0) {
        return '';
    }
    
    // Handle sarcasm combinations with gradient
    if (post.sentiment_colors.length > 1) {
        const color1 = post.sentiment_colors[0];
        const color2 = post.sentiment_colors[1];
        return `border-left: 4px solid ${color1}; background: linear-gradient(135deg, ${color1}22, ${color2}22);`;
    }
    
    // Single sentiment color
    const color = post.sentiment_colors[0];
    return `border-left: 4px solid ${color}; background: ${color}11;`;
}

function getSentimentTypeFromClass(sentimentClass) {
    const classToEmoji = {
        'sentiment-happy': '😊',
        'sentiment-joy': '😊',             // Joy - distinct from happy
        'sentiment-excited': '🤩',
        'sentiment-sad': '😢',             // Sadness
        'sentiment-angry': '😠',           // Anger
        'sentiment-confused': '🤔',
        'sentiment-fear': '😨',            // Fear
        'sentiment-disgust': '🤢',         // Disgust
        'sentiment-surprise': '😲',        // Surprise
        'sentiment-calm': '😌',
        'sentiment-affection': '💕',
        'sentiment-sarcastic': '😏',
        'sentiment-sarcastic-combo': '😏+'
    };
    
    return classToEmoji[sentimentClass] || '😐';
}

// Sentiment preview while typing
function previewSentiment() {
    const title = document.getElementById('postTitle').value;
    const content = document.getElementById('postContent').value;
    const preview = document.getElementById('sentimentPreview');
    
    const text = (title + ' ' + content).trim();
    
    if (text.length > 10) {
        // Simple client-side sentiment preview (not as accurate as backend)
        const sentiment = predictSentiment(text);
        preview.textContent = `Predicted mood: ${sentiment.emoji} ${sentiment.type} (${sentiment.confidence}% confidence)`;
    } else {
        preview.textContent = '';
    }
}

function predictSentiment(text) {
    const lowerText = text.toLowerCase();
    
    // Simple keyword-based sentiment prediction
    const patterns = {
        happy: ['happy', 'joy', 'great', 'awesome', 'love', 'amazing', 'wonderful', 'excited'],
        sad: ['sad', 'depressed', 'unhappy', 'cry', 'tears', 'disappointed'],
        angry: ['angry', 'mad', 'furious', 'rage', 'hate', 'annoyed'],
        fear: ['scared', 'afraid', 'terrified', 'anxious', 'worried', 'nervous'],
        affection: ['love', 'adore', 'care', 'sweet', 'dear', 'precious'],
        sarcastic: ['sarcastic', 'obviously', 'sure thing', 'yeah right']
    };
    
    let maxScore = 0;
    let predictedType = 'calm';
    
    for (const [type, keywords] of Object.entries(patterns)) {
        let score = 0;
        keywords.forEach(keyword => {
            if (lowerText.includes(keyword)) {
                score += 1;
            }
        });
        
        if (score > maxScore) {
            maxScore = score;
            predictedType = type;
        }
    }
    
    const confidence = Math.min(maxScore * 20 + 50, 95);
    const emojis = {
        happy: '😊', sad: '😢', angry: '😡', 
        fear: '😨', calm: '😌', affection: '💕', sarcastic: '😏'
    };
    
    return {
        type: predictedType,
        emoji: emojis[predictedType],
        confidence: confidence
    };
}

// Feed filtering
function filterFeed(sentiment) {
    // Update active filter button
    document.querySelectorAll('.filter-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    event.target.classList.add('active');
    
    if (sentiment === 'all') {
        renderPosts(posts);
    } else {
        const filtered = posts.filter(post => {
            const sentimentClass = getSentimentClass(post);
            
            // Handle sarcastic combinations
            if (sentiment === 'sarcastic' && sentimentClass === 'sentiment-sarcastic-combo') {
                return true;
            }
            
            return sentimentClass.includes(sentiment);
        });
        renderPosts(filtered);
    }
}

// Utility functions
function formatTimeAgo(dateString) {
    const now = new Date();
    const postDate = new Date(dateString);
    const diffMs = now - postDate;
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    
    return postDate.toLocaleDateString();
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function showToast(message, type = 'info') {
    const toast = document.getElementById('toast');
    const content = document.getElementById('toastContent');
    
    content.textContent = message;
    toast.className = `toast ${type} show`;
    
    setTimeout(() => {
        toast.classList.remove('show');
    }, 4000);
}

// Error handling for API calls
window.addEventListener('unhandledrejection', function(event) {
    console.error('Unhandled promise rejection:', event.reason);
    showToast('Something went wrong. Please try again.', 'error');
});

// Refresh posts periodically (every 30 seconds)
setInterval(() => {
    if (document.visibilityState === 'visible') {
        loadPosts();
    }
}, 30000);