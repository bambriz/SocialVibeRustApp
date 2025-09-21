# Social Pulse - Sentiment-Based Social Media Platform

## Overview
Social Pulse is a social media application designed to foster a safe and emotionally-aware online environment. It combines traditional social networking features like infinite scroll and hierarchical comment threading with advanced sentiment analysis and content moderation. The platform utilizes a performant Rust (Axum) backend and a vanilla JavaScript frontend, focusing on user well-being, content quality, and an engaging user experience with comprehensive optimistic UI.

## User Preferences
Preferred communication style: Simple, everyday language.

## System Architecture

### UI/UX Decisions
The frontend is a vanilla JavaScript Single Page Application (SPA) with a component-based, mobile-first design. It features modal-based interactions, real-time sentiment preview, infinite scroll, and optimistic UI for posts and comments. Visual feedback includes pending state animations and mobile-optimized sticky positioning. Emotion categories are represented by distinct single-color visuals, and typography uses the Inter Font Family.

### Technical Implementations
The backend is a modular monolith built with Rust's Axum framework, ensuring high performance, memory safety, and concurrency. A supervised Python subprocess handles content processing. Key implementations include JWT authentication, Argon2 password hashing, middleware-based authorization, RESTful API design, and secure pagination. The comment system uses a materialized path structure for hierarchical threading with database constraints (0-10 levels) and transactional creation. Optimistic UI is applied comprehensively across the platform.

### Feature Specifications
The platform includes:
- **Authentication**: User registration, login, and protected routes using JWT.
- **Posts**: CRUD operations with pagination and a sophisticated popularity scoring algorithm.
- **Comments**: Hierarchical, Reddit-style threading with sentiment analysis, CRUD operations.
- **Voting**: Emotion-based and content filter voting on posts and comments, with aggregation and engagement tracking.
- **Content Processing**: Real-time sentiment analysis and content moderation via a Python subprocess.
- **Emotion Detection**: Categorization into various emotions (joy, sad, angry, fear, disgust, surprise, confused, neutral, sarcastic, affectionate) using HuggingFace EmotionClassifier and pattern-based detection.
- **Data Management**: PostgreSQL as the primary database, optimized for social media interactions, featuring a user-centric model, hierarchical comment storage, and sentiment metadata.
- **Security**: Defense-in-depth including a content moderation pipeline, token-based authentication, middleware for security checks, and client/server-side input validation.

### System Design Choices
The architecture leverages a modular Rust/Axum backend with a supervised Python subprocess for performance and security. The vanilla JavaScript frontend ensures a lean client. PostgreSQL provides data integrity and scalability in a database-first architecture. Optimistic UI provides immediate feedback with eventual consistency. Cache optimization with proper invalidation is implemented. The system is designed for extensibility to integrate future ML APIs, CDNs, email, and real-time communication.

## External Dependencies

### Core Backend Dependencies
- **Axum**: Web framework
- **Tokio**: Async runtime
- **Serde**: Serialization/deserialization
- **Jsonwebtoken**: JWT implementation
- **Argon2**: Password hashing
- **Hyper**: HTTP implementation
- **Chrono**: Date and time handling

### Frontend Dependencies
- **Inter Font Family**: Typography
- **Native Browser APIs**: Fetch, Local Storage, DOM manipulation

### Content Processing Dependencies
- **Python 3**: For sentiment analysis and content moderation
- **Python Standard Library**: Regex and JSON modules