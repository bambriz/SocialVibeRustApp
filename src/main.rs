use social_media_app::{AppState, AppConfig};
use social_media_app::routes::create_routes;
use social_media_app::models::post::CreatePostRequest;
// Removed unused import
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error, Level};
use tracing_subscriber;
use std::net::SocketAddr;
use uuid::Uuid;

/// Populate the feed with sample posts during app startup
async fn populate_sample_posts(app_state: &AppState) {
    info!("Populating feed with sample posts...");
    
    // Sample users for post authors
    let sample_users = vec![
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(), "alex_martinez"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(), "sarah_chen"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440003").unwrap(), "mike_johnson"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440004").unwrap(), "emma_davis"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440005").unwrap(), "raj_patel"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440006").unwrap(), "zoe_williams"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440007").unwrap(), "carlos_rivera"),
        (Uuid::parse_str("550e8400-e29b-41d4-a716-446655440008").unwrap(), "nina_brooks"),
    ];
    
    // Sample posts designed to trigger different sentiment analysis results
    let sample_posts = vec![
        // Joy/Happy posts
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
    ];
    
    let mut posts_created = 0;
    let total_posts = sample_posts.len();
    
    for (i, post_request) in sample_posts.into_iter().enumerate() {
        let user_index = i % sample_users.len();
        let (author_id, author_username) = sample_users[user_index];
        
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Social Media App server...");

    // Load configuration from environment
    let config = AppConfig::from_env();
    info!("Server configuration loaded");

    // Initialize application state
    let app_state = AppState::new(config.clone()).await?;
    info!("Application state initialized");

    // Populate sample posts for demonstration purposes
    populate_sample_posts(&app_state).await;

    // Build our application with routes
    let app = create_routes()
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Bind to port 5000 on all interfaces as required by Replit
    let addr: SocketAddr = config.server_address().parse()?;
    let listener = TcpListener::bind(&addr).await?;
    
    info!("Server running on http://{}", config.server_address());
    
    // Start serving requests
    axum::serve(listener, app).await?;
    
    Ok(())
}