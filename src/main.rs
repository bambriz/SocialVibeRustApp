use social_media_app::{AppState, AppConfig, PythonServerMode};
use social_media_app::routes::create_routes;
use social_media_app::models::post::CreatePostRequest;
use social_media_app::models::User;
// Removed unused import
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error, Level};
use tracing_subscriber;
use std::net::SocketAddr;
use uuid::Uuid;
use reqwest;
use tokio::time::{sleep, Duration};

/// Wait for the Python sentiment analysis server to be ready
async fn wait_for_python_server(max_retries: u32, retry_delay_secs: u64) -> bool {
    info!("🔄 STARTUP: Waiting for Python sentiment analysis server...");
    
    for attempt in 1..=max_retries {
        match reqwest::get("http://127.0.0.1:8001/health").await {
            Ok(response) if response.status().is_success() => {
                match response.json::<serde_json::Value>().await {
                    Ok(health_data) => {
                        info!("✅ STARTUP: Python server is ready! Health check passed");
                        info!("   📚 Libraries: {:?}", health_data.get("libraries"));
                        info!("   🎯 Primary detector: {:?}", health_data.get("primary_detector"));
                        return true;
                    },
                    Err(e) => {
                        warn!("⚠️ STARTUP: Python server responded but with invalid JSON: {}", e);
                    }
                }
            },
            Ok(response) => {
                warn!("⚠️ STARTUP: Python server responded with status: {}", response.status());
            },
            Err(e) => {
                warn!("⚠️ STARTUP: Attempt {}/{}: Python server not ready yet: {}", attempt, max_retries, e);
            }
        }
        
        if attempt < max_retries {
            info!("⏳ STARTUP: Retrying in {} seconds...", retry_delay_secs);
            sleep(Duration::from_secs(retry_delay_secs)).await;
        }
    }
    
    error!("❌ STARTUP: Python server failed to become ready after {} attempts", max_retries);
    false
}

/// Populate the feed with sample posts during app startup (only if database is empty)
async fn populate_sample_posts(app_state: &AppState) {
    info!("🔍 STARTUP: Checking if database already contains posts...");
    
    // Check if posts already exist in the database
    match app_state.post_service.get_posts_feed(1, 0).await {
        Ok(posts) if !posts.is_empty() => {
            info!("✅ STARTUP: Database already contains posts, skipping sample post creation");
            return;
        }
        Ok(_) => {
            info!("🚀 STARTUP: Database is empty, populating with sample posts...");
        }
        Err(e) => {
            warn!("⚠️ STARTUP: Error checking existing posts: {}, proceeding with sample creation", e);
        }
    }
    
    // Create sample users in the database first
    let sample_users = vec![
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(), "alex_martinez", "alex@example.com"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(), "sarah_chen", "sarah@example.com"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap(), "mike_johnson", "mike@example.com"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440004").unwrap(), "emma_davis", "emma@example.com"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440005").unwrap(), "raj_patel", "raj@example.com"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440006").unwrap(), "zoe_williams", "zoe@example.com"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440007").unwrap(), "carlos_rivera", "carlos@example.com"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440008").unwrap(), "nina_brooks", "nina@example.com"),
    ];
    
    // Ensure users exist in database before creating posts
    info!("🔧 STARTUP: Creating sample users in database...");
    for (user_id, username, email) in &sample_users {
        // Check if user already exists
        match app_state.db.user_repo.get_user_by_id(*user_id).await {
            Ok(Some(_)) => {
                info!("✅ STARTUP: User {} already exists", username);
            }
            Ok(None) => {
                // User doesn't exist, create them
                let user = User {
                    id: *user_id,
                    username: username.to_string(),
                    email: email.to_string(),
                    password_hash: "sample_hash_123".to_string(), // Sample password hash
                    display_name: None,
                    bio: None,
                    avatar_url: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    is_active: true,
                };
                
                match app_state.db.user_repo.create_user(&user).await {
                    Ok(_) => {
                        info!("✅ STARTUP: Created sample user: {}", username);
                    }
                    Err(e) => {
                        warn!("⚠️ STARTUP: Failed to create sample user {}: {}", username, e);
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ STARTUP: Error checking user {}: {}", username, e);
            }
        }
    }
    
    // Sample posts designed to trigger different sentiment analysis results
    let sample_posts = vec![
        // Joy posts (primary positive emotion)
        CreatePostRequest {
            title: "Amazing day!".to_string(),
            content: "I am so excited and joyful today! Everything is wonderful and bright!".to_string(),
        },
        CreatePostRequest {
            title: "Big News!".to_string(),
            content: "Just got the promotion I've been working towards for months! Feeling incredibly grateful and happy right now. Hard work really does pay off!".to_string(),
        },
        
        // Affectionate combinations
        CreatePostRequest {
            title: "Weekend with Family".to_string(),
            content: "You make me so happy and I absolutely adore everything about you, my dear sweet love".to_string(),
        },
        CreatePostRequest {
            title: "Thank You".to_string(),
            content: "I love you so much and appreciate everything you do for our family. You're the best partner I could ask for.".to_string(),
        },
        
        // Sarcastic combinations  
        CreatePostRequest {
            title: "Living the Dream".to_string(),
            content: "Another wonderful day dealing with this amazing situation. Obviously this is exactly what I wanted to be doing with my time.".to_string(),
        },
        CreatePostRequest {
            title: "Perfect Timing".to_string(),
            content: "Oh sure, this is great. Just what I needed right now when everything else is falling apart.".to_string(),
        },
        
        // Sad/Disappointed posts
        CreatePostRequest {
            title: "Rough Week".to_string(),
            content: "Having a really tough time lately. Some days it feels like nothing goes right and I'm just exhausted from trying.".to_string(),
        },
        CreatePostRequest {
            title: "Missing Home".to_string(),
            content: "Feeling so homesick today. Sometimes the distance feels overwhelming and I just want to hug my family.".to_string(),
        },
        
        // Angry posts
        CreatePostRequest {
            title: "Frustrated".to_string(),
            content: "How are we still dealing with this? Broken tools. Zero communication. Decisions made in a vacuum. And somehow we're supposed to make it work.".to_string(),
        },
        CreatePostRequest {
            title: "Bad Service".to_string(),
            content: "this sucks".to_string(),
        },
        
        // Fear/Anxiety posts
        CreatePostRequest {
            title: "Nervous About Tomorrow".to_string(),
            content: "Starting something new tomorrow and honestly I'm terrified. What if I'm not ready? What if I mess up?".to_string(),
        },
        
        // Surprise posts
        CreatePostRequest {
            title: "Unexpected News".to_string(),
            content: "Well, that was definitely not what I was expecting to hear today! Life has a funny way of throwing curveballs.".to_string(),
        },
        
        // Disgust posts
        CreatePostRequest {
            title: "Cleanup Day".to_string(),
            content: "I don't know what's worse—the mess, the smell, or the fact that nobody seems to care. It's like basic hygiene is optional.".to_string(),
        },
        CreatePostRequest {
            title: "Spoiled Food".to_string(),
            content: "Opened the fridge and found something moldy and disgusting. The smell is absolutely nauseating and revolting.".to_string(),
        },
        CreatePostRequest {
            title: "Public Restroom".to_string(),
            content: "That was vile. I can't believe how gross and putrid it was in there. Makes me feel sick just thinking about it.".to_string(),
        },
        
        // Confused posts
        CreatePostRequest {
            title: "Lost in Translation".to_string(),
            content: "I'm totally bewildered and confused by these instructions. None of this makes any sense at all. What am I supposed to do?".to_string(),
        },
        CreatePostRequest {
            title: "Tech Problems".to_string(),
            content: "I'm completely puzzled and have no idea what just happened. This is absolutely no sense and I'm lost.".to_string(),
        },
        
        // Additional Neutral posts
        CreatePostRequest {
            title: "Regular Tuesday".to_string(),
            content: "Just a calm and peaceful day. Taking some deep breaths and feeling centered. Everything is balanced and serene.".to_string(),
        },
        
        // Additional Angry posts  
        CreatePostRequest {
            title: "Traffic Nightmare".to_string(),
            content: "These idiots can't drive! I'm absolutely furious and livid. This traffic is making me so angry I could scream.".to_string(),
        },
        
        // Additional Fear posts
        CreatePostRequest {
            title: "Doctor Visit".to_string(),
            content: "Waiting for test results and I'm terrified about what they might find. The anxiety is overwhelming and I'm scared.".to_string(),
        },
        
        // Additional Surprise posts
        CreatePostRequest {
            title: "Plot Twist".to_string(),
            content: "I did not see that coming at all! What a shocking and unexpected turn of events. I'm completely surprised!".to_string(),
        },
        
        // Additional Angry posts (to ensure we have 2+)
        CreatePostRequest {
            title: "Complete Disaster".to_string(),
            content: "These incompetent people are driving me absolutely furious! I'm so angry and livid right now. This is infuriating!".to_string(),
        },
        
        // Additional Confused posts (to ensure we have 2+)
        CreatePostRequest {
            title: "No Clue".to_string(),
            content: "I'm completely puzzled and bewildered by this situation. None of this makes any sense and I have no idea what's happening.".to_string(),
        },
        
        // Ensuring we have enough Angry posts
        CreatePostRequest {
            title: "Furious Customer".to_string(),
            content: "I'm absolutely LIVID and FURIOUS! This is the most infuriating experience ever! I'm ANGRY beyond words!".to_string(),
        },
        
        // Ensuring we have enough Disgust posts  
        CreatePostRequest {
            title: "Nasty Smell".to_string(),
            content: "That's absolutely DISGUSTING and NAUSEATING! The smell is so VILE and REPULSIVE I feel sick!".to_string(),
        },
    ];
    
    let mut posts_created = 0;
    let total_posts = sample_posts.len();
    
    for (i, post_request) in sample_posts.into_iter().enumerate() {
        let user_index = i % sample_users.len();
        let (author_id, author_username, _) = sample_users[user_index];
        
        match app_state.post_service.create_post(
            post_request, 
            author_id, 
            author_username.to_string()
        ).await {
            Ok(_) => {
                posts_created += 1;
                info!("Created sample post {}/{} by {}", posts_created, total_posts, author_username);
            }
            Err(e) => {
                warn!("Failed to create sample post by {}: {}. Continuing with remaining posts.", author_username, e);
            }
        }
    }
    
    info!("Sample post population completed: {}/{} posts created successfully", posts_created, total_posts);
    
    if posts_created == 0 {
        warn!("No sample posts were created. The feed will be empty on startup.");
    } else if posts_created < total_posts {
        warn!("Only {}/{} sample posts were created. Some posts failed to create.", posts_created, total_posts);
    } else {
        info!("All sample posts created successfully! Feed is now populated.");
    }
}

/// Run emotion migration on server startup if needed
async fn run_startup_migration(app_state: &AppState) {
    info!("🔄 STARTUP: Checking if emotion migration is needed...");
    
    // Check if post migration is needed
    let post_migration_needed = match app_state.post_service.is_migration_needed().await {
        Ok(needed) => needed,
        Err(e) => {
            error!("❌ STARTUP: Failed to check if post migration is needed: {}. Skipping migration.", e);
            return;
        }
    };
    
    // Check if comment migration is needed
    let comment_migration_needed = match app_state.comment_service.is_migration_needed().await {
        Ok(needed) => needed,
        Err(e) => {
            error!("❌ STARTUP: Failed to check if comment migration is needed: {}. Skipping migration.", e);
            return;
        }
    };
    
    if !post_migration_needed && !comment_migration_needed {
        info!("✅ STARTUP: No emotion migration needed - all posts and comments are up to date");
        return;
    }
    
    info!("🚀 STARTUP: Emotion migration needed. Starting migration process...");
    info!("   📊 Post migration needed: {}", post_migration_needed);
    info!("   📊 Comment migration needed: {}", comment_migration_needed);
    
    // Run post migration if needed
    if post_migration_needed {
        info!("🔄 STARTUP: Running post emotion migration...");
        match app_state.post_service.run_emotion_migration().await {
            Ok(result) => {
                info!("✅ STARTUP: Post migration completed successfully");
                info!("   📊 Posts checked: {}", result.total_posts_checked);
                info!("   🎯 Posts requiring migration: {}", result.posts_requiring_migration);
                info!("   ✅ Posts migrated: {}", result.posts_successfully_migrated);
                info!("   ❌ Posts failed: {}", result.posts_failed_migration);
                
                if !result.errors.is_empty() {
                    warn!("⚠️ STARTUP: {} errors occurred during post migration:", result.errors.len());
                    for error in &result.errors {
                        warn!("   - {}", error);
                    }
                }
            }
            Err(e) => {
                error!("❌ STARTUP: Post migration failed: {}. Server will continue but posts may have outdated emotion types.", e);
            }
        }
    }
    
    // Run comment migration if needed  
    if comment_migration_needed {
        info!("🔄 STARTUP: Running comment emotion migration...");
        match app_state.comment_service.run_emotion_migration().await {
            Ok(result) => {
                info!("✅ STARTUP: Comment migration completed successfully");
                info!("   📊 Comments checked: {}", result.total_comments_checked);
                info!("   🎯 Comments requiring migration: {}", result.comments_requiring_migration);
                info!("   ✅ Comments migrated: {}", result.comments_successfully_migrated);
                info!("   ❌ Comments failed: {}", result.comments_failed_migration);
                
                if !result.errors.is_empty() {
                    warn!("⚠️ STARTUP: {} errors occurred during comment migration:", result.errors.len());
                    for error in &result.errors {
                        warn!("   - {}", error);
                    }
                }
            }
            Err(e) => {
                error!("❌ STARTUP: Comment migration failed: {}. Server will continue but comments may have outdated emotion types.", e);
            }
        }
    }
    
    info!("🏁 STARTUP: Emotion migration process completed");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Social Media App server...");

    // Load configuration from environment
    let config = AppConfig::from_env();
    info!("Server configuration loaded: Python server mode = {:?}", config.python_server_mode);

    // Initialize application state
    let app_state = AppState::new(config.clone()).await?;
    info!("Application state initialized");

    // Start or wait for Python sentiment analysis server based on mode
    match config.python_server_mode {
        PythonServerMode::Subprocess => {
            if let Some(python_manager) = &app_state.python_manager {
                info!("🚀 STARTUP: Starting Python server in subprocess mode");
                match python_manager.start().await {
                    Ok(()) => {
                        info!("✅ STARTUP: Python server started successfully in subprocess mode");
                    }
                    Err(e) => {
                        error!("❌ STARTUP: Failed to start Python server subprocess: {}", e);
                        return Err(format!("Python server subprocess failed to start: {}", e).into());
                    }
                }
            } else {
                error!("❌ STARTUP: PythonManager not initialized despite subprocess mode");
                return Err("PythonManager not available in subprocess mode".into());
            }
        }
        PythonServerMode::External => {
            info!("🔄 STARTUP: Waiting for external Python server to be ready");
            if !wait_for_python_server(12, 30).await {
                error!("❌ STARTUP: Cannot start without external Python sentiment analysis server");
                return Err("External Python server dependency not available".into());
            }
        }
    }

    // Populate sample posts for demonstration purposes
    populate_sample_posts(&app_state).await;

    // Run emotion migration on startup to update any posts with old emotion types
    run_startup_migration(&app_state).await;

    // Set up graceful shutdown for Python server if in subprocess mode
    let python_manager_for_shutdown = app_state.python_manager.clone();

    // Build our application with routes
    let router = create_routes();
    let router_with_auth = social_media_app::routes::api::apply_auth_middleware(router, &app_state);
    let app = router_with_auth
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Bind to port 5000 on all interfaces as required by Replit
    // ARCHITECT GUIDANCE: Use tokio TcpListener directly to avoid blocking socket registration
    let addr: SocketAddr = config.server_address().parse()?;
    
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("✅ Server successfully bound to http://{}", config.server_address());
            listener
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                error!("❌ STARTUP: Port {} is already in use. Another server instance may be running.", config.server_port);
                error!("   🔍 Check if another Rust server is already bound to this port.");
                error!("   💡 Ensure only one workflow instance is running at a time.");
                
                // ARCHITECT GUIDANCE: Clean error handling with proper exit instead of implicit retries
                std::process::exit(1);
            } else {
                error!("❌ STARTUP: Failed to bind to {}: {}", config.server_address(), e);
                return Err(e.into());
            }
        }
    };
    
    // Start serving requests with graceful shutdown handling
    let server = axum::serve(listener, app);
    
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                error!("Server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal, stopping server...");
        }
    }
    
    // Gracefully shutdown Python server if running in subprocess mode
    if let Some(python_manager) = python_manager_for_shutdown {
        info!("Shutting down Python server subprocess...");
        if let Err(e) = python_manager.shutdown().await {
            warn!("Error during Python server shutdown: {}", e);
        }
    }
    
    info!("Server shutdown completed");
    Ok(())
}