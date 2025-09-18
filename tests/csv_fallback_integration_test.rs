use social_media_app::services::post_service::PostService;
use social_media_app::services::{SentimentService, ModerationService};
use social_media_app::db::repository::{PostRepository, CsvPostRepository};
use social_media_app::models::post::{CreatePostRequest, PostResponse};
use social_media_app::models::Post;
use social_media_app::{AppError, Result};
use uuid::Uuid;
use chrono::Utc;
use async_trait::async_trait;
use std::sync::Arc;
use std::fs;
use std::path::Path;
use tokio;

// Mock repository that always fails to force CSV fallback
pub struct FailingPostRepository;

#[async_trait]
impl PostRepository for FailingPostRepository {
    async fn create_post(&self, _post: &Post) -> Result<Post> {
        Err(AppError::InternalError("Primary repository failure simulation".to_string()))
    }

    async fn get_post_by_id(&self, _id: Uuid) -> Result<Option<Post>> {
        Err(AppError::InternalError("Primary repository failure simulation".to_string()))
    }

    async fn get_posts_paginated(&self, _limit: u32, _offset: u32) -> Result<Vec<Post>> {
        Err(AppError::InternalError("Primary repository failure simulation".to_string()))
    }

    async fn get_posts_by_popularity(&self, _limit: u32, _offset: u32) -> Result<Vec<Post>> {
        Err(AppError::InternalError("Primary repository failure simulation".to_string()))
    }

    async fn update_post(&self, _post: &Post) -> Result<Post> {
        Err(AppError::InternalError("Primary repository failure simulation".to_string()))
    }

    async fn delete_post(&self, _id: Uuid) -> Result<()> {
        Err(AppError::InternalError("Primary repository failure simulation".to_string()))
    }

    async fn increment_comment_count(&self, _post_id: Uuid) -> Result<()> {
        Err(AppError::InternalError("Primary repository failure simulation".to_string()))
    }

    async fn update_popularity_score(&self, _post_id: Uuid, _score: f64) -> Result<()> {
        Err(AppError::InternalError("Primary repository failure simulation".to_string()))
    }
}

#[tokio::test]
async fn test_csv_fallback_complete_crud_cycle() {
    // Setup test environment
    let test_csv_path = "test_posts_fallback.csv";
    
    // Clean up previous test files
    if Path::new(test_csv_path).exists() {
        fs::remove_file(test_csv_path).unwrap();
    }
    
    // Create test repositories and services
    let failing_primary_repo = Arc::new(FailingPostRepository);
    let csv_fallback_repo = Arc::new(CsvPostRepository::new(Some(test_csv_path.to_string())));
    let sentiment_service = Arc::new(SentimentService::new());
    let moderation_service = Arc::new(ModerationService::new());
    
    let post_service = PostService::new(
        failing_primary_repo,
        csv_fallback_repo.clone(),
        sentiment_service,
        moderation_service,
    );
    
    println!("🔬 TEST: Starting CSV fallback integration test");
    println!("📁 Test CSV file: {}", test_csv_path);
    
    // Test data
    let author_id = Uuid::new_v4();
    let author_username = "test_user".to_string();
    let post_request = CreatePostRequest {
        title: "Testing CSV Fallback".to_string(),
        content: "This post will test our CSV fallback system functionality and ensure everything works properly.".to_string(),
    };
    
    // TEST 1: Create post via CSV fallback
    println!("\n🧪 TEST 1: Creating post (should fallback to CSV)");
    let created_post = post_service.create_post(post_request, author_id, author_username.clone())
        .await
        .expect("Failed to create post via CSV fallback");
    
    println!("✅ Post created via CSV fallback: {}", created_post.id);
    
    // Verify CSV file was created and contains data
    assert!(Path::new(test_csv_path).exists(), "CSV file should exist after post creation");
    let csv_content = fs::read_to_string(test_csv_path).expect("Failed to read CSV file");
    println!("📄 CSV file contents after create:\n{}", csv_content);
    assert!(csv_content.contains(&created_post.id.to_string()), "CSV should contain the created post ID");
    assert!(csv_content.contains("Testing CSV Fallback"), "CSV should contain the post title");
    
    // TEST 2: Get post via CSV fallback
    println!("\n🧪 TEST 2: Getting post by ID (should fallback to CSV)");
    let retrieved_post = post_service.get_post(created_post.id)
        .await
        .expect("Failed to get post via CSV fallback")
        .expect("Post should exist");
    
    println!("✅ Post retrieved via CSV fallback: {}", retrieved_post.id);
    assert_eq!(retrieved_post.id, created_post.id);
    assert_eq!(retrieved_post.title, "Testing CSV Fallback");
    
    // TEST 3: Get posts feed via CSV fallback
    println!("\n🧪 TEST 3: Getting posts feed (should fallback to CSV)");
    let posts_feed = post_service.get_posts_feed(10, 0)
        .await
        .expect("Failed to get posts feed via CSV fallback");
    
    println!("✅ Posts feed retrieved via CSV fallback: {} posts", posts_feed.len());
    assert_eq!(posts_feed.len(), 1);
    assert_eq!(posts_feed[0].id, created_post.id);
    
    // TEST 4: Get posts paginated via CSV fallback
    println!("\n🧪 TEST 4: Getting posts paginated (should fallback to CSV)");
    let posts_paginated = post_service.get_posts_paginated(5, 0)
        .await
        .expect("Failed to get posts paginated via CSV fallback");
    
    println!("✅ Posts paginated retrieved via CSV fallback: {} posts", posts_paginated.len());
    assert_eq!(posts_paginated.len(), 1);
    assert_eq!(posts_paginated[0].id, created_post.id);
    
    // TEST 5: Update post via CSV fallback
    println!("\n🧪 TEST 5: Updating post (should fallback to CSV)");
    let update_request = CreatePostRequest {
        title: "Updated CSV Fallback Test".to_string(),
        content: "This post has been updated to verify CSV fallback update functionality.".to_string(),
    };
    
    let updated_post = post_service.update_post(created_post.id, update_request, author_id)
        .await
        .expect("Failed to update post via CSV fallback");
    
    println!("✅ Post updated via CSV fallback: {}", updated_post.id);
    assert_eq!(updated_post.title, "Updated CSV Fallback Test");
    
    // Verify CSV file contains updated data
    let updated_csv_content = fs::read_to_string(test_csv_path).expect("Failed to read updated CSV file");
    println!("📄 CSV file contents after update:\n{}", updated_csv_content);
    assert!(updated_csv_content.contains("Updated CSV Fallback Test"), "CSV should contain the updated title");
    
    // TEST 6: Test ownership enforcement for update
    println!("\n🧪 TEST 6: Testing ownership enforcement for update");
    let wrong_author_id = Uuid::new_v4();
    let ownership_test_result = post_service.update_post(created_post.id, update_request, wrong_author_id).await;
    
    match ownership_test_result {
        Err(AppError::AuthError(_)) => {
            println!("✅ Ownership enforcement working: Update rejected for wrong author");
        }
        _ => panic!("❌ Ownership enforcement failed: Update should have been rejected"),
    }
    
    // TEST 7: Test ownership enforcement for delete
    println!("\n🧪 TEST 7: Testing ownership enforcement for delete");
    let wrong_delete_result = post_service.delete_post(created_post.id, wrong_author_id).await;
    
    match wrong_delete_result {
        Err(AppError::AuthError(_)) => {
            println!("✅ Ownership enforcement working: Delete rejected for wrong author");
        }
        _ => panic!("❌ Ownership enforcement failed: Delete should have been rejected"),
    }
    
    // TEST 8: Delete post via CSV fallback (with correct author)
    println!("\n🧪 TEST 8: Deleting post (should fallback to CSV)");
    post_service.delete_post(created_post.id, author_id)
        .await
        .expect("Failed to delete post via CSV fallback");
    
    println!("✅ Post deleted via CSV fallback");
    
    // Verify post is no longer in CSV
    let final_csv_content = fs::read_to_string(test_csv_path).expect("Failed to read final CSV file");
    println!("📄 CSV file contents after delete:\n{}", final_csv_content);
    assert!(!final_csv_content.contains(&created_post.id.to_string()), "CSV should not contain the deleted post ID");
    
    // TEST 9: Verify CSV round-trip functionality
    println!("\n🧪 TEST 9: Testing CSV round-trip functionality");
    
    // Create a new post
    let roundtrip_request = CreatePostRequest {
        title: "Round-trip Test".to_string(),
        content: "Testing CSV write → read → verify cycle".to_string(),
    };
    
    let roundtrip_post = post_service.create_post(roundtrip_request, author_id, author_username.clone())
        .await
        .expect("Failed to create roundtrip test post");
    
    // Verify it can be read back
    let retrieved_roundtrip = post_service.get_post(roundtrip_post.id)
        .await
        .expect("Failed to retrieve roundtrip post")
        .expect("Roundtrip post should exist");
    
    assert_eq!(retrieved_roundtrip.title, "Round-trip Test");
    assert_eq!(retrieved_roundtrip.content, "Testing CSV write → read → verify cycle");
    
    println!("✅ CSV round-trip functionality verified");
    
    // Final verification - check CSV file structure
    let final_verification_csv = fs::read_to_string(test_csv_path).expect("Failed to read CSV for final verification");
    let lines: Vec<&str> = final_verification_csv.lines().collect();
    
    // Should have header + 1 data row
    assert_eq!(lines.len(), 2, "CSV should have header and one data row");
    assert!(lines[0].contains("id,title,content"), "CSV should have proper headers");
    assert!(lines[1].contains("Round-trip Test"), "CSV should contain the roundtrip test data");
    
    println!("✅ CSV file structure verified");
    
    // Clean up
    if Path::new(test_csv_path).exists() {
        fs::remove_file(test_csv_path).unwrap();
    }
    
    println!("\n🎉 ALL CSV FALLBACK TESTS PASSED!");
    println!("📋 EVIDENCE SUMMARY:");
    println!("   ✅ Primary repository failure forces CSV fallback");
    println!("   ✅ All CRUD operations work via CSV fallback");
    println!("   ✅ CSV file persists data correctly");
    println!("   ✅ CSV round-trip functionality verified");
    println!("   ✅ Ownership enforcement working correctly");
    println!("   ✅ CSV file structure is valid");
}

#[tokio::test]
async fn test_csv_repository_implements_all_methods() {
    println!("🔬 TEST: Verifying CsvPostRepository implements all PostRepository methods");
    
    let test_csv_path = "test_csv_completeness.csv";
    
    // Clean up
    if Path::new(test_csv_path).exists() {
        fs::remove_file(test_csv_path).unwrap();
    }
    
    let csv_repo = CsvPostRepository::new(Some(test_csv_path.to_string()));
    
    // Create test post
    let test_post = Post {
        id: Uuid::new_v4(),
        title: "Test Post".to_string(),
        content: "Test content".to_string(),
        author_id: Uuid::new_v4(),
        author_username: "test_author".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        comment_count: 0,
        sentiment_score: Some(0.5),
        sentiment_colors: vec!["#008000".to_string()],
        sentiment_type: Some("calm".to_string()),
        popularity_score: 1.0,
        is_blocked: false,
    };
    
    println!("✅ Testing create_post");
    let created = csv_repo.create_post(&test_post).await.expect("create_post should work");
    assert_eq!(created.id, test_post.id);
    
    println!("✅ Testing get_post_by_id");
    let retrieved = csv_repo.get_post_by_id(test_post.id).await.expect("get_post_by_id should work");
    assert!(retrieved.is_some());
    
    println!("✅ Testing get_posts_paginated");
    let paginated = csv_repo.get_posts_paginated(10, 0).await.expect("get_posts_paginated should work");
    assert_eq!(paginated.len(), 1);
    
    println!("✅ Testing get_posts_by_popularity");
    let by_popularity = csv_repo.get_posts_by_popularity(10, 0).await.expect("get_posts_by_popularity should work");
    assert_eq!(by_popularity.len(), 1);
    
    println!("✅ Testing update_post");
    let mut updated_post = test_post.clone();
    updated_post.title = "Updated Title".to_string();
    let updated = csv_repo.update_post(&updated_post).await.expect("update_post should work");
    assert_eq!(updated.title, "Updated Title");
    
    println!("✅ Testing increment_comment_count");
    csv_repo.increment_comment_count(test_post.id).await.expect("increment_comment_count should work");
    let after_increment = csv_repo.get_post_by_id(test_post.id).await.expect("get after increment should work").unwrap();
    assert_eq!(after_increment.comment_count, 1);
    
    println!("✅ Testing update_popularity_score");
    csv_repo.update_popularity_score(test_post.id, 2.5).await.expect("update_popularity_score should work");
    let after_popularity_update = csv_repo.get_post_by_id(test_post.id).await.expect("get after popularity update should work").unwrap();
    assert_eq!(after_popularity_update.popularity_score, 2.5);
    
    println!("✅ Testing delete_post");
    csv_repo.delete_post(test_post.id).await.expect("delete_post should work");
    let after_delete = csv_repo.get_post_by_id(test_post.id).await.expect("get after delete should work");
    assert!(after_delete.is_none());
    
    // Clean up
    if Path::new(test_csv_path).exists() {
        fs::remove_file(test_csv_path).unwrap();
    }
    
    println!("🎉 CsvPostRepository implements ALL PostRepository methods correctly!");
}