// Global state
let currentUser = null;
let authToken = localStorage.getItem('authToken');
let posts = [];

// Content filter state - tracks which toxicity types should be hidden
let contentFilters = {
    hiddenToxicityTypes: new Set() // Empty set means show all
};

// Pagination state for infinite scroll
let paginationState = {
    offset: 0,
    limit: 10,
    hasMore: true,
    isLoading: false
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
    
    // Infinite scroll detection
    window.addEventListener('scroll', handleScroll);
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

async function loadPosts(reset = true) {
    if (paginationState.isLoading) return;
    
    paginationState.isLoading = true;
    
    if (reset) {
        paginationState.offset = 0;
        paginationState.hasMore = true;
        posts = [];
    }
    
    const container = document.getElementById('postsContainer');
    const spinner = document.getElementById('loadingSpinner');
    
    if (spinner && reset) {
        spinner.classList.remove('hidden');
    }
    
    try {
        const url = `${API_BASE}/posts?limit=${paginationState.limit}&offset=${paginationState.offset}`;
        const response = await fetch(url);
        const data = await response.json();
        
        if (response.ok) {
            const newPosts = Array.isArray(data) ? data : data.posts || [];
            
            if (reset) {
                posts = newPosts;
            } else {
                posts = [...posts, ...newPosts];
            }
            
            // Update pagination state
            paginationState.hasMore = data.has_more !== false && newPosts.length === paginationState.limit;
            paginationState.offset += newPosts.length;
            
            // Apply active filters consistently
            const activeFilterBtn = document.querySelector('.filter-btn.active');
            if (activeFilterBtn) {
                const sentiment = activeFilterBtn.dataset.filter;
                filterFeed(sentiment); // This applies both emotion and content filters
            } else {
                // If no active emotion filter, just apply content filters
                renderPosts(applyContentFiltering(posts), reset);
            }
        } else {
            showToast('Failed to load posts', 'error');
            if (reset) renderEmptyState();
        }
    } catch (error) {
        console.error('Load posts error:', error);
        showToast('Failed to load posts', 'error');
        if (reset) renderEmptyState();
    } finally {
        paginationState.isLoading = false;
        if (spinner && reset) {
            spinner.classList.add('hidden');
        }
        hideInfiniteScrollLoader();
    }
}

function renderPosts(postsToRender, replace = true) {
    const container = document.getElementById('postsList');
    
    if (postsToRender.length === 0) {
        if (replace) {
            renderEmptyState();
        }
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
                    <button class="comment-toggle" onclick="toggleComments('${post.id}')">
                        💬 ${post.comment_count || 0} comments
                    </button>
                </div>
                
                <!-- Comments Section -->
                <div id="comments-${post.id}" class="comments-section hidden" style="margin-top: 15px; padding-top: 15px; border-top: 1px solid rgba(255,255,255,0.1);">
                    <div class="comment-form" ${!authToken ? 'style="display:none;"' : ''}>
                        <textarea id="comment-input-${post.id}" placeholder="Share your thoughts..." 
                                class="comment-textarea" rows="3" maxlength="2000"></textarea>
                        <div class="comment-form-actions">
                            <span class="comment-counter">0/2000</span>
                            <button onclick="postComment('${post.id}')" class="comment-submit-btn">Post Comment</button>
                        </div>
                    </div>
                    <div id="comments-list-${post.id}" class="comments-list">
                        <!-- Comments will be loaded here -->
                    </div>
                </div>
            </article>
        `;
    }).join('');
    
    if (replace) {
        container.innerHTML = postsHTML;
    } else {
        container.insertAdjacentHTML('beforeend', postsHTML);
    }
    
    // Setup comment input character counters
    document.querySelectorAll('.comment-textarea').forEach(textarea => {
        textarea.addEventListener('input', updateCommentCounter);
    });
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

// Infinite scroll functions
function showInfiniteScrollLoader() {
    let loader = document.getElementById('infiniteScrollLoader');
    if (!loader) {
        loader = document.createElement('div');
        loader.id = 'infiniteScrollLoader';
        loader.className = 'infinite-scroll-loader';
        loader.innerHTML = `
            <div style="text-align: center; padding: 2rem; color: #6b7280;">
                <div class="loading-spinner" style="display: inline-block; width: 20px; height: 20px; border: 2px solid #d1d5db; border-top: 2px solid #3b82f6; border-radius: 50%; animation: spin 1s linear infinite;"></div>
                <p style="margin: 1rem 0 0 0; font-size: 0.9rem;">Loading more posts...</p>
            </div>
        `;
        document.getElementById('postsList').appendChild(loader);
    }
    loader.classList.remove('hidden');
}

function hideInfiniteScrollLoader() {
    const loader = document.getElementById('infiniteScrollLoader');
    if (loader) {
        loader.classList.add('hidden');
    }
}

function handleScroll() {
    // Check if we're near the bottom of the page
    const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
    const windowHeight = window.innerHeight;
    const documentHeight = document.documentElement.scrollHeight;
    
    // Load more when user is within 200px of the bottom
    if (scrollTop + windowHeight >= documentHeight - 200) {
        if (paginationState.hasMore && !paginationState.isLoading) {
            showInfiniteScrollLoader();
            loadPosts(false); // Load more posts (don't reset)
        }
    }
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
        'toxicity': { emoji: '💩', label: 'Crude', color: '#f59e0b' },
        'super_toxic': { emoji: '🚨', label: 'Super Toxic', color: '#dc2626' },
        'obscene': { emoji: '🤬', label: 'Obscene', color: '#7c2d12' },
        'threat': { emoji: '⚡', label: 'Threatening', color: '#991b1b' },
        'threatening': { emoji: '⚡', label: 'Threatening', color: '#991b1b' },
        'insult': { emoji: '🖕', label: 'Insulting', color: '#c2410c' },
        'insulting': { emoji: '🖕', label: 'Insulting', color: '#c2410c' },
        'identity_attack': { emoji: '🛡️', label: 'Identity Attack', color: '#7f1d1d' },
        'severe_toxicity': { emoji: '☣️', label: 'Toxic', color: '#450a0a' }
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
                insult: { emoji: '🖕', label: 'Insulting', color: '#c2410c' },
                threat: { emoji: '⚡', label: 'Threatening', color: '#991b1b' },
                obscene: { emoji: '🤬', label: 'Obscene', color: '#7c2d12' },
                toxicity: { emoji: '💩', label: 'Crude', color: '#f59e0b' }
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

// ===== COMMENT SYSTEM =====

// Comment system state
let loadedComments = new Set(); // Track which post IDs have loaded comments

// Toggle comment section visibility
function toggleComments(postId) {
    const commentsSection = document.getElementById(`comments-${postId}`);
    const isHidden = commentsSection.classList.contains('hidden');
    
    if (isHidden) {
        commentsSection.classList.remove('hidden');
        // Load comments if not already loaded
        if (!loadedComments.has(postId)) {
            loadComments(postId);
        }
    } else {
        commentsSection.classList.add('hidden');
    }
}

// Load comments for a post
async function loadComments(postId) {
    if (loadedComments.has(postId)) return;
    
    const commentsList = document.getElementById(`comments-list-${postId}`);
    commentsList.innerHTML = '<div class="loading-comments">Loading comments...</div>';
    
    try {
        const response = await fetch(`${API_BASE}/posts/${postId}/comments`);
        const data = await response.json();
        
        if (response.ok) {
            loadedComments.add(postId);
            renderComments(postId, data.comments || []);
        } else {
            commentsList.innerHTML = '<div class="error-message">Failed to load comments</div>';
        }
    } catch (error) {
        console.error('Load comments error:', error);
        commentsList.innerHTML = '<div class="error-message">Failed to load comments</div>';
    }
}

// Post a new comment
async function postComment(postId) {
    if (!authToken) {
        showToast('Please log in to comment', 'error');
        return;
    }
    
    const textarea = document.getElementById(`comment-input-${postId}`);
    const content = textarea.value.trim();
    
    if (!content) {
        showToast('Please enter a comment', 'error');
        return;
    }
    
    if (content.length > 2000) {
        showToast('Comment is too long (max 2000 characters)', 'error');
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/posts/${postId}/comments`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({
                post_id: postId,
                content: content,
                parent_id: null // Top-level comment
            })
        });
        
        const data = await response.json();
        
        if (response.ok) {
            textarea.value = '';
            updateCommentCounter({ target: textarea });
            showToast('Comment posted!', 'success');
            
            // Reload comments to show the new one
            loadedComments.delete(postId);
            loadComments(postId);
            
            // Update comment count in post
            updatePostCommentCount(postId);
        } else {
            showToast(data.message || 'Failed to post comment', 'error');
        }
    } catch (error) {
        console.error('Post comment error:', error);
        showToast('Failed to post comment', 'error');
    }
}

// Render comments with nesting and emotion colors
function renderComments(postId, comments) {
    const commentsList = document.getElementById(`comments-list-${postId}`);
    
    if (!comments || comments.length === 0) {
        commentsList.innerHTML = '<div class="no-comments">No comments yet. Be the first to comment!</div>';
        return;
    }
    
    // Build nested comment structure (simplified for now - full nesting will be added later)
    const commentsHTML = comments.map(commentData => {
        const comment = commentData.comment;
        const author = commentData.author;
        const timeAgo = formatTimeAgo(comment.created_at);
        
        // Get sentiment styling
        const sentimentClass = getCommentSentimentClass(comment);
        const sentimentEmoji = getCommentSentimentEmoji(comment);
        
        return `
            <div class="comment ${sentimentClass}" data-comment-id="${comment.id}">
                <div class="comment-header">
                    <span class="comment-author">${escapeHtml(author?.username || 'Anonymous')}</span>
                    <span class="comment-time">${timeAgo}</span>
                    ${sentimentEmoji ? `<span class="comment-emotion">${sentimentEmoji}</span>` : ''}
                </div>
                <div class="comment-content">${escapeHtml(comment.content)}</div>
                <div class="comment-actions">
                    ${authToken ? `<button onclick="showReplyForm('${comment.id}')" class="reply-btn">Reply</button>` : ''}
                    <div class="emotion-voting" id="emotion-voting-${comment.id}">
                        ${renderEmotionVoting(comment)}
                    </div>
                </div>
                
                <!-- Reply form (initially hidden) -->
                ${authToken ? `
                <div id="reply-form-${comment.id}" class="reply-form hidden">
                    <textarea id="reply-input-${comment.id}" placeholder="Write a reply..." 
                            class="reply-textarea" rows="2" maxlength="2000"></textarea>
                    <div class="reply-form-actions">
                        <button onclick="cancelReply('${comment.id}')" class="cancel-btn">Cancel</button>
                        <button onclick="postReply('${comment.id}', '${postId}')" class="reply-submit-btn">Reply</button>
                    </div>
                </div>` : ''}
                
                <!-- Nested replies will go here -->
                <div id="replies-${comment.id}" class="replies-container">
                    ${renderReplies(commentData.replies || [])}
                </div>
            </div>
        `;
    }).join('');
    
    commentsList.innerHTML = commentsHTML;
}

// Get comment sentiment class for styling
function getCommentSentimentClass(comment) {
    if (!comment.sentiment_analysis) return '';
    
    try {
        const sentiments = JSON.parse(comment.sentiment_analysis);
        if (Array.isArray(sentiments) && sentiments.length > 0) {
            const primarySentiment = sentiments[0];
            return `sentiment-${primarySentiment.toLowerCase()}`;
        }
    } catch (e) {
        console.warn('Failed to parse comment sentiment:', e);
    }
    return '';
}

// Get comment sentiment emoji
function getCommentSentimentEmoji(comment) {
    if (!comment.sentiment_analysis) return '';
    
    try {
        const sentiments = JSON.parse(comment.sentiment_analysis);
        if (Array.isArray(sentiments) && sentiments.length > 0) {
            const primarySentiment = sentiments[0];
            const emojiMap = {
                'joy': '😊',
                'sad': '😢', 
                'angry': '😠',
                'fear': '😨',
                'surprise': '😲',
                'disgust': '🤢',
                'confused': '😕',
                'sarcastic': '😏',
                'affectionate': '🥰',
                'neutral': '😐'
            };
            return emojiMap[primarySentiment.toLowerCase()] || '';
        }
    } catch (e) {
        console.warn('Failed to parse comment sentiment:', e);
    }
    return '';
}

// Render emotion voting buttons (placeholder for now)
function renderEmotionVoting(comment) {
    // TODO: Implement full emotion voting system
    // For now, just show a simple like count
    return `<span class="vote-count">👍 0</span>`;
}

// Render nested replies
function renderReplies(replies) {
    if (!replies || replies.length === 0) return '';
    
    return replies.map(replyData => {
        const reply = replyData.comment;
        const author = replyData.author;
        const timeAgo = formatTimeAgo(reply.created_at);
        const sentimentClass = getCommentSentimentClass(reply);
        const sentimentEmoji = getCommentSentimentEmoji(reply);
        
        return `
            <div class="reply ${sentimentClass}" data-comment-id="${reply.id}">
                <div class="comment-header">
                    <span class="comment-author">${escapeHtml(author?.username || 'Anonymous')}</span>
                    <span class="comment-time">${timeAgo}</span>
                    ${sentimentEmoji ? `<span class="comment-emotion">${sentimentEmoji}</span>` : ''}
                </div>
                <div class="comment-content">${escapeHtml(reply.content)}</div>
            </div>
        `;
    }).join('');
}

// Update comment counter
function updateCommentCounter(event) {
    const textarea = event.target;
    const counter = textarea.closest('.comment-form, .reply-form').querySelector('.comment-counter, .reply-counter');
    if (counter) {
        const length = textarea.value.length;
        counter.textContent = `${length}/2000`;
        counter.style.color = length > 1800 ? '#ef4444' : '';
    }
}

// Update post comment count after posting
function updatePostCommentCount(postId) {
    const commentButton = document.querySelector(`button[onclick="toggleComments('${postId}')"]`);
    if (commentButton) {
        // Extract current count and increment
        const match = commentButton.textContent.match(/(\d+)/);
        if (match) {
            const currentCount = parseInt(match[1]);
            commentButton.innerHTML = `💬 ${currentCount + 1} comments`;
        }
    }
}

// Show reply form
function showReplyForm(commentId) {
    const replyForm = document.getElementById(`reply-form-${commentId}`);
    replyForm.classList.remove('hidden');
    const textarea = document.getElementById(`reply-input-${commentId}`);
    textarea.focus();
}

// Cancel reply
function cancelReply(commentId) {
    const replyForm = document.getElementById(`reply-form-${commentId}`);
    replyForm.classList.add('hidden');
    const textarea = document.getElementById(`reply-input-${commentId}`);
    textarea.value = '';
}

// Post reply (placeholder - will implement nested replies later)
async function postReply(parentCommentId, postId) {
    showToast('Nested replies coming soon!', 'info');
    cancelReply(parentCommentId);
}

// ===== END COMMENT SYSTEM =====

// Helper functions (moved here for clarity)
function formatTimeAgo(timestamp) {
    const now = new Date();
    const postTime = new Date(timestamp);
    const diffInSeconds = Math.floor((now - postTime) / 1000);
    
    if (diffInSeconds < 60) return 'just now';
    if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}m ago`;
    if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)}h ago`;
    if (diffInSeconds < 604800) return `${Math.floor(diffInSeconds / 86400)}d ago`;
    
    return postTime.toLocaleDateString();
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Refresh posts periodically (every 30 seconds)
setInterval(() => {
    if (document.visibilityState === 'visible') {
        loadPosts();
    }
}, 30000);