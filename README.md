# Social Pulse - Sentiment-Based Social Media Platform

Social Pulse is a modern social media application that combines traditional social networking features with AI-powered sentiment analysis, hierarchical comment system, emotion-based voting, and comprehensive content moderation. Built with a high-performance Rust backend using Axum framework, vanilla JavaScript frontend with advanced caching, and PostgreSQL database persistence, featuring HuggingFace EmotionClassifier and Detoxify for real-time content analysis.

## 🏗️ Technical Architecture

### Backend Architecture
- **Rust Web Server**: Asynchronous Axum framework with tokio runtime for high concurrency
- **Database Layer**: PostgreSQL with SQLx for type-safe database operations and connection pooling
- **Authentication**: Stateless JWT tokens with Argon2 password hashing
- **Middleware Stack**: Custom authentication, CORS, error handling, and request tracing
- **API Design**: RESTful endpoints with JSON serialization using serde

### AI Processing Architecture
- **Python Subprocess Management**: AI server runs as managed subprocess with health monitoring
- **Sentiment Analysis**: HuggingFace EmotionClassifier with 10 emotion categories
- **Content Moderation**: Detoxify for toxicity detection and content flagging
- **Model Caching**: Persistent HuggingFace model caching for faster startup times
- **Process Supervision**: Automatic restart with exponential backoff on Python process failure

### Frontend Architecture
- **Single Page Application**: Vanilla JavaScript with component-based organization
- **Advanced Caching**: LRU cache management for posts, comments, and votes
- **Optimistic UI**: Immediate feedback with rollback capability for failed operations
- **Infinite Scroll**: Efficient pagination with loading indicators and error handling
- **Responsive Design**: Mobile-first design with touch-optimized interactions

## 🚀 Features

### Core Social Features
- **User Authentication**: JWT-based secure authentication with registration and login
- **Post Creation & Management**: Create, view, edit, and delete posts with rich text content
- **Hierarchical Comment System**: Reddit-style threaded comments with replies and nested discussions
- **Emotion-Based Voting**: Interactive voting system with emotion tags (joy, sad, angry, etc.) and content filter tags
- **User Profiles**: Personalized "My Posts" pages with user-specific content management
- **Infinite Scroll Feed**: Instagram/Facebook/Reddit-style continuous feed loading

### AI-Powered Content Analysis
- **Advanced Sentiment Analysis**: HuggingFace EmotionClassifier with 10 emotion categories
- **Content Moderation**: Detoxify-based toxicity detection with automatic content flagging
- **Real-time Processing**: Content analyzed before publication with immediate sentiment feedback
- **Emotion Categories**: Joy, sad, angry, fear, disgust, surprise, confused, neutral, sarcastic, affectionate
- **Toxicity Detection**: Multi-tier toxicity detection with blocking and tagging systems

### Technical Features
- **PostgreSQL Database**: Production-ready database with automatic migrations
- **Subprocess Management**: Python AI server runs as managed subprocess with health monitoring
- **Model Caching**: HuggingFace model caching for faster startup times
- **Responsive Design**: Mobile-friendly interface with touch-optimized interactions
- **Delete Controls**: Context-aware delete permissions (only on "My Posts" page)

## 🛠️ Setup Instructions

### Prerequisites

**System Requirements:**
- **Rust 1.70+** with Cargo package manager
- **Node.js 16+** (for utility scripts)
- **Python 3.11+** with pip package manager
- **PostgreSQL 13+** database server
- **4GB+ RAM** (for AI model loading)

### Installation

#### 1. Clone the Repository
```bash
git clone <repository-url>
cd social-pulse
```

#### 2. Install Rust Dependencies
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build the project (downloads and compiles dependencies)
cargo build --release
```

#### 3. Install Python Dependencies
```bash
# Create virtual environment (recommended)
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install AI processing dependencies
pip install emotionclassifier hatesonar nrclex numpy opencv-python pillow scikit-learn scipy text2emotion textblob torch detoxify
```

#### 4. Database Setup

**Option A: Local PostgreSQL**
```bash
# Install PostgreSQL (Ubuntu/Debian)
sudo apt-get install postgresql postgresql-contrib

# Create database and user
sudo -u postgres psql
CREATE DATABASE social_media;
CREATE USER social_pulse WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE social_media TO social_pulse;
\q
```

**Option B: Docker PostgreSQL**
```bash
docker run --name social-pulse-db \
  -e POSTGRES_DB=social_media \
  -e POSTGRES_USER=social_pulse \
  -e POSTGRES_PASSWORD=your_password \
  -p 5432:5432 \
  -d postgres:15
```

#### 5. Environment Configuration
```bash
# Create .env file (optional - defaults provided)
cat > .env << EOF
# Database connection
DATABASE_URL=postgresql://social_pulse:your_password@localhost:5432/social_media

# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=5000

# AI server mode (subprocess or external)
PYTHON_SERVER_MODE=subprocess

# JWT secret (change in production)
JWT_SECRET=your-secure-secret-key-change-in-production
EOF
```

#### 6. Run the Application
```bash
# Start the complete application
cargo run

# Or run in release mode for better performance
cargo run --release
```

### First Run Process

The application performs these steps on startup:

1. **Database Schema Creation**: Automatically creates tables and indexes
2. **Python AI Server Startup**: Launches managed subprocess for AI processing
3. **Model Initialization**: Downloads and caches HuggingFace models (2-5 minutes first time)
4. **Web Server Start**: Binds to configured port (default: 5000)
5. **Health Check**: Verifies all components are operational

### Development Mode

For development with auto-recompilation:

```bash
# Install cargo-watch for file watching
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run
```

## 🗄️ Database Architecture

### Schema Design

The application uses **PostgreSQL** with the following core entities:

**Users Table**
- `id` (UUID): Primary key
- `email` (varchar): Unique email address
- `username` (varchar): Display name
- `password_hash` (varchar): Argon2 hashed password
- `created_at`, `updated_at` (timestamp): Audit fields

**Posts Table**
- `id` (UUID): Primary key
- `user_id` (UUID): Foreign key to users
- `title` (varchar): Post title
- `content` (text): Post content
- `sentiment_type` (varchar): AI-detected primary emotion
- `sentiment_colors` (text[]): All detected emotions
- `sentiment_score` (float): Confidence score
- `comment_count` (integer): Cached comment count
- `popularity_score` (float): Engagement-based ranking

**Comments Table**
- `id` (UUID): Primary key
- `post_id` (UUID): Foreign key to posts
- `parent_id` (UUID): Self-referencing for threading
- `user_id` (UUID): Foreign key to users
- `content` (text): Comment content
- `sentiment_type`, `sentiment_colors`, `sentiment_score`: AI analysis
- `depth` (integer): Nesting level for efficient queries

**Votes Table**
- `id` (UUID): Primary key
- `user_id` (UUID): Foreign key to users
- `target_id` (UUID): Posts or comments
- `target_type` (varchar): 'post' or 'comment'
- `vote_type` (varchar): 'upvote' or 'downvote'
- `tag` (varchar): Emotion tag (joy, sad, angry, etc.)

### Database Features

- **Automatic Schema Management**: Schema created from Rust models on startup
- **Connection Pooling**: SQLx connection pool for efficient database access
- **Prepared Statements**: All queries use prepared statements for security
- **Indexing Strategy**: Optimized indexes on foreign keys and query patterns
- **Transaction Support**: ACID compliance for data consistency

## 🤖 AI Processing Pipeline

### Sentiment Analysis Flow

1. **Content Submission**: User submits post/comment content
2. **Python AI Server**: Rust sends HTTP request to Python subprocess
3. **HuggingFace Processing**: EmotionClassifier analyzes text for emotions
4. **Pattern Detection**: Custom regex patterns detect sarcasm and affection
5. **Result Aggregation**: Combines ML results with rule-based detection
6. **Normalization**: Processes compound emotions (e.g., "sarcastic+joy" → "sarcastic")
7. **Database Storage**: Saves sentiment data alongside content

### Content Moderation Pipeline

1. **Toxicity Detection**: Detoxify analyzes content for harmful language
2. **Multi-tier Classification**: Toxicity, severe toxicity, obscene, threat, insult, identity attack
3. **Threshold Evaluation**: Configurable thresholds for blocking vs. tagging
4. **User Feedback**: Real-time warnings during content creation
5. **Automatic Actions**: Content flagging and optional blocking

### Model Specifications

**EmotionClassifier (HuggingFace)**
- **Architecture**: BERT-based transformer model
- **Categories**: joy, sadness, anger, fear, disgust, surprise, neutral, confused
- **Additional**: Custom sarcasm and affection detection
- **Memory Usage**: ~800MB RAM
- **Inference Time**: ~100ms per text

**Detoxify**
- **Architecture**: BERT-based toxicity classifier
- **Categories**: toxicity, severe_toxicity, obscene, threat, insult, identity_attack
- **Memory Usage**: ~400MB RAM
- **Inference Time**: ~50ms per text

### Python Server Architecture

The Python AI server (`python_scripts/async_server.py`) provides:

**Endpoints:**
- `POST /analyze`: Sentiment analysis
- `POST /moderate`: Content moderation
- `GET /health`: Health check

**Features:**
- **Async Processing**: FastAPI with async/await for concurrent requests
- **Model Caching**: Models loaded once and reused across requests
- **Error Handling**: Graceful fallbacks for model failures
- **Process Management**: Managed by Rust parent process with supervision

### Data Population Scripts

The `scripts/` directory contains utilities for testing and development:

```bash
# Populate with 12 users and 36 posts with realistic data
node scripts/populate_data.js

# Add threaded comments to existing posts
node scripts/populate_comments.js
```

## 📊 API Endpoints

### Authentication
- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login

### Posts
- `GET /api/posts` - List all posts with pagination
- `POST /api/posts` - Create new post (requires auth)
- `GET /api/posts/:id` - Get specific post
- `PUT /api/posts/:id` - Update post (requires auth)
- `DELETE /api/posts/:id` - Delete post (requires auth)
- `GET /api/posts/user/:user_id` - Get user's posts

### Comments
- `GET /api/posts/:post_id/comments` - Get comments for post
- `POST /api/posts/:post_id/comments` - Create comment (requires auth)
- `GET /api/comments/:comment_id` - Get specific comment
- `PUT /api/comments/:comment_id` - Update comment (requires auth)
- `DELETE /api/comments/:comment_id` - Delete comment (requires auth)
- `GET /api/comments/:comment_id/thread` - Get comment thread

### Voting System
- `POST /api/vote/:target_id/:target_type/:vote_type/:tag` - Cast vote (requires auth)
- `DELETE /api/vote/:target_id/:target_type/:vote_type/:tag` - Remove vote (requires auth)
- `GET /api/vote/:target_id/:target_type` - Get vote summary

### AI Services
- `POST /api/analyze` - Sentiment analysis (internal)
- `POST /api/moderate` - Content moderation (internal)

## 🏗️ Project Structure

```
social-pulse/
├── src/                          # Rust backend source code
│   ├── main.rs                   # Application entry point
│   ├── models/                   # Data models (User, Post, Comment, Vote)
│   ├── routes/                   # HTTP route handlers
│   ├── services/                 # Business logic services
│   ├── auth/                     # Authentication & middleware
│   └── db/                       # Database repositories
├── python_scripts/               # Python AI modules
│   ├── async_server.py          # HTTP server for AI endpoints
│   ├── sentiment_analyzer.py    # HuggingFace sentiment analysis
│   └── content_moderator.py     # Detoxify content moderation
├── static/                       # Frontend assets
│   ├── index.html               # Main HTML page
│   ├── app-v2.js               # JavaScript application
│   └── styles.css              # CSS styling
├── scripts/                      # Data population scripts
│   ├── populate_data.js         # Full data population
│   └── populate_comments.js     # Comment population
├── Cargo.toml                   # Rust dependencies
├── replit.nix                   # Nix configuration for Replit
└── README.md                   # This file
```

## 🎯 Usage

### Creating Content

1. **Register/Login**: Create an account or sign in
2. **Create Posts**: Click "Create New Post" and add content
3. **Real-time Analysis**: See sentiment analysis as you type
4. **Content Warnings**: Receive warnings for potentially problematic content

### Interacting with Content

1. **Voting**: Click emotion tags (😊 Joy, 😢 Sad, etc.) to vote on posts
2. **Comments**: Add threaded comments and replies to posts
3. **Delete Controls**: Delete your own content from the "My Posts" page

### Viewing Content

1. **Main Feed**: Scroll through posts with infinite loading
2. **User Posts**: Visit "My Posts" to see and manage your content
3. **Comment Threads**: Click comment counts to view discussions

## 🧪 Testing

### Frontend Testing
1. Open the application in Replit's webview
2. Create posts with different emotional content:
   - **Joy**: "I'm so happy and excited today!"
   - **Sarcasm**: "Oh great, just perfect timing"
   - **Affection**: "I love you so much"
   - **Toxicity**: Content with mild toxicity (tagged but not blocked)

### API Testing
```bash
# Test health endpoints
curl http://localhost:5000/api/health

# Register a new user
curl -X POST http://localhost:5000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","username":"testuser","password":"password123"}'

# Login to get JWT token
curl -X POST http://localhost:5000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'

# Test post creation (requires auth token)
curl -X POST http://localhost:5000/api/posts \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test Post","content":"This is a test post with sentiment analysis!"}'

# Test voting system
curl -X POST http://localhost:5000/api/vote/POST_ID/post/upvote/joy \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Test comment creation
curl -X POST http://localhost:5000/api/posts/POST_ID/comments \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"content":"This is a test comment"}'
```

## 🔧 Development

## ⚙️ Configuration

### Environment Variables

The application supports configuration through environment variables:

**Database Configuration:**
- `DATABASE_URL`: PostgreSQL connection string
  - Format: `postgresql://user:password@host:port/database`
  - Default: Uses individual PGHOST, PGPORT, etc. variables

**Individual Database Variables:**
- `PGHOST`: Database host (default: `localhost`)
- `PGPORT`: Database port (default: `5432`)
- `PGUSER`: Database username
- `PGPASSWORD`: Database password
- `PGDATABASE`: Database name (default: `social_media`)

**Server Configuration:**
- `SERVER_HOST`: Host to bind to (default: `0.0.0.0`)
- `SERVER_PORT` or `PORT`: Port to bind to (default: `5000`)

**Security Configuration:**
- `JWT_SECRET`: JWT signing secret (default: dev secret - **CHANGE IN PRODUCTION**)
- `SESSION_SECRET`: Session secret for additional security

**AI Configuration:**
- `PYTHON_SERVER_MODE`: `subprocess` (default) or `external`
- `PYTHON_SERVER_HOST`: External Python server host (if external mode)
- `PYTHON_SERVER_PORT`: External Python server port (if external mode)

### Configuration Modes

**Subprocess Mode (Default):**
- Python AI server runs as managed subprocess
- Automatic startup, supervision, and shutdown
- Logs integrated with main application
- Recommended for single-machine deployments

**External Mode:**
- Python AI server runs independently
- Requires manual startup and management
- Better for containerized or distributed deployments
- Configure `PYTHON_SERVER_HOST` and `PYTHON_SERVER_PORT`

### Production Configuration

For production deployment, ensure:

1. **Strong JWT Secret**: Generate cryptographically secure JWT_SECRET
2. **Database Security**: Use TLS connections and strong passwords
3. **Resource Limits**: Allocate sufficient RAM for AI models (4GB+ recommended)
4. **Environment Isolation**: Use environment-specific configuration files
5. **Monitoring**: Set up logging and health check monitoring

## 🚀 Deployment

### Production Deployment Options

**Option 1: Direct Deployment**
```bash
# Build optimized release binary
cargo build --release

# Set production environment variables
export JWT_SECRET="your-secure-production-secret"
export DATABASE_URL="postgresql://user:pass@host:5432/social_media"

# Run the application
./target/release/social-pulse
```

**Option 2: Docker Deployment**
```dockerfile
# Example Dockerfile structure
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM python:3.11-slim
RUN apt-get update && apt-get install -y libpq5
COPY --from=builder /app/target/release/social-pulse /usr/local/bin/
COPY python_scripts/ /app/python_scripts/
COPY static/ /app/static/
RUN pip install -r /app/requirements.txt
EXPOSE 5000
CMD ["social-pulse"]
```

**Option 3: Cloud Platform Deployment**
- **AWS**: EC2 instances with RDS PostgreSQL
- **Google Cloud**: Compute Engine with Cloud SQL
- **Azure**: Virtual Machines with Azure Database for PostgreSQL
- **Railway/Fly.io**: Platform-as-a-Service with managed databases

### Load Balancing and Scaling

**Horizontal Scaling:**
- Multiple Rust server instances behind load balancer
- Shared PostgreSQL database
- External Python AI server for model sharing

**Database Scaling:**
- Read replicas for query performance
- Connection pooling configuration
- Database performance monitoring

### Database Management

**Schema Management:**
- Automatic schema creation on startup
- Version-controlled schema changes
- Database migration best practices

**Backup and Recovery:**
```bash
# Create database backup
pg_dump $DATABASE_URL > backup.sql

# Restore from backup
psql $DATABASE_URL < backup.sql

# Continuous backup (production)
# Set up automated backups with pg_basebackup or cloud provider tools
```

**Performance Monitoring:**
- Database query performance analysis
- Connection pool monitoring
- Index usage optimization

### Python AI Server

The AI server runs automatically as a subprocess:

- **Startup**: Managed by Rust application
- **Health Monitoring**: Automatic restart on failure
- **Model Caching**: Models cached for faster subsequent starts
- **Logging**: Integrated with main application logs

## 🚨 Troubleshooting

### Common Issues

1. **Server Won't Start**
   ```bash
   # Check Python dependencies
   pip list | grep -E "(emotion|detoxify|torch)"
   
   # Verify database connection
   psql $DATABASE_URL -c "SELECT 1;"
   
   # Check Rust compilation
   cargo check
   
   # View detailed logs
   RUST_LOG=debug cargo run
   ```

2. **AI Models Not Loading**
   ```bash
   # First run downloads models (2-5 minutes)
   # Check available memory
   free -h
   
   # Monitor model download progress
   tail -f ~/.cache/huggingface/transformers/
   
   # Test Python AI server independently
   cd python_scripts && python async_server.py
   ```

3. **Database Connection Issues**
   ```bash
   # Test database connectivity
   pg_isready -h $PGHOST -p $PGPORT -U $PGUSER
   
   # Check database exists
   psql $DATABASE_URL -c "\l"
   
   # Verify schema
   psql $DATABASE_URL -c "\dt"
   
   # Check database logs
   sudo journalctl -u postgresql
   ```

4. **Frontend Not Loading**
   ```bash
   # Verify server is running
   curl -I http://localhost:5000/
   
   # Check static file serving
   curl http://localhost:5000/static/app-v2.js
   
   # Test API endpoints
   curl http://localhost:5000/api/health
   
   # Browser developer tools for client-side errors
   # Open Network tab and Console for debugging
   ```

5. **Permission Issues**
   ```bash
   # Python script permissions
   chmod +x python_scripts/*.py
   
   # Database permissions
   sudo -u postgres psql -c "GRANT ALL ON DATABASE social_media TO social_pulse;"
   
   # File system permissions
   ls -la static/
   ```

## ⚡ Performance Optimization

### Backend Performance

**Database Optimization:**
```sql
-- Key indexes automatically created
CREATE INDEX idx_posts_user_id ON posts(user_id);
CREATE INDEX idx_posts_created_at ON posts(created_at DESC);
CREATE INDEX idx_comments_post_id ON comments(post_id);
CREATE INDEX idx_votes_target ON votes(target_id, target_type);
```

**Connection Pooling:**
- SQLx connection pool with configurable limits
- Optimal pool size: CPU cores × 2
- Connection timeout and idle handling

**Memory Management:**
- AI models cached in memory (~1.2GB total)
- Rust zero-copy string operations
- Efficient JSON serialization with serde

### Frontend Performance

**Caching Strategy:**
```javascript
// LRU caches with configurable limits
const postsCache = new LRUCache(100);    // 100 posts
const commentsCache = new LRUCache(500); // 500 comments
const votesCache = new LRUCache(1000);   // 1000 vote states
```

**Optimistic UI Updates:**
- Immediate visual feedback for user actions
- Background API calls with rollback on failure
- Reduced perceived latency

**Efficient Rendering:**
- Virtual scrolling for large datasets
- Debounced search and input handling
- Minimal DOM manipulation

### AI Processing Performance

**Model Optimization:**
- HuggingFace model caching in `~/.cache/huggingface/`
- Batch processing for multiple requests
- Async processing prevents blocking

**Subprocess Management:**
- Persistent Python process avoids startup overhead
- Connection pooling for HTTP requests to AI server
- Graceful degradation on AI server unavailability

### Performance Monitoring

**Key Metrics:**
- Response time per endpoint
- Database query performance
- Memory usage (especially AI models)
- Cache hit rates
- Python subprocess health

**Optimization Tips:**
- Monitor database query patterns
- Use EXPLAIN ANALYZE for slow queries
- Profile memory usage with cargo instruments
- Benchmark API endpoints under load

## 🔒 Security Features

- **JWT Authentication**: Stateless authentication with secure token handling
- **Password Hashing**: Argon2 password hashing for user security
- **Content Moderation**: Automatic detection and flagging of toxic content
- **Input Validation**: Comprehensive validation on both client and server
- **SQL Injection Protection**: Parameterized queries prevent SQL injection
- **CORS Configuration**: Proper cross-origin resource sharing setup

## 📈 Monitoring and Observability

### Health Checks

**Application Health:**
```bash
# Overall application health
curl http://localhost:5000/api/health

# Database connectivity
curl http://localhost:5000/api/health/db

# Python AI server health
curl http://localhost:5000/api/health/ai
```

### Logging

**Structured Logging:**
- JSON-formatted logs with trace IDs
- Different log levels (ERROR, WARN, INFO, DEBUG)
- Request/response logging with timing
- Database query logging with performance metrics

**Log Configuration:**
```bash
# Set log level
export RUST_LOG=info

# Detailed debugging
export RUST_LOG=social_pulse=debug,sqlx=info

# Production logging
export RUST_LOG=warn,social_pulse=info
```

### Error Handling

**Error Categories:**
- **Validation Errors**: Input validation with detailed messages
- **Authentication Errors**: JWT token validation and expiration
- **Database Errors**: Connection issues, constraint violations
- **AI Processing Errors**: Model failures with fallback responses
- **Network Errors**: Timeout and connectivity issues

**Error Response Format:**
```json
{
  "error": "ValidationError",
  "message": "Invalid email format",
  "details": {
    "field": "email",
    "code": "INVALID_FORMAT"
  },
  "trace_id": "req_abc123"
}
```

### Performance Metrics

**Built-in Metrics:**
- Request duration and throughput
- Database query performance
- Memory usage tracking
- Cache hit/miss ratios
- AI processing latency

**Production Monitoring:**
- Set up Prometheus metrics collection
- Grafana dashboards for visualization
- AlertManager for critical alerts
- Log aggregation with ELK stack or similar

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes with proper tests
4. Run tests: `cargo test`
5. Submit a pull request

### Development Guidelines

- Follow Rust naming conventions and error handling patterns
- Add comprehensive tests for new features
- Update documentation for API changes
- Ensure all code passes formatting: `cargo fmt`
- Run clippy for linting: `cargo clippy`

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🔧 Advanced Configuration

### Custom AI Models

To use custom or different AI models:

1. **Replace EmotionClassifier:**
```python
# In python_scripts/sentiment_analyzer.py
from transformers import pipeline
classifier = pipeline('text-classification', model='your-custom-model')
```

2. **Alternative Toxicity Models:**
```python
# In python_scripts/content_moderator.py
from transformers import pipeline
toxicity_analyzer = pipeline('text-classification', model='your-toxicity-model')
```

### Database Customization

**Custom Database Schema:**
```rust
// Add custom fields to models in src/models/
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub custom_field: Option<String>, // Add custom fields
    // ... existing fields
}
```

### Frontend Customization

**Custom Themes:**
```css
/* Modify static/styles.css */
:root {
    --primary-color: #your-color;
    --secondary-color: #your-secondary;
    /* Custom CSS variables */
}
```

**Custom Components:**
```javascript
// Add to static/app-v2.js
function createCustomComponent(data) {
    // Custom UI components
}
```

## 🌟 Contributing

### Development Workflow

1. **Fork and Clone:**
```bash
git clone https://github.com/your-username/social-pulse.git
cd social-pulse
```

2. **Set Up Development Environment:**
```bash
# Install development dependencies
cargo install cargo-watch cargo-audit
pip install black isort flake8  # Python code formatting
```

3. **Code Quality:**
```bash
# Rust formatting and linting
cargo fmt
cargo clippy --all-targets --all-features

# Python formatting
black python_scripts/
isort python_scripts/

# Run tests
cargo test
python -m pytest python_scripts/tests/
```

4. **Commit Standards:**
```bash
# Use conventional commits
git commit -m "feat: add new emotion category"
git commit -m "fix: resolve database connection issue"
git commit -m "docs: update API documentation"
```

### Architecture Guidelines

- **Rust Backend**: Follow async/await patterns, use proper error handling
- **Database**: Use prepared statements, implement proper indexing
- **Frontend**: Maintain component-based structure, implement caching
- **AI Integration**: Handle model failures gracefully, implement fallbacks
- **Security**: Validate all inputs, use parameterized queries, secure authentication

### Testing

**Backend Tests:**
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Performance tests
cargo bench
```

**Frontend Tests:**
```javascript
// Manual testing with browser developer tools
// API testing with curl or Postman
// Load testing with tools like wrk or artillery
```

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**Social Pulse: Where AI meets authentic social interaction 🚀**