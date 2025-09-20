// Global state
let currentUser = null;
let authToken = localStorage.getItem('authToken');
let posts = [];
let currentView = 'feed'; // 'feed' or 'user_home'

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

// API Configuration - Use full domain for Replit environment
const API_BASE = window.location.origin + '/api';

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
    window.addEventListener('scroll', optimizedHandleScroll);
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
    currentView = 'feed'; // Reset to main feed
    showGuestInterface();
    showToast('Logged out successfully', 'info');
}

// User home page functions
function showUserHome() {
    if (!currentUser) {
        showToast('Please log in to view your posts', 'error');
        return;
    }
    
    currentView = 'user_home';
    document.getElementById('feedTitle').textContent = `My Posts (${currentUser.username})`;
    document.getElementById('feedControls').style.display = 'none'; // Hide filters for user posts
    loadPosts(true); // Reset and load user's posts
    showToast('Showing your posts', 'info');
}

function showMainFeed() {
    currentView = 'feed';
    document.getElementById('feedTitle').textContent = 'Vibe Check';
    document.getElementById('feedControls').style.display = 'block'; // Show filters
    loadPosts(true); // Reset and load main feed
    showToast('Showing main feed', 'info');
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
        // Show skeleton loading for initial load
        showSkeletonLoading(true);
    } else {
        // Show skeleton loading for additional posts
        showSkeletonLoading(false);
    }
    
    const container = document.getElementById('postsContainer');
    const spinner = document.getElementById('loadingSpinner');
    
    if (spinner && reset) {
        spinner.classList.remove('hidden');
    }
    
    try {
        // Choose URL based on current view
        let url;
        if (currentView === 'user_home' && currentUser) {
            url = `${API_BASE}/posts/user/${currentUser.id}?limit=${paginationState.limit}&offset=${paginationState.offset}`;
        } else {
            url = `${API_BASE}/posts?limit=${paginationState.limit}&offset=${paginationState.offset}`;
        }
        
        // Add a small delay for better UX (minimum loading time for skeletons to be visible)
        const [response] = await Promise.all([
            fetch(url),
            new Promise(resolve => setTimeout(resolve, reset ? 500 : 300))
        ]);
        
        const data = await response.json();
        
        if (response.ok) {
            const newPosts = Array.isArray(data) ? data : data.posts || [];
            
            if (reset) {
                posts = newPosts;
            } else {
                posts = [...posts, ...newPosts];
            }
            
            // Load vote data for new posts
            if (newPosts.length > 0) {
                await loadVoteDataForPosts(newPosts);
            }
            
            // Update pagination state
            paginationState.hasMore = data.has_more !== false && newPosts.length === paginationState.limit;
            paginationState.offset += newPosts.length;
            
            // Hide skeleton loading before showing real posts
            hideSkeletonLoading();
            
            // Small delay to allow skeleton removal animation to complete
            setTimeout(() => {
                // Apply active filters consistently
                const activeFilterBtn = document.querySelector('.filter-btn.active');
                if (activeFilterBtn) {
                    const sentiment = activeFilterBtn.dataset.filter;
                    filterFeed(sentiment); // This applies both emotion and content filters
                } else {
                    // If no active emotion filter, just apply content filters
                    renderPosts(applyContentFiltering(posts), reset);
                }
            }, 150);
        } else {
            hideSkeletonLoading();
            showRetryMessage(reset);
        }
    } catch (error) {
        console.error('Load posts error:', error);
        hideSkeletonLoading();
        showRetryMessage(reset);
    } finally {
        paginationState.isLoading = false;
        if (spinner && reset) {
            spinner.classList.add('hidden');
        }
        hideInfiniteScrollLoader();
    }
}

// Enhanced error handling with retry functionality
function showRetryMessage(isReset = true) {
    if (isReset) {
        const container = document.getElementById('postsList');
        container.innerHTML = `
            <div class="retry-state" style="text-align: center; padding: 3rem; color: #6b7280; animation: fadeIn 0.5s ease-out;">
                <div style="font-size: 3rem; margin-bottom: 1rem;">⚠️</div>
                <h3 style="margin-bottom: 0.5rem; color: #ef4444;">Failed to load posts</h3>
                <p style="margin-bottom: 2rem;">Something went wrong. Please try again.</p>
                <button onclick="retryLoadPosts()" class="btn btn-primary" style="padding: 0.75rem 1.5rem; border-radius: 0.5rem; background: #4f46e5; color: white; border: none; cursor: pointer; font-weight: 500;">
                    🔄 Retry
                </button>
            </div>
        `;
    } else {
        showToast('Failed to load more posts. Scroll down to try again.', 'error');
    }
}

function retryLoadPosts() {
    loadPosts(true);
}

// Performance optimization - batch DOM updates
function optimizedRenderPosts(postsToRender, replace = true) {
    // Use document fragment for better performance
    const fragment = document.createDocumentFragment();
    const container = document.getElementById('postsList');
    
    if (postsToRender.length === 0) {
        if (replace) {
            renderEmptyState();
        }
        return;
    }
    
    // Create all posts in memory first
    postsToRender.forEach(post => {
        const postElement = createPostElement(post);
        fragment.appendChild(postElement);
    });
    
    if (replace) {
        container.innerHTML = '';
        container.appendChild(fragment);
        animatePostsIn(container.children, 0);
    } else {
        const startIndex = container.children.length;
        container.appendChild(fragment);
        const newPosts = Array.from(container.children).slice(startIndex);
        animatePostsIn(newPosts, 200);
    }
    
    // Setup event listeners after rendering
    setupPostEventListeners();
}

function createPostElement(post) {
    const article = document.createElement('article');
    article.className = 'post-card';
    article.style.cssText = getSentimentBackground(post);
    
    const sentimentClass = getSentimentClass(post);
    const sentimentLabel = getSentimentLabel(post);
    const timeAgo = formatTimeAgo(post.created_at);
    
    // Get toxicity tags for this post
    const toxicityTags = getToxicityTags(post);
    const toxicityTagsHTML = renderToxicityTags(toxicityTags, post.id);
    
    // Show delete controls only for posts owned by current user AND only on "My posts" page
    const isOwner = currentUser && post.author_id === currentUser.id;
    const isMyPostsPage = currentView === 'user_home';
    const deleteControlsHTML = (isOwner && isMyPostsPage) ? `
        <div class="delete-controls">
            <input type="checkbox" class="delete-checkbox" data-type="post" data-id="${post.id}" 
                   onchange="toggleDeleteControls()">
            <button class="delete-icon" onclick="deletePost('${post.id}')" title="Delete Post">🗑️</button>
        </div>
    ` : '';
    
    article.innerHTML = `
        <div class="post-header">
            <div class="post-author-section">
                <div class="post-author">${escapeHtml(post.author_username)}</div>
                <div class="post-time">${timeAgo}</div>
            </div>
            <div class="post-header-right">
                <div class="post-badges">
                    ${sentimentLabel ? renderVotableEmotionTag(post, sentimentClass, sentimentLabel) : ''}
                </div>
                ${deleteControlsHTML}
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
    `;
    
    return article;
}

function setupPostEventListeners() {
    // Setup comment input character counters
    document.querySelectorAll('.comment-textarea').forEach(textarea => {
        if (!textarea.dataset.listenerAttached) {
            textarea.addEventListener('input', updateCommentCounter);
            textarea.dataset.listenerAttached = 'true';
        }
    });
}

function renderPosts(postsToRender, replace = true) {
    const container = document.getElementById('postsList');
    
    if (postsToRender.length === 0) {
        if (replace) {
            renderEmptyState();
        }
        return;
    }
    
    // If replacing content, clear existing posts first
    if (replace) {
        container.innerHTML = '';
    }
    
    const postsHTML = postsToRender.map(post => {
        const sentimentClass = getSentimentClass(post);
        const sentimentLabel = getSentimentLabel(post);
        const backgroundStyle = getSentimentBackground(post);
        const timeAgo = formatTimeAgo(post.created_at);
        
        // Get toxicity tags for this post
        const toxicityTags = getToxicityTags(post);
        const toxicityTagsHTML = renderToxicityTags(toxicityTags, post.id);
        
        // Show delete controls only for posts owned by current user AND only on "My posts" page
        const isOwner = currentUser && post.author_id === currentUser.id;
        const isMyPostsPage = currentView === 'user_home';
        const deleteControlsHTML = (isOwner && isMyPostsPage) ? `
            <div class="delete-controls">
                <input type="checkbox" class="delete-checkbox" data-type="post" data-id="${post.id}" 
                       onchange="toggleDeleteControls()">
                <button class="delete-icon" onclick="deletePost('${post.id}')" title="Delete Post">🗑️</button>
            </div>
        ` : '';
        
        return `
            <article class="post-card" style="${backgroundStyle}">
                <div class="post-header">
                    <div class="post-author-section">
                        <div class="post-author">${escapeHtml(post.author_username)}</div>
                        <div class="post-time">${timeAgo}</div>
                    </div>
                    <div class="post-header-right">
                        <div class="post-badges">
                            ${sentimentLabel ? renderVotableEmotionTag(post, sentimentClass, sentimentLabel) : ''}
                        </div>
                        ${deleteControlsHTML}
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
        // Animate in new posts with staggered timing
        animatePostsIn(container.children, 0);
    } else {
        const startIndex = container.children.length;
        container.insertAdjacentHTML('beforeend', postsHTML);
        // Animate only the newly added posts
        const newPosts = Array.from(container.children).slice(startIndex);
        animatePostsIn(newPosts, 200); // Small delay for append
    }
    
    // Setup comment input character counters
    document.querySelectorAll('.comment-textarea').forEach(textarea => {
        textarea.addEventListener('input', updateCommentCounter);
    });
}

// Post Animation Functions
function animatePostsIn(elements, initialDelay = 0) {
    Array.from(elements).forEach((element, index) => {
        if (element.classList.contains('skeleton-post')) {
            return; // Skip skeleton posts
        }
        
        // Initially hide the post
        element.style.opacity = '0';
        element.style.transform = 'translateY(20px)';
        element.style.transition = 'all 0.4s ease-out';
        
        // Animate in with staggered timing
        setTimeout(() => {
            element.style.opacity = '1';
            element.style.transform = 'translateY(0)';
        }, initialDelay + (index * 100)); // 100ms stagger between posts
    });
}

function addPostLoadingEffect(postElement) {
    postElement.classList.add('loading');
    setTimeout(() => {
        postElement.classList.remove('loading');
    }, 300);
}

function renderEmptyState() {
    const container = document.getElementById('postsList');
    container.innerHTML = `
        <div class="empty-state" style="text-align: center; padding: 3rem; color: #6b7280; animation: fadeIn 0.5s ease-out;">
            <div style="font-size: 3rem; margin-bottom: 1rem;">📝</div>
            <h3 style="margin-bottom: 0.5rem;">No posts yet</h3>
            <p style="margin: 0;">Be the first to share something!</p>
        </div>
    `;
}

// Skeleton Loading Functions
function renderSkeletonPosts(count = 3) {
    const skeletons = Array.from({ length: count }, (_, index) => `
        <div class="skeleton-post" id="skeleton-${index}">
            <div class="skeleton-header">
                <div class="skeleton-element skeleton-avatar"></div>
                <div class="skeleton-element skeleton-author"></div>
                <div class="skeleton-element skeleton-time"></div>
            </div>
            <div class="skeleton-element skeleton-title"></div>
            <div class="skeleton-element skeleton-content"></div>
            <div class="skeleton-element skeleton-content"></div>
            <div class="skeleton-element skeleton-content"></div>
            <div class="skeleton-footer">
                <div class="skeleton-element skeleton-badge"></div>
                <div class="skeleton-element skeleton-comment-btn"></div>
            </div>
        </div>
    `).join('');
    
    return skeletons;
}

function showSkeletonLoading(replace = true) {
    const container = document.getElementById('postsList');
    const skeletonHTML = renderSkeletonPosts(replace ? 6 : 3);
    
    if (replace) {
        container.innerHTML = skeletonHTML;
    } else {
        container.insertAdjacentHTML('beforeend', skeletonHTML);
    }
}

function hideSkeletonLoading() {
    const skeletons = document.querySelectorAll('.skeleton-post');
    skeletons.forEach((skeleton, index) => {
        setTimeout(() => {
            if (skeleton.parentNode) {
                skeleton.remove();
            }
        }, index * 50); // Staggered removal for smooth effect
    });
}

// Enhanced Infinite scroll functions
function showInfiniteScrollLoader() {
    let loader = document.getElementById('infiniteScrollLoader');
    if (!loader) {
        loader = document.createElement('div');
        loader.id = 'infiniteScrollLoader';
        loader.className = 'infinite-scroll-loader enhanced-loader';
        loader.innerHTML = `
            <div class="enhanced-spinner"></div>
            <div class="loading-text loading-dots">Loading more posts</div>
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
    
    // Load more when user is within 400px of the bottom (increased for better UX)
    if (scrollTop + windowHeight >= documentHeight - 400) {
        if (paginationState.hasMore && !paginationState.isLoading) {
            loadPosts(false); // Load more posts (don't reset)
        }
    }
}

// Optimized scroll handler with throttling
let scrollTimeout;
function optimizedHandleScroll() {
    if (scrollTimeout) {
        clearTimeout(scrollTimeout);
    }
    
    scrollTimeout = setTimeout(() => {
        handleScroll();
    }, 100); // Throttle scroll events to every 100ms
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

// Function to render toxicity tags HTML with voting
function renderToxicityTags(toxicityTags, postId) {
    if (toxicityTags.length === 0) {
        return '';
    }
    
    const tagsHTML = toxicityTags.map(tag => 
        renderVotableContentTag(postId, tag)
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

// === VOTING SYSTEM ===

// Store vote data for each post
const voteData = new Map(); // postId -> { emotionVotes: [], contentVotes: [] }
const userVotes = new Map(); // postId -> { voteType_tag -> isUpvote }

// Render votable emotion tag (for sentiment badges)
function renderVotableEmotionTag(post, sentimentClass, sentimentLabel) {
    // Extract emotion tag from sentiment class
    const emotionTag = sentimentClass.replace('sentiment-', '');
    const voteKey = `emotion_${emotionTag}`;
    const userVote = getUserVote(post.id, voteKey);
    const voteCount = getVoteCount(post.id, 'emotion', emotionTag);
    const voteCountDisplay = voteCount > 0 ? ` ${formatVoteCount(voteCount)}` : '';
    
    const votedClass = userVote ? 'voted agreed' : '';
    
    return `
        <div class="sentiment-badge ${sentimentClass} votable-tag ${votedClass}" 
             onclick="voteOnTag('${post.id}', 'post', 'emotion', '${emotionTag}')"
             title="Click to agree this emotion matches the content. Click again to remove your agreement.">
            ${sentimentLabel}${voteCountDisplay}
        </div>
    `;
}

// Render votable content tag (for toxicity tags)
function renderVotableContentTag(postId, tag) {
    const voteKey = `content_filter_${tag.tag}`;
    const userVote = getUserVote(postId, voteKey);
    const voteCount = getVoteCount(postId, 'content_filter', tag.tag);
    const voteCountDisplay = voteCount > 0 ? ` ${formatVoteCount(voteCount)}` : '';
    
    const votedClass = userVote ? 'voted agreed' : '';
    
    return `
        <span class="toxicity-tag votable-tag ${votedClass}" 
              style="background-color: ${tag.color}20; border: 1px solid ${tag.color}60; color: ${tag.color}"
              onclick="voteOnTag('${postId}', 'post', 'content_filter', '${tag.tag}')"
              title="Click to agree this content tag matches. Click again to remove your agreement.">
            ${tag.displayText}${voteCountDisplay}
        </span>
    `;
}

// Vote on a tag (emotion or content) - Simple agree/disagree toggle
async function voteOnTag(targetId, targetType, voteType, tag) {
    if (!authToken) {
        showToast('Please log in to vote', 'error');
        return;
    }
    
    try {
        const voteKey = `${voteType}_${tag}`;
        const currentVote = getUserVote(targetId, voteKey);
        
        // If user already agreed, remove the vote (toggle off)
        if (currentVote) {
            await removeVote(targetId, targetType, voteType, tag);
            setUserVote(targetId, voteKey, null);
        } else {
            // Cast agreement vote
            const response = await fetch('/api/vote', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${authToken}`
                },
                body: JSON.stringify({
                    target_id: targetId,
                    target_type: targetType,
                    vote_type: voteType,
                    tag: tag,
                    is_upvote: true  // Always true since we only support agreement
                })
            });
            
            if (response.ok) {
                const voteSummary = await response.json();
                setUserVote(targetId, voteKey, true);
                updateVoteData(targetId, voteSummary);
                refreshPostVoting(targetId);
            } else {
                throw new Error('Failed to cast vote');
            }
        }
    } catch (error) {
        console.error('Vote error:', error);
        showToast('Failed to vote. Please try again.', 'error');
    }
}

// Remove a vote
async function removeVote(targetId, targetType, voteType, tag) {
    const response = await fetch(`/api/vote/${targetId}/${targetType}/${voteType}/${tag}`, {
        method: 'DELETE',
        headers: {
            'Authorization': `Bearer ${authToken}`
        }
    });
    
    if (response.ok) {
        const voteSummary = await response.json();
        updateVoteData(targetId, voteSummary);
        refreshPostVoting(targetId);
    } else {
        throw new Error('Failed to remove vote');
    }
}

// Get user's vote on a specific tag (simplified for agreement-only)
function getUserVote(targetId, voteKey) {
    const postVotes = userVotes.get(targetId) || {};
    // Return true if user has agreed, null if no vote
    return postVotes[voteKey] === true ? true : null;
}

// Set user's vote on a specific tag (simplified for agreement-only)
function setUserVote(targetId, voteKey, vote) {
    if (!userVotes.has(targetId)) {
        userVotes.set(targetId, {});
    }
    if (vote === null || vote === false) {
        delete userVotes.get(targetId)[voteKey];
    } else {
        userVotes.get(targetId)[voteKey] = true; // Only store agreements
    }
}

// Get vote count for a tag (agreement count only)
function getVoteCount(targetId, voteType, tag) {
    const data = voteData.get(targetId);
    if (!data) return 0;
    
    const votes = voteType === 'emotion' ? data.emotion_votes : data.content_filter_votes;
    const tagVote = votes.find(v => v.tag === tag);
    // Since we only support agreements, show upvotes count as total agreement count
    return tagVote ? tagVote.upvotes : 0;
}

// Update vote data from server response
function updateVoteData(targetId, voteSummary) {
    voteData.set(targetId, voteSummary);
}

// Format vote count display
function formatVoteCount(count) {
    if (count >= 1000000) {
        return `${(count / 1000000).toFixed(1)}M`.replace('.0M', 'M');
    } else if (count >= 1000) {
        return `${(count / 1000).toFixed(1)}k`.replace('.0k', 'k');
    }
    return count.toString();
}

// Refresh voting display for a post
function refreshPostVoting(targetId) {
    // Find and update the post's voting elements
    const postElement = document.querySelector(`[data-post-id="${targetId}"]`);
    if (postElement) {
        // Force re-render of the post to update vote counts
        loadPosts(true); // Reload posts to refresh vote counts
    }
}

// Load vote data for posts when they're displayed
async function loadVoteDataForPosts(posts) {
    if (!authToken) return;
    
    try {
        for (const post of posts) {
            // Load vote summary for each post
            const response = await fetch(`/api/vote/${post.id}/post`, {
                headers: {
                    'Authorization': `Bearer ${authToken}`
                }
            });
            
            if (response.ok) {
                const voteSummary = await response.json();
                updateVoteData(post.id, voteSummary);
                
                // Load user's votes for emotion and content tags
                await loadUserVotesForPost(post);
            }
        }
    } catch (error) {
        console.warn('Failed to load vote data:', error);
    }
}

// Load user's specific votes for a post
async function loadUserVotesForPost(post) {
    try {
        // Load user votes for emotion tags
        if (post.sentiment_colors && post.sentiment_colors.length > 0) {
            const sentimentClass = getSentimentClass(post);
            const emotionTag = sentimentClass.replace('sentiment-', '');
            
            const emotionResponse = await fetch(`/api/vote/user/${post.id}/post/emotion/${emotionTag}`, {
                headers: { 'Authorization': `Bearer ${authToken}` }
            });
            
            if (emotionResponse.ok) {
                const userVote = await emotionResponse.json();
                if (userVote) {
                    setUserVote(post.id, `emotion_${emotionTag}`, userVote.is_upvote);
                }
            }
        }
        
        // Load user votes for content tags
        if (post.toxicity_tags && post.toxicity_tags.length > 0) {
            for (const tag of post.toxicity_tags) {
                const normalized = tag.toLowerCase().replace(/\s+/g, '_');
                const contentResponse = await fetch(`/api/vote/user/${post.id}/post/content_filter/${normalized}`, {
                    headers: { 'Authorization': `Bearer ${authToken}` }
                });
                
                if (contentResponse.ok) {
                    const userVote = await contentResponse.json();
                    if (userVote) {
                        setUserVote(post.id, `content_filter_${normalized}`, userVote.is_upvote);
                    }
                }
            }
        }
    } catch (error) {
        console.warn('Failed to load user votes for post:', post.id, error);
    }
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
        
        // Get sentiment styling (same as posts)
        const sentimentClass = getCommentSentimentClass(comment);
        const sentimentEmoji = getCommentSentimentEmoji(comment);
        const sentimentStyle = getCommentSentimentStyle(comment);
        
        // Show delete controls only for comments owned by current user AND only on "My posts" page
        const isOwner = currentUser && comment.user_id === currentUser.id;
        const isMyPostsPage = currentView === 'user_home';
        const deleteControlsHTML = (isOwner && isMyPostsPage) ? `
            <div class="delete-controls comment-delete-controls">
                <input type="checkbox" class="delete-checkbox" data-type="comment" data-id="${comment.id}" 
                       onchange="toggleDeleteControls()">
                <button class="delete-icon" onclick="deleteComment('${comment.id}')" title="Delete Comment">🗑️</button>
            </div>
        ` : '';
        
        return `
            <div class="comment ${sentimentClass}" data-comment-id="${comment.id}" style="${sentimentStyle}">
                <div class="comment-header">
                    <div class="comment-header-left">
                        <span class="comment-author">${escapeHtml(author?.username || 'Anonymous')}</span>
                        <span class="comment-time">${timeAgo}</span>
                        ${sentimentEmoji ? `<span class="comment-emotion">${sentimentEmoji}</span>` : ''}
                    </div>
                    ${deleteControlsHTML}
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

// Get comment sentiment class for styling (same as posts)
function getCommentSentimentClass(comment) {
    if (!comment.sentiment_type) return '';
    
    // Use the same sentiment class mapping as posts
    return `sentiment-${comment.sentiment_type.toLowerCase()}`;
}

// Get comment sentiment emoji (same as posts)
function getCommentSentimentEmoji(comment) {
    if (!comment.sentiment_type) return '';
    
    const emojiMap = {
        'joy': '😊',
        'happy': '😊',  // Alternative mapping
        'sad': '😢', 
        'angry': '😠',
        'fear': '😨',
        'surprise': '😲',
        'disgust': '🤢',
        'confused': '😕',
        'sarcastic': '😏',
        'affectionate': '🥰',
        'affection': '🥰',  // Alternative mapping
        'neutral': '😐'
    };
    return emojiMap[comment.sentiment_type.toLowerCase()] || '';
}

// Get comment sentiment style (same as posts)
function getCommentSentimentStyle(comment) {
    if (!comment.sentiment_colors || comment.sentiment_colors.length === 0) {
        return '';
    }
    
    // Use single sentiment color (first color if multiple exist)
    const color = comment.sentiment_colors[0];
    return `border-left: 4px solid ${color}; background: ${color}11;`;
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
        const sentimentStyle = getCommentSentimentStyle(reply);
        
        return `
            <div class="reply ${sentimentClass}" data-comment-id="${reply.id}" style="${sentimentStyle}">
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

// ===== DELETE FUNCTIONALITY =====

// Delete functionality
function toggleDeleteControls() {
    const selectedItems = document.querySelectorAll('.delete-checkbox:checked');
    const bulkDeleteBtn = document.getElementById('bulkDeleteBtn');
    
    if (selectedItems.length > 0) {
        showBulkDeleteButton();
    } else {
        hideBulkDeleteButton();
    }
}

function showBulkDeleteButton() {
    let bulkDeleteBtn = document.getElementById('bulkDeleteBtn');
    if (!bulkDeleteBtn) {
        bulkDeleteBtn = document.createElement('div');
        bulkDeleteBtn.id = 'bulkDeleteBtn';
        bulkDeleteBtn.className = 'bulk-delete-container';
        bulkDeleteBtn.innerHTML = `
            <div class="bulk-delete-info">
                <span id="selectedCount">0</span> items selected
            </div>
            <button class="bulk-delete-btn" onclick="confirmBulkDelete()">
                🗑️ Delete Selected
            </button>
            <button class="bulk-cancel-btn" onclick="clearAllSelections()">
                Cancel
            </button>
        `;
        document.body.appendChild(bulkDeleteBtn);
    }
    
    const selectedCount = document.querySelectorAll('.delete-checkbox:checked').length;
    document.getElementById('selectedCount').textContent = selectedCount;
    bulkDeleteBtn.style.display = 'flex';
}

function hideBulkDeleteButton() {
    const bulkDeleteBtn = document.getElementById('bulkDeleteBtn');
    if (bulkDeleteBtn) {
        bulkDeleteBtn.style.display = 'none';
    }
}

function clearAllSelections() {
    document.querySelectorAll('.delete-checkbox:checked').forEach(checkbox => {
        checkbox.checked = false;
    });
    hideBulkDeleteButton();
}

async function deletePost(postId) {
    if (!confirm('Are you sure you want to delete this post?')) {
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/posts/${postId}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json'
            }
        });
        
        if (response.ok) {
            showToast('Post deleted successfully', 'success');
            // Remove post from UI
            const postElement = document.querySelector(`[data-post-id="${postId}"]`);
            if (postElement) {
                postElement.remove();
            }
            // Reload posts to refresh the view
            loadPosts(true);
        } else {
            const data = await response.json();
            showToast(data.message || 'Failed to delete post', 'error');
        }
    } catch (error) {
        console.error('Delete post error:', error);
        showToast('Failed to delete post. Please try again.', 'error');
    }
}

async function deleteComment(commentId) {
    if (!confirm('Are you sure you want to delete this comment?')) {
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/comments/${commentId}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json'
            }
        });
        
        if (response.ok) {
            showToast('Comment deleted successfully', 'success');
            // Remove comment from UI
            const commentElement = document.querySelector(`[data-comment-id="${commentId}"]`);
            if (commentElement) {
                commentElement.remove();
            }
        } else {
            const data = await response.json();
            showToast(data.message || 'Failed to delete comment', 'error');
        }
    } catch (error) {
        console.error('Delete comment error:', error);
        showToast('Failed to delete comment. Please try again.', 'error');
    }
}

function confirmBulkDelete() {
    const selectedItems = document.querySelectorAll('.delete-checkbox:checked');
    const postCount = Array.from(selectedItems).filter(item => item.dataset.type === 'post').length;
    const commentCount = Array.from(selectedItems).filter(item => item.dataset.type === 'comment').length;
    
    let confirmMessage = 'Are you sure you want to delete ';
    if (postCount > 0 && commentCount > 0) {
        confirmMessage += `${postCount} post(s) and ${commentCount} comment(s)?`;
    } else if (postCount > 0) {
        confirmMessage += `${postCount} post(s)?`;
    } else {
        confirmMessage += `${commentCount} comment(s)?`;
    }
    
    if (confirm(confirmMessage)) {
        bulkDelete();
    }
}

async function bulkDelete() {
    const selectedItems = document.querySelectorAll('.delete-checkbox:checked');
    const deletePromises = [];
    
    selectedItems.forEach(item => {
        const type = item.dataset.type;
        const id = item.dataset.id;
        
        if (type === 'post') {
            deletePromises.push(deletePostSilent(id));
        } else if (type === 'comment') {
            deletePromises.push(deleteCommentSilent(id));
        }
    });
    
    try {
        await Promise.all(deletePromises);
        showToast(`Successfully deleted ${selectedItems.length} item(s)`, 'success');
        clearAllSelections();
        loadPosts(true); // Refresh the view
    } catch (error) {
        console.error('Bulk delete error:', error);
        showToast('Some items could not be deleted. Please try again.', 'error');
    }
}

// Silent delete functions for bulk operations (no individual confirmations)
async function deletePostSilent(postId) {
    const response = await fetch(`${API_BASE}/posts/${postId}`, {
        method: 'DELETE',
        headers: {
            'Authorization': `Bearer ${authToken}`,
            'Content-Type': 'application/json'
        }
    });
    
    if (!response.ok) {
        throw new Error(`Failed to delete post ${postId}`);
    }
}

async function deleteCommentSilent(commentId) {
    const response = await fetch(`${API_BASE}/comments/${commentId}`, {
        method: 'DELETE',
        headers: {
            'Authorization': `Bearer ${authToken}`,
            'Content-Type': 'application/json'
        }
    });
    
    if (!response.ok) {
        throw new Error(`Failed to delete comment ${commentId}`);
    }
}