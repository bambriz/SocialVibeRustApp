/*!
 * Comment Model for Social Pulse - Reddit-Style Hierarchical Comments
 * 
 * LEARNING FOCUS: Understanding Rust data modeling and hierarchical structures
 * 
 * This module defines the Comment data structure for our Reddit-style comment system.
 * Comments form tree hierarchies where users can reply to posts or other comments.
 */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The main Comment struct - represents a single comment in our hierarchical system
/// 
/// LEARNING PURPOSE: This shows how to design data structures for tree-like data
/// KEY CONCEPTS: Foreign keys, hierarchical paths, optional fields, timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Unique identifier for this comment
    /// LEARNING: UUIDs provide better security than auto-incrementing IDs
    pub id: Uuid,
    
    /// Which post this comment belongs to
    /// LEARNING: Foreign key relationship - every comment belongs to exactly one post  
    pub post_id: Uuid,
    
    /// The actual comment text content
    /// LEARNING: String owns the data, unlike &str which borrows
    pub content: String,
    
    /// Who wrote this comment
    /// LEARNING: Another foreign key - links to users table
    pub user_id: Uuid, // Updated from author_id to match our new schema
    
    /// If this is a reply to another comment, store the parent's ID
    /// LEARNING: Option<T> enum - None for root comments, Some(id) for replies
    pub parent_id: Option<Uuid>,
    
    /// Materialized path for efficient tree queries (e.g., "1/3/7/")
    /// LEARNING: Trade storage space for query speed - avoid recursive lookups
    /// WHY THIS WORKS: We can find all descendants with a single LIKE query
    pub path: String, // Updated from thread_path for clarity
    
    /// How deep this comment is nested (0 = root, 1 = reply to root, etc.)
    /// LEARNING: Simple integer counter for UI indentation and collapse logic
    pub depth: i32, // Updated from u8 to i32 to match database
    
    /// Sentiment analysis score (-1.0 to 1.0)
    /// LEARNING: Normalized sentiment score for consistent processing
    pub sentiment_score: Option<f64>,
    
    /// Color codes for sentiment display
    /// LEARNING: Store hex color codes for frontend styling
    pub sentiment_colors: Vec<String>,
    
    /// Human-readable sentiment name (e.g., "angry", "joy", "sarcastic+happy")
    /// LEARNING: Descriptive sentiment types for UI display
    pub sentiment_type: Option<String>,
    
    /// Whether this comment has been blocked by moderation
    /// LEARNING: Boolean flags for quick filtering in queries
    pub is_blocked: bool,
    
    /// Toxicity categories (e.g., "toxicity", "insult", "threat")
    /// LEARNING: Store specific violation types for transparency
    pub toxicity_tags: Vec<String>,
    
    /// Complete diagnostic data from moderation system
    /// LEARNING: Store full analysis results for debugging
    pub toxicity_scores: Option<serde_json::Value>,
    
    /// When this comment was originally created
    /// LEARNING: DateTime<Utc> for timezone-aware timestamps
    pub created_at: DateTime<Utc>,
    
    /// When this comment was last updated
    /// LEARNING: Automatic updates via database triggers
    pub updated_at: DateTime<Utc>,
    
    /// Count of direct replies to this comment
    /// LEARNING: Denormalized data - store counts for UI performance
    pub reply_count: i32,
    
    /// Calculated popularity score for feed ranking (includes votes, sentiment, recency)
    /// LEARNING: Dynamic scoring for content ranking and discovery
    pub popularity_score: f64,
}

/// Request structure for creating a new comment
/// 
/// LEARNING PURPOSE: Separate input/output models from domain models
/// WHY: Users don't provide IDs, timestamps, or computed fields
#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    /// Which post this comment is for
    /// LEARNING: Required field - every comment must belong to a post
    pub post_id: Uuid,
    
    /// The comment content from the user
    /// LEARNING: This will be validated before processing
    pub content: String,
    
    /// If replying to another comment, provide the parent ID
    /// LEARNING: Option type allows both root comments and replies
    pub parent_id: Option<Uuid>,
}

/// Response structure for returning comment data to frontend
/// 
/// LEARNING PURPOSE: API response shaping and nested data structures  
/// WHY SEPARATE: Responses often need computed fields and related data
#[derive(Debug, Serialize)]
pub struct CommentResponse {
    /// Basic comment information
    /// LEARNING: #[serde(flatten)] includes all fields from Comment struct
    #[serde(flatten)]
    pub comment: Comment,
    
    /// Author information for UI display  
    /// LEARNING: Join related data to avoid N+1 query problems
    pub author: Option<crate::models::user::User>,
    
    /// Nested replies for tree structure display
    /// LEARNING: Recursive data structure - CommentResponse contains Vec<CommentResponse>
    pub replies: Vec<CommentResponse>,
    
    /// Whether current user can modify this comment
    /// LEARNING: Authorization checks computed at response time
    pub can_modify: bool,
    
    /// UI state for Reddit-style collapsing
    /// LEARNING: Frontend state hints from backend
    pub is_collapsed: bool,
}

/// Tree structure for efficiently building comment hierarchies
/// 
/// LEARNING PURPOSE: Understanding tree data structures in Rust
/// USE CASE: Converting flat database results into nested trees
#[derive(Debug, Clone)]
pub struct CommentTreeNode {
    pub comment: Comment,
    pub children: Vec<CommentTreeNode>,
    pub metadata: CommentTreeMetadata,
}

/// Metadata for comment tree operations
/// 
/// LEARNING: Additional computed data for tree operations
#[derive(Debug, Clone)]
pub struct CommentTreeMetadata {
    /// Total descendants count (children + grandchildren + ...)
    pub total_descendants: usize,
    /// Maximum depth in this subtree
    pub max_depth: i32,
    /// Whether this branch should be collapsed in UI
    pub should_collapse: bool,
}

impl Comment {
    /// Generate the materialized path for a new reply to this comment
    /// 
    /// LEARNING EXERCISE: Path generation for hierarchical data
    /// YOUR TASK: Implement this method to generate the next sequential path
    /// 
    /// HOW IT WORKS:
    /// - Root comment: "1/", "2/", "3/" etc.
    /// - Replies: "1/1/", "1/2/", "2/1/" etc.
    /// - Deep replies: "1/2/3/", "1/2/4/" etc.
    pub fn generate_reply_path(&self, sibling_count: i32) -> String {
        // TODO: YOUR IMPLEMENTATION HERE
        // 
        // HINTS:
        // 1. Take this comment's path as base
        // 2. Add the next sequential number (sibling_count + 1)
        // 3. Add trailing slash
        // 
        // EXAMPLE:
        // If self.path = "1/3/" and sibling_count = 2
        // Return "1/3/3/" (next reply path)
        
        format!("{}{}/", self.path, sibling_count + 1)
    }
    
    /// Calculate depth from materialized path
    /// 
    /// LEARNING EXERCISE: String parsing and mathematical operations
    /// YOUR TASK: Count slashes to determine hierarchy depth
    pub fn calculate_depth_from_path(path: &str) -> i32 {
        // TODO: YOUR IMPLEMENTATION HERE
        //
        // HINTS:
        // 1. Count "/" characters in the path
        // 2. Subtract 1 (root path "1/" = depth 0)
        // 3. Handle empty/invalid paths gracefully
        //
        // EXAMPLES:
        // "1/" -> 0 (root comment)
        // "1/2/" -> 1 (first-level reply) 
        // "1/2/3/" -> 2 (second-level reply)
        
        if path.is_empty() {
            return 0;
        }
        (path.matches('/').count() as i32).saturating_sub(1)
    }
    
    /// Check if this comment is an ancestor of another
    /// 
    /// LEARNING EXERCISE: Hierarchical relationship checking
    /// YOUR TASK: Use path prefixes to determine ancestry
    pub fn is_ancestor_of(&self, other: &Comment) -> bool {
        // TODO: YOUR IMPLEMENTATION HERE
        //
        // LOGIC:
        // 1. Check if other's path starts with this path
        // 2. Ensure they're not the same comment
        // 3. This enables "collapse thread" functionality
        //
        // EXAMPLES:
        // self="1/", other="1/2/" -> true (parent)
        // self="1/2/", other="1/2/3/" -> true (grandparent)  
        // self="1/", other="2/" -> false (siblings)
        
        other.path.starts_with(&self.path) && other.path != self.path
    }
    
    /// Get the immediate parent path from this comment's path
    /// 
    /// LEARNING EXERCISE: String manipulation for tree traversal
    /// YOUR TASK: Extract parent path by removing the last segment
    pub fn get_parent_path(&self) -> Option<String> {
        // TODO: YOUR IMPLEMENTATION HERE
        //
        // ALGORITHM:
        // 1. Remove trailing slash
        // 2. Find last slash
        // 3. Return everything up to (and including) last slash
        // 4. Return None if this is a root comment
        //
        // EXAMPLES:
        // "1/2/3/" -> Some("1/2/")
        // "1/" -> None (root comment)
        
        if self.depth == 0 {
            return None;
        }
        
        // Remove trailing slash, find last slash, rebuild path
        let without_trailing = self.path.trim_end_matches('/');
        if let Some(last_slash_pos) = without_trailing.rfind('/') {
            Some(format!("{}/", &without_trailing[..last_slash_pos + 1]))
        } else {
            None
        }
    }
}

impl CreateCommentRequest {
    /// Validate comment content and structure
    /// 
    /// LEARNING EXERCISE: Input validation patterns
    /// YOUR TASK: Implement comprehensive validation
    pub fn validate(&self) -> Result<(), String> {
        // TODO: YOUR IMPLEMENTATION HERE
        //
        // VALIDATION RULES:
        // 1. Content must not be empty after trimming
        // 2. Content length between 1-2000 characters
        // 3. No prohibited patterns (spam, abuse, etc.)
        // 4. Valid UUID format for post_id and parent_id
        //
        // RETURN:
        // - Ok(()) if valid
        // - Err(descriptive_message) if invalid
        
        let content = self.content.trim();
        
        if content.is_empty() {
            return Err("Comment content cannot be empty".to_string());
        }
        
        if content.len() > 2000 {
            return Err("Comment content exceeds 2000 character limit".to_string());
        }
        
        // Additional validations you can implement:
        // - Check for spam patterns
        // - Validate against blocked words
        // - Rate limiting checks
        // - Content policy enforcement
        
        Ok(())
    }
}

impl CommentTreeNode {
    /// Build a comment tree from flat list of comments
    /// 
    /// LEARNING EXERCISE: Tree construction algorithms
    /// YOUR TASK: Convert flat database results into nested tree structure
    pub fn build_tree(comments: Vec<Comment>) -> Vec<CommentTreeNode> {
        // TODO: YOUR IMPLEMENTATION HERE
        //
        // ALGORITHM:
        // 1. Create a map of comment_id -> CommentTreeNode
        // 2. Iterate through comments, building tree structure
        // 3. Use parent_id to link children to parents
        // 4. Return root nodes (those with parent_id = None)
        //
        // CHALLENGE: Handle comments that arrive out of order
        // (child before parent in the list)
        
        // This is a complex algorithm - start with a simple version
        // and gradually add optimizations
        
        vec![] // Placeholder - implement the tree building logic
    }
    
    /// Flatten tree back to a list (for API responses)
    /// 
    /// LEARNING EXERCISE: Tree traversal algorithms  
    /// YOUR TASK: Implement depth-first or breadth-first traversal
    pub fn flatten(&self) -> Vec<&Comment> {
        // TODO: YOUR IMPLEMENTATION HERE
        //
        // OPTIONS:
        // 1. Depth-first: Current node, then all children recursively
        // 2. Breadth-first: Current level, then next level
        // 3. Path-ordered: Sort by materialized path
        //
        // CHOOSE: Depth-first matches Reddit's comment ordering
        
        vec![] // Placeholder
    }
}

/* 
 * ADVANCED LEARNING EXERCISES FOR YOU:
 * 
 * 1. COMMENT RANKING SYSTEM:
 *    Implement `calculate_reddit_score()` that combines:
 *    - Reply count (engagement)
 *    - Sentiment positivity 
 *    - Time decay (newer = slight boost)
 *    - Author reputation
 *    
 * 2. THREAD SUMMARIZATION:
 *    Create `CommentThreadSummary` with:
 *    - Dominant emotion across thread
 *    - Average discussion depth  
 *    - Most active contributors
 *    - Controversy score (mixed sentiments)
 *    
 * 3. MODERATION FEATURES:
 *    Add methods for:
 *    - `auto_moderate()` - Apply AI moderation
 *    - `cascade_action()` - Apply action to thread
 *    - `requires_review()` - Flag for human review
 *    
 * 4. PERFORMANCE OPTIMIZATIONS:
 *    - Implement comment pagination
 *    - Add caching for hot threads
 *    - Lazy loading for deep threads
 *    - Database query optimizations
 */