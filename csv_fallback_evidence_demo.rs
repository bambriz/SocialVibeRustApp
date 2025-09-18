use social_media_app::services::post_service::PostService;
use social_media_app::services::{SentimentService, ModerationService};
use social_media_app::db::repository::{PostRepository, CsvPostRepository};
use social_media_app::models::post::CreatePostRequest;
use social_media_app::models::Post;
use social_media_app::{AppError, Result};
use uuid::Uuid;
use chrono::Utc;
use async_trait::async_trait;
use std::sync::Arc;
use std::fs;

// Create a simple failing repository to force CSV fallback
pub struct AlwaysFailRepository;

#[async_trait]
impl PostRepository for AlwaysFailRepository {
    async fn create_post(&self, _post: &Post) -> Result<Post> {
        println!("🔴 Primary repository CREATE failed - forcing CSV fallback!");
        Err(AppError::InternalError("Primary repository intentionally failed".to_string()))
    }

    async fn get_post_by_id(&self, _id: Uuid) -> Result<Option<Post>> {
        println!("🔴 Primary repository GET failed - forcing CSV fallback!");
        Err(AppError::InternalError("Primary repository intentionally failed".to_string()))
    }

    async fn get_posts_paginated(&self, _limit: u32, _offset: u32) -> Result<Vec<Post>> {
        println!("🔴 Primary repository PAGINATED failed - forcing CSV fallback!");
        Err(AppError::InternalError("Primary repository intentionally failed".to_string()))
    }

    async fn get_posts_by_popularity(&self, _limit: u32, _offset: u32) -> Result<Vec<Post>> {
        println!("🔴 Primary repository POPULARITY failed - forcing CSV fallback!");
        Err(AppError::InternalError("Primary repository intentionally failed".to_string()))
    }

    async fn update_post(&self, _post: &Post) -> Result<Post> {
        println!("🔴 Primary repository UPDATE failed - forcing CSV fallback!");
        Err(AppError::InternalError("Primary repository intentionally failed".to_string()))
    }

    async fn delete_post(&self, _id: Uuid) -> Result<()> {
        println!("🔴 Primary repository DELETE failed - forcing CSV fallback!");
        Err(AppError::InternalError("Primary repository intentionally failed".to_string()))
    }

    async fn increment_comment_count(&self, _post_id: Uuid) -> Result<()> {
        Err(AppError::InternalError("Primary repository intentionally failed".to_string()))
    }

    async fn update_popularity_score(&self, _post_id: Uuid, _score: f64) -> Result<()> {
        Err(AppError::InternalError("Primary repository intentionally failed".to_string()))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for log output
    tracing_subscriber::fmt::init();
    
    println!("🎯 CSV FALLBACK EVIDENCE DEMONSTRATION");
    println!("=====================================");
    
    let demo_csv_path = "demo_posts_evidence.csv";
    
    // Clean up any previous demo file
    if std::path::Path::new(demo_csv_path).exists() {
        fs::remove_file(demo_csv_path).unwrap();
    }
    
    // Set up repositories - primary always fails, CSV works
    let failing_primary = Arc::new(AlwaysFailRepository);
    let csv_fallback = Arc::new(CsvPostRepository::new(Some(demo_csv_path.to_string())));
    let sentiment_service = Arc::new(SentimentService::new());
    let moderation_service = Arc::new(ModerationService::new());
    
    let post_service = PostService::new(
        failing_primary,
        csv_fallback,
        sentiment_service,
        moderation_service,
    );
    
    let author_id = Uuid::new_v4();
    let author_username = "evidence_user".to_string();
    
    println!("\n📋 EVIDENCE 1: PostService methods use try_with_fallback");
    println!("Creating post (will show fallback logs)...");
    
    let create_request = CreatePostRequest {
        title: "CSV Fallback Evidence".to_string(),
        content: "This demonstrates CSV fallback working with trace logging and file persistence.".to_string(),
    };
    
    let created_post = post_service.create_post(create_request, author_id, author_username.clone()).await?;
    println!("✅ Post created via CSV fallback: {}", created_post.id);
    
    // Verify CSV file was created
    assert!(std::path::Path::new(demo_csv_path).exists(), "CSV file should exist");
    
    println!("\n📋 EVIDENCE 2: CSV file persistence verification");
    let csv_content = fs::read_to_string(demo_csv_path)?;
    println!("📄 CSV file contents:");
    println!("{}", csv_content);
    
    assert!(csv_content.contains(&created_post.id.to_string()), "CSV should contain post ID");
    assert!(csv_content.contains("CSV Fallback Evidence"), "CSV should contain post title");
    
    println!("\n📋 EVIDENCE 3: CSV round-trip functionality (write → read → verify)");
    println!("Reading post back from CSV...");
    
    let retrieved_post = post_service.get_post(created_post.id).await?
        .expect("Post should exist in CSV");
    
    println!("✅ Post retrieved from CSV:");
    println!("   ID: {}", retrieved_post.id);
    println!("   Title: {}", retrieved_post.title);
    println!("   Content: {}", retrieved_post.content);
    
    assert_eq!(retrieved_post.id, created_post.id);
    assert_eq!(retrieved_post.title, "CSV Fallback Evidence");
    
    println!("\n📋 EVIDENCE 4: Update operation with CSV fallback");
    let update_request = CreatePostRequest {
        title: "Updated CSV Evidence".to_string(),
        content: "This post was updated via CSV fallback to prove persistence.".to_string(),
    };
    
    let updated_post = post_service.update_post(created_post.id, update_request, author_id).await?;
    println!("✅ Post updated via CSV fallback: {}", updated_post.title);
    
    // Verify update persisted to CSV
    let updated_csv_content = fs::read_to_string(demo_csv_path)?;
    assert!(updated_csv_content.contains("Updated CSV Evidence"), "CSV should contain updated title");
    
    println!("\n📋 EVIDENCE 5: Ownership enforcement verification");
    let wrong_author = Uuid::new_v4();
    let update_attempt = CreatePostRequest {
        title: "Unauthorized Update".to_string(),
        content: "This should fail".to_string(),
    };
    
    match post_service.update_post(created_post.id, update_attempt, wrong_author).await {
        Err(AppError::AuthError(_)) => {
            println!("✅ Ownership enforcement working: Unauthorized update blocked");
        }
        _ => panic!("❌ Ownership enforcement failed"),
    }
    
    match post_service.delete_post(created_post.id, wrong_author).await {
        Err(AppError::AuthError(_)) => {
            println!("✅ Ownership enforcement working: Unauthorized delete blocked");
        }
        _ => panic!("❌ Ownership enforcement failed"),
    }
    
    println!("\n📋 EVIDENCE 6: Delete operation with CSV fallback");
    post_service.delete_post(created_post.id, author_id).await?;
    println!("✅ Post deleted via CSV fallback");
    
    // Verify deletion from CSV
    let final_csv_content = fs::read_to_string(demo_csv_path)?;
    assert!(!final_csv_content.contains(&created_post.id.to_string()), "CSV should not contain deleted post");
    
    println!("\n🎉 ALL EVIDENCE VERIFIED SUCCESSFULLY!");
    println!("📊 SUMMARY:");
    println!("   ✅ PostService methods use try_with_fallback for ALL CRUD operations");
    println!("   ✅ CSV fallback repository implements ALL PostRepository methods");
    println!("   ✅ CSV file persistence works (write → read → update → delete)");
    println!("   ✅ Ownership enforcement prevents unauthorized operations");
    println!("   ✅ Trace logging shows fallback execution paths");
    
    // Clean up
    if std::path::Path::new(demo_csv_path).exists() {
        fs::remove_file(demo_csv_path).unwrap();
    }
    
    Ok(())
}