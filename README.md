# Social Pulse - Sentiment-Based Social Media Platform

Social Pulse is a modern social media application that combines traditional social networking features with AI-powered sentiment analysis, hierarchical comment system, emotion-based voting, and comprehensive content moderation. Built with Rust (Axum backend), vanilla JavaScript frontend, and PostgreSQL database, featuring HuggingFace EmotionClassifier and Detoxify for real-time content analysis.

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

This application is designed to run on **Replit** and requires:
- **Rust toolchain** (automatically provided by Replit)
- **Node.js** (for npm package management)
- **Python 3.11+** (for AI processing)
- **PostgreSQL database** (automatically configured on Replit)

### Quick Start on Replit

1. **Fork/Import this repository** to your Replit workspace

2. **Install required languages:**
   ```bash
   # Rust and Node.js are automatically detected and installed
   # Python dependencies will be installed automatically
   ```

3. **Configure environment variables** (if needed):
   ```bash
   # Database is automatically configured with:
   # DATABASE_URL, PGHOST, PGPORT, PGUSER, PGPASSWORD, PGDATABASE
   
   # Optional: Configure Python server mode
   export PYTHON_SERVER_MODE=subprocess  # (default)
   ```

4. **Install Python dependencies:**
   ```bash
   pip install emotionclassifier hatesonar nrclex numpy opencv-python pillow scikit-learn scipy text2emotion textblob torch detoxify
   ```

5. **Build and run the application:**
   ```bash
   cargo run
   ```

The application will:
- Start the Python AI server automatically
- Initialize HuggingFace and Detoxify models (1-2 minutes first time)
- Start the Rust web server on port 5000
- Set up PostgreSQL database schema automatically

### Database Setup

The application uses **PostgreSQL** with automatic schema management:

- **Database**: Automatically created and configured on Replit
- **Schema**: Auto-generated from Rust models with proper relations
- **Migrations**: Handled automatically on startup
- **Sample Data**: Use provided scripts for population (see Scripts section)

### Scripts

The `scripts/` directory contains data population utilities:

```bash
# Populate with 12 users and 36 posts
node scripts/populate_data.js

# Add comments to existing posts
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
curl https://your-repl-name.replit.dev/api/health

# Test post creation (requires auth token)
curl -X POST https://your-repl-name.replit.dev/api/posts \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test Post","content":"This is a test post"}'
```

## 🔧 Development

### Environment Configuration

The application supports configuration through environment variables:

- `DATABASE_URL`: PostgreSQL connection string (auto-configured)
- `PYTHON_SERVER_MODE`: `subprocess` (default) or `external`
- `SESSION_SECRET`: JWT signing secret (auto-generated)

### Database Management

PostgreSQL database is automatically managed:

- **Schema Creation**: Automatic on startup
- **Data Persistence**: All data stored in PostgreSQL
- **Backup**: Use PostgreSQL tools or Replit's database export

### Python AI Server

The AI server runs automatically as a subprocess:

- **Startup**: Managed by Rust application
- **Health Monitoring**: Automatic restart on failure
- **Model Caching**: Models cached for faster subsequent starts
- **Logging**: Integrated with main application logs

## 🚨 Troubleshooting

### Common Issues

1. **Server Won't Start**
   - Check if Python dependencies are installed: `pip list`
   - Verify database connection in Replit's database tab
   - Check logs for specific error messages

2. **AI Models Not Loading**
   - First run may take 2-5 minutes to download HuggingFace models
   - Ensure stable internet connection
   - Check available memory (models require ~1GB RAM)

3. **Database Issues**
   - Verify PostgreSQL is running in Replit's database tab
   - Check environment variables are properly set
   - Review database logs for connection errors

4. **Frontend Not Loading**
   - Ensure server is running on port 5000
   - Check webview configuration in Replit
   - Verify static files are being served correctly

### Performance Tips

- **Model Caching**: Models are cached after first load for faster startup
- **Database Indexing**: Key database fields are automatically indexed
- **Memory Management**: Close unused tabs to free memory for AI models
- **Pagination**: Large datasets are paginated for better performance

## 🔒 Security Features

- **JWT Authentication**: Stateless authentication with secure token handling
- **Password Hashing**: Argon2 password hashing for user security
- **Content Moderation**: Automatic detection and flagging of toxic content
- **Input Validation**: Comprehensive validation on both client and server
- **SQL Injection Protection**: Parameterized queries prevent SQL injection
- **CORS Configuration**: Proper cross-origin resource sharing setup

## 📈 Monitoring

The application includes comprehensive monitoring:

- **Health Endpoints**: `/api/health` for application status
- **Structured Logging**: Detailed logs with trace IDs
- **Error Handling**: Graceful error handling with user-friendly messages
- **Performance Metrics**: Built-in performance tracking

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

---

**Ready to build the future of social media with AI-powered insights! 🚀**