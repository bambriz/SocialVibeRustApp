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

// === POSTS CACHE SYSTEM ===

// Cache configuration
const CACHE_CONFIG = {
    maxPostsPerView: 500,        // Maximum posts to cache per view
    maxTotalMemoryMB: 50,        // Rough memory limit in MB
    staleTimeMinutes: 10,        // Cache freshness time
    preloadPages: 2              // Pages to preload ahead
};

// Posts cache class for intelligent caching and filtering
class PostsCache {
    constructor() {
        this.caches = new Map(); // viewKey -> ViewCache
        this.lastCleanup = Date.now();
        this.cleanupInterval = 5 * 60 * 1000; // 5 minutes
    }

    // Get or create cache for a specific view
    getViewCache(viewType, userId = null) {
        const viewKey = viewType === 'user_home' && userId ? `user_${userId}` : 'main_feed';
        
        if (!this.caches.has(viewKey)) {
            this.caches.set(viewKey, new ViewCache(viewKey, viewType));
        }
        
        return this.caches.get(viewKey);
    }

    // Get all cached posts across views for global filtering
    getAllCachedPosts() {
        const allPosts = [];
        for (const cache of this.caches.values()) {
            allPosts.push(...cache.getAllPosts());
        }
        return allPosts;
    }

    // Clear all caches (hard refresh)
    clearAll() {
        this.caches.clear();
        console.log('🗑️ Cache: All caches cleared');
    }

    // Clear specific view cache
    clearView(viewType, userId = null) {
        const viewKey = viewType === 'user_home' && userId ? `user_${userId}` : 'main_feed';
        this.caches.delete(viewKey);
        console.log(`🗑️ Cache: Cleared cache for ${viewKey}`);
    }

    // Memory management - periodic cleanup with global memory enforcement
    performCleanup() {
        const now = Date.now();
        if (now - this.lastCleanup < this.cleanupInterval) return;

        console.log('🧹 Cache: Performing cleanup...');
        
        // Clean each view cache first
        for (const [viewKey, cache] of this.caches.entries()) {
            const sizeBefore = cache.posts.size;
            cache.cleanup();
            const sizeAfter = cache.posts.size;
            
            if (sizeBefore > sizeAfter) {
                console.log(`🧹 Cache: ${viewKey} cleaned ${sizeBefore - sizeAfter} posts`);
            }
        }

        // Enforce global memory limit
        this.enforceGlobalMemoryLimit();

        this.lastCleanup = now;
    }

    // Enforce global memory limit across all views
    enforceGlobalMemoryLimit() {
        let stats = this.getStats();
        if (stats.memoryEstimateMB <= CACHE_CONFIG.maxTotalMemoryMB) return;

        console.log(`🧹 Cache: Memory limit exceeded (${stats.memoryEstimateMB.toFixed(2)}MB > ${CACHE_CONFIG.maxTotalMemoryMB}MB), enforcing global cleanup`);

        // Loop until memory is below limit or no caches remain
        let attempts = 0;
        const maxAttempts = 10; // Safety valve to prevent infinite loops
        
        while (stats.memoryEstimateMB > CACHE_CONFIG.maxTotalMemoryMB && stats.totalPosts > 0 && attempts < maxAttempts) {
            // Sort views by last access and clean least recently used first
            const viewsByAccess = Array.from(this.caches.entries())
                .filter(([_, cache]) => cache.posts.size > 0) // Only process views with posts
                .sort((a, b) => a[1].lastAccess - b[1].lastAccess);
            
            if (viewsByAccess.length === 0) break;
            
            // Calculate adaptive removal size based on memory overage
            const overageMB = stats.memoryEstimateMB - CACHE_CONFIG.maxTotalMemoryMB;
            const avgPostSizeMB = stats.memoryEstimateMB / stats.totalPosts;
            const targetRemoval = Math.max(10, Math.ceil(overageMB / avgPostSizeMB));
            
            // Remove from least recently used view
            const [viewKey, cache] = viewsByAccess[0];
            const postsToRemove = Math.min(targetRemoval, cache.posts.size);
            cache.forceCleanup(postsToRemove);
            
            console.log(`🧹 Cache: Global cleanup removed ${postsToRemove} posts from ${viewKey} (${overageMB.toFixed(2)}MB overage)`);
            
            // Recalculate stats and increment attempt counter
            stats = this.getStats();
            attempts++;
        }
        
        if (attempts >= maxAttempts) {
            console.warn('🚨 Cache: Max cleanup attempts reached, may still exceed memory limit');
        } else if (stats.memoryEstimateMB <= CACHE_CONFIG.maxTotalMemoryMB) {
            console.log(`✅ Cache: Memory now within limit (${stats.memoryEstimateMB.toFixed(2)}MB)`);
        }
    }

    // Get cache statistics
    getStats() {
        const stats = {
            totalViews: this.caches.size,
            totalPosts: 0,
            memoryEstimateMB: 0,
            views: {}
        };

        for (const [viewKey, cache] of this.caches.entries()) {
            const viewStats = cache.getStats();
            stats.totalPosts += viewStats.postCount;
            stats.memoryEstimateMB += viewStats.memoryEstimateMB;
            stats.views[viewKey] = viewStats;
        }

        return stats;
    }
}

// View-specific cache (main feed, user posts, etc.)
class ViewCache {
    constructor(viewKey, viewType) {
        this.viewKey = viewKey;
        this.viewType = viewType;
        this.posts = new Map(); // postId -> post
        this.postOrder = []; // Array of post IDs in order
        this.paginationRanges = new Set(); // Track loaded ranges "offset-limit"
        this.lastAccess = Date.now();
        this.lastFetch = 0;
        this.hasMore = true;
    }

    // Add posts to cache
    addPosts(newPosts, offset, limit) {
        const now = Date.now();
        this.lastAccess = now;
        this.lastFetch = now;

        // Mark this range as loaded
        this.paginationRanges.add(`${offset}-${limit}`);

        // Add posts to cache
        newPosts.forEach(post => {
            if (!this.posts.has(post.id)) {
                this.posts.set(post.id, { ...post, _cached_at: now, _last_access: now });
                this.postOrder.push(post.id);
            }
        });

        // Update hasMore based on response (don't rely just on length check)
        // This will be properly set by the calling code with server response
        this.hasMore = newPosts.length === limit;

        console.log(`📦 Cache: Added ${newPosts.length} posts to ${this.viewKey} (total: ${this.posts.size})`);
    }

    // Check if a specific range is already cached
    hasRange(offset, limit) {
        return this.paginationRanges.has(`${offset}-${limit}`);
    }

    // Get posts for a specific range with LRU tracking
    getPostsInRange(offset, limit) {
        const now = Date.now();
        this.lastAccess = now;
        
        const endIndex = offset + limit;
        const requestedIds = this.postOrder.slice(offset, endIndex);
        
        // Update last access time for accessed posts (true LRU)
        return requestedIds.map(id => {
            const post = this.posts.get(id);
            if (post) {
                post._last_access = now;
            }
            return post;
        }).filter(Boolean);
    }

    // Get all cached posts with LRU tracking
    getAllPosts() {
        const now = Date.now();
        this.lastAccess = now;
        
        // Update last access time for all accessed posts (true LRU)
        return this.postOrder.map(id => {
            const post = this.posts.get(id);
            if (post) {
                post._last_access = now;
            }
            return post;
        });
    }

    // Check if cache is fresh
    isFresh() {
        const staleTime = CACHE_CONFIG.staleTimeMinutes * 60 * 1000;
        return (Date.now() - this.lastFetch) < staleTime;
    }

    // Clean up old posts (proper LRU based on last access)
    cleanup() {
        if (this.posts.size <= CACHE_CONFIG.maxPostsPerView) return;
        
        const toRemove = this.posts.size - CACHE_CONFIG.maxPostsPerView;
        this.forceCleanup(toRemove);
    }

    // Force cleanup of specific number of posts (for global memory management)
    forceCleanup(postsToRemove) {
        if (postsToRemove <= 0 || this.posts.size === 0) return;

        // Sort by last access time (true LRU), remove least recently used
        const postsArray = Array.from(this.posts.entries());
        postsArray.sort((a, b) => (a[1]._last_access || a[1]._cached_at || 0) - (b[1]._last_access || b[1]._cached_at || 0));

        const actualRemove = Math.min(postsToRemove, postsArray.length);
        for (let i = 0; i < actualRemove; i++) {
            const [postId] = postsArray[i];
            this.posts.delete(postId);
            
            // Also remove from order array
            const orderIndex = this.postOrder.indexOf(postId);
            if (orderIndex !== -1) {
                this.postOrder.splice(orderIndex, 1);
            }
        }

        // Clear pagination ranges since we've modified the cache
        this.paginationRanges.clear();
    }

    // Update hasMore from server response
    updateHasMore(serverHasMore) {
        this.hasMore = serverHasMore;
    }

    // Clear all posts from this view cache
    clear() {
        this.posts.clear();
        this.postOrder = [];
        this.paginationRanges.clear();
        this.lastAccess = Date.now();
        this.lastFetch = 0;
        this.hasMore = true;
        console.log(`🗑️ Cache: Cleared view cache for ${this.viewKey}`);
    }

    // Get cache statistics
    getStats() {
        const postCount = this.posts.size;
        const avgPostSize = 2000; // Rough estimate: 2KB per post
        const memoryEstimateMB = (postCount * avgPostSize) / (1024 * 1024);

        return {
            viewKey: this.viewKey,
            postCount,
            memoryEstimateMB: Math.round(memoryEstimateMB * 100) / 100,
            ranges: this.paginationRanges.size,
            hasMore: this.hasMore,
            isFresh: this.isFresh(),
            lastAccess: new Date(this.lastAccess).toLocaleTimeString()
        };
    }
}

// Initialize global cache
const postsCache = new PostsCache();

// Enhanced function to load comments for multiple posts efficiently
async function loadCommentsForPosts(posts) {
    const startTime = performance.now();
    console.log(`🔄 COMMENT_BATCH_LOAD: Starting batch comment loading for ${posts.length} posts`);
    
    // Create promises for all comment fetches
    const commentPromises = posts.map(async (post) => {
        try {
            // Check if comments are already cached and fresh
            if (commentsCache.isFresh(post.id)) {
                const cachedComments = commentsCache.get(post.id);
                if (cachedComments) {
                    post._comments = cachedComments; // Attach to post object
                    loadedComments.add(post.id);
                    return { postId: post.id, success: true, cached: true };
                }
            }
            
            // Fetch comments from API
            const response = await fetch(`${API_BASE}/posts/${post.id}/comments`);
            const data = await response.json();
            
            if (response.ok) {
                // Fix: API returns array directly, not wrapped in { comments: [] }
                const comments = Array.isArray(data) ? data : data.comments || [];
                
                console.log(`🔍 COMMENT_DEBUG: Post ${post.id} - API returned ${comments.length} comments`);
                
                // Cache the comments
                commentsCache.set(post.id, comments);
                loadedComments.add(post.id);
                
                // Attach comments directly to post object for unified caching
                post._comments = comments;
                
                return { postId: post.id, success: true, count: comments.length, cached: false };
            } else {
                console.warn(`⚠️ Failed to load comments for post ${post.id}:`, data);
                return { postId: post.id, success: false, error: data.message };
            }
        } catch (error) {
            console.error(`❌ Error loading comments for post ${post.id}:`, error);
            return { postId: post.id, success: false, error: error.message };
        }
    });
    
    // Wait for all comment fetches to complete
    const results = await Promise.all(commentPromises);
    
    // Log results
    const successful = results.filter(r => r.success);
    const cached = results.filter(r => r.cached);
    const fetched = results.filter(r => r.success && !r.cached);
    const failed = results.filter(r => !r.success);
    
    const duration = performance.now() - startTime;
    console.log(`✅ COMMENT_BATCH_LOAD: Completed in ${duration.toFixed(2)}ms`);
    console.log(`   📊 Results: ${successful.length}/${posts.length} successful`);
    console.log(`   🗂️  From cache: ${cached.length}, Fetched: ${fetched.length}, Failed: ${failed.length}`);
    
    if (fetched.length > 0) {
        const totalComments = fetched.reduce((sum, r) => sum + (r.count || 0), 0);
        console.log(`   💬 Total comments loaded: ${totalComments}`);
    }
    
    return results;
}

// Start automatic cache cleanup interval
setInterval(() => {
    postsCache.performCleanup();
}, 5 * 60 * 1000); // Every 5 minutes

// API Configuration - Use full domain for Replit environment
const API_BASE = window.location.origin + '/api';

// Initialize app
document.addEventListener('DOMContentLoaded', function() {
    loadContentFilterState(); // Load saved filter preferences first
    initializeApp();
    setupEventListeners();
    initializeCommentToggling(); // Initialize collapsible reply functionality
    
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
    
    // Cache debug function accessible from console
    window.debugCache = function() {
        const stats = postsCache.getStats();
        console.log('=== CACHE DEBUG INFO ===');
        console.log('Total views cached:', stats.totalViews);
        console.log('Total posts cached:', stats.totalPosts);
        console.log('Memory estimate:', stats.memoryEstimateMB.toFixed(2) + ' MB');
        console.log('Cache config:', CACHE_CONFIG);
        console.log('View details:');
        for (const [viewKey, viewStats] of Object.entries(stats.views)) {
            console.log(`  ${viewKey}:`, viewStats);
        }
        console.log('Current view:', currentView);
        console.log('Current posts array length:', posts.length);
        console.log('========================');
    };
    
    // Cache management functions accessible from console
    window.clearCache = function() {
        postsCache.clearAll();
        commentsCache.clearAll();
        console.log('🗑️ All caches cleared manually');
    };
    
    window.clearCurrentViewCache = function() {
        postsCache.clearView(currentView, currentUser?.id);
        console.log(`🗑️ Cache cleared for current view: ${currentView}`);
    };
    
    // Comments cache debug function
    window.debugCommentsCache = function() {
        const stats = commentsCache.getStats();
        console.log('=== COMMENTS CACHE DEBUG ===');
        console.log('Total posts with cached comments:', stats.totalPosts);
        console.log('Total cached comments:', stats.totalComments);
        console.log('Memory estimate:', stats.memoryEstimateMB.toFixed(2) + ' MB');
        console.log('Cache config:', COMMENTS_CACHE_CONFIG);
        console.log('============================');
    };
    
    window.clearCommentsCache = function() {
        commentsCache.clearAll();
        console.log('🗑️ Comments cache cleared manually');
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
    
    // Touch gesture detection for mobile swipe interactions
    document.addEventListener('touchstart', handleTouchStart, { passive: false });
    document.addEventListener('touchmove', handleTouchMove, { passive: false });
    document.addEventListener('touchend', handleTouchEnd, { passive: false });
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
    clearOptimisticVoteState(); // Clear optimistic voting state on logout
    postsCache.clearAll(); // Clear all cached data on logout
    commentsCache.clearAll(); // Clear comments cache on logout
    showGuestInterface();
    showToast('Logged out successfully', 'info');
}

// User home page functions
async function showUserHome() {
    if (!currentUser) {
        showToast('Please log in to view your posts', 'error');
        return;
    }
    
    const previousView = currentView;
    currentView = 'user_home';
    document.getElementById('feedTitle').textContent = `My Posts (${currentUser.username})`;
    // Hide vibe check component for user posts
    const vibeCheckComponent = document.getElementById('vibeCheckComponent');
    if (vibeCheckComponent) {
        vibeCheckComponent.style.display = 'none';
    }
    
    // Check if we're switching views and have cached data
    if (previousView !== currentView) {
        const cache = postsCache.getViewCache(currentView, currentUser.id);
        if (cache.posts.size > 0 && cache.isFresh()) {
            console.log('📦 Cache: Switching to user view with cached data');
            posts = cache.getAllPosts();
            paginationState.offset = posts.length;
            paginationState.hasMore = cache.hasMore;
            
            // Load vote data for cached posts to ensure freshness
            if (posts.length > 0) {
                await loadVoteDataForPosts(posts);
            }
            
            applyCurrentFilters();
            return;
        }
    }
    
    loadPosts(true); // Reset and load user's posts
    showToast('Showing your posts', 'info');
}

async function showMainFeed() {
    const previousView = currentView;
    currentView = 'feed';
    document.getElementById('feedTitle').textContent = 'Social Pulse Waves';
    // Show vibe check component for main feed
    const vibeCheckComponent = document.getElementById('vibeCheckComponent');
    if (vibeCheckComponent) {
        vibeCheckComponent.style.display = 'block';
    }
    
    // Check if we're switching views and have cached data
    if (previousView !== currentView) {
        const cache = postsCache.getViewCache(currentView, null);
        if (cache.posts.size > 0 && cache.isFresh()) {
            console.log('📦 Cache: Switching to main feed with cached data');
            posts = cache.getAllPosts();
            paginationState.offset = posts.length;
            paginationState.hasMore = cache.hasMore;
            
            // Load vote data for cached posts to ensure freshness
            if (posts.length > 0) {
                await loadVoteDataForPosts(posts);
            }
            
            applyCurrentFilters();
            return;
        }
    }
    
    loadPosts(true); // Reset and load main feed
    showToast('Showing main feed', 'info');
}

function showUserInterface() {
    // Desktop navigation
    document.getElementById('navActions').classList.add('hidden');
    document.getElementById('navUser').classList.remove('hidden');
    document.getElementById('navUsername').textContent = `Hello, ${currentUser.username}!`;
    
    // Mobile navigation - sync with desktop
    document.getElementById('mobileNavActions').classList.add('hidden');
    document.getElementById('mobileNavUser').classList.remove('hidden');
    document.getElementById('mobileNavUsername').textContent = `Hello, ${currentUser.username}!`;
    
    document.getElementById('postCreator').classList.remove('hidden');
    
    // Update sticky positions after UI change that could affect navbar height
    setTimeout(updateStickyPositions, 100);
}

// Toggle post creator between collapsed and expanded states
function togglePostCreator() {
    const postCreator = document.getElementById('postCreator');
    const isCollapsed = postCreator.classList.contains('collapsed');
    
    if (isCollapsed) {
        // Expanding
        postCreator.classList.remove('collapsed');
        postCreator.classList.add('expanded');
        // Focus on the title input when expanded
        setTimeout(() => {
            document.getElementById('postTitle').focus();
        }, 300);
        
        // Update sticky positioning dynamically based on actual element heights
        updateStickyPositions();
    } else {
        // Collapsing with smooth animation
        postCreator.classList.add('collapsing');
        postCreator.classList.remove('expanded');
        
        // Clear form when collapsing
        document.getElementById('postForm').reset();
        document.getElementById('sentimentPreview').textContent = '';
        
        // Wait for animation to complete, then apply collapsed state
        setTimeout(() => {
            postCreator.classList.remove('collapsing');
            postCreator.classList.add('collapsed');
            updateStickyPositions();
        }, 300);
    }
}

// Toggle vibe check component between collapsed and expanded states
function toggleVibeCheck() {
    const vibeCheckComponent = document.getElementById('vibeCheckComponent');
    const isCollapsed = vibeCheckComponent.classList.contains('collapsed');
    
    if (isCollapsed) {
        // Expanding
        vibeCheckComponent.classList.remove('collapsed');
        vibeCheckComponent.classList.add('expanded');
        
        // Update sticky positioning dynamically based on actual element heights
        updateStickyPositions();
    } else {
        // Collapsing with smooth animation
        vibeCheckComponent.classList.add('collapsing');
        vibeCheckComponent.classList.remove('expanded');
        
        // Wait for animation to complete, then apply collapsed state
        setTimeout(() => {
            vibeCheckComponent.classList.remove('collapsing');
            vibeCheckComponent.classList.add('collapsed');
            updateStickyPositions();
        }, 300);
    }
}

// Dynamically update sticky positioning based on actual element heights
function updateStickyPositions() {
    const navbar = document.querySelector('.navbar');
    const postCreator = document.getElementById('postCreator');
    const vibeCheckComponent = document.getElementById('vibeCheckComponent');
    
    if (navbar && postCreator) {
        const navbarHeight = navbar.offsetHeight;
        const postCreatorHeight = postCreator.offsetHeight;
        const vibeCheckHeight = vibeCheckComponent ? vibeCheckComponent.offsetHeight : 0;
        
        // Update CSS variables for responsive sticky positioning
        document.documentElement.style.setProperty('--post-creator-top', `${navbarHeight}px`);
        document.documentElement.style.setProperty('--vibe-check-top', `${navbarHeight + postCreatorHeight + 10}px`);
        document.documentElement.style.setProperty('--feed-header-top', `${navbarHeight + postCreatorHeight + vibeCheckHeight + 30}px`);
    }
}

// Initialize sticky positions on page load and window resize
window.addEventListener('load', updateStickyPositions);
window.addEventListener('resize', updateStickyPositions);

function showGuestInterface() {
    // Desktop navigation
    document.getElementById('navActions').classList.remove('hidden');
    document.getElementById('navUser').classList.add('hidden');
    
    // Mobile navigation - sync with desktop
    document.getElementById('mobileNavActions').classList.remove('hidden');
    document.getElementById('mobileNavUser').classList.add('hidden');
    
    document.getElementById('postCreator').classList.add('hidden');
    
    // Update sticky positions after UI change that could affect navbar height
    setTimeout(updateStickyPositions, 100);
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
    
    // Create optimistic post
    const optimisticPost = {
        id: `temp_${Date.now()}`,
        title: title,
        content: content,
        author_username: currentUser.username,
        author_id: currentUser.id,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        sentiment_analysis: { sentiment_type: null },
        moderation_result: { is_blocked: false, toxicity_tags: [] },
        popularity_score: 8.5, // Realistic score for brand new posts (will be corrected by server)
        comment_count: 0,
        isPending: true
    };
    
    // Clear form and collapse creator
    document.getElementById('postForm').reset();
    document.getElementById('sentimentPreview').textContent = '';
    
    const postCreator = document.getElementById('postCreator');
    if (postCreator.classList.contains('expanded')) {
        postCreator.classList.add('collapsing');
        postCreator.classList.remove('expanded');
        
        setTimeout(() => {
            postCreator.classList.remove('collapsing');
            postCreator.classList.add('collapsed');
            updateStickyPositions();
        }, 300);
    }
    
    // Add optimistic post to UI immediately
    addOptimisticPost(optimisticPost);
    showToast('📝 Creating post...', 'info');
    
    // Fire-and-forget async database save - don't wait for response
    savePostToDatabase(optimisticPost.id, title, content);
}

// Async database operations - eventual consistency pattern
async function savePostToDatabase(optimisticId, title, content) {
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
            // Mark post as saved and trigger refresh from database
            markPostAsSaved(optimisticId, data.post);
            showToast('✅ Post saved to database!', 'success');
            
            // Refresh data from database to ensure consistency
            await refreshPostsFromDatabase();
        } else {
            console.log('Database save failed:', data);
            markPostAsFailed(optimisticId, data);
            
            // Check if this is a content moderation error
            if (data.error_type === 'content_moderation') {
                showToast('❌ ' + (data.error || 'Post blocked due to content violation'), 'error');
            } else {
                showToast('❌ Failed to save post to database', 'error');
            }
        }
    } catch (error) {
        console.error('Database save error:', error);
        markPostAsFailed(optimisticId, { error: 'Network error' });
        showToast('❌ Network error saving post', 'error');
    }
}

async function saveCommentToDatabase(optimisticId, postId, content, parentId = null) {
    console.log('💾 DATABASE_SAVE_DIAGNOSTIC: Starting comment save to database');
    console.log(`   🆔 Optimistic ID: ${optimisticId}`);
    console.log(`   📍 Post ID: ${postId}`);
    console.log(`   📄 Content Length: ${content.length}`);
    console.log(`   👤 Parent ID: ${parentId || 'null (root comment)'}`);
    
    const startTime = performance.now();
    
    try {
        const url = `${API_BASE}/posts/${postId}/comments`;
        console.log(`   🌐 Request URL: ${url}`);
        
        const requestBody = { content, post_id: postId, parent_id: parentId };
        console.log('   📦 Request Body:', requestBody);
        
        console.log('   🚀 Sending HTTP POST request...');
        const response = await fetch(url, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify(requestBody)
        });
        
        const requestDuration = performance.now() - startTime;
        console.log(`   ⏱️  Request completed in ${requestDuration.toFixed(2)}ms`);
        console.log(`   📊 Response Status: ${response.status} ${response.statusText}`);
        console.log('   📋 Response Headers:', Object.fromEntries(response.headers.entries()));
        
        let data;
        try {
            console.log('   🔄 Parsing JSON response...');
            data = await response.json();
            console.log('   ✅ JSON parsed successfully');
            console.log('   📝 Comment Response Data:', {
                success: data.success,
                comment_id: data.comment?.id,
                message: data.message,
                comment_created_at: data.comment?.created_at,
                sentiment_type: data.comment?.sentiment_type
            });
        } catch (jsonError) {
            console.error('   ❌ JSON parsing failed:', jsonError);
            const responseText = await response.text();
            console.error('   📄 Raw Response Text:', responseText.substring(0, 500));
            throw new Error(`Server returned non-JSON response (${response.status}): ${responseText.substring(0, 100)}...`);
        }
        
        if (response.ok) {
            console.log('   ✅ Database save successful!');
            
            const totalDuration = performance.now() - startTime;
            console.log(`   ⏱️  Total save operation completed in ${totalDuration.toFixed(2)}ms`);
            
            // Replace optimistic comment with real data
            console.log('   🔄 Replacing optimistic comment with real data...');
            replaceOptimisticComment(postId, optimisticId, data.comment);
            markCommentAsSaved(optimisticId, postId, data.comment);
            showToast('✅ Comment saved to database!', 'success');
            
            // Update cache with new comment but don't refresh UI (it's already updated)
            console.log('   🗃️  Updating comment cache...');
            const comments = commentsCache.get(postId) || [];
            comments.unshift({
                comment: data.comment,
                author: currentUser.username,
                can_modify: true,
                is_collapsed: false,
                replies: []
            });
            commentsCache.set(postId, comments);
            
            console.log('   ✅ Comment cache updated successfully');
            console.log('   🎯 DIAGNOSTIC SUMMARY: Comment save completed successfully');
            
        } else {
            console.error('   ❌ Database save failed!');
            console.error('   📊 Response not OK:', response.status, response.statusText);
            console.error('   📄 Error Data:', data);
            
            markCommentAsFailed(optimisticId, postId, data);
            showToast(`❌ Failed to save comment: ${data.message || 'Unknown error'}`, 'error');
        }
    } catch (error) {
        const totalDuration = performance.now() - startTime;
        console.error('   💥 CRITICAL ERROR in comment database save:');
        console.error('   🕐 Error occurred after:', totalDuration.toFixed(2) + 'ms');
        console.error('   🔍 Error Details:', error);
        console.error('   📚 Error Stack:', error.stack);
        
        markCommentAsFailed(optimisticId, postId, { error: error.message });
        showToast(`❌ Network error saving comment: ${error.message}`, 'error');
    }
}

// Eventual consistency helpers
function markPostAsSaved(optimisticId, realPost) {
    const optimisticElement = document.querySelector(`[data-post-id="${optimisticId}"]`);
    if (optimisticElement) {
        optimisticElement.classList.add('saved');
        optimisticElement.classList.remove('post-pending');
        
        // Update posts array
        const postIndex = posts.findIndex(p => p.id === optimisticId);
        if (postIndex !== -1) {
            posts[postIndex] = { ...realPost, isSaved: true };
        }
    }
}

function markPostAsFailed(optimisticId, errorData) {
    const optimisticElement = document.querySelector(`[data-post-id="${optimisticId}"]`);
    if (optimisticElement) {
        optimisticElement.classList.add('failed');
        optimisticElement.title = 'Failed to save: ' + (errorData.error || 'Unknown error');
        
        // Add retry button
        const postActions = optimisticElement.querySelector('.post-actions');
        if (postActions && !postActions.querySelector('.retry-btn')) {
            postActions.insertAdjacentHTML('beforeend', `
                <button class="retry-btn" onclick="retryPostSave('${optimisticId}')">🔄 Retry Save</button>
            `);
        }
    }
}

function markCommentAsSaved(optimisticId, postId, realComment) {
    const optimisticElement = document.querySelector(`[data-comment-id="${optimisticId}"]`);
    if (optimisticElement) {
        // Update the data attribute to use the real comment ID
        optimisticElement.setAttribute('data-comment-id', realComment.id);
        optimisticElement.classList.add('saved');
        optimisticElement.classList.remove('comment-pending');
        
        console.log(`✅ Comment ${optimisticId} marked as saved with real ID ${realComment.id}`);
    }
}

function markCommentAsFailed(optimisticId, postId, errorData) {
    const optimisticElement = document.querySelector(`[data-comment-id="${optimisticId}"]`);
    if (optimisticElement) {
        optimisticElement.classList.remove('comment-pending');
        optimisticElement.classList.add('failed');
        optimisticElement.title = 'Failed to save: ' + (errorData.error || 'Unknown error');
        
        // Add retry button to comment actions
        const commentActions = optimisticElement.querySelector('.comment-actions');
        if (commentActions && !commentActions.querySelector('.retry-btn')) {
            const retryBtn = document.createElement('button');
            retryBtn.className = 'retry-btn';
            retryBtn.textContent = '🔄 Retry Save';
            retryBtn.onclick = () => retryCommentSave(optimisticId, postId);
            commentActions.appendChild(retryBtn);
        }
    }
}

// Database refresh functions - only called after saves are confirmed
async function refreshPostsFromDatabase() {
    console.log('🔄 Refreshing posts from database after save confirmation');
    
    // Clear cache to force fresh fetch
    postsCache.clearAll();
    
    // Reload current view from database
    if (currentView === 'main_feed') {
        await loadPostsFeed(true); // Force refresh
    } else if (currentView === 'user_home' && currentUser) {
        await loadUserPosts(currentUser.id, true); // Force refresh
    }
}

// Feed loading function - wrapper around loadPosts for main feed loading
async function loadPostsFeed(reset = true) {
    console.log(`🔄 Loading main feed posts (reset: ${reset})`);
    
    // Ensure we're in main feed view
    const previousView = currentView;
    currentView = 'feed';
    
    try {
        await loadPosts(reset);
    } catch (error) {
        console.error('❌ Failed to load feed posts:', error);
        currentView = previousView;
        throw error;
    }
}

// User posts loading function - wrapper around loadPosts for user-specific loading
async function loadUserPosts(userId, reset = true) {
    console.log(`🔄 Loading posts for user ${userId} (reset: ${reset})`);
    
    // Ensure we're in user_home view
    const previousView = currentView;
    currentView = 'user_home';
    
    try {
        // Use the main loadPosts function which handles user view correctly
        await loadPosts(reset);
    } catch (error) {
        console.error('❌ Failed to load user posts:', error);
        // Restore previous view on error
        currentView = previousView;
        throw error;
    }
}

async function refreshCommentsFromDatabase(postId) {
    console.log(`🔄 Refreshing comments for post ${postId} from database after save confirmation`);
    
    // Clear comments cache for this post to force fresh fetch
    commentsCache.clear(postId);
    loadedComments.delete(postId);
    
    // Add small delay to ensure database consistency
    setTimeout(async () => {
        try {
            // Force reload comments from database
            await loadComments(postId);
            console.log(`✅ Comments refreshed successfully for post ${postId}`);
        } catch (error) {
            console.error(`❌ Failed to refresh comments for post ${postId}:`, error);
        }
    }, 500);
}

// Retry mechanisms with exponential backoff
const retryQueues = {
    posts: new Map(),
    comments: new Map()
};

async function retryPostSave(optimisticId) {
    const post = posts.find(p => p.id === optimisticId);
    if (post) {
        // Remove failed styling
        const element = document.querySelector(`[data-post-id="${optimisticId}"]`);
        if (element) {
            element.classList.remove('failed');
            element.classList.add('post-pending');
            
            // Remove retry button
            const retryBtn = element.querySelector('.retry-btn');
            if (retryBtn) retryBtn.remove();
        }
        
        showToast('🔄 Retrying post save...', 'info');
        
        // Retry save with exponential backoff
        await savePostToDatabase(optimisticId, post.title, post.content);
    }
}

async function retryCommentSave(optimisticId, postId) {
    // Find comment content from the UI since we don't store comment data
    const commentElement = document.querySelector(`[data-comment-id="${optimisticId}"]`);
    if (commentElement) {
        const contentElement = commentElement.querySelector('.comment-content');
        const content = contentElement ? contentElement.textContent.trim() : '';
        
        if (content) {
            // Remove failed styling
            commentElement.classList.remove('failed');
            commentElement.classList.add('comment-pending');
            
            // Remove retry button
            const retryBtn = commentElement.querySelector('.retry-btn');
            if (retryBtn) retryBtn.remove();
            
            showToast('🔄 Retrying comment save...', 'info');
            
            // Retry save
            await saveCommentToDatabase(optimisticId, postId, content);
        }
    }
}

// Automatic retry with exponential backoff for network errors
async function autoRetryWithBackoff(operation, maxRetries = 3, baseDelay = 1000) {
    for (let attempt = 1; attempt <= maxRetries; attempt++) {
        try {
            return await operation();
        } catch (error) {
            console.warn(`Attempt ${attempt}/${maxRetries} failed:`, error);
            
            if (attempt === maxRetries) {
                throw error; // Final attempt failed
            }
            
            // Exponential backoff: 1s, 2s, 4s
            const delay = baseDelay * Math.pow(2, attempt - 1);
            await new Promise(resolve => setTimeout(resolve, delay));
        }
    }
}

// Optimistic post helper functions
function addOptimisticPost(post) {
    const container = document.getElementById('postsList');
    if (!container) return;
    
    // Create post HTML with pending styling
    const postHTML = createOptimisticPostHTML(post);
    
    // Add to top of posts list
    container.insertAdjacentHTML('afterbegin', postHTML);
    
    // Add to posts array
    posts.unshift(post);
    
    // Apply pending styling explicitly
    setTimeout(() => {
        const optimisticElement = document.querySelector(`[data-post-id="${post.id}"]`);
        if (optimisticElement) {
            optimisticElement.classList.add('post-pending');
        }
    }, 10);
}

function replaceOptimisticPost(tempId, realPost) {
    const tempElement = document.querySelector(`[data-post-id="${tempId}"]`);
    if (tempElement && realPost) {
        // Find and update in posts array
        const postIndex = posts.findIndex(p => p.id === tempId);
        if (postIndex !== -1) {
            posts[postIndex] = realPost;
        }
        
        // Replace in DOM using createPostElement
        const realElement = createPostElement(realPost);
        tempElement.replaceWith(realElement);
        
        // Setup event listeners for the new element
        setupPostEventListeners();
    }
}

function removeOptimisticPost(tempId) {
    const tempElement = document.querySelector(`[data-post-id="${tempId}"]`);
    if (tempElement) {
        tempElement.remove();
        
        // Remove from posts array
        const postIndex = posts.findIndex(p => p.id === tempId);
        if (postIndex !== -1) {
            posts.splice(postIndex, 1);
        }
    }
}

function createOptimisticPostHTML(post) {
    const timeAgo = formatTimeAgo(post.created_at);
    
    return `
        <article class="post-card post-pending" data-post-id="${escapeHtml(post.id)}">
            <div class="post-header">
                <div class="post-author-section">
                    <div class="post-author">${escapeHtml(post.author_username)}</div>
                    <div class="post-time">${timeAgo}</div>
                </div>
                <div class="post-header-right">
                    <span class="pending-indicator">⏳ Posting...</span>
                </div>
            </div>
            <h3 class="post-title">${escapeHtml(post.title)}</h3>
            <p class="post-content">${escapeHtml(post.content)}</p>
            <div class="post-footer">
                <div>Popularity: ${(post.popularity_score || 1.0).toFixed(1)}</div>
                <button class="comment-toggle" onclick="toggleComments('${escapeJavaScript(post.id)}')">
                    💬 0 comments
                </button>
            </div>
            
            <!-- Comments Section -->
            <div id="comments-${escapeHtml(post.id)}" class="comments-section hidden">
                <div class="comments-list">
                    <div class="no-comments">No comments yet. Be the first to comment!</div>
                </div>
            </div>
        </article>
    `;
}

// Cache-aware posts loading with smart caching and filtering
async function loadPosts(reset = true) {
    if (paginationState.isLoading) return;
    
    paginationState.isLoading = true;
    
    // Get appropriate cache for current view
    const cache = postsCache.getViewCache(currentView, currentUser?.id);
    
    if (reset) {
        paginationState.offset = 0;
        paginationState.hasMore = true;
        posts = [];
        clearOptimisticVoteState(); // Clear optimistic state when resetting posts
        
        // Robust hard refresh detection - clear cache on hard refresh
        const isHardRefresh = (
            (performance.navigation && performance.navigation.type === performance.navigation.TYPE_RELOAD) ||
            (performance.getEntriesByType && performance.getEntriesByType('navigation')[0]?.type === 'reload') ||
            document.referrer === '' && window.history.length === 1
        );
        
        if (isHardRefresh) {
            console.log('🔄 Cache: Hard refresh detected, clearing cache');
            postsCache.clearAll();
        }
        
        // Check if we have fresh cached data for initial load
        if (cache.posts.size > 0 && cache.isFresh()) {
            console.log('📦 Cache: Using cached data for reset load');
            posts = cache.getAllPosts();
            paginationState.hasMore = cache.hasMore;
            paginationState.offset = posts.length;
            
            // Load vote data for cached posts to ensure freshness
            if (posts.length > 0) {
                await loadVoteDataForPosts(posts);
            }
            
            // Apply filters and render immediately from cache
            setTimeout(() => {
                applyCurrentFilters();
            }, 50);
            
            paginationState.isLoading = false;
            return;
        }
        
        // Show skeleton loading for initial load
        showSkeletonLoading(true);
    } else {
        // For infinite scroll, check if we already have this range cached
        if (cache.hasRange(paginationState.offset, paginationState.limit)) {
            console.log(`📦 Cache: Using cached data for range ${paginationState.offset}-${paginationState.limit}`);
            
            const cachedPosts = cache.getPostsInRange(paginationState.offset, paginationState.limit);
            posts = [...posts, ...cachedPosts];
            paginationState.offset += cachedPosts.length;
            paginationState.hasMore = cache.hasMore;
            
            // Load vote data for cached posts to keep them fresh
            if (cachedPosts.length > 0) {
                await loadVoteDataForPosts(cachedPosts);
            }
            
            // Apply filters and render
            setTimeout(() => {
                applyCurrentFilters();
            }, 50);
            
            paginationState.isLoading = false;
            return;
        }
        
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
        
        console.log(`🌐 Cache: Fetching from server - ${url}`);
        
        // Add a small delay for better UX (minimum loading time for skeletons to be visible)
        const [response] = await Promise.all([
            fetch(url),
            new Promise(resolve => setTimeout(resolve, reset ? 500 : 300))
        ]);
        
        const data = await response.json();
        
        if (response.ok) {
            const newPosts = Array.isArray(data) ? data : data.posts || [];
            
            // Add to cache with proper hasMore handling
            cache.addPosts(newPosts, paginationState.offset, paginationState.limit);
            
            // Update cache hasMore from server response
            const serverHasMore = data.has_more !== false && newPosts.length === paginationState.limit;
            cache.updateHasMore(serverHasMore);
            
            if (reset) {
                posts = newPosts;
            } else {
                posts = [...posts, ...newPosts];
            }
            
            // Load vote data for new posts
            if (newPosts.length > 0) {
                await loadVoteDataForPosts(newPosts);
            }
            
            // ENHANCED: Load comments for each post to enable full caching
            console.log(`📦 POST_COMMENT_INTEGRATION: Loading comments for ${newPosts.length} posts`);
            await loadCommentsForPosts(newPosts);
            console.log(`📦 POST_COMMENT_INTEGRATION: Comments attached to ${newPosts.filter(p => p._comments).length}/${newPosts.length} posts`);
            
            // Update comment counts for all posts after loading comments
            console.log(`🔄 POST_COMMENT_INTEGRATION: Updating comment counts for all posts`);
            newPosts.forEach(post => {
                const comments = commentsCache.get(post.id);
                if (comments) {
                    const rootCount = calculateRootCommentCount(comments);
                    updatePostCommentCountInUI(post.id, rootCount);
                }
            });
            
            // Update pagination state from server truth
            paginationState.hasMore = serverHasMore;
            paginationState.offset += newPosts.length;
            
            // Hide skeleton loading before showing real posts
            hideSkeletonLoading();
            
            // Small delay to allow skeleton removal animation to complete
            setTimeout(() => {
                applyCurrentFilters();
            }, 150);
            
            // Perform periodic cache cleanup
            postsCache.performCleanup();
            
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

// Apply current active filters (separated for reuse)
function applyCurrentFilters() {
    const activeFilterBtn = document.querySelector('.filter-btn.active');
    if (activeFilterBtn) {
        const sentiment = activeFilterBtn.dataset.filter;
        filterFeed(sentiment); // This applies both emotion and content filters
    } else {
        // If no active emotion filter, just apply content filters
        renderPosts(applyContentFiltering(posts));
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
            <input type="checkbox" class="delete-checkbox" data-type="post" data-id="${escapeHtml(post.id)}" 
                   onchange="toggleDeleteControls()">
            <button class="delete-icon" onclick="deletePost('${escapeJavaScript(post.id)}')" title="Delete Post">🗑️</button>
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
            <button class="comment-toggle" onclick="toggleComments('${escapeJavaScript(post.id)}')">
                💬 ${post.comment_count || 0} comments
            </button>
        </div>
        
        <!-- Comments Section -->
        <div id="comments-${escapeHtml(post.id)}" class="comments-section hidden" style="margin-top: 15px; padding-top: 15px; border-top: 1px solid rgba(255,255,255,0.1);">
            <div class="comment-form" ${!authToken ? 'style="display:none;"' : ''}>
                <textarea id="comment-input-${escapeHtml(post.id)}" placeholder="Share your thoughts..." 
                        class="comment-textarea" rows="3" maxlength="2000"></textarea>
                <div class="comment-form-actions">
                    <span class="comment-counter">0/2000</span>
                    <button onclick="postComment('${escapeJavaScript(post.id)}')" class="comment-submit-btn">Post Comment</button>
                </div>
            </div>
            <div id="comments-list-${escapeHtml(post.id)}" class="comments-list">
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
                <input type="checkbox" class="delete-checkbox" data-type="post" data-id="${escapeHtml(post.id)}" 
                       onchange="toggleDeleteControls()">
                <button class="delete-icon" onclick="deletePost('${escapeJavaScript(post.id)}')" title="Delete Post">🗑️</button>
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
                    <button class="comment-toggle" onclick="toggleComments('${escapeJavaScript(post.id)}')">
                        💬 ${post.comment_count || 0} comments
                    </button>
                </div>
                
                <!-- Comments Section -->
                <div id="comments-${escapeHtml(post.id)}" class="comments-section hidden" style="margin-top: 15px; padding-top: 15px; border-top: 1px solid rgba(255,255,255,0.1);">
                    <div class="comment-form" ${!authToken ? 'style="display:none;"' : ''}>
                        <textarea id="comment-input-${escapeHtml(post.id)}" placeholder="Share your thoughts..." 
                                class="comment-textarea" rows="3" maxlength="2000"></textarea>
                        <div class="comment-form-actions">
                            <span class="comment-counter">0/2000</span>
                            <button onclick="postComment('${escapeJavaScript(post.id)}')" class="comment-submit-btn">Post Comment</button>
                        </div>
                    </div>
                    <div id="comments-list-${escapeHtml(post.id)}" class="comments-list">
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
    const count = replace ? 6 : 3;
    
    if (replace) {
        container.replaceChildren();
    }
    
    // Create skeleton elements safely using DOM methods
    for (let index = 0; index < count; index++) {
        const skeletonPost = document.createElement('div');
        skeletonPost.className = 'skeleton-post';
        skeletonPost.id = `skeleton-${index}`;
        
        // Create header section
        const header = document.createElement('div');
        header.className = 'skeleton-header';
        ['skeleton-avatar', 'skeleton-author', 'skeleton-time'].forEach(cls => {
            const elem = document.createElement('div');
            elem.className = `skeleton-element ${cls}`;
            header.appendChild(elem);
        });
        skeletonPost.appendChild(header);
        
        // Create title
        const title = document.createElement('div');
        title.className = 'skeleton-element skeleton-title';
        skeletonPost.appendChild(title);
        
        // Create content lines
        for (let i = 0; i < 3; i++) {
            const content = document.createElement('div');
            content.className = 'skeleton-element skeleton-content';
            skeletonPost.appendChild(content);
        }
        
        // Create footer section
        const footer = document.createElement('div');
        footer.className = 'skeleton-footer';
        ['skeleton-badge', 'skeleton-comment-btn'].forEach(cls => {
            const elem = document.createElement('div');
            elem.className = `skeleton-element ${cls}`;
            footer.appendChild(elem);
        });
        skeletonPost.appendChild(footer);
        
        container.appendChild(skeletonPost);
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
        
        // Create spinner element
        const spinner = document.createElement('div');
        spinner.className = 'enhanced-spinner';
        loader.appendChild(spinner);
        
        // Create loading text element
        const loadingText = document.createElement('div');
        loadingText.className = 'loading-text loading-dots';
        loadingText.textContent = 'Loading more posts';
        loader.appendChild(loadingText);
        
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

// Pull-to-refresh visual indicators
function showPullToRefreshIndicator() {
    let indicator = document.getElementById('pullToRefreshIndicator');
    if (!indicator) {
        indicator = document.createElement('div');
        indicator.id = 'pullToRefreshIndicator';
        indicator.className = 'pull-to-refresh-indicator';
        
        // Create arrow element
        const arrow = document.createElement('div');
        arrow.className = 'pull-refresh-arrow';
        arrow.textContent = '↓';
        indicator.appendChild(arrow);
        
        // Create text element
        const text = document.createElement('div');
        text.className = 'pull-refresh-text';
        text.textContent = 'Release to refresh';
        indicator.appendChild(text);
        
        document.body.insertBefore(indicator, document.body.firstChild);
    }
    indicator.classList.remove('hidden');
    indicator.classList.add('active');
}

function hidePullToRefreshIndicator() {
    const indicator = document.getElementById('pullToRefreshIndicator');
    if (indicator) {
        indicator.classList.remove('active');
        indicator.classList.add('hidden');
    }
}

// Swipe-to-load visual indicators
function showSwipeToLoadIndicator() {
    let indicator = document.getElementById('swipeToLoadIndicator');
    if (!indicator) {
        indicator = document.createElement('div');
        indicator.id = 'swipeToLoadIndicator';
        indicator.className = 'swipe-to-load-indicator';
        indicator.innerHTML = `
            <div class="swipe-load-arrow">↑</div>
            <div class="swipe-load-text">Loading more...</div>
        `;
        const postsList = document.getElementById('postsList');
        if (postsList) {
            postsList.appendChild(indicator);
        } else {
            console.warn('Posts list container not found for swipe indicator');
            return;
        }
    }
    indicator.classList.remove('hidden');
    indicator.classList.add('active');
}

function hideSwipeToLoadIndicator() {
    const indicator = document.getElementById('swipeToLoadIndicator');
    if (indicator) {
        indicator.classList.remove('active');
        setTimeout(() => {
            if (indicator && indicator.classList.contains('hidden') === false) {
                indicator.classList.add('hidden');
            }
        }, 1000); // Keep visible for 1 second after swipe
    }
}

// Refresh feed function for pull-to-refresh
async function refreshFeed() {
    console.log('🔄 Refreshing feed via pull-to-refresh');
    
    // Clear current view cache to ensure fresh data
    const cache = postsCache.getViewCache(currentView, currentUser?.id);
    cache.clear();
    
    // Reset pagination and reload posts
    paginationState.offset = 0;
    paginationState.hasMore = true;
    
    // Show refreshing indicator
    showToast('Refreshing feed...', 'info');
    
    // Load fresh posts
    await loadPosts(true);
    
    showToast('Feed refreshed!', 'success');
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

// Touch/Swipe gesture handling for mobile-friendly feed navigation
let touchStartY = null;
let touchStartX = null;
let touchStartTime = null;
let isPullToRefreshActive = false;
let isSwipeToLoadActive = false;
let pullToRefreshThreshold = 80; // Minimum distance for pull-to-refresh
let swipeToLoadThreshold = 60; // Minimum distance for swipe-to-load

function handleTouchStart(e) {
    const touch = e.touches[0];
    touchStartY = touch.clientY;
    touchStartX = touch.clientX;
    touchStartTime = Date.now();
}

function handleTouchMove(e) {
    if (!touchStartY || !touchStartX) return;

    const touch = e.touches[0];
    const deltaY = touch.clientY - touchStartY;
    const deltaX = touch.clientX - touchStartX;
    const absDeltaY = Math.abs(deltaY);
    const absDeltaX = Math.abs(deltaX);

    // Only process vertical swipes (ignore horizontal swipes)
    if (absDeltaX > absDeltaY) return;

    const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
    const windowHeight = window.innerHeight;
    const documentHeight = document.documentElement.scrollHeight;

    // Pull-to-refresh at the top (swipe down)
    if (deltaY > pullToRefreshThreshold && scrollTop <= 10 && !isPullToRefreshActive && !paginationState.isLoading) {
        isPullToRefreshActive = true;
        showPullToRefreshIndicator();
        // Only prevent default when we're actively handling the gesture
        e.preventDefault();
        console.log('🔄 Pull-to-refresh activated via swipe');
    }

    // Swipe-to-load at the bottom (swipe up) 
    if (deltaY < -swipeToLoadThreshold && scrollTop + windowHeight >= documentHeight - 150 && !isSwipeToLoadActive && paginationState.hasMore && !paginationState.isLoading) {
        isSwipeToLoadActive = true;
        showSwipeToLoadIndicator();
        // Trigger load more posts
        console.log('📱 Swipe-to-load activated');
        loadPosts(false);
    }
}

function handleTouchEnd(e) {
    if (isPullToRefreshActive) {
        isPullToRefreshActive = false;
        hidePullToRefreshIndicator();
        // Trigger refresh
        console.log('🔄 Executing pull-to-refresh');
        refreshFeed();
    }

    if (isSwipeToLoadActive) {
        isSwipeToLoadActive = false;
        hideSwipeToLoadIndicator();
    }

    // Reset touch tracking
    touchStartY = null;
    touchStartX = null;
    touchStartTime = null;
}

function getSentimentClass(post) {
    // Check if we have sentiment type from the backend API
    if (!post.sentiment_type) {
        return 'sentiment-neutral';
    }
    
    // Use the sentiment type from the API response
    const sentimentType = post.sentiment_type.toLowerCase();
    
    // Map emotion names to sentiment classes
    const emotionToClass = {
        'joy': 'sentiment-joy',
        'happy': 'sentiment-joy',        // Alias for joy
        'sad': 'sentiment-sad',
        'angry': 'sentiment-angry',
        'confused': 'sentiment-confused',
        'disgust': 'sentiment-disgust',
        'surprise': 'sentiment-surprise',
        'fear': 'sentiment-fear',
        'neutral': 'sentiment-neutral',
        'affection': 'sentiment-affection',
        'affectionate': 'sentiment-affection',  // Alias
        'sarcastic': 'sentiment-sarcastic'
    };
    
    return emotionToClass[sentimentType] || 'sentiment-neutral';
}

function getSentimentLabel(post) {
    // Use the actual sentiment detected by our enhanced analysis system
    if (post.sentiment_type) {
        // Get the sentiment class and convert to display type
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

// Helper function to get color from emotion
function getEmotionColor(emotion) {
    const emotionToColor = {
        'joy': '#fbbf24',        // Bright yellow/gold
        'happy': '#fbbf24',      // Alias for joy
        'sad': '#1e3a8a',        // Dark blue
        'angry': '#dc2626',      // Red
        'confused': '#a16207',   // Brown/amber
        'disgust': '#84cc16',    // Lime green
        'surprise': '#f97316',   // Orange
        'fear': '#374151',       // Dark grey
        'neutral': '#6b7280',    // Neutral gray
        'affection': '#ec4899',  // Pink
        'affectionate': '#ec4899', // Alias
        'sarcastic': '#7c3aed'   // Purple
    };
    
    return emotionToColor[emotion?.toLowerCase()] || '#6b7280';
}

// Function to handle single color backgrounds (no more gradients)
function getSentimentBackground(post) {
    if (!post.sentiment_colors || post.sentiment_colors.length === 0) {
        return '';
    }
    
    // Use the first color from sentiment_colors (API now provides proper colors)
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

// Optimistic voting state management
const optimisticVoteState = new Map(); // postId -> { votes: { voteType: { tag: { count, agreed, baseline: { count, agreed }, pending, opSeq, abortController } } } }

// Global operation sequence counter for handling rapid clicks
let globalOpSeq = 0;

// Clear optimistic voting state (call on logout, post refresh)
function clearOptimisticVoteState() {
    optimisticVoteState.clear();
    globalOpSeq = 0;
}

// Clear optimistic state for a specific post (call when fresh vote data arrives)
function clearOptimisticVoteStateForPost(postId) {
    optimisticVoteState.delete(postId);
}

// Initialize optimistic state for a post
function initOptimisticVoteState(postId) {
    if (!optimisticVoteState.has(postId)) {
        optimisticVoteState.set(postId, { votes: {} });
    }
}

// Get optimistic vote state for a specific tag
function getOptimisticState(postId, voteType, tag) {
    const postState = optimisticVoteState.get(postId);
    if (!postState || !postState.votes[voteType] || !postState.votes[voteType][tag]) {
        return null;
    }
    return postState.votes[voteType][tag];
}

// Set optimistic vote state for a specific tag
function setOptimisticState(postId, voteType, tag, state) {
    initOptimisticVoteState(postId);
    const postState = optimisticVoteState.get(postId);
    
    if (!postState.votes[voteType]) {
        postState.votes[voteType] = {};
    }
    
    postState.votes[voteType][tag] = state;
}

// Get current vote count (optimistic if available, otherwise baseline)
function getCurrentVoteCount(postId, voteType, tag) {
    const optimistic = getOptimisticState(postId, voteType, tag);
    if (optimistic) {
        return optimistic.count;
    }
    return getVoteCount(postId, voteType, tag);
}

// Get current user agreement state (optimistic if available, otherwise baseline)
function getCurrentUserAgreement(postId, voteType, tag) {
    const optimistic = getOptimisticState(postId, voteType, tag);
    if (optimistic) {
        return optimistic.agreed;
    }
    const voteKey = `${voteType}_${tag}`;
    return getUserVote(postId, voteKey);
}

// Render votable emotion tag (for sentiment badges)
function renderVotableEmotionTag(post, sentimentClass, sentimentLabel) {
    // Extract emotion tag from sentiment class
    const emotionTag = sentimentClass.replace('sentiment-', '');
    const userVote = getCurrentUserAgreement(post.id, 'emotion', emotionTag);
    const voteCount = getCurrentVoteCount(post.id, 'emotion', emotionTag);
    const voteCountDisplay = voteCount > 0 ? ` ${formatVoteCount(voteCount)}` : '';
    
    // Check if there's a pending operation
    const optimistic = getOptimisticState(post.id, 'emotion', emotionTag);
    const pendingClass = optimistic && optimistic.pending ? 'pending' : '';
    const votedClass = userVote ? 'voted agreed' : '';
    
    return `
        <div class="sentiment-badge ${sentimentClass} votable-tag ${votedClass} ${pendingClass}" 
             onclick="voteOnTag('${escapeJavaScript(post.id)}', 'post', 'emotion', '${escapeJavaScript(emotionTag)}')"
             title="Click to agree this emotion matches the content. Click again to remove your agreement.">
            ${sentimentLabel}${voteCountDisplay}
        </div>
    `;
}

// Render votable content tag (for toxicity tags)
function renderVotableContentTag(postId, tag) {
    const userVote = getCurrentUserAgreement(postId, 'content_filter', tag.tag);
    const voteCount = getCurrentVoteCount(postId, 'content_filter', tag.tag);
    const voteCountDisplay = voteCount > 0 ? ` ${formatVoteCount(voteCount)}` : '';
    
    // Check if there's a pending operation
    const optimistic = getOptimisticState(postId, 'content_filter', tag.tag);
    const pendingClass = optimistic && optimistic.pending ? 'pending' : '';
    const votedClass = userVote ? 'voted agreed' : '';
    
    return `
        <span class="toxicity-tag votable-tag ${votedClass} ${pendingClass}" 
              style="background-color: ${tag.color}20; border: 1px solid ${tag.color}60; color: ${tag.color}"
              onclick="voteOnTag('${escapeJavaScript(postId)}', 'post', 'content_filter', '${escapeJavaScript(tag.tag)}')"
              title="Click to agree this content tag matches. Click again to remove your agreement.">
            ${tag.displayText}${voteCountDisplay}
        </span>
    `;
}

// Vote on a tag (emotion or content) - Optimistic UI with background sync
async function voteOnTag(targetId, targetType, voteType, tag) {
    if (!authToken) {
        showToast('Please log in to vote', 'error');
        return;
    }
    
    // Get current state
    const currentAgreed = getCurrentUserAgreement(targetId, voteType, tag);
    const currentCount = getCurrentVoteCount(targetId, voteType, tag);
    const desiredAgreed = !currentAgreed;
    
    // Generate new operation sequence
    const opSeq = ++globalOpSeq;
    
    // Get baseline state for network decision
    const voteKey = `${voteType}_${tag}`;
    const baselineAgreed = getUserVote(targetId, voteKey);
    const baselineCount = getVoteCount(targetId, voteType, tag);
    
    // Apply optimistic update immediately with proper clamping
    const newCount = Math.max(0, currentCount + (desiredAgreed ? 1 : -1));
    
    // Cancel any existing request for this tag
    const existingState = getOptimisticState(targetId, voteType, tag);
    if (existingState && existingState.abortController) {
        existingState.abortController.abort();
    }
    
    // Create new abort controller
    const abortController = new AbortController();
    
    // Set optimistic state
    setOptimisticState(targetId, voteType, tag, {
        count: newCount,
        agreed: desiredAgreed,
        baseline: {
            count: baselineCount,
            agreed: baselineAgreed
        },
        pending: true,
        opSeq: opSeq,
        abortController: abortController
    });
    
    // Update UI immediately
    refreshPostVotingOptimistic(targetId);
    
    // Decide network action based on baseline vs desired
    let networkAction = null;
    if (baselineAgreed !== desiredAgreed) {
        networkAction = desiredAgreed ? 'add' : 'remove';
    }
    
    // If no network action needed, just mark as complete
    if (!networkAction) {
        const state = getOptimisticState(targetId, voteType, tag);
        if (state && state.opSeq === opSeq) {
            state.pending = false;
            refreshPostVotingOptimistic(targetId);
        }
        return;
    }
    
    // Execute network request in background
    try {
        let response;
        
        if (networkAction === 'add') {
            response = await fetch('/api/vote', {
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
                    is_upvote: true
                }),
                signal: abortController.signal
            });
        } else {
            response = await fetch(`/api/vote/${targetId}/${targetType}/${voteType}/${tag}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${authToken}`
                },
                signal: abortController.signal
            });
        }
        
        if (response.ok) {
            const voteSummary = await response.json();
            
            // Check if this response is still relevant (not superseded by newer clicks)
            const currentState = getOptimisticState(targetId, voteType, tag);
            if (currentState && currentState.opSeq === opSeq) {
                // Update baseline with server truth
                setUserVote(targetId, voteKey, desiredAgreed);
                updateVoteData(targetId, voteSummary);
                
                // Update optimistic state with server data
                currentState.baseline.count = getVoteCount(targetId, voteType, tag);
                currentState.baseline.agreed = desiredAgreed;
                currentState.pending = false;
                
                refreshPostVotingOptimistic(targetId);
            }
        } else {
            throw new Error(`Failed to ${networkAction} vote`);
        }
    } catch (error) {
        // Check if request was aborted (superseded by newer click)
        if (error.name === 'AbortError' || error.message.includes('abort')) {
            return; // Ignore aborted requests, don't rollback
        }
        
        // Check if this error is still relevant (not superseded by newer operation)
        const currentState = getOptimisticState(targetId, voteType, tag);
        if (currentState && currentState.opSeq === opSeq) {
            // Rollback to baseline on network failure
            currentState.count = Math.max(0, currentState.baseline.count); // Ensure count clamping
            currentState.agreed = currentState.baseline.agreed;
            currentState.pending = false;
            
            refreshPostVotingOptimistic(targetId);
            showToast('Failed to save vote. Please try again.', 'error');
            console.warn('Vote network error:', error);
        }
    }
}

// Optimistic UI refresh - updates voting display immediately without full reload
function refreshPostVotingOptimistic(targetId) {
    // Find the post element
    const postElement = document.querySelector(`[data-post-id="${targetId}"]`);
    if (!postElement) return;
    
    // Find and update sentiment badges
    const sentimentBadges = postElement.querySelectorAll('.sentiment-badge.votable-tag');
    sentimentBadges.forEach(badge => {
        const onClick = badge.getAttribute('onclick');
        if (onClick) {
            // Extract tag from onclick attribute
            const match = onClick.match(/voteOnTag\('[^']*',\s*'[^']*',\s*'emotion',\s*'([^']*)'\)/);
            if (match) {
                const emotionTag = match[1];
                updateVotableElementOptimistic(badge, targetId, 'emotion', emotionTag);
            }
        }
    });
    
    // Find and update toxicity tags
    const toxicityTags = postElement.querySelectorAll('.toxicity-tag.votable-tag');
    toxicityTags.forEach(tag => {
        const onClick = tag.getAttribute('onclick');
        if (onClick) {
            // Extract tag from onclick attribute
            const match = onClick.match(/voteOnTag\('[^']*',\s*'[^']*',\s*'content_filter',\s*'([^']*)'\)/);
            if (match) {
                const contentTag = match[1];
                updateVotableElementOptimistic(tag, targetId, 'content_filter', contentTag);
            }
        }
    });
}

// Update a specific votable element with optimistic state
function updateVotableElementOptimistic(element, targetId, voteType, tag) {
    const userVote = getCurrentUserAgreement(targetId, voteType, tag);
    const voteCount = Math.max(0, getCurrentVoteCount(targetId, voteType, tag)); // Ensure count clamping
    const optimistic = getOptimisticState(targetId, voteType, tag);
    
    // Update classes
    element.classList.toggle('voted', !!userVote);
    element.classList.toggle('agreed', !!userVote);
    element.classList.toggle('pending', optimistic && optimistic.pending);
    
    // Update vote count in text content
    const currentText = element.textContent;
    const baseText = currentText.replace(/\s+\d+[kM]?$/, ''); // Remove existing count
    const voteCountDisplay = voteCount > 0 ? ` ${formatVoteCount(voteCount)}` : '';
    element.textContent = baseText + voteCountDisplay;
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
        if (post.sentiment_type) {
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

// Enhanced cache-aware feed filtering
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
    
    // Use cache-aware filtering for better performance and more comprehensive results
    const sourceData = getCachedPostsForCurrentView();
    
    if (sentiment === 'all') {
        renderPosts(applyContentFiltering(sourceData));
    } else {
        const filtered = sourceData.filter(post => {
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

// Get cached posts for current view - falls back to current posts array if cache is empty
function getCachedPostsForCurrentView() {
    const cache = postsCache.getViewCache(currentView, currentUser?.id);
    const cachedPosts = cache.getAllPosts();
    
    // If cache has data, use it for more comprehensive filtering
    // Otherwise fall back to current posts array
    if (cachedPosts.length > 0) {
        console.log(`📦 Cache: Filtering ${cachedPosts.length} cached posts`);
        return cachedPosts;
    } else {
        console.log(`📦 Cache: No cached data, filtering ${posts.length} current posts`);
        return posts;
    }
}

// Enhanced cache-aware content filtering
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
    
    // Re-apply current filters using cache-aware filtering
    const activeFilterBtn = document.querySelector('.filter-btn.active');
    if (activeFilterBtn) {
        const currentFilter = activeFilterBtn.dataset.filter || 'all';
        
        // Use cache-aware filtering for comprehensive results
        const sourceData = getCachedPostsForCurrentView();
        let currentPosts = sourceData;
        
        if (currentFilter !== 'all') {
            currentPosts = sourceData.filter(post => {
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

function escapeJavaScript(text) {
    // Escape text for safe use in JavaScript string contexts
    return text.toString().replace(/\\/g, '\\\\').replace(/'/g, "\\'")
               .replace(/"/g, '\\"').replace(/\n/g, '\\n')
               .replace(/\r/g, '\\r').replace(/\t/g, '\\t');
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

// Comment system state and caching
let loadedComments = new Set(); // Track which post IDs have loaded comments

// Comments cache configuration
const COMMENTS_CACHE_CONFIG = {
    staleTimeMinutes: 5, // Comments become stale after 5 minutes
    maxCommentsPerPost: 200, // Maximum comments to cache per post
    maxCachedPosts: 50 // Maximum posts to cache comments for
};

// Comments cache class for intelligent caching
class CommentsCache {
    constructor() {
        this.cache = new Map(); // postId -> { comments, lastFetch, isLoading }
        this.lastCleanup = Date.now();
        this.cleanupInterval = 5 * 60 * 1000; // 5 minutes
    }

    // Get cache key for a post
    getCacheKey(postId) {
        return `post_${postId}`;
    }

    // Check if cached comments are fresh
    isFresh(postId) {
        const cached = this.cache.get(this.getCacheKey(postId));
        if (!cached) return false;
        
        const staleTime = COMMENTS_CACHE_CONFIG.staleTimeMinutes * 60 * 1000;
        return (Date.now() - cached.lastFetch) < staleTime;
    }

    // Get cached comments
    get(postId) {
        const cached = this.cache.get(this.getCacheKey(postId));
        return cached ? cached.comments : null;
    }

    // Set comments in cache
    set(postId, comments) {
        const cacheKey = this.getCacheKey(postId);
        this.cache.set(cacheKey, {
            comments: comments.slice(0, COMMENTS_CACHE_CONFIG.maxCommentsPerPost), // Limit cache size
            lastFetch: Date.now(),
            isLoading: false
        });
        
        this.performCleanup();
    }

    // Set loading state
    setLoading(postId, isLoading) {
        const cacheKey = this.getCacheKey(postId);
        const cached = this.cache.get(cacheKey) || { comments: null, lastFetch: 0 };
        cached.isLoading = isLoading;
        this.cache.set(cacheKey, cached);
    }

    // Check if currently loading
    isLoading(postId) {
        const cached = this.cache.get(this.getCacheKey(postId));
        return cached ? cached.isLoading : false;
    }

    // Clear cache for a specific post
    clear(postId) {
        this.cache.delete(this.getCacheKey(postId));
        loadedComments.delete(postId);
    }

    // Clear all cache
    clearAll() {
        this.cache.clear();
        loadedComments.clear();
        console.log('🗑️ Comments cache: All caches cleared');
    }

    // Add a new comment to cache (for optimistic updates)
    addComment(postId, comment) {
        const cached = this.cache.get(this.getCacheKey(postId));
        if (cached && cached.comments) {
            cached.comments.unshift(comment); // Add to beginning
            this.cache.set(this.getCacheKey(postId), cached);
        }
    }

    // Periodic cleanup of old cache entries
    performCleanup() {
        const now = Date.now();
        if (now - this.lastCleanup < this.cleanupInterval) return;

        const cacheEntries = Array.from(this.cache.entries());
        
        // Remove stale entries
        let removedCount = 0;
        for (const [key, value] of cacheEntries) {
            const staleTime = COMMENTS_CACHE_CONFIG.staleTimeMinutes * 60 * 1000;
            if (now - value.lastFetch > staleTime * 2) { // Remove if twice as old as stale time
                this.cache.delete(key);
                removedCount++;
            }
        }

        // Limit cache size (LRU style)
        if (this.cache.size > COMMENTS_CACHE_CONFIG.maxCachedPosts) {
            const sortedEntries = Array.from(this.cache.entries())
                .sort((a, b) => a[1].lastFetch - b[1].lastFetch);
            
            const toRemove = this.cache.size - COMMENTS_CACHE_CONFIG.maxCachedPosts;
            for (let i = 0; i < toRemove; i++) {
                this.cache.delete(sortedEntries[i][0]);
                removedCount++;
            }
        }

        if (removedCount > 0) {
            console.log(`🧹 Comments cache: Cleaned up ${removedCount} entries`);
        }

        this.lastCleanup = now;
    }

    // Get cache statistics
    getStats() {
        const totalPosts = this.cache.size;
        let totalComments = 0;
        for (const cached of this.cache.values()) {
            if (cached.comments) {
                totalComments += cached.comments.length;
            }
        }
        
        return {
            totalPosts,
            totalComments,
            memoryEstimateMB: ((totalPosts * 1000) + (totalComments * 500)) / (1024 * 1024) // Rough estimate
        };
    }
}

// Initialize comments cache
const commentsCache = new CommentsCache();

// Enhanced toggle comment section visibility with cached comments support
function toggleComments(postId) {
    const commentsSection = document.getElementById(`comments-${postId}`);
    const isHidden = commentsSection.classList.contains('hidden');
    
    if (isHidden) {
        commentsSection.classList.remove('hidden');
        
        // UNIFIED CACHING: Use cached comments from post object first
        const cachedPost = posts.find(p => p.id === postId);
        if (cachedPost && cachedPost._comments !== undefined) {
            console.log(`📦 ENHANCED_COMMENTS: Using cached comments from post object for ${postId} (${cachedPost._comments.length} comments)`);
            renderComments(postId, cachedPost._comments);
            loadedComments.add(postId);
            
            // Sync with separate comments cache to maintain consistency
            commentsCache.set(postId, cachedPost._comments);
        } else if (!loadedComments.has(postId)) {
            // Fall back to separate comment loading only if absolutely needed
            console.log(`📦 ENHANCED_COMMENTS: Loading comments separately for ${postId} - no cached comments on post`);
            loadComments(postId);
        }
    } else {
        commentsSection.classList.add('hidden');
    }
}

// Load comments for a post with caching
async function loadComments(postId) {
    // Check if already loading
    if (commentsCache.isLoading(postId)) {
        console.log(`📦 Comments: Already loading comments for post ${postId}`);
        return;
    }

    // Check cache first
    if (commentsCache.isFresh(postId)) {
        console.log(`📦 Comments: Using cached comments for post ${postId}`);
        const cachedComments = commentsCache.get(postId);
        if (cachedComments) {
            renderComments(postId, cachedComments);
            loadedComments.add(postId); // Ensure tracking state is updated
            return;
        }
    }
    
    const commentsList = document.getElementById(`comments-list-${postId}`);
    commentsList.innerHTML = `
        <div class="loading-comments">
            <div class="loading-spinner"></div>
            <span>💬 Loading comments...</span>
        </div>
    `;
    
    commentsCache.setLoading(postId, true);
    
    try {
        const response = await fetch(`${API_BASE}/posts/${postId}/comments`);
        const data = await response.json();
        
        if (response.ok) {
            // Fix: API returns array directly, not wrapped in { comments: [] }
            const comments = Array.isArray(data) ? data : data.comments || [];
            console.log(`🔍 COMMENT_LOAD_DEBUG: Post ${postId} - API returned ${comments.length} comments`);
            
            // Calculate and update root comment count (depth = 0 only)  
            const rootCommentCount = calculateRootCommentCount(comments);
            console.log(`📊 COMMENT_COUNT: Post ${postId} - ${rootCommentCount} root comments, ${comments.length} total`);
            updatePostCommentCountInUI(postId, rootCommentCount);
            
            // Cache the comments
            commentsCache.set(postId, comments);
            loadedComments.add(postId);
            
            console.log(`📦 Comments: Loaded and cached ${comments.length} comments for post ${postId}`);
            renderComments(postId, comments);
        } else {
            commentsList.innerHTML = `
                <div class="error-message">
                    <span>❌ Failed to load comments</span>
                    <button onclick="loadComments('${escapeJavaScript(postId)}')" class="retry-btn">🔄 Retry</button>
                </div>
            `;
        }
    } catch (error) {
        console.error('Load comments error:', error);
        commentsList.innerHTML = `
            <div class="error-message">
                <span>❌ Network error loading comments</span>
                <button onclick="loadComments('${escapeJavaScript(postId)}')" class="retry-btn">🔄 Retry</button>
            </div>
        `;
    } finally {
        commentsCache.setLoading(postId, false);
    }
}

// Post a new comment with optimistic UI
async function postComment(postId) {
    console.log('🚀 COMMENT_DIAGNOSTIC: Starting comment creation process');
    console.log(`   📍 Post ID: ${postId}`);
    console.log(`   👤 Auth Token Present: ${!!authToken}`);
    console.log(`   🌐 API Base: ${API_BASE}`);
    
    if (!authToken) {
        console.error('❌ COMMENT_DIAGNOSTIC: No auth token found');
        showToast('Please log in to comment', 'error');
        return;
    }
    
    const textarea = document.getElementById(`comment-input-${postId}`);
    if (!textarea) {
        console.error(`❌ COMMENT_DIAGNOSTIC: Textarea not found for post ${postId}`);
        showToast('Comment form not found', 'error');
        return;
    }
    
    const content = textarea.value.trim();
    console.log(`   📝 Content Length: ${content.length} characters`);
    console.log(`   📄 Content Preview: "${content.substring(0, 100)}${content.length > 100 ? '...' : ''}"`);
    
    if (!content) {
        console.warn('⚠️ COMMENT_DIAGNOSTIC: Empty content provided');
        showToast('Please enter a comment', 'error');
        return;
    }
    
    if (content.length > 2000) {
        console.warn(`⚠️ COMMENT_DIAGNOSTIC: Content too long: ${content.length} > 2000`);
        showToast('Comment is too long (max 2000 characters)', 'error');
        return;
    }
    
    console.log('🔄 COMMENT_DIAGNOSTIC: Creating optimistic comment object');
    
    // Create optimistic comment
    const optimisticId = `temp_${Date.now()}`;
    const optimisticComment = {
        comment: {
            id: optimisticId,
            post_id: postId,
            user_id: currentUser.id,
            content: content,
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
            parent_id: null,
            path: '001',
            depth: 0,
            sentiment_analysis: { sentiment_type: null },
            moderation_result: { is_blocked: false, toxicity_tags: [] },
            is_flagged: false,
            reply_count: 0,
            popularity_score: 1.0
        },
        author: currentUser.username,
        can_modify: true,
        is_collapsed: false,
        replies: [],
        isPending: true // Mark as pending for UI
    };
    
    console.log(`✅ COMMENT_DIAGNOSTIC: Optimistic comment created with temp ID: ${optimisticId}`);
    console.log('   🔍 Comment Structure:', {
        id: optimisticComment.comment.id,
        post_id: optimisticComment.comment.post_id,
        content: optimisticComment.comment.content.substring(0, 50) + '...',
        user_id: optimisticComment.comment.user_id
    });
    
    // Clear input and update UI optimistically
    console.log('🔄 COMMENT_DIAGNOSTIC: Clearing textarea and updating UI');
    textarea.value = '';
    updateCommentCounter({ target: textarea });
    
    // Add optimistic comment to UI immediately
    console.log('🎨 COMMENT_DIAGNOSTIC: Adding optimistic comment to UI');
    addOptimisticComment(postId, optimisticComment);
    updatePostCommentCount(postId);
    showToast('💬 Creating comment...', 'info');
    
    // Fire-and-forget async database save - don't wait for response
    console.log('🚀 COMMENT_DIAGNOSTIC: Starting async database save (fire-and-forget)');
    saveCommentToDatabase(optimisticId, postId, content);
}

// Add optimistic comment to UI immediately
function addOptimisticComment(postId, commentData) {
    const commentsList = document.getElementById(`comments-list-${postId}`);
    if (!commentsList) return;
    
    // Create comment HTML with pending styling
    const commentHTML = createCommentHTML(commentData, true);
    
    // Add to top of comments list
    if (commentsList.innerHTML.includes('no-comments')) {
        commentsList.innerHTML = commentHTML;
    } else {
        commentsList.insertAdjacentHTML('afterbegin', commentHTML);
    }
    
    // Mark this post as having comments loaded
    loadedComments.add(postId);
    
    // Apply pending styling explicitly
    setTimeout(() => {
        const optimisticElement = document.querySelector(`[data-comment-id="${commentData.comment.id}"]`);
        if (optimisticElement) {
            optimisticElement.classList.add('comment-pending');
        }
    }, 10);
}

// Replace optimistic comment with real one
function replaceOptimisticComment(postId, tempId, realComment) {
    const tempElement = document.querySelector(`[data-comment-id="${tempId}"]`);
    if (tempElement && realComment) {
        // Create real comment data structure
        const realCommentData = {
            comment: realComment,
            author: currentUser.username,
            can_modify: true,
            is_collapsed: false,
            replies: [],
            isPending: false
        };
        
        // Update the data attribute to use the real comment ID
        tempElement.setAttribute('data-comment-id', realComment.id);
        
        // Remove pending styling and add sentiment if available
        tempElement.classList.remove('comment-pending');
        tempElement.classList.add('saved');
        
        // ENHANCED: Update sentiment badge with real data from server and make it votable
        const sentimentBadge = tempElement.querySelector('.comment-sentiment-badge');
        if (realComment.sentiment_type) {
            if (sentimentBadge) {
                // Replace existing badge with votable version
                const badgesContainer = tempElement.querySelector('.comment-badges');
                if (badgesContainer) {
                    badgesContainer.innerHTML = renderVotableCommentSentimentTag(realComment);
                }
            } else {
                // Add new votable badge if pending comment didn't have one
                const badgesContainer = tempElement.querySelector('.comment-badges');
                if (badgesContainer) {
                    badgesContainer.innerHTML = renderVotableCommentSentimentTag(realComment);
                }
            }
        }
        
        console.log(`✅ Replaced optimistic comment ${tempId} with real comment ${realComment.id}`);
    }
}

// Remove failed optimistic comment
function removeOptimisticComment(postId, tempId) {
    const tempElement = document.querySelector(`[data-comment-id="${tempId}"]`);
    if (tempElement) {
        tempElement.remove();
        
        // Check if no comments left
        const commentsList = document.getElementById(`comments-list-${postId}`);
        if (commentsList && commentsList.children.length === 0) {
            commentsList.innerHTML = '<div class="no-comments">No comments yet. Be the first to comment!</div>';
        }
    }
}

// Create comment HTML
function createCommentHTML(commentData, isPending = false) {
    const comment = commentData.comment;
    const author = commentData.author || 'Unknown';
    const timeAgo = formatTimeAgo(comment.created_at);
    
    // Get sentiment styling
    const sentimentClass = getCommentSentimentClass(comment);
    const sentimentEmoji = getCommentSentimentEmoji(comment);
    const sentimentStyle = getCommentSentimentStyle(comment);
    
    const pendingClass = isPending ? 'comment-pending' : '';
    const pendingIndicator = isPending ? '<span class="pending-indicator">⏳ Posting...</span>' : '';
    
    return `
        <div class="comment ${pendingClass}" data-comment-id="${comment.id}" style="${sentimentStyle}">
            <div class="comment-header">
                <div class="comment-author">${escapeHtml(author)}</div>
                <div class="comment-time">${timeAgo}</div>
                ${pendingIndicator}
                ${sentimentEmoji ? `<span class="comment-sentiment">${sentimentEmoji}</span>` : ''}
            </div>
            <div class="comment-content">${escapeHtml(comment.content)}</div>
        </div>
    `;
}


// Render comments with nesting and emotion colors
function renderComments(postId, comments) {
    const commentsList = document.getElementById(`comments-list-${postId}`);
    
    if (!comments || comments.length === 0) {
        commentsList.innerHTML = '<div class="no-comments">No comments yet. Be the first to comment!</div>';
        return;
    }
    
    // Build nested comment structure - API returns comments directly, not wrapped
    const commentsHTML = comments.map(comment => {
        // API returns comment objects directly, not wrapped in commentData structure
        const timeAgo = formatTimeAgo(comment.created_at);
        
        // Get sentiment styling (same as posts)
        const sentimentClass = getCommentSentimentClass(comment);
        const sentimentEmoji = getCommentSentimentEmoji(comment);
        const sentimentStyle = getCommentSentimentStyle(comment);
        
        // ENHANCED: Render votable sentiment tag like posts
        const sentimentTagHTML = renderVotableCommentSentimentTag(comment);
        
        // Get toxicity tags for this comment (like posts)
        const toxicityTags = getCommentToxicityTags(comment);
        const toxicityTagsHTML = renderVotableCommentToxicityTags(comment.id, toxicityTags);
        
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
        
        // Handle author - API returns author object or use fallback
        const authorName = comment.author?.username || 'Anonymous';
        
        return `
            <div class="comment ${sentimentClass}" data-comment-id="${comment.id}" style="${sentimentStyle}">
                <div class="comment-header">
                    <div class="comment-header-left">
                        <span class="comment-author">${escapeHtml(authorName)}</span>
                        <span class="comment-time">${timeAgo}</span>
                    </div>
                    <div class="comment-header-right">
                        <div class="comment-badges">
                            ${sentimentTagHTML}
                        </div>
                        ${deleteControlsHTML}
                    </div>
                </div>
                <div class="comment-content">${escapeHtml(comment.content)}</div>
                ${toxicityTagsHTML ? `<div class="comment-toxicity-tags">${toxicityTagsHTML}</div>` : ''}
                <div class="comment-actions">
                    ${authToken ? `<button onclick="showReplyForm('${comment.id}')" class="reply-btn">Reply</button>` : ''}
                    <div class="comment-stats">
                        <span class="comment-popularity">⭐ ${(comment.popularity_score || comment.comment?.popularity_score || 1.0).toFixed(1)}</span>
                        ${(() => {
                            const replyCount = comment.replies ? comment.replies.length : 0;
                            return replyCount > 0 ? `<span class="comment-replies clickable" 
                                data-comment-id="${comment.id}" 
                                data-reply-count="${replyCount}"
                                role="button" 
                                tabindex="0" 
                                aria-expanded="false"
                                aria-controls="replies-${comment.id}"
                                title="Click to expand ${replyCount} replies">↳ ${replyCount} replies</span>` : '';
                        })()}
                    </div>
                </div>
                
                <!-- Reply form (initially hidden) -->
                ${authToken ? `
                <div id="reply-form-${comment.id}" class="reply-form hidden">
                    <textarea id="reply-input-${comment.id}" placeholder="Write a reply..." 
                            class="reply-textarea" rows="2" maxlength="2000"></textarea>
                    <div class="reply-form-actions">
                        <button onclick="cancelReply('${comment.id}')" class="cancel-btn">Cancel</button>
                        <button onclick="postReply('${escapeJavaScript(comment.id)}', '${escapeJavaScript(postId)}')" class="reply-submit-btn">Reply</button>
                    </div>
                </div>` : ''}
                
                <!-- Nested replies will go here (initially hidden) -->
                <div id="replies-${comment.id}" class="replies-container hidden">
                    ${renderReplies(comment.replies || [], postId)}
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
    if (!comment.sentiment_type) {
        return '';
    }
    
    // Get color from the sentiment type
    const color = getEmotionColor(comment.sentiment_type);
    return `border-left: 4px solid ${color}; background: ${color}11;`;
}

// ENHANCED: Render votable comment sentiment tag like posts
function renderVotableCommentSentimentTag(comment) {
    if (!comment.sentiment_type) return '';
    
    const sentimentClass = getCommentSentimentClass(comment);
    const sentimentEmoji = getCommentSentimentEmoji(comment);
    const sentimentColor = getEmotionColor(comment.sentiment_type);
    
    // Extract emotion tag from sentiment type for voting
    const emotionTag = comment.sentiment_type.toLowerCase();
    const userVote = getCurrentUserAgreement(comment.id, 'emotion', emotionTag);
    const voteCount = getCurrentVoteCount(comment.id, 'emotion', emotionTag);
    const voteCountDisplay = voteCount > 0 ? ` ${formatVoteCount(voteCount)}` : '';
    
    const votedClass = userVote ? 'voted agreed' : '';
    
    return `<div class="sentiment-badge comment-sentiment-badge votable-tag ${sentimentClass} ${votedClass}" 
                 style="background-color: ${sentimentColor}; color: white; font-size: 0.75rem; padding: 2px 6px; border-radius: 12px; font-weight: 500; cursor: pointer;"
                 onclick="voteOnTag('${comment.id}', 'comment', 'emotion', '${emotionTag}')"
                 title="Click to agree this emotion matches the comment. Click again to remove your agreement.">
        ${sentimentEmoji} ${comment.sentiment_type}${voteCountDisplay}
    </div>`;
}

// Get toxicity tags for comments (like posts)
function getCommentToxicityTags(comment) {
    const tags = [];
    if (comment.toxicity_tags && comment.toxicity_tags.length > 0) {
        comment.toxicity_tags.forEach(tag => {
            tags.push({
                type: tag,
                label: tag.charAt(0).toUpperCase() + tag.slice(1).replace('_', ' ')
            });
        });
    }
    return tags;
}

// Render votable toxicity tags for comments
function renderVotableCommentToxicityTags(commentId, toxicityTags) {
    if (!toxicityTags || toxicityTags.length === 0) return '';
    
    return toxicityTags.map(tag => {
        const userVote = getCurrentUserAgreement(commentId, 'content_filter', tag.type);
        const voteCount = getCurrentVoteCount(commentId, 'content_filter', tag.type);
        const voteCountDisplay = voteCount > 0 ? ` ${formatVoteCount(voteCount)}` : '';
        
        const votedClass = userVote ? 'voted agreed' : '';
        
        return `
            <span class="toxicity-tag comment-toxicity-tag votable-tag ${votedClass}" 
                  style="background-color: #ef4444; color: white; font-size: 0.7rem; padding: 1px 4px; border-radius: 8px; margin-right: 4px; cursor: pointer;"
                  onclick="voteOnTag('${commentId}', 'comment', 'content_filter', '${tag.type}')"
                  title="Click to agree this content tag matches. Click again to remove your agreement.">
                ⚠️ ${tag.label}${voteCountDisplay}
            </span>
        `;
    }).join('');
}

// Update comment voting UI after vote (similar to post voting)
function refreshCommentVotingOptimistic(commentId) {
    const commentElement = document.querySelector(`[data-comment-id="${commentId}"]`);
    if (!commentElement) return;
    
    // Update emotion tags with current vote state
    const sentimentBadges = commentElement.querySelectorAll('.sentiment-badge.votable-tag');
    sentimentBadges.forEach(badge => {
        const onClick = badge.getAttribute('onclick');
        if (onClick) {
            // Extract tag from onclick attribute
            const match = onClick.match(/voteOnTag\('[^']*',\s*'comment',\s*'emotion',\s*'([^']*)'\)/);
            if (match) {
                const emotionTag = match[1];
                updateVotableElementOptimistic(badge, commentId, 'emotion', emotionTag);
            }
        }
    });
    
    // Update toxicity tags with current vote state
    const toxicityTags = commentElement.querySelectorAll('.toxicity-tag.votable-tag');
    toxicityTags.forEach(tag => {
        const onClick = tag.getAttribute('onclick');
        if (onClick) {
            // Extract tag from onclick attribute
            const match = onClick.match(/voteOnTag\('[^']*',\s*'comment',\s*'content_filter',\s*'([^']*)'\)/);
            if (match) {
                const contentTag = match[1];
                updateVotableElementOptimistic(tag, commentId, 'content_filter', contentTag);
            }
        }
    });
}

// Enhanced render nested replies with full hierarchical support
function renderReplies(replies, postId = null, depth = 1) {
    if (!replies || replies.length === 0) return '';
    
    const maxDisplayDepth = 5; // Limit visual nesting for readability
    const actualDepth = Math.min(depth, maxDisplayDepth);
    
    return replies.map(reply => {
        // API returns reply objects directly, not wrapped
        const timeAgo = formatTimeAgo(reply.created_at);
        const sentimentClass = getCommentSentimentClass(reply);
        const sentimentEmoji = getCommentSentimentEmoji(reply);
        const sentimentStyle = getCommentSentimentStyle(reply);
        const authorName = reply.author?.username || 'Anonymous';
        
        // Show delete controls for replies owned by current user
        const isOwner = currentUser && reply.user_id === currentUser.id;
        const isMyPostsPage = currentView === 'user_home';
        const deleteControlsHTML = (isOwner && isMyPostsPage) ? `
            <div class="delete-controls comment-delete-controls">
                <input type="checkbox" class="delete-checkbox" data-type="comment" data-id="${reply.id}" 
                       onchange="toggleDeleteControls()">
                <button class="delete-icon" onclick="deleteComment('${reply.id}')" title="Delete Reply">🗑️</button>
            </div>
        ` : '';
        
        return `
            <div class="reply ${sentimentClass}" data-comment-id="${reply.id}" data-depth="${actualDepth}" style="${sentimentStyle}; margin-left: ${actualDepth * 20}px;">
                <div class="comment-header">
                    <div class="comment-header-left">
                        <span class="comment-author">${escapeHtml(authorName)}</span>
                        <span class="comment-time">${timeAgo}</span>
                        ${sentimentEmoji ? `<span class="comment-emotion">${sentimentEmoji}</span>` : ''}
                    </div>
                    ${deleteControlsHTML}
                </div>
                <div class="comment-content">${escapeHtml(reply.content)}</div>
                
                <div class="comment-actions">
                    ${authToken && actualDepth < maxDisplayDepth ? `<button onclick="showReplyForm('${reply.id}')" class="reply-btn">Reply</button>` : ''}
                    <div class="comment-stats">
                        <span class="comment-popularity">⭐ ${(reply.popularity_score || 1.0).toFixed(1)}</span>
                        ${reply.replies && reply.replies.length > 0 ? `<span class="comment-replies clickable" 
                            data-comment-id="${reply.id}" 
                            data-reply-count="${reply.replies.length}"
                            role="button" 
                            tabindex="0" 
                            aria-expanded="false"
                            aria-controls="replies-${reply.id}"
                            title="Click to expand ${reply.replies.length} replies">↳ ${reply.replies.length} replies</span>` : ''}
                    </div>
                    <!-- Emotion voting removed temporarily -->
                </div>
                
                <!-- Reply form for nested replies -->
                ${authToken && actualDepth < maxDisplayDepth ? `
                <div id="reply-form-${reply.id}" class="reply-form hidden">
                    <textarea id="reply-input-${reply.id}" placeholder="Write a reply..." 
                            class="reply-textarea" rows="2" maxlength="2000"></textarea>
                    <div class="reply-form-actions">
                        <button onclick="cancelReply('${reply.id}')" class="cancel-btn">Cancel</button>
                        <button onclick="postReply('${escapeJavaScript(reply.id)}', '${escapeJavaScript(postId || 'unknown')}')" class="reply-submit-btn">Reply</button>
                    </div>
                </div>` : ''}
                
                <!-- Recursive nested replies (initially hidden) -->
                ${reply.replies && reply.replies.length > 0 ? `
                <div id="replies-${reply.id}" class="replies-container hidden">
                    ${renderReplies(reply.replies, postId, depth + 1)}
                </div>` : ''}
            </div>
        `;
    }).join('');
}

// Toggle replies visibility for a comment using data attributes
function toggleReplies(commentId) {
    const repliesContainer = document.getElementById(`replies-${commentId}`);
    const repliesButton = document.querySelector(`[data-comment-id="${commentId}"].comment-replies.clickable`);
    
    if (!repliesContainer || !repliesButton) return;
    
    // Prevent rapid clicks during animation
    if (repliesButton.hasAttribute('data-toggling')) return;
    repliesButton.setAttribute('data-toggling', 'true');
    
    const isHidden = repliesContainer.classList.contains('hidden');
    const replyCount = repliesButton.getAttribute('data-reply-count') || '0';
    
    // Remove any existing animation event listeners
    repliesContainer.removeEventListener('animationend', handleExpandComplete);
    repliesContainer.removeEventListener('animationend', handleCollapseComplete);
    
    if (isHidden) {
        // Show replies with smooth animation
        repliesContainer.classList.remove('hidden');
        repliesContainer.classList.add('expanding');
        repliesButton.setAttribute('title', `Click to hide ${replyCount} replies`);
        repliesButton.innerHTML = `▼ Hide ${replyCount} replies`;
        
        // Listen for animation completion
        function handleExpandComplete() {
            repliesContainer.classList.remove('expanding');
            repliesContainer.classList.add('expanded');
            repliesButton.setAttribute('aria-expanded', 'true');
            repliesButton.removeAttribute('data-toggling');
            repliesContainer.removeEventListener('animationend', handleExpandComplete);
        }
        repliesContainer.addEventListener('animationend', handleExpandComplete);
        
    } else {
        // Hide replies with smooth animation
        repliesContainer.classList.remove('expanded');
        repliesContainer.classList.add('collapsing');
        repliesButton.setAttribute('title', `Click to expand ${replyCount} replies`);
        repliesButton.innerHTML = `↳ ${replyCount} replies`;
        
        // Listen for animation completion
        function handleCollapseComplete() {
            repliesContainer.classList.remove('collapsing');
            repliesContainer.classList.add('hidden');
            repliesButton.setAttribute('aria-expanded', 'false');
            repliesButton.removeAttribute('data-toggling');
            repliesContainer.removeEventListener('animationend', handleCollapseComplete);
        }
        repliesContainer.addEventListener('animationend', handleCollapseComplete);
    }
}

// Initialize delegated event handlers for collapsible replies
function initializeCommentToggling() {
    // Remove any existing handlers to prevent duplicates
    document.removeEventListener('click', handleReplyToggle);
    document.removeEventListener('keydown', handleReplyToggle);
    
    // Add delegated event handlers for reply toggles
    document.addEventListener('click', handleReplyToggle);
    document.addEventListener('keydown', handleReplyToggle);
}

// Handle reply toggle clicks and keyboard events using event delegation
function handleReplyToggle(event) {
    // For keyboard events, only proceed if it's Enter or Space
    if (event.type === 'keydown' && event.key !== 'Enter' && event.key !== ' ') {
        return;
    }
    
    const target = event.target.closest('.comment-replies.clickable');
    if (!target) return;
    
    event.preventDefault();
    const commentId = target.getAttribute('data-comment-id');
    if (commentId) {
        toggleReplies(commentId);
    }
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

// Update post comment count in UI with specific count  
function updatePostCommentCountInUI(postId, count) {
    const commentButton = document.querySelector(`button[onclick="toggleComments('${postId}')"]`);
    if (commentButton) {
        commentButton.innerHTML = `💬 ${count} comments`;
        console.log(`📊 Updated post ${postId} comment count to ${count}`);
    }
}

// Calculate root comment count from comments array (depth 0 or no parent)
function calculateRootCommentCount(comments) {
    if (!comments || !Array.isArray(comments)) return 0;
    return comments.filter(comment => {
        // Use depth if available, otherwise fallback to parent_id check
        if (comment.depth !== undefined) {
            return comment.depth === 0;
        }
        return comment.parent_id === null || comment.parent_id === undefined;
    }).length;
}

// Calculate direct reply count for a specific comment
function calculateDirectReplyCount(comments, commentId) {
    if (!comments || !Array.isArray(comments)) return 0;
    return comments.filter(comment => comment.parent_id === commentId).length;
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

// Post reply to a comment
async function postReply(parentCommentId, postId) {
    if (!authToken) {
        showToast('Please log in to reply', 'error');
        return;
    }
    
    const textarea = document.getElementById(`reply-input-${parentCommentId}`);
    const content = textarea.value.trim();
    
    if (!content) {
        showToast('Please enter a reply', 'error');
        return;
    }
    
    if (content.length > 2000) {
        showToast('Reply is too long (max 2000 characters)', 'error');
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
                parent_id: parentCommentId // This is a reply to another comment
            })
        });
        
        const data = await response.json();
        
        if (response.ok) {
            textarea.value = '';
            cancelReply(parentCommentId);
            showToast('💬 Reply posted!', 'success');
            
            // ENHANCED: Invalidate both comment cache AND post cache for this post
            commentsCache.clear(postId);
            
            // Also clear post from posts cache to reload fresh data including comment count
            const currentPost = posts.find(p => p.id === postId);
            if (currentPost) {
                // Clear cached comments on post object
                delete currentPost._comments;
            }
            
            // Reload comments to show the new reply
            loadComments(postId);
            
            // Update comment count in post
            updatePostCommentCount(postId);
        } else {
            showToast(data.message || 'Failed to post reply', 'error');
        }
    } catch (error) {
        console.error('Post reply error:', error);
        showToast('Failed to post reply', 'error');
    }
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

// Auto-refresh disabled per user request
// Users can manually refresh using pull-to-refresh or reload buttons
// setInterval(() => {
//     if (document.visibilityState === 'visible') {
//         loadPosts();
//     }
// }, 30000);

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

// === MOBILE NAVIGATION FUNCTIONS ===

function toggleMobileMenu() {
    const mobileNav = document.getElementById('mobileNav');
    const mobileMenuToggle = document.getElementById('mobileMenuToggle');
    
    if (mobileNav.classList.contains('show')) {
        // Closing menu
        mobileNav.classList.remove('show');
        mobileNav.classList.add('hidden');
        mobileMenuToggle.classList.remove('active');
        document.removeEventListener('click', handleClickOutside);
    } else {
        // Opening menu
        mobileNav.classList.remove('hidden');
        mobileNav.classList.add('show');
        mobileMenuToggle.classList.add('active');
        // Add click outside listener
        setTimeout(() => {
            document.addEventListener('click', handleClickOutside);
        }, 0);
    }
}

function closeMobileMenu() {
    const mobileNav = document.getElementById('mobileNav');
    const mobileMenuToggle = document.getElementById('mobileMenuToggle');
    
    mobileNav.classList.remove('show');
    mobileNav.classList.add('hidden');
    mobileMenuToggle.classList.remove('active');
    document.removeEventListener('click', handleClickOutside);
}

function handleClickOutside(event) {
    const mobileNav = document.getElementById('mobileNav');
    const mobileMenuToggle = document.getElementById('mobileMenuToggle');
    
    // Check if click is outside the mobile nav and toggle button
    if (!mobileNav.contains(event.target) && !mobileMenuToggle.contains(event.target)) {
        closeMobileMenu();
    }
}