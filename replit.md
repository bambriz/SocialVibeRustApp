# Social Pulse - Sentiment-Based Social Media Platform

## Overview

Social Pulse is a modern social media application that combines traditional social networking features with sentiment analysis and content moderation capabilities. The platform allows users to create posts, interact with content, and provides real-time sentiment analysis of user-generated content. Built with a Rust backend using Axum framework and a vanilla JavaScript frontend, the application focuses on creating a safe and emotionally-aware social environment with Instagram/Facebook/Reddit-style infinite scroll functionality, hierarchical comment threading, and comprehensive optimistic UI for immediate user feedback.

## User Preferences

Preferred communication style: Simple, everyday language.

## System Architecture

### Backend Architecture
The application uses a **modular monolith architecture** with Axum as the primary web framework and a supervised Python subprocess for content processing. This choice provides memory safety, high performance, and excellent concurrency handling for a social media platform that needs to process multiple user requests simultaneously.

**Key architectural decisions:**
- **Axum Framework**: Selected for its modern async/await support, type-safe routing, and middleware system
- **JWT Authentication**: Implements stateless authentication using JSON Web Tokens for scalability
- **Argon2 Password Hashing**: Uses industry-standard password hashing for security
- **Middleware-based Authorization**: Implements authentication checks through Axum middleware for consistent security across endpoints
- **RESTful API Design**: Follows REST principles for predictable and maintainable API endpoints
- **Secure Pagination System**: Implements validated pagination with bounds checking (limit: 1-50, offset: ≤10000) and division-by-zero protection

### Frontend Architecture
The frontend uses **vanilla JavaScript with a Single Page Application (SPA) approach**. This decision prioritizes simplicity and reduces build complexity while maintaining modern user experience patterns.

**Key architectural decisions:**
- **Vanilla JavaScript**: Chosen over frameworks to minimize complexity and bundle size
- **Component-based UI**: Organizes code into reusable functions despite not using a formal framework
- **Local Storage for Auth**: Stores JWT tokens locally for session persistence
- **Modal-based Interactions**: Uses overlay modals for forms and detailed views
- **Real-time Sentiment Preview**: Provides immediate feedback on content sentiment as users type
- **Infinite Scroll Feed**: Implements continuous scrolling pagination with loading indicators for smooth Instagram/Facebook/Reddit-style user experience
- **Optimistic UI**: Comprehensive optimistic UI for both posts and comments with immediate visual feedback and graceful error handling
- **Mobile-First Design**: Touch-friendly interactions with pull-to-refresh and swipe-to-load gestures for enhanced mobile experience

### Comment System Architecture
The application implements a **sophisticated hierarchical comment system** following Reddit-style threading patterns with optimistic UI for immediate user feedback.

**Key architectural decisions:**
- **Materialized Path Structure**: Uses path-based hierarchy (e.g., "1/", "1/001/", "1/002/") stored in `path` TEXT column with atomic sibling index allocation for consistent threading
- **Database Constraints**: Enforces path validation via `comments_path_valid` constraint, depth limits (0-10) via `comments_depth_limit` constraint, and foreign key integrity via `comments_post_id_fkey` and `comments_parent_id_fkey`
- **Transactional Comment Creation**: Single-transaction insert implemented in `src/services/comment_service.rs` with automatic path generation and reply count updates for data integrity
- **Depth-Limited Threading**: Maximum comment depth (10 levels) enforced at database level and service level (CommentService::create_comment_atomic)
- **Optimistic Comment Posting**: Immediate UI updates with temporary IDs, replaced by server responses or rolled back on failure (implemented in static/app-v2.js)
- **Sentiment-Aware Comments**: Each comment includes sentiment analysis with visual emotion indicators and popularity scoring
- **Collapsible Comment Threads**: Expandable/collapsible comment sections with smooth animations for better content organization
- **Touch-Friendly Mobile UX**: Optimized comment interactions for mobile devices with proper touch targets and responsive design

### Content Processing Architecture
The application implements a **subprocess-managed content processing system** with tight integration between Rust and Python for advanced text analysis.

**Key architectural decisions:**
- **Subprocess Management**: Python server runs as a managed subprocess of the Rust application with automatic startup, health checking, and supervision
- **Unified Logging**: Python subprocess logs are piped into Rust tracing with [PY] prefix for complete operational transparency
- **Robust Supervision**: Implements bounded restart logic with exponential backoff when Python subprocess crashes (max 3 attempts)
- **Graceful Process Lifecycle**: Automatic startup coordination and clean shutdown handling eliminate process management complexity
- **Configuration Flexibility**: Supports both subprocess mode (default) and external server mode via PYTHON_SERVER_MODE environment variable
- **Rule-based Analysis**: Uses pattern matching for both sentiment analysis and content moderation
- **Persistent Caching**: HuggingFace model caching dramatically reduces startup time on subsequent launches
- **Real-time Processing**: Content is analyzed before storage to provide immediate feedback

### Emotion Processing Architecture
The platform implements **sophisticated emotion detection** with a streamlined architecture for categorizing user sentiment.

**Key architectural decisions:**
- **Standalone Emotion Categories**: Each emotion (joy, sad, angry, fear, disgust, surprise, confused, neutral, sarcastic, affectionate) is treated as an independent category with distinct visual representation
- **Single-Color Design**: Each emotion has its own dedicated color (#fbbf24 for joy, #7c3aed for sarcastic, #ec4899 for affectionate, etc.) eliminating gradient complexity
- **Python-Rust Integration**: Python sentiment analysis returns raw emotion data, with Rust backend parsing and normalizing sarcastic+X and affectionate+X patterns into standalone "sarcastic" and "affectionate" categories
- **Simplified Frontend Processing**: Emotion display logic treats all sentiments uniformly without special combo handling, using single colors and emoji representations
- **HuggingFace EmotionClassifier**: Primary sentiment detection using pre-trained models with pattern-based detection for sarcasm and affection
- **Emotion Migration System**: Automatic migration of legacy combo emotions to standalone categories during startup

### Data Management
The application uses **PostgreSQL as the primary database** with a sophisticated schema design optimized for social media interactions and hierarchical content.

**Key architectural decisions:**
- **PostgreSQL Production Database**: Uses Replit's managed PostgreSQL service with rollback capabilities and production-grade reliability
- **User-centric Data Model**: Organizes data around user entities with posts, comments, authentication, and profile information
- **Hierarchical Comment Storage**: Implements materialized path pattern for efficient comment threading with path validation constraints
- **Timestamp Tracking**: Includes created_at/updated_at fields for audit trails and chronological ordering
- **Sentiment Metadata Storage**: Stores sentiment analysis results alongside posts and comments for quick retrieval and filtering
- **Atomic Transaction Support**: Uses database transactions for complex operations like comment creation with path generation
- **Database-Only Persistence**: Eliminated CSV backup dependency for improved performance and data consistency

### Security Architecture
The platform implements **defense-in-depth security** with multiple layers of protection.

**Key architectural decisions:**
- **Content Moderation Pipeline**: Automatically screens content before publication
- **Token-based Authentication**: Stateless authentication reduces server-side session management complexity
- **Middleware Security**: Centralized authentication and authorization checking
- **Input Validation**: Both client-side and server-side validation for data integrity

## External Dependencies

### Core Backend Dependencies
- **Axum**: Modern async web framework for Rust applications
- **Tokio**: Async runtime for handling concurrent connections
- **Serde**: Serialization framework for JSON API communication
- **Jsonwebtoken**: JWT implementation for secure authentication
- **Argon2**: Password hashing library for secure credential storage
- **Hyper**: HTTP implementation underlying Axum
- **Chrono**: Date and time handling for timestamps and token expiration

### Frontend Dependencies
- **Inter Font Family**: Modern typography from Google Fonts for improved readability
- **Native Browser APIs**: Utilizes Fetch API, Local Storage, and DOM manipulation without external libraries

### Content Processing Dependencies
- **Python 3**: Required for sentiment analysis and content moderation scripts
- **Python Standard Library**: Uses built-in regex and JSON modules for text processing

## Setup & Configuration

### Environment Variables
Required environment variables for local development:
- **DATABASE_URL**: PostgreSQL connection string (automatically provided in Replit environment)
- **JWT_SECRET**: Secret key for JWT token signing (automatically provided)
- **PYTHON_SERVER_MODE**: Controls sentiment analysis mode - "disabled" uses internal subprocess (default), "external" connects to standalone Python server
- **RUST_LOG**: Set to "info" or "debug" for application logging

### Database Setup
- The application uses Replit's managed PostgreSQL database (production-ready)
- Database schema is automatically initialized on first startup
- No manual database setup required in Replit environment

### Running the Application
```bash
# Start the application (in Replit, this runs automatically via the configured workflow)
cargo run

# The application will:
# 1. Start Rust server on port 5000
# 2. Initialize PostgreSQL connection
# 3. Start Python sentiment analysis subprocess (if enabled)
# 4. Serve static files and API endpoints
```

### Sample Data
The application includes rich sample data with:
- Multiple user accounts with various post types
- Hierarchical comment threads demonstrating nested conversations
- Diverse sentiment examples across different emotion categories
- Test account available: email `frontend@test.com`, password `test123`

## API Reference

### Authentication Endpoints
- **POST /api/register**: User registration
- **POST /api/login**: User authentication  
- **POST /api/logout**: User logout

### Posts Endpoints
- **GET /api/posts**: Retrieve posts with pagination (limit: 1-50, offset: ≤10000)
- **POST /api/posts**: Create new post (requires authentication)
- **GET /api/posts/{id}**: Get specific post
- **PUT /api/posts/{id}**: Update post (requires ownership)
- **DELETE /api/posts/{id}**: Delete post (requires ownership)

### Comments Endpoints
- **GET /api/posts/{post_id}/comments**: Get hierarchical comments for a post
- **POST /api/posts/{post_id}/comments**: Create new comment (requires authentication)
- **PUT /api/comments/{id}**: Update comment (requires ownership)
- **DELETE /api/comments/{id}**: Delete comment (requires ownership)

## Recent Enhancements (September 2025)

### Completed Features
- **✅ Hierarchical Comment System**: Full Reddit-style threaded comments with materialized path storage and optimistic UI
- **✅ PostgreSQL Migration**: Complete migration from SQLite to managed PostgreSQL with enhanced data integrity
- **✅ Optimistic UI Framework**: Comprehensive immediate feedback system for posts and comments with graceful error handling
- **✅ Mobile-Responsive Design**: Touch-friendly interface with pull-to-refresh and swipe-to-load gestures
- **✅ Enhanced Sample Data**: Rich test data with nested comments and diverse sentiment examples
- **✅ Collapsible UI Components**: Independent Share Your Thoughts and Vibe Check sections with smooth animations
- **✅ Eventual Consistency**: Fire-and-forget database operations with optimistic UI updates and background synchronization
- **✅ Comment Persistence**: Fixed comment display issues - comments now persist properly in their posts without UI refresh conflicts

### Technical Improvements
- **Database-First Architecture**: Eliminated CSV backup dependency for improved performance
- **Touch Gesture System**: Full mobile gesture support with threshold-based swipe detection
- **Visual Feedback Enhancements**: Pending state animations and mobile-optimized sticky positioning with save/failed states
- **Cache Optimization**: Improved caching system with proper invalidation and consistency management
- **Surgical Comment Updates**: Comments update in-place without full UI refresh, maintaining user experience continuity

### Potential Future Integrations
The architecture is designed to accommodate:
- **Machine Learning APIs**: For advanced sentiment analysis and content moderation
- **CDN Services**: For static asset delivery and improved performance
- **Email Services**: For user notifications and verification
- **Real-time Communication**: WebSocket support for live updates and messaging
- **Push Notifications**: Mobile push notification support for engagement
- **Advanced Moderation**: AI-powered content moderation with custom rule engines