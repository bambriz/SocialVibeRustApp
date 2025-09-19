# Social Pulse - Sentiment-Based Social Media Platform

Social Pulse is a modern social media application that combines traditional social networking features with advanced AI-powered sentiment analysis and content moderation capabilities. Built with Rust (backend) and vanilla JavaScript (frontend), featuring HuggingFace EmotionClassifier and Detoxify for comprehensive content analysis.

## 🚀 Features

- **Advanced Sentiment Analysis**: HuggingFace EmotionClassifier with combo emotions (sarcastic+joy, affectionate+sad)
- **AI-Powered Content Moderation**: Detoxify-based toxicity detection with dual-tier blocking system
- **Real-time Processing**: Content analyzed before publication with immediate feedback
- **Emotion Categories**: Joy, sad, angry, fear, disgust, surprise, confused, neutral, sarcastic, affectionate
- **Toxicity Tagging**: Identity attack blocking (≥0.8) + toxicity tagging (≥0.5) for other categories
- **Persistent Caching**: HuggingFace model caching for faster startup times
- **Dual Storage**: Primary SQLite with CSV backup for data persistence

## 🛠️ Setup Instructions for Windows with RustRover

### Prerequisites

1. **Install RustRover** (JetBrains IDE for Rust)
   - Download from: https://www.jetbrains.com/rust/
   - Install with default settings

2. **Install PyCharm Plugin** for RustRover
   - Open RustRover → File → Settings → Plugins
   - Search for "Python" or "PyCharm"
   - Install the official Python plugin
   - Restart RustRover

3. **Install Rust Toolchain**
   ```powershell
   # Install rustup (Rust installer) 
   winget install Rustlang.Rust.MSVC
   # Or download from: https://rustup.rs/
   
   # Verify installation
   rustc --version
   cargo --version
   ```

4. **Install Python 3.11+**
   ```powershell
   # Using winget
   winget install Python.Python.3.11
   
   # Or download from: https://www.python.org/downloads/
   # Make sure Python is added to PATH
   
   # Verify installation
   python --version
   pip --version
   ```

### Project Setup

1. **Clone and Open Project**
   ```powershell
   git clone <your-repo-url>
   cd social-pulse
   ```
   - Open RustRover → Open → Select the `social-pulse` folder

2. **Configure Python Environment in RustRover**
   - File → Settings → Build, Execution, Deployment → Python Interpreter
   - Add new Python interpreter (use system Python 3.11+)
   - Set working directory to project root

3. **Install Python Dependencies**
   ```powershell
   # Navigate to project directory
   cd social-pulse
   
   # Install Python dependencies
   pip install -r requirements.txt
   # OR using the project dependencies:
   pip install emotionclassifier>=0.1.4 hatesonar>=0.0.7 nrclex>=3.0.0 numpy>=1.26.4 opencv-python>=4.11.0.86 pillow>=11.3.0 scikit-learn>=1.7.2 scipy>=1.16.2 text2emotion>=0.0.5 textblob>=0.19.0 torch>=2.8.0 detoxify>=0.5.0
   
   # For PyTorch CPU (Windows):
   pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu
   ```

4. **Install Rust Dependencies**
   ```powershell
   # This will download and compile all Rust dependencies
   cargo build
   ```

### Running the Application

1. **Start the Application**
   ```powershell
   # Build and run (debug mode)
   cargo run
   
   # OR build and run (release mode for better performance)
   cargo run --release
   ```

   The application will:
   - Start the Python AI server (sentiment analysis + content moderation)
   - Initialize HuggingFace and Detoxify models (may take 1-2 minutes first time)
   - Start the Rust web server on `http://localhost:5000`
   - Populate sample posts for testing

2. **Access the Application**
   - **Frontend**: http://localhost:5000
   - **Health Check**: http://localhost:5000/health
   - **Python AI Server**: http://localhost:8001/health

### Testing the System

#### Frontend Testing
1. Open http://localhost:5000 in your browser
2. Create a new post with different emotions:
   - **Joy**: "I'm so happy and excited today!"
   - **Sarcasm**: "Oh great, just perfect timing"
   - **Affection**: "I love you so much, my dear"
   - **Toxicity**: "You are stupid" (will be tagged, not blocked)
   - **Hate Speech**: "I hate all [identity group]" (will be blocked)

#### API Testing with curl/PowerShell
```powershell
# Test sentiment analysis
curl -X POST http://localhost:8001/analyze -H "Content-Type: application/json" -d '{\"text\":\"I am happy today!\"}'

# Test content moderation  
curl -X POST http://localhost:8001/moderate -H "Content-Type: application/json" -d '{\"text\":\"You are an idiot\"}'

# Check health status
curl http://localhost:8001/health
```

#### Expected Responses
- **Sentiment**: `{"sentiment_type":"joy","confidence":0.8,"is_sarcastic":false,"is_combo":false}`
- **Moderation**: `{"is_blocked":false,"toxicity_tags":["toxicity","insult"],"confidence":0.05}`
- **Health**: `{"status":"healthy","primary_detector":"huggingface-emotionclassifier"}`

### Development in RustRover

#### Rust Configuration
1. **Build Configuration**
   - Run → Edit Configurations → Add "Cargo Command"
   - Command: `run`
   - Working directory: project root

2. **Debug Configuration**
   - Set breakpoints in Rust code
   - Use RustRover's integrated debugger
   - Environment variables can be set in run configuration

#### Python Configuration  
1. **Python Scripts**
   - Navigate to `python_scripts/` folder
   - Right-click Python files → "Run" or "Debug"
   - Use PyCharm plugin features for Python development

2. **Python Module Testing**
   ```python
   # Test individual modules
   cd python_scripts
   python -c "from sentiment_analyzer import SentimentAnalyzer; sa = SentimentAnalyzer(); print(sa.analyze_sentiment('I am happy'))"
   ```

### Project Structure

```
social-pulse/
├── src/                          # Rust backend source code
│   ├── main.rs                   # Application entry point
│   ├── models/                   # Data models (User, Post, etc.)
│   ├── handlers/                 # HTTP request handlers
│   ├── services/                 # Business logic services
│   └── middleware/               # Authentication, logging
├── python_scripts/               # Python AI modules
│   ├── async_server.py          # HTTP server for AI endpoints
│   ├── sentiment_analyzer.py    # HuggingFace sentiment analysis
│   ├── content_moderator.py     # Detoxify content moderation
│   └── model_cache.py          # Model caching utilities
├── static/                       # Frontend assets
│   ├── index.html               # Main HTML page
│   ├── app-v2.js               # JavaScript application
│   └── styles.css              # CSS styling
├── tests/                        # Rust integration tests
├── Cargo.toml                   # Rust dependencies
├── pyproject.toml              # Python dependencies
└── README.md                   # This file
```

### Key Components

- **Rust Backend**: Axum web framework with JWT authentication, post management, and Python integration
- **Python AI Server**: Modular sentiment analysis and content moderation using state-of-the-art models
- **Frontend**: Vanilla JavaScript SPA with real-time sentiment preview and toxicity warnings
- **Data Storage**: SQLite database with CSV backup for data persistence
- **Caching System**: Persistent model caching for faster startup times

### Troubleshooting

#### Common Issues

1. **Python Server Won't Start**
   ```powershell
   # Check Python path
   python --version
   
   # Install missing dependencies
   pip install torch detoxify emotionclassifier
   
   # Test Python server directly
   cd python_scripts
   python async_server.py
   ```

2. **Rust Compilation Errors**
   ```powershell
   # Update Rust toolchain
   rustup update
   
   # Clean build
   cargo clean
   cargo build
   ```

3. **Model Download Issues**
   - First run may take 2-5 minutes to download HuggingFace models
   - Ensure stable internet connection
   - Models are cached in `/tmp/social_pulse_cache/`

4. **Port Conflicts**
   - Rust server: Port 5000
   - Python AI server: Port 8001
   - Check if ports are available: `netstat -an | findstr :5000`

#### Performance Tips
- Use `cargo run --release` for better performance
- Models are cached after first load for faster startup
- Close unused applications to free memory for AI models

### Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature-name`
3. Make changes with proper tests
4. Run tests: `cargo test`
5. Submit pull request

### License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**Happy coding with Social Pulse! 🚀**