// Global state
let currentUser = null;
let authToken = localStorage.getItem('authToken');
let posts = [];

// Content filter state - tracks which toxicity types should be hidden
let contentFilters = {
    hiddenToxicityTypes: new Set() // Empty set means show all
};

// API Configuration
const API_BASE = '/api';

// Initialize app
document.addEventListener('DOMContentLoaded', function() {
    loadContentFilterState(); // Load saved filter preferences first
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
            console.log('Error details:', data);
            
            // Check if this is a content moderation error
            if (data.error_type === 'content_moderation') {
                // Show specific violation reason for content moderation errors
                showToast(data.error || 'Post blocked due to content violation', 'error');
            } else {
                // Show generic message for other errors
                showToast('Failed to create post. Please try again.', 'error');
            }
        }
    } catch (error) {
        console.error('Create post error:', error);
        console.error('Error details:', error.message);
        console.error('Auth token present:', !!authToken);
        
        // For network errors or other exceptions, show generic message
        showToast('Failed to create post. Please check your connection and try again.', 'error');
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
            
            // Apply active filters consistently on reload
            const activeFilterBtn = document.querySelector('.filter-btn.active');
            if (activeFilterBtn) {
                const sentiment = activeFilterBtn.dataset.filter;
                filterFeed(sentiment); // This applies both emotion and content filters
            } else {
                // If no active emotion filter, just apply content filters
                renderPosts(applyContentFiltering(posts));
            }
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
        
        // Get toxicity tags for this post
        const toxicityTags = getToxicityTags(post);
        const toxicityTagsHTML = renderToxicityTags(toxicityTags);
        
        return `
            <article class="post-card" style="${backgroundStyle}">
                <div class="post-header">
                    <div class="post-author-section">
                        <div class="post-author">${escapeHtml(post.author_username)}</div>
                        <div class="post-time">${timeAgo}</div>
                    </div>
                    <div class="post-badges">
                        ${sentimentLabel ? `<div class="sentiment-badge ${sentimentClass}">${sentimentLabel}</div>` : ''}
                    </div>
                </div>
                <h3 class="post-title">${escapeHtml(post.title)}</h3>
                <p class="post-content">${escapeHtml(post.content)}</p>
                ${toxicityTagsHTML}
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
        return 'sentiment-neutral';
    }
    
    // Use the first sentiment color to determine class
    const primaryColor = post.sentiment_colors[0];
    
    // Map colors to sentiment classes (updated with new backend colors)
    const colorToClass = {
        '#fbbf24': 'sentiment-joy',        // Bright yellow/gold - Joy 😊 (matches backend)
        '#1e3a8a': 'sentiment-sad',        // Dark blue - Sad
        '#dc2626': 'sentiment-angry',      // Red - Angry
        '#a16207': 'sentiment-confused',   // Brown/amber - Confused
        '#84cc16': 'sentiment-disgust',    // Lime green - Disgust 🤢
        '#f97316': 'sentiment-surprise',   // Orange - Surprise 😲
        '#374151': 'sentiment-fear',       // Dark grey - Fear
        '#6b7280': 'sentiment-neutral',    // Neutral gray - Neutral (matches backend)
        '#ec4899': 'sentiment-affection',  // Pink - Affection
        '#7c3aed': 'sentiment-sarcastic'   // Purple - Sarcastic
    };
    
    return colorToClass[primaryColor] || 'sentiment-neutral';
}

function getSentimentLabel(post) {
    // Use the actual sentiment detected by our enhanced analysis system
    if (post.sentiment_colors && post.sentiment_colors.length > 0) {
        // Show the emoji for the detected emotion (no more combo logic)
        const sentimentClass = getSentimentClass(post);
        const sentimentType = getSentimentTypeFromClass(sentimentClass);
        return sentimentType;
    }
    
    // If no sentiment data, show neutral
    return '😐 Neutral';
}

// New function to get toxicity tags for display
function getToxicityTags(post) {
    if (!post.toxicity_tags || post.toxicity_tags.length === 0) {
        return [];
    }
    
    // Map toxicity categories to display info
    const toxicityMap = {
        'toxicity': { emoji: '⚠️', label: 'Toxic', color: '#f59e0b' },
        'super_toxic': { emoji: '🚨', label: 'Super Toxic', color: '#dc2626' },
        'obscene': { emoji: '🚫', label: 'Obscene', color: '#7c2d12' },
        'threat': { emoji: '⚡', label: 'Threatening', color: '#991b1b' },
        'threatening': { emoji: '⚡', label: 'Threatening', color: '#991b1b' },
        'insult': { emoji: '💢', label: 'Insulting', color: '#c2410c' },
        'insulting': { emoji: '💢', label: 'Insulting', color: '#c2410c' },
        'identity_attack': { emoji: '🛡️', label: 'Identity Attack', color: '#7f1d1d' },
        'severe_toxicity': { emoji: '💀', label: 'Severe', color: '#450a0a' }
    };
    
    return post.toxicity_tags.map(tag => {
        const normalized = tag.toLowerCase().replace(/\s+/g, '_');
        const config = toxicityMap[normalized] || { 
            emoji: '⚠️', 
            label: tag, 
            color: '#6b7280' 
        };
        
        return {
            tag: normalized,
            emoji: config.emoji,
            label: config.label,
            color: config.color,
            displayText: `${config.emoji} ${config.label}`
        };
    });
}

// Function to render toxicity tags HTML
function renderToxicityTags(toxicityTags) {
    if (toxicityTags.length === 0) {
        return '';
    }
    
    const tagsHTML = toxicityTags.map(tag => 
        `<span class="toxicity-tag" style="background-color: ${tag.color}20; border: 1px solid ${tag.color}60; color: ${tag.color}">
            ${tag.displayText}
        </span>`
    ).join('');
    
    return `<div class="toxicity-tags-container">${tagsHTML}</div>`;
}

// Function to handle single color backgrounds (no more gradients)
function getSentimentBackground(post) {
    if (!post.sentiment_colors || post.sentiment_colors.length === 0) {
        return '';
    }
    
    // Use single sentiment color (first color if multiple exist)
    const color = post.sentiment_colors[0];
    return `border-left: 4px solid ${color}; background: ${color}11;`;
}

function getSentimentTypeFromClass(sentimentClass) {
    const classToDisplay = {
        'sentiment-joy': '😊 Happy',
        'sentiment-sad': '😢 Sad',
        'sentiment-angry': '😠 Angry',
        'sentiment-confused': '🤔 Confused',
        'sentiment-fear': '😨 Fear',
        'sentiment-disgust': '🤢 Disgust',
        'sentiment-surprise': '😲 Surprise',
        'sentiment-neutral': '😐 Neutral',
        'sentiment-affection': '💕 Affection',
        'sentiment-sarcastic': '😏 Sarcastic'
    };
    
    return classToDisplay[sentimentClass] || '😐 Neutral';
}

// New helper function to get emoji directly from color
function getEmojiFromColor(color) {
    const colorToEmoji = {
        '#fbbf24': '😊',      // Joy - bright yellow/gold (displays as Happy, matches backend)
        '#1e3a8a': '😢',      // Sad - dark blue
        '#dc2626': '😠',      // Angry - red
        '#a16207': '🤔',      // Confused - brown/amber
        '#84cc16': '🤢',      // Disgust - lime green
        '#f97316': '😲',      // Surprise - orange
        '#374151': '😨',      // Fear - dark grey
        '#6b7280': '😐',      // Neutral - gray (matches backend)
        '#ec4899': '💕',      // Affection - pink
        '#7c3aed': '😏'       // Sarcastic - purple
    };
    
    return colorToEmoji[color] || '😐';
}

// Enhanced preview with both sentiment and toxicity
function previewSentiment() {
    const title = document.getElementById('postTitle').value;
    const content = document.getElementById('postContent').value;
    const preview = document.getElementById('sentimentPreview');
    
    const text = (title + ' ' + content).trim();
    
    if (text.length > 10) {
        // Simple client-side sentiment preview (not as accurate as backend)
        const sentiment = predictSentiment(text);
        const toxicity = predictToxicity(text);
        
        let previewText = `Preview: ${sentiment.emoji} ${sentiment.displayText} (${sentiment.confidence}% confidence)`;
        
        if (toxicity.tags.length > 0) {
            const toxicityText = toxicity.tags.map(tag => tag.displayText).join(', ');
            previewText += ` | ⚠️ Toxicity: ${toxicityText}`;
        }
        
        preview.innerHTML = previewText;
    } else {
        preview.textContent = '';
    }
}

// Simple client-side toxicity prediction for preview
function predictToxicity(text) {
    const lowerText = text.toLowerCase();
    const detectedTags = [];
    
    // Simple keyword-based toxicity detection
    const toxicityPatterns = {
        insult: ['stupid', 'idiot', 'moron', 'dumb', 'loser', 'pathetic'],
        threat: ['kill', 'die', 'hurt', 'destroy', 'eliminate'],
        obscene: ['damn', 'hell', 'crap'],
        toxicity: ['hate', 'suck', 'terrible', 'awful', 'disgusting']
    };
    
    for (const [category, keywords] of Object.entries(toxicityPatterns)) {
        let hasMatch = keywords.some(keyword => lowerText.includes(keyword));
        if (hasMatch) {
            const config = {
                insult: { emoji: '💢', label: 'Insulting', color: '#c2410c' },
                threat: { emoji: '⚡', label: 'Threatening', color: '#991b1b' },
                obscene: { emoji: '🚫', label: 'Obscene', color: '#7c2d12' },
                toxicity: { emoji: '⚠️', label: 'Toxic', color: '#f59e0b' }
            }[category];
            
            detectedTags.push({
                tag: category,
                emoji: config.emoji,
                label: config.label,
                color: config.color,
                displayText: `${config.emoji} ${config.label}`
            });
        }
    }
    
    return { tags: detectedTags };
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
    let predictedType = 'neutral';
    
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
        sad: '😢', angry: '😠', 
        fear: '😨', neutral: '😐', affection: '💕', sarcastic: '😏'
    };
    
    return {
        type: predictedType,
        emoji: emojis[predictedType],
        displayText: predictedType.charAt(0).toUpperCase() + predictedType.slice(1),
        confidence: confidence
    };
}

// Feed filtering
function filterFeed(sentiment, buttonElement = null) {
    // Update active filter button
    document.querySelectorAll('.filter-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    
    // If buttonElement is provided, use it; otherwise find the button by data-filter
    if (buttonElement) {
        buttonElement.classList.add('active');
    } else {
        const targetBtn = document.querySelector(`[data-filter="${sentiment}"]`);
        if (targetBtn) {
            targetBtn.classList.add('active');
        }
    }
    
    if (sentiment === 'all') {
        renderPosts(applyContentFiltering(posts));
    } else {
        const filtered = posts.filter(post => {
            const sentimentClass = getSentimentClass(post);
            
            // Handle sarcastic combinations - fixed class name
            if (sentiment === 'sarcastic' && sentimentClass === 'sentiment-sarcastic') {
                return true;
            }
            
            return sentimentClass.includes(sentiment);
        });
        renderPosts(applyContentFiltering(filtered));
    }
}

// Content filtering functions
function applyContentFilters() {
    // Update the hidden toxicity types based on unchecked checkboxes
    contentFilters.hiddenToxicityTypes.clear();
    
    const filterCheckboxes = [
        'filter-toxicity',
        'filter-severe_toxicity', 
        'filter-obscene',
        'filter-threat',
        'filter-insult'
    ];
    
    filterCheckboxes.forEach(checkboxId => {
        const checkbox = document.getElementById(checkboxId);
        if (!checkbox.checked) {
            // Extract toxicity type from checkbox ID
            const toxicityType = checkboxId.replace('filter-', '');
            contentFilters.hiddenToxicityTypes.add(toxicityType);
        }
    });
    
    // Save filter state to localStorage
    saveContentFilterState();
    
    // Re-apply current filters
    const activeFilterBtn = document.querySelector('.filter-btn.active');
    if (activeFilterBtn) {
        const currentFilter = activeFilterBtn.dataset.filter || 'all';
        
        // Get currently filtered posts by sentiment
        let currentPosts = posts;
        if (currentFilter !== 'all') {
            currentPosts = posts.filter(post => {
                const sentimentClass = getSentimentClass(post);
                if (currentFilter === 'sarcastic' && sentimentClass === 'sentiment-sarcastic') {
                    return true;
                }
                return sentimentClass.includes(currentFilter);
            });
        }
        
        // Apply content filtering and render
        renderPosts(applyContentFiltering(currentPosts));
    }
}

function applyContentFiltering(postsArray) {
    if (contentFilters.hiddenToxicityTypes.size === 0) {
        return postsArray; // No content filters applied
    }
    
    return postsArray.filter(post => {
        if (!post.toxicity_tags || post.toxicity_tags.length === 0) {
            return true; // Show posts with no toxicity tags
        }
        
        // Check if any of the post's toxicity tags should hide it
        const hasHiddenToxicity = post.toxicity_tags.some(tag => {
            const normalized = tag.toLowerCase().replace(/\s+/g, '_');
            return contentFilters.hiddenToxicityTypes.has(normalized);
        });
        
        return !hasHiddenToxicity; // Show post only if it doesn't have hidden toxicity types
    });
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

// Content filter persistence functions
function saveContentFilterState() {
    const filterState = {
        hiddenToxicityTypes: Array.from(contentFilters.hiddenToxicityTypes)
    };
    localStorage.setItem('socialPulse_contentFilters', JSON.stringify(filterState));
}

function loadContentFilterState() {
    try {
        const saved = localStorage.getItem('socialPulse_contentFilters');
        if (saved) {
            const filterState = JSON.parse(saved);
            contentFilters.hiddenToxicityTypes = new Set(filterState.hiddenToxicityTypes || []);
            
            // Update checkbox states to match saved preferences
            const filterCheckboxes = [
                'filter-toxicity',
                'filter-severe_toxicity',
                'filter-obscene',
                'filter-threat',
                'filter-insult'
            ];
            
            filterCheckboxes.forEach(checkboxId => {
                const checkbox = document.getElementById(checkboxId);
                const toxicityType = checkboxId.replace('filter-', '');
                if (checkbox && contentFilters.hiddenToxicityTypes.has(toxicityType)) {
                    checkbox.checked = false; // Uncheck boxes for hidden types
                }
            });
        }
    } catch (error) {
        console.warn('Failed to load content filter preferences:', error);
        // Reset to default state on error
        contentFilters.hiddenToxicityTypes.clear();
    }
}

// Refresh posts periodically (every 30 seconds)
setInterval(() => {
    if (document.visibilityState === 'visible') {
        loadPosts();
    }
}, 30000);