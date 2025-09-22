# Social Pulse - Sentiment-Based Social Media Platform

## Executive Summary

Social Pulse is a modern social media platform that revolutionizes online interactions by integrating advanced sentiment analysis and content moderation directly into the user experience. Built with performance, safety, and emotional intelligence at its core, the platform creates a more mindful and engaging social environment.

### Key Value Propositions
- **Emotional Intelligence**: Real-time sentiment analysis helps users understand the emotional impact of their content
- **Safety First**: Automated content moderation prevents harmful content before it spreads
- **Performance**: High-performance Rust backend ensures fast, responsive interactions
- **Transparency**: Users see sentiment scores and moderation decisions, fostering accountability
- **Engagement**: Emotion-based voting and sophisticated popularity algorithms drive meaningful interactions

## User Features

### 🎭 Emotion-Aware Posting
- **Real-time Sentiment Preview**: See how your post will be perceived before publishing
- **Emotion Categories**: Joy, Anger, Sadness, Fear, Disgust, Surprise, Confusion, Sarcasm, Affection, and Neutral
- **Visual Emotion Indicators**: Color-coded sentiment tags for immediate emotional context
- **Sentiment Confidence Scores**: Transparency in AI emotion detection

### 🛡️ Smart Content Moderation
- **Automatic Toxicity Detection**: AI-powered content screening for harmful material
- **Toxicity Categories**: Detailed classification (toxicity, insults, threats, hate speech)
- **Transparent Moderation**: Users see moderation decisions and can understand why content was flagged
- **Fail-Safe Design**: System defaults to safety when AI services are unavailable

### 💬 Hierarchical Comment System
- **Reddit-Style Threading**: Nested conversations up to 10 levels deep
- **Materialized Path Storage**: Efficient tree queries for fast comment loading
- **Emotion Analysis on Comments**: Every comment gets sentiment analysis
- **Context-Aware Replies**: See parent comment context in mobile views

### 🗳️ Emotion-Based Voting
- **Multi-Dimensional Voting**: Vote on different emotional aspects of content
- **Popularity Algorithm**: Sophisticated scoring that balances recency, engagement, and quality
- **Vote Aggregation**: Community sentiment reflected in aggregate emotion scores
- **Anti-Gaming Measures**: One vote per user per post/comment

### 🚀 Modern User Experience
- **Infinite Scroll**: Seamless content discovery with optimized pagination
- **Optimistic UI**: Immediate feedback for posts, comments, and votes
- **Mobile-First Design**: Responsive interface optimized for all devices
- **Real-Time Updates**: Live sentiment analysis as you type
- **Fast Load Times**: Efficient caching and database optimization

## Technology Stack

### Backend Architecture

#### Core Framework: Rust + Axum
- **Language**: Rust for memory safety, performance, and concurrency
- **Web Framework**: Axum for high-performance HTTP handling
- **Async Runtime**: Tokio for efficient concurrent operations
- **Performance Benefits**: Zero-cost abstractions, fearless concurrency, memory safety

#### Database Layer
- **Primary Database**: PostgreSQL with advanced features
- **ORM**: SQLx for compile-time verified queries
- **Connection Pooling**: Configured for 20 concurrent connections with timeouts
- **Data Models**: Optimized schemas for social media workloads

#### AI/ML Integration
- **Python Microservice**: Dedicated subprocess for AI operations
- **Sentiment Analysis**: HuggingFace EmotionClassifier + custom pattern detection
- **Content Moderation**: HateSonar + TextBlob for toxicity detection
- **Process Management**: Supervised subprocess with health checks and auto-restart
- **Fallback Strategy**: Graceful degradation when AI services are unavailable

#### Security & Authentication
- **Authentication**: JWT tokens with HS256 algorithm
- **Password Security**: Argon2 hashing with secure salt generation
- **Authorization**: Middleware-based route protection
- **Session Management**: 24-hour token expiration with secure claims

### Frontend Architecture

#### Technology Choice: Vanilla JavaScript SPA
- **Framework**: Pure JavaScript for minimal bundle size and maximum performance
- **Architecture**: Component-based design with modular organization
- **State Management**: Local state with optimistic updates
- **Styling**: CSS3 with Inter font family for modern typography

#### Key Frontend Features
- **Modal-Based Interactions**: Clean, focused user interactions
- **Real-Time Sentiment Preview**: Live emotion analysis while typing
- **Infinite Scroll**: Efficient pagination with lazy loading
- **Optimistic UI**: Immediate feedback for all user actions
- **Mobile Optimization**: Touch-friendly interface with responsive design

## System Design

### Request Flow Architecture

```
Client (Browser)
    ↓ HTTP/HTTPS
Axum Web Server
    ↓ Route Matching
Middleware Layer (Auth, CORS, Logging)
    ↓ Business Logic
Service Layer (Posts, Comments, Auth, etc.)
    ↓ Data Access
Repository Pattern
    ↓ SQL Queries
PostgreSQL Database

    ↕ Python Subprocess Communication
Python AI Server (Sentiment + Moderation)
```

### Data Flow for Content Creation

1. **User Input**: User types content in frontend
2. **Real-Time Analysis**: Frontend calls sentiment preview API
3. **Python Processing**: Rust spawns Python analysis subprocess
4. **Sentiment Response**: Emotion scores and colors returned
5. **UI Update**: Frontend shows live sentiment preview
6. **Post Submission**: User submits with full content analysis
7. **Moderation Check**: Content moderation pipeline runs
8. **Database Storage**: Post saved with sentiment and moderation metadata
9. **Feed Update**: New post appears in feeds with popularity scoring

### Database Schema Design

#### Posts Table
```sql
CREATE TABLE posts (
    id UUID PRIMARY KEY,
    title VARCHAR NOT NULL,
    content TEXT NOT NULL,
    author_id UUID REFERENCES users(id),
    author_username VARCHAR NOT NULL, -- Denormalized for performance
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    comment_count INTEGER DEFAULT 0,
    sentiment_score DOUBLE PRECISION, -- -1.0 to 1.0
    sentiment_colors TEXT[], -- Color codes for UI
    sentiment_type VARCHAR, -- Human-readable emotion
    sentiment_analysis JSONB, -- Full analysis data
    popularity_score DOUBLE PRECISION DEFAULT 1.0,
    is_blocked BOOLEAN DEFAULT FALSE,
    toxicity_tags TEXT[], -- Moderation categories
    toxicity_scores JSONB -- Full moderation data
);
```

#### Hierarchical Comments
```sql
CREATE TABLE comments (
    id UUID PRIMARY KEY,
    post_id UUID REFERENCES posts(id),
    content TEXT NOT NULL,
    user_id UUID REFERENCES users(id),
    parent_id UUID REFERENCES comments(id), -- NULL for root comments
    path VARCHAR NOT NULL, -- Materialized path: "1/3/7/"
    depth INTEGER NOT NULL, -- Nesting level (0-10)
    sentiment_score DOUBLE PRECISION,
    sentiment_colors TEXT[],
    sentiment_type VARCHAR,
    is_blocked BOOLEAN DEFAULT FALSE,
    toxicity_tags TEXT[],
    toxicity_scores JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### AI Service Architecture

#### Python Subprocess Management
- **Process Supervision**: Automatic restart on crashes
- **Health Monitoring**: Regular health checks with HTTP endpoints
- **Timeout Handling**: 5-second timeouts prevent hanging
- **Graceful Shutdown**: Clean subprocess termination on server shutdown
- **Error Recovery**: Fallback responses when AI services fail

#### Sentiment Analysis Pipeline
1. **Text Preprocessing**: Clean and normalize input text
2. **Emotion Classification**: HuggingFace model inference
3. **Pattern Detection**: Custom rules for sarcasm and complex emotions
4. **Confidence Scoring**: Return confidence levels for transparency
5. **Color Mapping**: Convert emotions to UI color codes
6. **Response Formatting**: Structure data for frontend consumption

#### Content Moderation Pipeline
1. **Toxicity Detection**: HateSonar analysis for harmful content
2. **Category Classification**: Specific violation types (insults, threats, etc.)
3. **Scoring**: Numerical toxicity scores for transparency
4. **Blocking Decision**: Binary block/allow with detailed reasoning
5. **Audit Trail**: Complete moderation data stored for review

## Security & Privacy

### Authentication Security
- **JWT Implementation**: Secure token-based authentication
- **Password Protection**: Argon2 hashing with crypto-secure salts
- **Token Expiration**: 24-hour token lifetime
- **Secure Headers**: CORS and security headers configured

### Data Protection
- **Input Validation**: Server-side validation for all user inputs
- **SQL Injection Prevention**: Parameterized queries with SQLx
- **XSS Protection**: Content sanitization and secure headers
- **Token-Based Security**: JWT tokens in Authorization headers (no CSRF tokens needed for stateless API)

### Content Moderation Security
- **Proactive Filtering**: Content blocked before publication
- **Transparency**: Users see moderation decisions
- **Appeal Process**: Clear moderation reasoning provided
- **Privacy**: Sensitive data handled securely

## Performance & Scalability

### Database Optimization
- **Connection Pooling**: 20 concurrent connections with lifecycle management
- **Query Optimization**: Indexed queries for fast lookups
- **Pagination**: Efficient cursor-based pagination for large datasets
- **Denormalization**: Strategic data duplication for performance

### Caching Strategy
- **Application-Level Caching**: In-memory caching of frequently accessed data
- **Database Query Optimization**: Efficient indexes and query patterns
- **Static Asset Caching**: Frontend assets cached with proper headers
- **AI Response Caching**: Cache sentiment analysis for duplicate content

### Popularity Algorithm
```rust
// Sophisticated popularity scoring with time decay
let age_hours = (Utc::now() - created_at).num_hours() as f64;
let time_decay = (-age_hours / 24.0).exp(); // Exponential decay
let engagement_boost = (comment_count as f64 + 1.0).ln();
let sentiment_factor = sentiment_score.unwrap_or(0.0).abs() * 0.5 + 1.0;

popularity_score = base_score * time_decay * engagement_boost * sentiment_factor;
```

### Scalability Features
- **Async Processing**: Non-blocking I/O for high concurrency
- **Horizontal Scaling**: Stateless design supports load balancing
- **Database Scaling**: Read replicas and connection pooling
- **Microservice Ready**: AI services can be extracted to separate services

## Deployment Architecture

### Environment Configuration
```bash
# Database
DATABASE_URL=postgresql://user:pass@localhost/social_pulse

# Authentication
JWT_SECRET=your-secure-secret-here

# Python AI Services
PYTHON_SERVER_MODE=subprocess  # or external/disabled
PYTHON_SERVER_HOST=127.0.0.1
PYTHON_SERVER_PORT=8001

# Server Configuration
RUST_LOG=info
PORT=5000

# Production (if needed)
RELEASE_MODE=true
STATIC_FILE_CACHE_CONTROL=no-cache
```

### Quick Start Guide

1. **Install Dependencies**:
   ```bash
   # Install Rust and Python dependencies
   cargo build
   pip install -r requirements.txt
   ```

2. **Setup Database**:
   ```bash
   # Create PostgreSQL database
   createdb social_pulse
   # Run migrations
   sqlx migrate run
   ```

3. **Start Services**:
   ```bash
   # Start the main application (includes Python subprocess)
   cargo run
   ```

4. **Access Application**:
   - Web Interface: http://localhost:5000
   - API Endpoints: http://localhost:5000/api
   - Health Check: http://localhost:5000/api/health

### Startup Sequence
1. **Configuration Loading**: Environment variables and config validation
2. **Database Connection**: PostgreSQL pool initialization and health check
3. **Python Service**: Start and wait for AI subprocess health
4. **Sample Data**: Populate demonstration posts if needed
5. **Server Launch**: Bind to port and start accepting requests
6. **Health Monitoring**: Background health checks and supervision

### Production Considerations
- **Process Supervision**: systemd or Docker for process management
- **Load Balancing**: Multiple instances behind reverse proxy
- **Database Backup**: Regular PostgreSQL backups and point-in-time recovery
- **Monitoring**: Application metrics and health endpoints
- **Logging**: Structured logging with log aggregation

## Performance Metrics & SLOs

### Response Time Targets
- **API Endpoints**: < 100ms for simple queries, < 500ms for complex
- **Sentiment Analysis**: < 2 seconds for real-time preview
- **Content Moderation**: < 3 seconds for post submission
- **Page Load**: < 2 seconds for initial page load

### Availability Targets
- **Overall System**: 99.9% uptime
- **Core Features**: 99.95% availability for posting and viewing
- **AI Services**: 99% availability with graceful degradation

### Capacity Planning
- **Concurrent Users**: Designed for 1000+ concurrent users
- **Database Connections**: 20 concurrent connections per instance
- **Post Throughput**: 100+ posts per minute
- **Comment Throughput**: 500+ comments per minute

## Development & Maintenance

### Code Quality
- **Type Safety**: Rust's compile-time guarantees prevent runtime errors
- **Error Handling**: Comprehensive error types and graceful degradation
- **Testing**: Unit tests for business logic and integration tests for APIs
- **Documentation**: Inline documentation and architectural decision records

### Monitoring & Observability
- **Logging**: Structured logging with correlation IDs
- **Metrics**: Application performance metrics
- **Health Checks**: Endpoint health monitoring
- **Error Tracking**: Comprehensive error logging and alerting

## Future Roadmap

### Short-Term Enhancements (3-6 months)
- **Real-Time Features**: WebSocket support for live comments
- **Advanced Moderation**: Machine learning model improvements
- **Mobile App**: Native mobile applications
- **Content Search**: Full-text search with sentiment filtering

### Long-Term Vision (6-12 months)
- **Recommendation Engine**: Personalized content recommendations
- **Multi-Language Support**: International expansion
- **Enterprise Features**: Organization accounts and moderation dashboards
- **API Platform**: Third-party developer APIs

### Technical Improvements
- **Microservices Migration**: Extract AI services to independent services
- **CDN Integration**: Global content delivery network
- **Advanced Analytics**: User behavior analytics and sentiment trends
- **Machine Learning Pipeline**: Continuous model improvement

---

*Social Pulse represents the future of social media: intelligent, safe, and emotionally aware. By combining cutting-edge technology with human-centered design, we're creating a platform where meaningful connections can flourish.*