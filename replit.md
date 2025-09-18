# Social Pulse - Sentiment-Based Social Media Platform

## Overview

Social Pulse is a modern social media application that combines traditional social networking features with sentiment analysis and content moderation capabilities. The platform allows users to create posts, interact with content, and provides real-time sentiment analysis of user-generated content. Built with a Rust backend using Axum framework and a vanilla JavaScript frontend, the application focuses on creating a safe and emotionally-aware social environment.

## User Preferences

Preferred communication style: Simple, everyday language.

## System Architecture

### Backend Architecture
The application uses a **Rust-based microservices architecture** with Axum as the primary web framework. This choice provides memory safety, high performance, and excellent concurrency handling for a social media platform that needs to process multiple user requests simultaneously.

**Key architectural decisions:**
- **Axum Framework**: Selected for its modern async/await support, type-safe routing, and middleware system
- **JWT Authentication**: Implements stateless authentication using JSON Web Tokens for scalability
- **Argon2 Password Hashing**: Uses industry-standard password hashing for security
- **Middleware-based Authorization**: Implements authentication checks through Axum middleware for consistent security across endpoints
- **RESTful API Design**: Follows REST principles for predictable and maintainable API endpoints

### Frontend Architecture
The frontend uses **vanilla JavaScript with a Single Page Application (SPA) approach**. This decision prioritizes simplicity and reduces build complexity while maintaining modern user experience patterns.

**Key architectural decisions:**
- **Vanilla JavaScript**: Chosen over frameworks to minimize complexity and bundle size
- **Component-based UI**: Organizes code into reusable functions despite not using a formal framework
- **Local Storage for Auth**: Stores JWT tokens locally for session persistence
- **Modal-based Interactions**: Uses overlay modals for forms and detailed views
- **Real-time Sentiment Preview**: Provides immediate feedback on content sentiment as users type

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

### Data Management
The application is designed with **flexible data storage** in mind, currently structured for SQLite but architectured to support PostgreSQL migration.

**Key architectural decisions:**
- **User-centric Data Model**: Organizes data around user entities with posts, authentication, and profile information
- **Timestamp Tracking**: Includes created_at/updated_at fields for audit trails and chronological ordering
- **Sentiment Metadata Storage**: Stores sentiment analysis results alongside posts for quick retrieval and filtering

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

### Potential Future Integrations
The architecture is designed to accommodate:
- **PostgreSQL Database**: For production-scale data storage
- **Machine Learning APIs**: For advanced sentiment analysis and content moderation
- **CDN Services**: For static asset delivery and improved performance
- **Email Services**: For user notifications and verification
- **Real-time Communication**: WebSocket support for live updates and messaging