# Migration Guide: PostgreSQL to Azure Cosmos DB

## Table of Contents
1. [Executive Summary](#executive-summary)
2. [Architecture Comparison](#architecture-comparison)
3. [Data Model Migration](#data-model-migration)
4. [Implementation Strategy](#implementation-strategy)
5. [Query Pattern Migration](#query-pattern-migration)
6. [Performance Optimization](#performance-optimization)
7. [Migration Execution Plan](#migration-execution-plan)
8. [Testing & Validation](#testing--validation)
9. [Rollback Strategy](#rollback-strategy)
10. [Appendix: Code Examples](#appendix-code-examples)

## Executive Summary

### Migration Overview
This guide provides a comprehensive roadmap for migrating Social Pulse from PostgreSQL to Azure Cosmos DB. The migration involves transitioning from a relational database model to a document-based NoSQL approach while maintaining functional parity and improving global scalability.

### Key Benefits of Migration
- **Global Distribution**: Multi-region replication with low latency worldwide
- **Elastic Scaling**: Automatic scaling based on demand with predictable performance
- **Multiple APIs**: SQL API, MongoDB API, Cassandra API, Gremlin API support
- **Integrated Analytics**: Real-time analytics with Azure Synapse Link
- **Cost Optimization**: Pay-per-RU model with serverless options
- **High Availability**: 99.999% availability SLA with multi-master replication

### Migration Scope
- **Posts Collection**: User-generated content with sentiment and moderation data
- **Comments Collection**: Hierarchical comment threading with materialized paths
- **Users Collection**: User authentication and profile data
- **Votes Collection**: Emotion-based voting and engagement tracking
- **Application Layer**: Repository pattern migration with feature flags

### Migration Timeline
- **Phase 1** (Week 1-2): Infrastructure setup and data modeling
- **Phase 2** (Week 3-4): Dual-write implementation and data migration
- **Phase 3** (Week 5-6): Read traffic migration and optimization
- **Phase 4** (Week 7-8): Full cutover and PostgreSQL deprecation

## Architecture Comparison

### Current PostgreSQL Architecture

```sql
-- Relational Schema with Foreign Keys
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR UNIQUE NOT NULL,
    email VARCHAR UNIQUE NOT NULL,
    password_hash VARCHAR NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE posts (
    id UUID PRIMARY KEY,
    title VARCHAR NOT NULL,
    content TEXT NOT NULL,
    author_id UUID REFERENCES users(id),
    author_username VARCHAR NOT NULL, -- Denormalized
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    comment_count INTEGER DEFAULT 0,
    sentiment_score DOUBLE PRECISION,
    sentiment_colors TEXT[],
    sentiment_type VARCHAR,
    sentiment_analysis JSONB,
    popularity_score DOUBLE PRECISION DEFAULT 1.0,
    is_blocked BOOLEAN DEFAULT FALSE,
    toxicity_tags TEXT[],
    toxicity_scores JSONB
);

CREATE TABLE comments (
    id UUID PRIMARY KEY,
    post_id UUID REFERENCES posts(id),
    content TEXT NOT NULL,
    user_id UUID REFERENCES users(id),
    parent_id UUID REFERENCES comments(id),
    path VARCHAR NOT NULL, -- Materialized path: "1/3/7/"
    depth INTEGER NOT NULL,
    sentiment_score DOUBLE PRECISION,
    sentiment_colors TEXT[],
    sentiment_type VARCHAR,
    is_blocked BOOLEAN DEFAULT FALSE,
    toxicity_tags TEXT[],
    toxicity_scores JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE votes (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    target_id UUID NOT NULL, -- Can reference posts or comments
    target_type VARCHAR NOT NULL, -- 'post' or 'comment'
    vote_type VARCHAR NOT NULL, -- emotion type
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, target_id, target_type)
);
```

### Target Cosmos DB Architecture

```json
// Document-based Collections with Embedded Relationships

// Users Collection
{
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "pk": "user:550e8400-e29b-41d4-a716-446655440000", // Partition key
    "type": "user",
    "username": "john_doe",
    "email": "john@example.com",
    "passwordHash": "$argon2id$v=19$...",
    "profile": {
        "displayName": "John Doe",
        "bio": "Software developer",
        "avatarUrl": null
    },
    "metadata": {
        "createdAt": "2025-09-22T10:00:00Z",
        "lastLoginAt": "2025-09-22T15:30:00Z",
        "isActive": true
    },
    "_ts": 1695384000
}

// Posts Collection
{
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "pk": "user:550e8400-e29b-41d4-a716-446655440000", // Partition by author for user-centric queries
    "type": "post",
    "title": "Exploring the Future of AI",
    "content": "Artificial intelligence is rapidly evolving...",
    "author": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "username": "john_doe",
        "displayName": "John Doe"
    },
    "engagement": {
        "commentCount": 15,
        "voteCount": 42,
        "popularityScore": 7.5
    },
    "sentiment": {
        "score": 0.8,
        "colors": ["#fbbf24", "#10b981"],
        "type": "joy",
        "analysis": {
            "primaryEmotion": "joy",
            "confidence": 0.85,
            "emotions": {
                "joy": 0.8,
                "surprise": 0.3,
                "neutral": 0.1
            }
        }
    },
    "moderation": {
        "isBlocked": false,
        "toxicityTags": [],
        "scores": null,
        "reviewedAt": "2025-09-22T10:05:00Z"
    },
    "timestamps": {
        "createdAt": "2025-09-22T10:00:00Z",
        "updatedAt": "2025-09-22T10:00:00Z"
    },
    "_ts": 1695384000
}

// Comments Collection
{
    "id": "789e0123-e89b-12d3-a456-426614174001",
    "pk": "post:123e4567-e89b-12d3-a456-426614174000", // Partition by post for comment threads
    "type": "comment",
    "postId": "123e4567-e89b-12d3-a456-426614174000",
    "content": "Great insights! I particularly enjoyed...",
    "author": {
        "id": "777e8400-e29b-41d4-a716-446655440000",
        "username": "jane_smith",
        "displayName": "Jane Smith"
    },
    "threading": {
        "parentId": null, // null for root comments
        "path": "789e0123-e89b-12d3-a456-426614174001/",
        "depth": 0,
        "childCount": 3
    },
    "sentiment": {
        "score": 0.6,
        "colors": ["#fbbf24"],
        "type": "joy"
    },
    "moderation": {
        "isBlocked": false,
        "toxicityTags": [],
        "scores": null
    },
    "timestamps": {
        "createdAt": "2025-09-22T10:15:00Z",
        "updatedAt": "2025-09-22T10:15:00Z"
    },
    "_ts": 1695384900
}

// Votes Collection  
{
    "id": "vote_550e8400_123e4567_post_joy",
    "pk": "target:123e4567-e89b-12d3-a456-426614174000", // Partition by target for aggregation
    "type": "vote",
    "userId": "550e8400-e29b-41d4-a716-446655440000",
    "targetId": "123e4567-e89b-12d3-a456-426614174000",
    "targetType": "post",
    "voteType": "joy",
    "user": {
        "username": "john_doe",
        "displayName": "John Doe"
    },
    "timestamps": {
        "createdAt": "2025-09-22T10:30:00Z"
    },
    "_ts": 1695385800
}
```

### Key Architectural Differences

| Aspect | PostgreSQL | Cosmos DB |
|--------|------------|----------|
| **Data Model** | Relational with foreign keys | Document with embedded relationships |
| **Schema** | Fixed schema with DDL | Flexible schema per document |
| **Transactions** | ACID across tables | ACID within partition |
| **Queries** | SQL with JOINs | SQL without JOINs, document-centric |
| **Scaling** | Vertical + read replicas | Horizontal with partitioning |
| **Consistency** | Strong consistency | Tunable consistency (Session, Eventual, Strong) |
| **Indexing** | B-tree indexes | Automatic + custom indexes |
| **Cost Model** | Fixed compute + storage | Request Units (RU) + storage |

## Data Model Migration

### Partition Key Strategy

Choosing the right partition key is critical for performance and cost optimization in Cosmos DB.

#### Users Collection
- **Partition Key Path**: `/pk`
- **Document Structure**: Each user document has `"pk": "user:{user_id}"` 
- **Reasoning**: User operations are always user-specific, ensuring even distribution
- **RU Allocation**: 400 RU/s (low write volume, mostly reads)

#### Posts Collection
- **Partition Key Path**: `/pk` 
- **Document Structure**: Each post document has `"pk": "user:{author_id}"` for user-centric queries
- **Reasoning**: User post queries ("my posts", user profiles) are efficient single-partition operations. Global feed requires cross-partition queries but can be optimized with composite indexes
- **RU Allocation**: 1000 RU/s (moderate write volume, high read volume)

#### Comments Collection
- **Partition Key Path**: `/pk`
- **Document Structure**: Each comment document has `"pk": "post:{post_id}"` 
- **Reasoning**: Comments are always queried by post, enabling efficient single-partition operations for comment threads
- **RU Allocation**: 600 RU/s (high write volume for active posts)

#### Votes Collection
- **Partition Key Path**: `/pk`
- **Document Structure**: Each vote document has `"pk": "target:{target_id}"` 
- **Reasoning**: Votes are aggregated by target (post/comment), enabling efficient vote counting and user vote lookups
- **RU Allocation**: 400 RU/s (moderate write volume, batch reads)

### Index Policy Configuration

#### Posts Collection Indexing
```json
{
    "indexingMode": "consistent",
    "automatic": true,
    "includedPaths": [
        {
            "path": "/*"
        }
    ],
    "excludedPaths": [
        {
            "path": "/sentiment/analysis/*"
        },
        {
            "path": "/moderation/scores/*"
        }
    ],
    "compositeIndexes": [
        [
            {
                "path": "/engagement/popularityScore",
                "order": "descending"
            },
            {
                "path": "/timestamps/createdAt",
                "order": "descending"
            }
        ],
        [
            {
                "path": "/engagement/popularityScore",
                "order": "ascending"
            },
            {
                "path": "/timestamps/createdAt",
                "order": "descending"
            }
        ]
    ]
}
```

#### Comments Collection Indexing
```json
{
    "indexingMode": "consistent",
    "automatic": true,
    "includedPaths": [
        {
            "path": "/*"
        }
    ],
    "excludedPaths": [
        {
            "path": "/sentiment/*"
        },
        {
            "path": "/moderation/scores/*"
        }
    ],
    "compositeIndexes": [
        [
            {
                "path": "/threading/path",
                "order": "ascending"
            },
            {
                "path": "/threading/path",
                "order": "ascending"
            },
            {
                "path": "/timestamps/createdAt",
                "order": "ascending"
            }
        ]
    ]
}
```

### Data Denormalization Strategy

Cosmos DB's document model encourages strategic denormalization for query efficiency.

#### User Information Embedding
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbeddedUser {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
}

// Usage in posts and comments
#[derive(Debug, Serialize, Deserialize)]
pub struct CosmosPost {
    pub id: Uuid,
    pub pk: String, // "user:{author_id}"
    pub r#type: String, // "post"
    pub title: String,
    pub content: String,
    pub author: EmbeddedUser, // Denormalized user data
    // ... other fields
}
```

#### Aggregated Metrics Embedding
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct EngagementMetrics {
    pub comment_count: u32,
    pub vote_count: u32,
    pub popularity_score: f64,
    pub last_activity_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommentThreading {
    pub parent_id: Option<Uuid>,
    pub path: String,
    pub depth: u32,
    pub child_count: u32, // Denormalized count
}
```

## Implementation Strategy

### Repository Pattern Abstraction

#### Database Trait Definition
```rust
#[async_trait]
pub trait Database: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    // User operations
    async fn create_user(&self, user: &CreateUserRequest) -> Result<User, Self::Error>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, Self::Error>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Self::Error>;
    async fn update_user(&self, id: Uuid, updates: &UpdateUserRequest) -> Result<User, Self::Error>;
    
    // Post operations
    async fn create_post(&self, post: &CreatePostRequest) -> Result<Post, Self::Error>;
    async fn get_post_by_id(&self, id: Uuid) -> Result<Option<Post>, Self::Error>;
    async fn get_posts_paginated(&self, limit: u32, offset: Option<String>) -> Result<(Vec<Post>, Option<String>), Self::Error>;
    async fn get_posts_by_user(&self, user_id: Uuid, limit: u32, offset: Option<String>) -> Result<(Vec<Post>, Option<String>), Self::Error>;
    async fn update_post(&self, id: Uuid, updates: &UpdatePostRequest) -> Result<Post, Self::Error>;
    async fn delete_post(&self, id: Uuid) -> Result<(), Self::Error>;
    
    // Comment operations
    async fn create_comment(&self, comment: &CreateCommentRequest) -> Result<Comment, Self::Error>;
    async fn get_comments_by_post(&self, post_id: Uuid, limit: u32, offset: Option<String>) -> Result<(Vec<Comment>, Option<String>), Self::Error>;
    async fn update_comment(&self, id: Uuid, updates: &UpdateCommentRequest) -> Result<Comment, Self::Error>;
    async fn delete_comment(&self, id: Uuid) -> Result<(), Self::Error>;
    
    // Vote operations
    async fn create_or_update_vote(&self, vote: &CreateVoteRequest) -> Result<Vote, Self::Error>;
    async fn get_user_vote(&self, user_id: Uuid, target_id: Uuid, target_type: &str) -> Result<Option<Vote>, Self::Error>;
    async fn get_votes_by_target(&self, target_id: Uuid, target_type: &str) -> Result<Vec<Vote>, Self::Error>;
    async fn delete_vote(&self, user_id: Uuid, target_id: Uuid, target_type: &str) -> Result<(), Self::Error>;
}
```

#### Cosmos DB Implementation

**Required Dependencies** (add to Cargo.toml):
```toml
[dependencies]
azure_data_cosmos = "0.8"
azure_core = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
```

**Cosmos DB Client Setup**:
```rust
use azure_data_cosmos::prelude::*;
use azure_core::auth::TokenCredential;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct CosmosDatabase {
    client: CosmosClient,
    database_name: String,
    containers: CosmosContainers,
}

pub struct CosmosContainers {
    pub users: ContainerClient,
    pub posts: ContainerClient,
    pub comments: ContainerClient,
    pub votes: ContainerClient,
}

impl CosmosDatabase {
    pub async fn new(
        account: &str,
        key: &str,
        database_name: &str,
    ) -> Result<Self, CosmosError> {
        let authorization_token = AuthorizationToken::primary_key(key)?;
        let client = CosmosClient::new(account, authorization_token);
        
        // Verify database connection
        let database = client.database_client(database_name);
        
        let containers = CosmosContainers {
            users: database.container_client("users"),
            posts: database.container_client("posts"),
            comments: database.container_client("comments"),
            votes: database.container_client("votes"),
        };
        
        // Test connectivity
        containers.users.read_container().await?;
        
        Ok(Self {
            client,
            database_name: database_name.to_string(),
            containers,
        })
    }
    
    pub async fn create_containers(&self) -> Result<(), CosmosError> {
        let database = self.client.database_client(&self.database_name);
        
        // Create Users container
        database
            .create_container("users", "/pk")
            .offer(Offer::Throughput(400))
            .execute()
            .await?;
        
        // Create Posts container  
        database
            .create_container("posts", "/pk")
            .offer(Offer::Throughput(1000))
            .execute()
            .await?;
        
        // Create Comments container
        database
            .create_container("comments", "/pk")
            .offer(Offer::Throughput(600))
            .execute()
            .await?;
        
        // Create Votes container
        database
            .create_container("votes", "/pk")
            .offer(Offer::Throughput(400))
            .execute()
            .await?;
        
        Ok(())
    }
}

// Cosmos DB Document Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CosmosUser {
    pub id: String,
    pub pk: String, // Partition key: "user:{user_id}"
    pub r#type: String, // "user"
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub profile: UserProfile,
    pub metadata: UserMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CosmosPost {
    pub id: String,
    pub pk: String, // Partition key: "user:{author_id}"
    pub r#type: String, // "post"
    pub title: String,
    pub content: String,
    pub author: EmbeddedUser,
    pub engagement: EngagementMetrics,
    pub sentiment: Option<SentimentData>,
    pub moderation: ModerationData,
    pub timestamps: Timestamps,
}

#[async_trait]
impl Database for CosmosDatabase {
    type Error = CosmosError;
    
    async fn create_user(&self, request: &CreateUserRequest) -> Result<User, Self::Error> {
        let user_id = Uuid::new_v4();
        let cosmos_user = CosmosUser {
            id: user_id.to_string(),
            pk: format!("user:{}", user_id), // Proper partition key format
            r#type: "user".to_string(),
            username: request.username.clone(),
            email: request.email.clone(),
            password_hash: request.password_hash.clone(),
            profile: UserProfile {
                display_name: request.display_name.clone(),
                bio: None,
                avatar_url: None,
            },
            metadata: UserMetadata {
                created_at: Utc::now(),
                last_login_at: None,
                is_active: true,
            },
        };
        
        let partition_key = PartitionKey::from(&cosmos_user.pk);
        let response = self.containers.users
            .create_document(cosmos_user.clone())
            .consistency_level(ConsistencyLevel::Session)
            .execute(&partition_key)
            .await?;
        
        Ok(cosmos_user.into())
    }
    
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, Self::Error> {
        // Direct document read using id and partition key (most efficient)
        let partition_key = PartitionKey::from(&format!("user:{}", id));
        
        match self.containers.users
            .document_client(id.to_string(), &partition_key)
            .read_document::<CosmosUser>()
            .execute()
            .await {
            Ok(response) => Ok(Some(response.document.into())),
            Err(CosmosError::ResourceNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Self::Error> {
        // Cross-partition query for username lookup
        let query = format!("SELECT * FROM c WHERE c.username = '{}' AND c.type = 'user'", username);
        
        let mut query_response = self.containers.users
            .query_documents(query)
            .max_item_count(1)
            .execute::<CosmosUser>()
            .await?;
        
        Ok(query_response.results.drain(..).next().map(|u| u.into()))
    }
    
    async fn create_post(&self, request: &CreatePostRequest) -> Result<Post, Self::Error> {
        let post_id = Uuid::new_v4();
        let cosmos_post = CosmosPost {
            id: post_id.to_string(),
            pk: format!("user:{}", request.author_id), // Partition by author for user-centric queries
            r#type: "post".to_string(),
            title: request.title.clone(),
            content: request.content.clone(),
            author: EmbeddedUser {
                id: request.author_id,
                username: request.author_username.clone(),
                display_name: request.author_display_name.clone(),
            },
            engagement: EngagementMetrics {
                comment_count: 0,
                vote_count: 0,
                popularity_score: 1.0,
                last_activity_at: Utc::now(),
            },
            sentiment: request.sentiment.clone(),
            moderation: request.moderation.clone(),
            timestamps: Timestamps {
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        };
        
        let partition_key = PartitionKey::from(&cosmos_post.pk);
        let response = self.containers.posts
            .create_document(cosmos_post.clone())
            .consistency_level(ConsistencyLevel::Session)
            .execute(&partition_key)
            .await?;
        
        Ok(cosmos_post.into())
    }
    
    async fn get_posts_paginated(
        &self, 
        limit: u32, 
        continuation_token: Option<String>
    ) -> Result<(Vec<Post>, Option<String>), Self::Error> {
        let query = "SELECT * FROM c WHERE c.type = 'post' AND c.moderation.isBlocked = false ORDER BY c.engagement.popularityScore DESC, c.timestamps.createdAt DESC";
        
        let mut query_builder = self.containers.posts
            .query_documents(query.to_string())
            .max_item_count(limit as i32);
        
        if let Some(token) = continuation_token {
            query_builder = query_builder.continuation_token(token);
        }
        
        let response = query_builder.execute::<CosmosPost>().await?;
        
        let posts: Vec<Post> = response.results.into_iter().map(|p| p.into()).collect();
        
        Ok((posts, response.continuation_token))
    }
    
    async fn get_posts_by_user(
        &self,
        user_id: Uuid,
        limit: u32,
        continuation_token: Option<String>
    ) -> Result<(Vec<Post>, Option<String>), Self::Error> {
        let query = format!(
            "SELECT * FROM c WHERE c.type = 'post' AND c.author.id = '{}' ORDER BY c.timestamps.createdAt DESC",
            user_id
        );
        
        let partition_key = PartitionKey::from(&format!("user:{}", user_id));
        let mut query_builder = self.containers.posts
            .query_documents(query)
            .partition_key(&partition_key) // Single partition query - very efficient!
            .max_item_count(limit as i32);
        
        if let Some(token) = continuation_token {
            query_builder = query_builder.continuation_token(token);
        }
        
        let response = query_builder.execute::<CosmosPost>().await?;
        
        let posts: Vec<Post> = response.results.into_iter().map(|p| p.into()).collect();
        
        Ok((posts, response.continuation_token))
    }
}
```

### Feature Flag Integration

```rust
pub struct DatabaseManager {
    postgres: PostgresDatabase,
    cosmos: Option<CosmosDatabase>,
    feature_flags: Arc<FeatureFlagManager>,
}

impl DatabaseManager {
    pub async fn get_database(&self, user_id: &str, operation: &str) -> Box<dyn Database> {
        let use_cosmos = self.feature_flags
            .is_enabled("cosmos_db_migration", user_id, &RequestContext::default())
            .await;
        
        if use_cosmos {
            if let Some(cosmos) = &self.cosmos {
                info!("Using Cosmos DB for operation: {}", operation);
                return Box::new(cosmos.clone());
            }
        }
        
        info!("Using PostgreSQL for operation: {}", operation);
        Box::new(self.postgres.clone())
    }
}

// Usage in service layer
#[async_trait]
impl PostService for PostServiceImpl {
    async fn create_post(&self, request: &CreatePostRequest, user_id: &str) -> Result<Post, ServiceError> {
        let database = self.database_manager.get_database(user_id, "create_post").await;
        
        let post = database.create_post(request).await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        
        // Log metrics for comparison
        self.metrics.record_database_operation("create_post", &database.database_type());
        
        Ok(post)
    }
}
```

## Query Pattern Migration

### Pagination Strategy

#### PostgreSQL Pagination (Current)
```sql
-- Offset-based pagination
SELECT * FROM posts 
WHERE is_blocked = false 
ORDER BY popularity_score DESC, created_at DESC 
LIMIT 20 OFFSET 100;
```

#### Cosmos DB Pagination (Target)
```rust
// Continuation token-based pagination
pub struct PaginationRequest {
    pub limit: u32,
    pub continuation_token: Option<String>,
}

pub struct PaginationResponse<T> {
    pub items: Vec<T>,
    pub continuation_token: Option<String>,
    pub has_more: bool,
}

// Implementation
async fn get_posts_paginated(
    &self,
    request: &PaginationRequest
) -> Result<PaginationResponse<Post>, DatabaseError> {
    let query = "SELECT * FROM c WHERE c.type = 'post' AND c.moderation.isBlocked = false ORDER BY c.engagement.popularityScore DESC, c.timestamps.createdAt DESC";
    
    let mut query_request = self.containers.posts
        .query_documents()
        .query_cross_partition(true)
        .max_item_count(request.limit as i32);
    
    if let Some(token) = &request.continuation_token {
        query_request = query_request.continuation(token);
    }
    
    let response = query_request.execute::<CosmosPost>(query).await?;
    
    let posts: Vec<Post> = response.results.into_iter().map(|p| p.into()).collect();
    let has_more = response.continuation_token.is_some();
    
    Ok(PaginationResponse {
        items: posts,
        continuation_token: response.continuation_token,
        has_more,
    })
}
```

### Comment Threading Migration

#### PostgreSQL Hierarchical Queries
```sql
-- Get comment thread with materialized path
SELECT * FROM comments 
WHERE post_id = $1 
ORDER BY path, created_at;

-- Get comment children
SELECT * FROM comments 
WHERE post_id = $1 AND path LIKE $2 || '%' AND depth = $3;
```

#### Cosmos DB Comment Threading
```rust
// Single-partition query for all comments in a post
async fn get_comments_by_post(
    &self,
    post_id: Uuid,
    request: &PaginationRequest
) -> Result<PaginationResponse<Comment>, DatabaseError> {
    let query = format!(
        "SELECT * FROM c WHERE c.type = 'comment' AND c.postId = '{}' " +
        "ORDER BY c.threading.path ASC, c.timestamps.createdAt ASC",
        post_id
    );
    
    let mut query_request = self.containers.comments
        .query_documents()
        .execute_with_partition_key(&post_id) // Single partition - efficient!
        .max_item_count(request.limit as i32);
    
    if let Some(token) = &request.continuation_token {
        query_request = query_request.continuation(token);
    }
    
    let response = query_request.execute::<CosmosComment>(query).await?;
    
    let comments: Vec<Comment> = response.results.into_iter().map(|c| c.into()).collect();
    
    Ok(PaginationResponse {
        items: comments,
        continuation_token: response.continuation_token,
        has_more: response.continuation_token.is_some(),
    })
}

// Get specific comment thread
async fn get_comment_thread(
    &self,
    post_id: Uuid,
    parent_path: &str,
    max_depth: u32
) -> Result<Vec<Comment>, DatabaseError> {
    let query = format!(
        "SELECT * FROM c WHERE c.type = 'comment' AND c.postId = '{}' " +
        "AND STARTSWITH(c.threading.path, '{}') AND c.threading.depth <= {} " +
        "ORDER BY c.threading.path ASC, c.timestamps.createdAt ASC",
        post_id, parent_path, max_depth
    );
    
    let response = self.containers.comments
        .query_documents()
        .execute_with_partition_key(&post_id)
        .execute::<CosmosComment>(&query)
        .await?;
    
    Ok(response.results.into_iter().map(|c| c.into()).collect())
}
```

### Vote Aggregation

#### PostgreSQL Vote Aggregation
```sql
-- Count votes by type for a post
SELECT vote_type, COUNT(*) as count
FROM votes 
WHERE target_id = $1 AND target_type = 'post'
GROUP BY vote_type;

-- Check if user voted
SELECT vote_type FROM votes 
WHERE user_id = $1 AND target_id = $2 AND target_type = $3;
```

#### Cosmos DB Vote Aggregation
```rust
// Single-partition aggregation
async fn get_vote_summary(
    &self,
    target_id: Uuid,
    target_type: &str
) -> Result<VoteSummary, DatabaseError> {
    let query = format!(
        "SELECT c.voteType, COUNT(1) as count " +
        "FROM c WHERE c.type = 'vote' AND c.targetId = '{}' AND c.targetType = '{}' " +
        "GROUP BY c.voteType",
        target_id, target_type
    );
    
    let response = self.containers.votes
        .query_documents()
        .execute_with_partition_key(&target_id)
        .execute::<VoteCount>(&query)
        .await?;
    
    let vote_counts: HashMap<String, u32> = response.results
        .into_iter()
        .map(|vc| (vc.vote_type, vc.count))
        .collect();
    
    Ok(VoteSummary {
        target_id,
        target_type: target_type.to_string(),
        vote_counts,
        total_votes: vote_counts.values().sum(),
    })
}

// Check user vote with composite document ID
async fn get_user_vote(
    &self,
    user_id: Uuid,
    target_id: Uuid,
    target_type: &str
) -> Result<Option<Vote>, DatabaseError> {
    let vote_id = format!("vote_{}_{}_{}_{}", user_id, target_id, target_type, "any");
    
    // First try direct document read (most efficient)
    let query = format!(
        "SELECT * FROM c WHERE c.type = 'vote' AND c.userId = '{}' " +
        "AND c.targetId = '{}' AND c.targetType = '{}'",
        user_id, target_id, target_type
    );
    
    let response = self.containers.votes
        .query_documents()
        .execute_with_partition_key(&target_id)
        .execute::<CosmosVote>(&query)
        .await?;
    
    Ok(response.results.into_iter().next().map(|v| v.into()))
}
```

## Performance Optimization

### Request Unit (RU) Optimization

#### RU Cost Analysis
```rust
pub struct RUMetrics {
    pub operation_type: String,
    pub ru_consumed: f64,
    pub latency_ms: u64,
    pub document_size_kb: f64,
    pub timestamp: DateTime<Utc>,
}

pub struct RUMonitor {
    metrics: Arc<Mutex<Vec<RUMetrics>>>,
    alert_threshold: f64, // Alert if operation exceeds this many RUs
}

impl RUMonitor {
    pub fn record_operation(
        &self,
        operation_type: &str,
        ru_consumed: f64,
        latency_ms: u64,
        document_size_kb: f64,
    ) {
        let metric = RUMetrics {
            operation_type: operation_type.to_string(),
            ru_consumed,
            latency_ms,
            document_size_kb,
            timestamp: Utc::now(),
        };
        
        if ru_consumed > self.alert_threshold {
            warn!(
                "High RU consumption: {} RUs for operation {} (document size: {} KB)",
                ru_consumed, operation_type, document_size_kb
            );
        }
        
        self.metrics.lock().unwrap().push(metric);
    }
    
    pub fn get_average_ru_by_operation(&self) -> HashMap<String, f64> {
        let metrics = self.metrics.lock().unwrap();
        let mut operation_rus: HashMap<String, Vec<f64>> = HashMap::new();
        
        for metric in metrics.iter() {
            operation_rus
                .entry(metric.operation_type.clone())
                .or_insert_with(Vec::new)
                .push(metric.ru_consumed);
        }
        
        operation_rus
            .into_iter()
            .map(|(op, rus)| {
                let average = rus.iter().sum::<f64>() / rus.len() as f64;
                (op, average)
            })
            .collect()
    }
}
```

#### Query Optimization Strategies

```rust
// Optimized query examples
pub struct OptimizedQueries;

impl OptimizedQueries {
    // Use single-partition queries when possible
    pub fn get_user_posts_optimized(user_id: Uuid) -> String {
        // This query operates within a single partition (author.id)
        format!(
            "SELECT * FROM c WHERE c.type = 'post' AND c.author.id = '{}' " +
            "ORDER BY c.timestamps.createdAt DESC",
            user_id
        )
    }
    
    // Use composite indexes for complex ordering
    pub fn get_popular_posts_optimized() -> String {
        // Composite index on (type, popularityScore DESC, createdAt DESC)
        "SELECT * FROM c WHERE c.type = 'post' AND c.moderation.isBlocked = false " +
        "ORDER BY c.engagement.popularityScore DESC, c.timestamps.createdAt DESC"
    }
    
    // Optimize with SELECT projection
    pub fn get_post_summaries_optimized() -> String {
        "SELECT c.id, c.title, c.author, c.engagement, c.timestamps.createdAt " +
        "FROM c WHERE c.type = 'post' AND c.moderation.isBlocked = false " +
        "ORDER BY c.engagement.popularityScore DESC"
    }
    
    // Use STARTSWITH for hierarchical queries
    pub fn get_comment_thread_optimized(post_id: Uuid, parent_path: &str) -> String {
        format!(
            "SELECT * FROM c WHERE c.type = 'comment' AND c.postId = '{}' " +
            "AND STARTSWITH(c.threading.path, '{}') " +
            "ORDER BY c.threading.path ASC",
            post_id, parent_path
        )
    }
}
```

### Caching Strategy for Cosmos DB

```rust
use redis::{AsyncCommands, Client as RedisClient};

pub struct CosmosDbWithCache {
    cosmos: CosmosDatabase,
    redis: RedisClient,
    cache_ttl: Duration,
}

impl CosmosDbWithCache {
    pub async fn get_post_cached(&self, id: Uuid) -> Result<Option<Post>, DatabaseError> {
        let cache_key = format!("post:{}", id);
        
        // Try cache first
        let mut redis_conn = self.redis.get_async_connection().await
            .map_err(|e| DatabaseError::CacheError(e.to_string()))?;
        
        if let Ok(cached_post) = redis_conn.get::<String, String>(&cache_key).await {
            if let Ok(post) = serde_json::from_str::<Post>(&cached_post) {
                return Ok(Some(post));
            }
        }
        
        // Cache miss - fetch from Cosmos DB
        let post = self.cosmos.get_post_by_id(id).await?;
        
        // Update cache
        if let Some(ref post) = post {
            if let Ok(serialized) = serde_json::to_string(post) {
                let _: () = redis_conn
                    .set_ex(&cache_key, serialized, self.cache_ttl.as_secs())
                    .await
                    .unwrap_or_default();
            }
        }
        
        Ok(post)
    }
    
    pub async fn invalidate_post_cache(&self, id: Uuid) -> Result<(), DatabaseError> {
        let cache_key = format!("post:{}", id);
        
        let mut redis_conn = self.redis.get_async_connection().await
            .map_err(|e| DatabaseError::CacheError(e.to_string()))?;
        
        let _: () = redis_conn.del(&cache_key).await
            .map_err(|e| DatabaseError::CacheError(e.to_string()))?;
        
        Ok(())
    }
}
```

## Migration Execution Plan

### Phase 1: Infrastructure Setup (Week 1-2)

#### Cosmos DB Resource Creation
```bash
#!/bin/bash
# Azure CLI script for Cosmos DB setup

# Variables
RESOURCE_GROUP="social-pulse-rg"
COSMOS_ACCOUNT="social-pulse-cosmos"
DATABASE_NAME="social_pulse"
LOCATION="East US"

# Create Cosmos DB account
az cosmosdb create \
    --resource-group $RESOURCE_GROUP \
    --name $COSMOS_ACCOUNT \
    --kind GlobalDocumentDB \
    --locations regionName="$LOCATION" failoverPriority=0 isZoneRedundant=False \
    --default-consistency-level "Session" \
    --enable-multiple-write-locations true \
    --enable-automatic-failover true

# Create database
az cosmosdb sql database create \
    --account-name $COSMOS_ACCOUNT \
    --resource-group $RESOURCE_GROUP \
    --name $DATABASE_NAME \
    --throughput 4000

# Create containers
az cosmosdb sql container create \
    --account-name $COSMOS_ACCOUNT \
    --database-name $DATABASE_NAME \
    --resource-group $RESOURCE_GROUP \
    --name "users" \
    --partition-key-path "/pk" \
    --throughput 1000

az cosmosdb sql container create \
    --account-name $COSMOS_ACCOUNT \
    --database-name $DATABASE_NAME \
    --resource-group $RESOURCE_GROUP \
    --name "posts" \
    --partition-key-path "/pk" \
    --throughput 2000

az cosmosdb sql container create \
    --account-name $COSMOS_ACCOUNT \
    --database-name $DATABASE_NAME \
    --resource-group $RESOURCE_GROUP \
    --name "comments" \
    --partition-key-path "/pk" \
    --throughput 1000

az cosmosdb sql container create \
    --account-name $COSMOS_ACCOUNT \
    --database-name $DATABASE_NAME \
    --resource-group $RESOURCE_GROUP \
    --name "votes" \
    --partition-key-path "/pk" \
    --throughput 1000
```

#### Environment Configuration
```bash
# .env additions for Cosmos DB
COSMOS_DB_ENABLED=true
COSMOS_DB_ACCOUNT=social-pulse-cosmos
COSMOS_DB_KEY=your-cosmos-primary-key
COSMOS_DB_DATABASE=social_pulse
COSMOS_DB_REGION=East US

# Feature flags for gradual migration
COSMOS_DB_USERS_ENABLED=false
COSMOS_DB_POSTS_ENABLED=false
COSMOS_DB_COMMENTS_ENABLED=false
COSMOS_DB_VOTES_ENABLED=false

# Migration control
DUAL_WRITE_ENABLED=false
MIGRATION_BATCH_SIZE=1000
MIGRATION_DELAY_MS=100
```

### Phase 2: Data Migration (Week 3-4)

#### Bulk Migration Tool
```rust
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};

pub struct MigrationTool {
    postgres: PostgresDatabase,
    cosmos: CosmosDatabase,
    batch_size: usize,
    delay_between_batches: Duration,
}

impl MigrationTool {
    pub async fn migrate_all_data(&self) -> Result<MigrationReport, MigrationError> {
        let start_time = Instant::now();
        let mut report = MigrationReport::new();
        
        info!("Starting full data migration from PostgreSQL to Cosmos DB");
        
        // Migrate in dependency order
        report.users = self.migrate_users().await?;
        report.posts = self.migrate_posts().await?;
        report.comments = self.migrate_comments().await?;
        report.votes = self.migrate_votes().await?;
        
        report.total_duration = start_time.elapsed();
        
        info!("Migration completed in {:?}", report.total_duration);
        Ok(report)
    }
    
    async fn migrate_users(&self) -> Result<EntityMigrationReport, MigrationError> {
        info!("Migrating users...");
        
        let total_count = self.postgres.count_users().await?;
        let progress = ProgressBar::new(total_count as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} users ({eta})")
                .unwrap()
        );
        
        let mut offset = 0;
        let mut migrated_count = 0;
        let mut error_count = 0;
        
        loop {
            let users = self.postgres
                .get_users_batch(self.batch_size, offset)
                .await?;
            
            if users.is_empty() {
                break;
            }
            
            // Process users in parallel batches
            let migration_results = stream::iter(users)
                .map(|user| async {
                    self.migrate_single_user(user).await
                })
                .buffer_unordered(10) // Process 10 users concurrently
                .collect::<Vec<_>>()
                .await;
            
            for result in migration_results {
                match result {
                    Ok(_) => migrated_count += 1,
                    Err(e) => {
                        error_count += 1;
                        warn!("Failed to migrate user: {}", e);
                    }
                }
                progress.inc(1);
            }
            
            offset += self.batch_size;
            
            // Rate limiting
            tokio::time::sleep(self.delay_between_batches).await;
        }
        
        progress.finish_with_message("Users migration completed");
        
        Ok(EntityMigrationReport {
            entity_type: "users".to_string(),
            total_count,
            migrated_count,
            error_count,
        })
    }
    
    async fn migrate_single_user(&self, postgres_user: PostgresUser) -> Result<(), MigrationError> {
        let cosmos_user = CosmosUser {
            id: postgres_user.id,
            partition_key: postgres_user.id, // Same as id for users
            r#type: "user".to_string(),
            username: postgres_user.username,
            email: postgres_user.email,
            password_hash: postgres_user.password_hash,
            profile: UserProfile {
                display_name: postgres_user.username.clone(), // Default display name
                bio: None,
                avatar_url: None,
            },
            metadata: UserMetadata {
                created_at: postgres_user.created_at,
                last_login_at: None,
                is_active: true,
            },
        };
        
        self.cosmos.containers.users
            .create_document()
            .is_upsert(true) // Allow overwrite for retries
            .execute_with_partition_key(&cosmos_user.partition_key, &cosmos_user)
            .await
            .map_err(|e| MigrationError::CosmosError(e))?;
        
        Ok(())
    }
    
    async fn migrate_posts(&self) -> Result<EntityMigrationReport, MigrationError> {
        info!("Migrating posts...");
        
        let total_count = self.postgres.count_posts().await?;
        let progress = ProgressBar::new(total_count as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} posts ({eta})")
                .unwrap()
        );
        
        let mut offset = 0;
        let mut migrated_count = 0;
        let mut error_count = 0;
        
        loop {
            let posts = self.postgres
                .get_posts_batch(self.batch_size, offset)
                .await?;
            
            if posts.is_empty() {
                break;
            }
            
            let migration_results = stream::iter(posts)
                .map(|post| async {
                    self.migrate_single_post(post).await
                })
                .buffer_unordered(5) // Lower concurrency for larger documents
                .collect::<Vec<_>>()
                .await;
            
            for result in migration_results {
                match result {
                    Ok(_) => migrated_count += 1,
                    Err(e) => {
                        error_count += 1;
                        warn!("Failed to migrate post: {}", e);
                    }
                }
                progress.inc(1);
            }
            
            offset += self.batch_size;
            tokio::time::sleep(self.delay_between_batches).await;
        }
        
        progress.finish_with_message("Posts migration completed");
        
        Ok(EntityMigrationReport {
            entity_type: "posts".to_string(),
            total_count,
            migrated_count,
            error_count,
        })
    }
    
    async fn migrate_single_post(&self, postgres_post: PostgresPost) -> Result<(), MigrationError> {
        // Get author information
        let author = self.postgres
            .get_user_by_id(postgres_post.author_id)
            .await?
            .ok_or(MigrationError::DataIntegrityError("Author not found".to_string()))?;
        
        let cosmos_post = CosmosPost {
            id: postgres_post.id,
            partition_key: postgres_post.author_id, // Partition by author
            r#type: "post".to_string(),
            title: postgres_post.title,
            content: postgres_post.content,
            author: EmbeddedUser {
                id: author.id,
                username: author.username,
                display_name: author.username.clone(),
            },
            engagement: EngagementMetrics {
                comment_count: postgres_post.comment_count,
                vote_count: 0, // Will be calculated from votes
                popularity_score: postgres_post.popularity_score,
                last_activity_at: postgres_post.updated_at,
            },
            sentiment: if let Some(analysis) = postgres_post.sentiment_analysis {
                Some(SentimentData {
                    score: postgres_post.sentiment_score,
                    colors: postgres_post.sentiment_colors.unwrap_or_default(),
                    r#type: postgres_post.sentiment_type,
                    analysis,
                })
            } else {
                None
            },
            moderation: ModerationData {
                is_blocked: postgres_post.is_blocked,
                toxicity_tags: postgres_post.toxicity_tags.unwrap_or_default(),
                scores: postgres_post.toxicity_scores,
                reviewed_at: postgres_post.updated_at,
            },
            timestamps: Timestamps {
                created_at: postgres_post.created_at,
                updated_at: postgres_post.updated_at,
            },
        };
        
        self.cosmos.containers.posts
            .create_document()
            .is_upsert(true)
            .execute_with_partition_key(&cosmos_post.partition_key, &cosmos_post)
            .await
            .map_err(|e| MigrationError::CosmosError(e))?;
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct MigrationReport {
    pub users: EntityMigrationReport,
    pub posts: EntityMigrationReport,
    pub comments: EntityMigrationReport,
    pub votes: EntityMigrationReport,
    pub total_duration: Duration,
}

#[derive(Debug)]
pub struct EntityMigrationReport {
    pub entity_type: String,
    pub total_count: usize,
    pub migrated_count: usize,
    pub error_count: usize,
}

impl MigrationReport {
    pub fn success_rate(&self) -> f64 {
        let total_migrated = self.users.migrated_count + 
                           self.posts.migrated_count + 
                           self.comments.migrated_count + 
                           self.votes.migrated_count;
        
        let total_attempted = self.users.total_count + 
                            self.posts.total_count + 
                            self.comments.total_count + 
                            self.votes.total_count;
        
        if total_attempted == 0 {
            1.0
        } else {
            total_migrated as f64 / total_attempted as f64
        }
    }
    
    pub fn print_summary(&self) {
        println!("\n=== Migration Summary ===");
        println!("Total Duration: {:?}", self.total_duration);
        println!("Overall Success Rate: {:.2}%", self.success_rate() * 100.0);
        println!();
        
        for report in [&self.users, &self.posts, &self.comments, &self.votes] {
            let success_rate = if report.total_count == 0 {
                100.0
            } else {
                (report.migrated_count as f64 / report.total_count as f64) * 100.0
            };
            
            println!(
                "{}: {}/{} migrated ({:.2}% success, {} errors)",
                report.entity_type,
                report.migrated_count,
                report.total_count,
                success_rate,
                report.error_count
            );
        }
    }
}
```

### Phase 3: Dual-Write Implementation (Week 5)

```rust
pub struct DualWriteManager {
    postgres: Arc<PostgresDatabase>,
    cosmos: Arc<CosmosDatabase>,
    write_strategy: DualWriteStrategy,
    metrics: Arc<DualWriteMetrics>,
}

#[derive(Debug, Clone)]
pub enum DualWriteStrategy {
    PostgresFirst,  // Write to PostgreSQL first, then Cosmos DB
    CosmosFirst,    // Write to Cosmos DB first, then PostgreSQL
    Parallel,       // Write to both simultaneously
}

#[async_trait]
impl Database for DualWriteManager {
    type Error = DualWriteError;
    
    async fn create_post(&self, request: &CreatePostRequest) -> Result<Post, Self::Error> {
        let start_time = Instant::now();
        
        match self.write_strategy {
            DualWriteStrategy::PostgresFirst => {
                // Primary write to PostgreSQL
                let post = self.postgres.create_post(request).await
                    .map_err(|e| DualWriteError::PrimaryWriteFailed(e.to_string()))?;
                
                // Secondary write to Cosmos DB (best effort)
                if let Err(e) = self.cosmos.create_post(request).await {
                    warn!("Cosmos DB write failed (PostgreSQL succeeded): {}", e);
                    self.metrics.record_secondary_failure("create_post", "cosmos").await;
                }
                
                self.metrics.record_success("create_post", start_time.elapsed()).await;
                Ok(post)
            },
            
            DualWriteStrategy::CosmosFirst => {
                // Primary write to Cosmos DB
                let post = self.cosmos.create_post(request).await
                    .map_err(|e| DualWriteError::PrimaryWriteFailed(e.to_string()))?;
                
                // Secondary write to PostgreSQL (best effort)
                if let Err(e) = self.postgres.create_post(request).await {
                    warn!("PostgreSQL write failed (Cosmos succeeded): {}", e);
                    self.metrics.record_secondary_failure("create_post", "postgres").await;
                }
                
                self.metrics.record_success("create_post", start_time.elapsed()).await;
                Ok(post)
            },
            
            DualWriteStrategy::Parallel => {
                // Write to both databases simultaneously
                let (postgres_result, cosmos_result) = tokio::join!(
                    self.postgres.create_post(request),
                    self.cosmos.create_post(request)
                );
                
                match (postgres_result, cosmos_result) {
                    (Ok(postgres_post), Ok(_cosmos_post)) => {
                        self.metrics.record_success("create_post", start_time.elapsed()).await;
                        Ok(postgres_post) // Return PostgreSQL result for now
                    },
                    (Ok(postgres_post), Err(cosmos_error)) => {
                        warn!("Cosmos DB write failed: {}", cosmos_error);
                        self.metrics.record_secondary_failure("create_post", "cosmos").await;
                        Ok(postgres_post)
                    },
                    (Err(postgres_error), Ok(cosmos_post)) => {
                        warn!("PostgreSQL write failed: {}", postgres_error);
                        self.metrics.record_secondary_failure("create_post", "postgres").await;
                        
                        // Convert Cosmos post to PostgreSQL format
                        Ok(cosmos_post.into())
                    },
                    (Err(postgres_error), Err(cosmos_error)) => {
                        error!("Both writes failed - PostgreSQL: {}, Cosmos: {}", postgres_error, cosmos_error);
                        self.metrics.record_total_failure("create_post").await;
                        Err(DualWriteError::BothWritesFailed {
                            postgres_error: postgres_error.to_string(),
                            cosmos_error: cosmos_error.to_string(),
                        })
                    }
                }
            }
        }
    }
}
```

### Phase 4: Read Traffic Migration (Week 6)

```rust
pub struct ReadMigrationManager {
    postgres: Arc<PostgresDatabase>,
    cosmos: Arc<CosmosDatabase>,
    feature_flags: Arc<FeatureFlagManager>,
    fallback_strategy: ReadFallbackStrategy,
    metrics: Arc<ReadMigrationMetrics>,
}

#[derive(Debug, Clone)]
pub enum ReadFallbackStrategy {
    PostgresOnError,  // Fall back to PostgreSQL if Cosmos DB fails
    CosmosOnError,    // Fall back to Cosmos DB if PostgreSQL fails
    NoFallback,       // No fallback, return error
}

#[async_trait]
impl Database for ReadMigrationManager {
    type Error = ReadMigrationError;
    
    async fn get_post_by_id(&self, id: Uuid) -> Result<Option<Post>, Self::Error> {
        // Determine which database to read from
        let use_cosmos = self.feature_flags
            .is_enabled("cosmos_db_reads", &id.to_string(), &RequestContext::default())
            .await;
        
        let start_time = Instant::now();
        
        if use_cosmos {
            // Primary read from Cosmos DB
            match self.cosmos.get_post_by_id(id).await {
                Ok(post) => {
                    self.metrics.record_read_success("cosmos", "get_post_by_id", start_time.elapsed()).await;
                    Ok(post)
                },
                Err(e) => {
                    warn!("Cosmos DB read failed for post {}: {}", id, e);
                    self.metrics.record_read_failure("cosmos", "get_post_by_id").await;
                    
                    match self.fallback_strategy {
                        ReadFallbackStrategy::PostgresOnError => {
                            info!("Falling back to PostgreSQL for post {}", id);
                            let result = self.postgres.get_post_by_id(id).await
                                .map_err(|e| ReadMigrationError::FallbackFailed(e.to_string()))?;
                            
                            self.metrics.record_fallback_success("postgres", "get_post_by_id").await;
                            Ok(result)
                        },
                        _ => Err(ReadMigrationError::PrimaryReadFailed(e.to_string()))
                    }
                }
            }
        } else {
            // Primary read from PostgreSQL
            match self.postgres.get_post_by_id(id).await {
                Ok(post) => {
                    self.metrics.record_read_success("postgres", "get_post_by_id", start_time.elapsed()).await;
                    Ok(post)
                },
                Err(e) => {
                    warn!("PostgreSQL read failed for post {}: {}", id, e);
                    self.metrics.record_read_failure("postgres", "get_post_by_id").await;
                    
                    match self.fallback_strategy {
                        ReadFallbackStrategy::CosmosOnError => {
                            info!("Falling back to Cosmos DB for post {}", id);
                            let result = self.cosmos.get_post_by_id(id).await
                                .map_err(|e| ReadMigrationError::FallbackFailed(e.to_string()))?;
                            
                            self.metrics.record_fallback_success("cosmos", "get_post_by_id").await;
                            Ok(result)
                        },
                        _ => Err(ReadMigrationError::PrimaryReadFailed(e.to_string()))
                    }
                }
            }
        }
    }
}
```

## Testing & Validation

### Migration Validation Checklist

#### Pre-Migration Container Validation
```bash
# Verify all containers use /pk partition key path
az cosmosdb sql container show --account-name $COSMOS_ACCOUNT --database-name $DATABASE_NAME --resource-group $RESOURCE_GROUP --name "users" --query "resource.partitionKey"
az cosmosdb sql container show --account-name $COSMOS_ACCOUNT --database-name $DATABASE_NAME --resource-group $RESOURCE_GROUP --name "posts" --query "resource.partitionKey"
az cosmosdb sql container show --account-name $COSMOS_ACCOUNT --database-name $DATABASE_NAME --resource-group $RESOURCE_GROUP --name "comments" --query "resource.partitionKey"
az cosmosdb sql container show --account-name $COSMOS_ACCOUNT --database-name $DATABASE_NAME --resource-group $RESOURCE_GROUP --name "votes" --query "resource.partitionKey"

# Expected output for all: {"paths": ["/pk"], "kind": "Hash"}
```

#### Performance Validation
```rust
// Test query performance patterns
pub async fn validate_partition_performance(&self) -> Result<PartitionPerformanceReport, CosmosError> {
    let user_id = "550e8400-e29b-41d4-a716-446655440000";
    
    // Test 1: Single-partition query (should be fast, low RU)
    let start = std::time::Instant::now();
    let user_posts = self.get_posts_by_user(Uuid::parse_str(user_id)?, 10, None).await?;
    let single_partition_time = start.elapsed();
    
    // Test 2: Cross-partition query (higher RU, acceptable latency)
    let start = std::time::Instant::now();
    let global_feed = self.get_posts_paginated(10, None).await?;
    let cross_partition_time = start.elapsed();
    
    // Test 3: Aggregation within partition (efficient)
    let start = std::time::Instant::now();
    let post_id = "123e4567-e89b-12d3-a456-426614174000";
    let vote_summary = self.get_vote_summary(Uuid::parse_str(post_id)?, "post").await?;
    let aggregation_time = start.elapsed();
    
    Ok(PartitionPerformanceReport {
        single_partition_ms: single_partition_time.as_millis() as u32,
        cross_partition_ms: cross_partition_time.as_millis() as u32,
        aggregation_ms: aggregation_time.as_millis() as u32,
        efficiency_ratio: cross_partition_time.as_millis() as f64 / single_partition_time.as_millis() as f64,
    })
}

#[derive(Debug)]
pub struct PartitionPerformanceReport {
    pub single_partition_ms: u32,  // Target: < 10ms
    pub cross_partition_ms: u32,   // Target: < 100ms
    pub aggregation_ms: u32,       // Target: < 20ms
    pub efficiency_ratio: f64,     // Should be 2-10x (cross vs single)
}
```

#### RU Consumption Monitoring
```bash
# Monitor RU consumption patterns
az monitor metrics list \
    --resource "/subscriptions/$SUBSCRIPTION_ID/resourceGroups/$RESOURCE_GROUP/providers/Microsoft.DocumentDB/databaseAccounts/$COSMOS_ACCOUNT" \
    --metric "TotalRequestUnits" \
    --interval PT1M \
    --aggregation Average,Maximum

# Check for hot partitions
az monitor metrics list \
    --resource "/subscriptions/$SUBSCRIPTION_ID/resourceGroups/$RESOURCE_GROUP/providers/Microsoft.DocumentDB/databaseAccounts/$COSMOS_ACCOUNT" \
    --metric "NormalizedRUConsumption" \
    --interval PT1M \
    --aggregation Maximum

# Target: No partition consistently above 80% RU consumption
```

### Data Consistency Validation

```rust
pub struct ConsistencyValidator {
    postgres: Arc<PostgresDatabase>,
    cosmos: Arc<CosmosDatabase>,
}

impl ConsistencyValidator {
    pub async fn validate_all_data(&self) -> Result<ConsistencyReport, ValidationError> {
        let mut report = ConsistencyReport::new();
        
        info!("Starting data consistency validation...");
        
        // Validate each entity type
        report.users = self.validate_users().await?;
        report.posts = self.validate_posts().await?;
        report.comments = self.validate_comments().await?;
        report.votes = self.validate_votes().await?;
        
        info!("Data consistency validation completed");
        Ok(report)
    }
    
    async fn validate_posts(&self) -> Result<EntityConsistencyReport, ValidationError> {
        let postgres_posts = self.postgres.get_all_posts().await?;
        let mut report = EntityConsistencyReport::new("posts");
        
        let progress = ProgressBar::new(postgres_posts.len() as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} posts validated")
                .unwrap()
        );
        
        for postgres_post in postgres_posts {
            progress.inc(1);
            
            match self.cosmos.get_post_by_id(postgres_post.id).await {
                Ok(Some(cosmos_post)) => {
                    if self.posts_are_equivalent(&postgres_post, &cosmos_post) {
                        report.consistent_count += 1;
                    } else {
                        report.inconsistent_count += 1;
                        report.inconsistencies.push(InconsistencyDetail {
                            entity_id: postgres_post.id.to_string(),
                            field_differences: self.compare_posts(&postgres_post, &cosmos_post),
                        });
                    }
                },
                Ok(None) => {
                    report.missing_in_cosmos += 1;
                    report.missing_entities.push(postgres_post.id.to_string());
                },
                Err(e) => {
                    report.cosmos_errors += 1;
                    warn!("Error fetching post {} from Cosmos DB: {}", postgres_post.id, e);
                }
            }
        }
        
        progress.finish_with_message("Posts validation completed");
        Ok(report)
    }
    
    fn posts_are_equivalent(&self, postgres_post: &PostgresPost, cosmos_post: &CosmosPost) -> bool {
        postgres_post.id == cosmos_post.id &&
        postgres_post.title == cosmos_post.title &&
        postgres_post.content == cosmos_post.content &&
        postgres_post.author_id == cosmos_post.author.id &&
        postgres_post.created_at == cosmos_post.timestamps.created_at &&
        postgres_post.is_blocked == cosmos_post.moderation.is_blocked
        // Add more field comparisons as needed
    }
    
    fn compare_posts(&self, postgres_post: &PostgresPost, cosmos_post: &CosmosPost) -> Vec<String> {
        let mut differences = Vec::new();
        
        if postgres_post.title != cosmos_post.title {
            differences.push(format!(
                "title: postgres='{}', cosmos='{}'",
                postgres_post.title, cosmos_post.title
            ));
        }
        
        if postgres_post.content != cosmos_post.content {
            differences.push(format!(
                "content: lengths differ - postgres={}, cosmos={}",
                postgres_post.content.len(), cosmos_post.content.len()
            ));
        }
        
        if postgres_post.popularity_score != cosmos_post.engagement.popularity_score {
            differences.push(format!(
                "popularity_score: postgres={}, cosmos={}",
                postgres_post.popularity_score, cosmos_post.engagement.popularity_score
            ));
        }
        
        differences
    }
}

#[derive(Debug)]
pub struct ConsistencyReport {
    pub users: EntityConsistencyReport,
    pub posts: EntityConsistencyReport,
    pub comments: EntityConsistencyReport,
    pub votes: EntityConsistencyReport,
}

#[derive(Debug)]
pub struct EntityConsistencyReport {
    pub entity_type: String,
    pub consistent_count: usize,
    pub inconsistent_count: usize,
    pub missing_in_cosmos: usize,
    pub cosmos_errors: usize,
    pub inconsistencies: Vec<InconsistencyDetail>,
    pub missing_entities: Vec<String>,
}

#[derive(Debug)]
pub struct InconsistencyDetail {
    pub entity_id: String,
    pub field_differences: Vec<String>,
}

impl ConsistencyReport {
    pub fn overall_consistency_rate(&self) -> f64 {
        let total_consistent = self.users.consistent_count + 
                             self.posts.consistent_count + 
                             self.comments.consistent_count + 
                             self.votes.consistent_count;
        
        let total_compared = total_consistent + 
                           self.users.inconsistent_count + 
                           self.posts.inconsistent_count + 
                           self.comments.inconsistent_count + 
                           self.votes.inconsistent_count;
        
        if total_compared == 0 {
            1.0
        } else {
            total_consistent as f64 / total_compared as f64
        }
    }
    
    pub fn is_migration_ready(&self) -> bool {
        let consistency_threshold = 0.99; // 99% consistency required
        self.overall_consistency_rate() >= consistency_threshold
    }
}
```

### Performance Testing

```rust
pub struct PerformanceComparison {
    postgres: Arc<PostgresDatabase>,
    cosmos: Arc<CosmosDatabase>,
}

impl PerformanceComparison {
    pub async fn run_comprehensive_benchmark(&self) -> Result<BenchmarkReport, BenchmarkError> {
        let mut report = BenchmarkReport::new();
        
        info!("Starting comprehensive performance benchmark...");
        
        // Test different query patterns
        report.single_record_reads = self.benchmark_single_record_reads().await?;
        report.paginated_queries = self.benchmark_paginated_queries().await?;
        report.complex_filters = self.benchmark_complex_filters().await?;
        report.write_operations = self.benchmark_write_operations().await?;
        
        info!("Performance benchmark completed");
        Ok(report)
    }
    
    async fn benchmark_single_record_reads(&self) -> Result<OperationBenchmark, BenchmarkError> {
        let test_post_ids = self.get_random_post_ids(100).await?;
        
        // Benchmark PostgreSQL
        let postgres_times = self.measure_operation_times(
            "PostgreSQL single reads",
            &test_post_ids,
            |id| async move {
                self.postgres.get_post_by_id(*id).await
            }
        ).await;
        
        // Benchmark Cosmos DB
        let cosmos_times = self.measure_operation_times(
            "Cosmos DB single reads",
            &test_post_ids,
            |id| async move {
                self.cosmos.get_post_by_id(*id).await
            }
        ).await;
        
        Ok(OperationBenchmark {
            operation_type: "single_record_reads".to_string(),
            postgres_metrics: PerformanceMetrics::from_times(postgres_times),
            cosmos_metrics: PerformanceMetrics::from_times(cosmos_times),
        })
    }
    
    async fn benchmark_paginated_queries(&self) -> Result<OperationBenchmark, BenchmarkError> {
        let page_sizes = vec![10, 20, 50, 100];
        let mut postgres_times = Vec::new();
        let mut cosmos_times = Vec::new();
        
        for page_size in page_sizes {
            // PostgreSQL pagination
            let start_time = Instant::now();
            let _ = self.postgres.get_posts_paginated(page_size, None).await?;
            postgres_times.push(start_time.elapsed());
            
            // Cosmos DB pagination
            let start_time = Instant::now();
            let _ = self.cosmos.get_posts_paginated(page_size, None).await?;
            cosmos_times.push(start_time.elapsed());
        }
        
        Ok(OperationBenchmark {
            operation_type: "paginated_queries".to_string(),
            postgres_metrics: PerformanceMetrics::from_times(postgres_times),
            cosmos_metrics: PerformanceMetrics::from_times(cosmos_times),
        })
    }
    
    async fn measure_operation_times<F, Fut, T>(
        &self,
        operation_name: &str,
        test_data: &[Uuid],
        operation: F,
    ) -> Vec<Duration>
    where
        F: Fn(&Uuid) -> Fut + Send + Sync,
        Fut: Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>> + Send,
    {
        let progress = ProgressBar::new(test_data.len() as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template(&format!("{{spinner:.green}} [{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{pos}}/{{len}} {}", operation_name))
                .unwrap()
        );
        
        let mut times = Vec::new();
        
        for id in test_data {
            let start_time = Instant::now();
            let _ = operation(id).await;
            times.push(start_time.elapsed());
            progress.inc(1);
        }
        
        progress.finish();
        times
    }
}

#[derive(Debug)]
pub struct BenchmarkReport {
    pub single_record_reads: OperationBenchmark,
    pub paginated_queries: OperationBenchmark,
    pub complex_filters: OperationBenchmark,
    pub write_operations: OperationBenchmark,
}

#[derive(Debug)]
pub struct OperationBenchmark {
    pub operation_type: String,
    pub postgres_metrics: PerformanceMetrics,
    pub cosmos_metrics: PerformanceMetrics,
}

#[derive(Debug)]
pub struct PerformanceMetrics {
    pub mean_duration: Duration,
    pub median_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub sample_count: usize,
}

impl PerformanceMetrics {
    pub fn from_times(mut times: Vec<Duration>) -> Self {
        times.sort();
        let len = times.len();
        
        if len == 0 {
            return Self {
                mean_duration: Duration::ZERO,
                median_duration: Duration::ZERO,
                p95_duration: Duration::ZERO,
                p99_duration: Duration::ZERO,
                min_duration: Duration::ZERO,
                max_duration: Duration::ZERO,
                sample_count: 0,
            };
        }
        
        let sum: Duration = times.iter().sum();
        let mean_duration = sum / len as u32;
        
        let median_duration = times[len / 2];
        let p95_duration = times[(len as f64 * 0.95) as usize];
        let p99_duration = times[(len as f64 * 0.99) as usize];
        let min_duration = times[0];
        let max_duration = times[len - 1];
        
        Self {
            mean_duration,
            median_duration,
            p95_duration,
            p99_duration,
            min_duration,
            max_duration,
            sample_count: len,
        }
    }
    
    pub fn improvement_over(&self, other: &PerformanceMetrics) -> f64 {
        let self_mean_ms = self.mean_duration.as_millis() as f64;
        let other_mean_ms = other.mean_duration.as_millis() as f64;
        
        if other_mean_ms == 0.0 {
            return 0.0;
        }
        
        (other_mean_ms - self_mean_ms) / other_mean_ms
    }
}

impl BenchmarkReport {
    pub fn print_summary(&self) {
        println!("\n=== Performance Benchmark Summary ===");
        
        for benchmark in [
            &self.single_record_reads,
            &self.paginated_queries,
            &self.complex_filters,
            &self.write_operations,
        ] {
            println!("\n{}:", benchmark.operation_type);
            
            let improvement = benchmark.cosmos_metrics.improvement_over(&benchmark.postgres_metrics);
            let improvement_symbol = if improvement > 0.0 { "🟢" } else { "🔴" };
            
            println!("  PostgreSQL: {:.2}ms (mean), {:.2}ms (p95)", 
                    benchmark.postgres_metrics.mean_duration.as_millis(),
                    benchmark.postgres_metrics.p95_duration.as_millis());
            
            println!("  Cosmos DB:  {:.2}ms (mean), {:.2}ms (p95)", 
                    benchmark.cosmos_metrics.mean_duration.as_millis(),
                    benchmark.cosmos_metrics.p95_duration.as_millis());
            
            println!("  {} Cosmos DB is {:.1}% {} than PostgreSQL",
                    improvement_symbol,
                    improvement.abs() * 100.0,
                    if improvement > 0.0 { "faster" } else { "slower" });
        }
    }
}
```

## Rollback Strategy

### Automated Rollback Triggers

```rust
pub struct RollbackManager {
    feature_flags: Arc<FeatureFlagManager>,
    metrics: Arc<MigrationMetrics>,
    alert_system: Arc<AlertSystem>,
    rollback_config: RollbackConfig,
}

#[derive(Debug, Clone)]
pub struct RollbackConfig {
    pub error_rate_threshold: f64,      // Trigger rollback if error rate exceeds this
    pub latency_increase_threshold: f64, // Trigger rollback if latency increases by this %
    pub data_loss_threshold: f64,       // Trigger rollback if data consistency drops below this
    pub monitoring_window: Duration,     // Time window for evaluating metrics
    pub rollback_delay: Duration,       // Grace period before executing rollback
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            error_rate_threshold: 0.05, // 5% error rate
            latency_increase_threshold: 0.50, // 50% latency increase
            data_loss_threshold: 0.95, // 95% data consistency
            monitoring_window: Duration::from_minutes(10),
            rollback_delay: Duration::from_minutes(2),
        }
    }
}

impl RollbackManager {
    pub async fn start_monitoring(&self) {
        let mut interval = tokio::time::interval(Duration::from_minutes(1));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_rollback_conditions().await {
                error!("Error checking rollback conditions: {}", e);
            }
        }
    }
    
    async fn check_rollback_conditions(&self) -> Result<(), RollbackError> {
        let current_metrics = self.metrics
            .get_metrics_for_window(self.rollback_config.monitoring_window)
            .await?;
        
        let mut rollback_reasons = Vec::new();
        
        // Check error rate
        if current_metrics.error_rate > self.rollback_config.error_rate_threshold {
            rollback_reasons.push(format!(
                "Error rate too high: {:.2}% (threshold: {:.2}%)",
                current_metrics.error_rate * 100.0,
                self.rollback_config.error_rate_threshold * 100.0
            ));
        }
        
        // Check latency increase
        let baseline_latency = self.metrics.get_baseline_latency().await?;
        let latency_increase = (current_metrics.avg_latency - baseline_latency) / baseline_latency;
        
        if latency_increase > self.rollback_config.latency_increase_threshold {
            rollback_reasons.push(format!(
                "Latency increased by {:.1}% (threshold: {:.1}%)",
                latency_increase * 100.0,
                self.rollback_config.latency_increase_threshold * 100.0
            ));
        }
        
        // Check data consistency
        if current_metrics.data_consistency_rate < self.rollback_config.data_loss_threshold {
            rollback_reasons.push(format!(
                "Data consistency too low: {:.2}% (threshold: {:.2}%)",
                current_metrics.data_consistency_rate * 100.0,
                self.rollback_config.data_loss_threshold * 100.0
            ));
        }
        
        if !rollback_reasons.is_empty() {
            warn!("Rollback conditions detected: {:?}", rollback_reasons);
            
            // Send alert
            let alert = Alert::RollbackTriggered {
                reasons: rollback_reasons.clone(),
                severity: AlertSeverity::Critical,
                timestamp: Utc::now(),
            };
            
            self.alert_system.send_alert(alert).await?;
            
            // Wait for grace period
            info!("Waiting {:?} before executing rollback...", self.rollback_config.rollback_delay);
            tokio::time::sleep(self.rollback_config.rollback_delay).await;
            
            // Execute rollback
            self.execute_rollback(rollback_reasons).await?;
        }
        
        Ok(())
    }
    
    async fn execute_rollback(&self, reasons: Vec<String>) -> Result<(), RollbackError> {
        error!("EXECUTING AUTOMATIC ROLLBACK - Reasons: {:?}", reasons);
        
        // 1. Disable Cosmos DB reads immediately
        self.feature_flags.disable_flag("cosmos_db_reads").await?;
        info!("✅ Disabled Cosmos DB reads");
        
        // 2. Disable Cosmos DB writes
        self.feature_flags.disable_flag("cosmos_db_writes").await?;
        info!("✅ Disabled Cosmos DB writes");
        
        // 3. Switch all traffic back to PostgreSQL
        self.feature_flags.disable_flag("cosmos_db_migration").await?;
        info!("✅ Switched all traffic back to PostgreSQL");
        
        // 4. Send completion alert
        let alert = Alert::RollbackCompleted {
            reasons,
            timestamp: Utc::now(),
            recovery_actions: vec![
                "All database traffic routed to PostgreSQL".to_string(),
                "Cosmos DB integration disabled".to_string(),
                "System stability restored".to_string(),
            ],
        };
        
        self.alert_system.send_alert(alert).await?;
        
        info!("🔄 ROLLBACK COMPLETED - System restored to PostgreSQL");
        Ok(())
    }
    
    pub async fn manual_rollback(&self, reason: &str) -> Result<(), RollbackError> {
        info!("EXECUTING MANUAL ROLLBACK - Reason: {}", reason);
        
        self.execute_rollback(vec![format!("Manual rollback: {}", reason)]).await
    }
}

#[derive(Debug)]
pub struct CurrentMetrics {
    pub error_rate: f64,
    pub avg_latency: f64,
    pub data_consistency_rate: f64,
    pub throughput: f64,
}

#[derive(Debug, Clone)]
pub enum Alert {
    RollbackTriggered {
        reasons: Vec<String>,
        severity: AlertSeverity,
        timestamp: DateTime<Utc>,
    },
    RollbackCompleted {
        reasons: Vec<String>,
        timestamp: DateTime<Utc>,
        recovery_actions: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}
```

### Data Recovery Procedures

```rust
pub struct DataRecoveryManager {
    postgres: Arc<PostgresDatabase>,
    cosmos: Arc<CosmosDatabase>,
    backup_storage: Arc<BackupStorage>,
}

impl DataRecoveryManager {
    pub async fn recover_lost_data(&self, time_range: TimeRange) -> Result<RecoveryReport, RecoveryError> {
        info!("Starting data recovery for time range: {:?}", time_range);
        
        let mut report = RecoveryReport::new();
        
        // 1. Find data that exists in PostgreSQL but not in Cosmos DB
        let missing_data = self.find_missing_data_in_cosmos(time_range).await?;
        report.total_missing = missing_data.len();
        
        // 2. Recover missing data
        for missing_item in missing_data {
            match self.recover_single_item(&missing_item).await {
                Ok(_) => report.recovered_count += 1,
                Err(e) => {
                    report.failed_count += 1;
                    warn!("Failed to recover item {}: {}", missing_item.id, e);
                }
            }
        }
        
        // 3. Find and fix data inconsistencies
        let inconsistencies = self.find_data_inconsistencies(time_range).await?;
        report.total_inconsistencies = inconsistencies.len();
        
        for inconsistency in inconsistencies {
            match self.fix_inconsistency(&inconsistency).await {
                Ok(_) => report.fixed_count += 1,
                Err(e) => {
                    report.fix_failed_count += 1;
                    warn!("Failed to fix inconsistency {}: {}", inconsistency.id, e);
                }
            }
        }
        
        info!("Data recovery completed: {:?}", report);
        Ok(report)
    }
    
    async fn find_missing_data_in_cosmos(&self, time_range: TimeRange) -> Result<Vec<MissingDataItem>, RecoveryError> {
        let postgres_items = self.postgres.get_items_in_range(time_range).await?;
        let mut missing_items = Vec::new();
        
        for item in postgres_items {
            match item.entity_type {
                EntityType::Post => {
                    if self.cosmos.get_post_by_id(item.id).await?.is_none() {
                        missing_items.push(MissingDataItem {
                            id: item.id,
                            entity_type: EntityType::Post,
                            created_at: item.created_at,
                        });
                    }
                },
                EntityType::Comment => {
                    if self.cosmos.get_comment_by_id(item.id).await?.is_none() {
                        missing_items.push(MissingDataItem {
                            id: item.id,
                            entity_type: EntityType::Comment,
                            created_at: item.created_at,
                        });
                    }
                },
                // Handle other entity types...
                _ => {}
            }
        }
        
        Ok(missing_items)
    }
    
    async fn recover_single_item(&self, missing_item: &MissingDataItem) -> Result<(), RecoveryError> {
        match missing_item.entity_type {
            EntityType::Post => {
                // Get the post from PostgreSQL
                if let Some(postgres_post) = self.postgres.get_post_by_id(missing_item.id).await? {
                    // Convert and insert into Cosmos DB
                    let cosmos_post = self.convert_postgres_post_to_cosmos(&postgres_post).await?;
                    self.cosmos.create_post(&cosmos_post.into()).await?;
                    info!("Recovered post: {}", missing_item.id);
                }
            },
            EntityType::Comment => {
                if let Some(postgres_comment) = self.postgres.get_comment_by_id(missing_item.id).await? {
                    let cosmos_comment = self.convert_postgres_comment_to_cosmos(&postgres_comment).await?;
                    self.cosmos.create_comment(&cosmos_comment.into()).await?;
                    info!("Recovered comment: {}", missing_item.id);
                }
            },
            // Handle other types...
            _ => {}
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct RecoveryReport {
    pub total_missing: usize,
    pub recovered_count: usize,
    pub failed_count: usize,
    pub total_inconsistencies: usize,
    pub fixed_count: usize,
    pub fix_failed_count: usize,
}

#[derive(Debug)]
pub struct MissingDataItem {
    pub id: Uuid,
    pub entity_type: EntityType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}
```

## Appendix: Code Examples

### Complete Migration CLI Tool

```rust
use clap::{Parser, Subcommand};
use tokio;

#[derive(Parser)]
#[command(name = "cosmos-migration")]
#[command(about = "A tool for migrating Social Pulse from PostgreSQL to Cosmos DB")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Migrate data from PostgreSQL to Cosmos DB
    Migrate {
        /// Entity types to migrate (users, posts, comments, votes, or all)
        #[arg(short, long, default_value = "all")]
        entities: String,
        
        /// Batch size for migration
        #[arg(short, long, default_value = "1000")]
        batch_size: usize,
        
        /// Delay between batches in milliseconds
        #[arg(short, long, default_value = "100")]
        delay: u64,
        
        /// Dry run (don't actually migrate)
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Validate data consistency between PostgreSQL and Cosmos DB
    Validate {
        /// Entity types to validate
        #[arg(short, long, default_value = "all")]
        entities: String,
        
        /// Sample size for validation (0 = validate all)
        #[arg(short, long, default_value = "0")]
        sample_size: usize,
    },
    
    /// Run performance benchmarks
    Benchmark {
        /// Number of operations per test
        #[arg(short, long, default_value = "100")]
        operations: usize,
        
        /// Include write operations in benchmark
        #[arg(long)]
        include_writes: bool,
    },
    
    /// Execute rollback to PostgreSQL
    Rollback {
        /// Reason for rollback
        #[arg(short, long, default_value = "Manual rollback")]
        reason: String,
        
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    
    /// Recover lost data
    Recover {
        /// Start time for recovery (ISO 8601 format)
        #[arg(short, long)]
        start_time: String,
        
        /// End time for recovery (ISO 8601 format)
        #[arg(short, long)]
        end_time: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();
    
    let cli = Cli::parse();
    
    // Initialize databases
    let postgres = PostgresDatabase::new(&std::env::var("DATABASE_URL")?).await?;
    let cosmos = CosmosDatabase::new(
        &std::env::var("COSMOS_DB_ACCOUNT")?,
        &std::env::var("COSMOS_DB_KEY")?,
        &std::env::var("COSMOS_DB_DATABASE")?,
    ).await?;
    
    match cli.command {
        Commands::Migrate { entities, batch_size, delay, dry_run } => {
            let migration_tool = MigrationTool {
                postgres: Arc::new(postgres),
                cosmos: Arc::new(cosmos),
                batch_size,
                delay_between_batches: Duration::from_millis(delay),
            };
            
            if dry_run {
                println!("DRY RUN: Would migrate {} with batch size {}", entities, batch_size);
                return Ok(());
            }
            
            let report = migration_tool.migrate_all_data().await?;
            report.print_summary();
        },
        
        Commands::Validate { entities, sample_size } => {
            let validator = ConsistencyValidator {
                postgres: Arc::new(postgres),
                cosmos: Arc::new(cosmos),
            };
            
            let report = validator.validate_all_data().await?;
            
            println!("\n=== Validation Results ===");
            println!("Overall consistency rate: {:.2}%", report.overall_consistency_rate() * 100.0);
            println!("Migration ready: {}", if report.is_migration_ready() { "✅ Yes" } else { "❌ No" });
        },
        
        Commands::Benchmark { operations, include_writes } => {
            let benchmark = PerformanceComparison {
                postgres: Arc::new(postgres),
                cosmos: Arc::new(cosmos),
            };
            
            let report = benchmark.run_comprehensive_benchmark().await?;
            report.print_summary();
        },
        
        Commands::Rollback { reason, force } => {
            if !force {
                print!("Are you sure you want to rollback to PostgreSQL? (y/N): ");
                use std::io::{self, Write};
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Rollback cancelled");
                    return Ok(());
                }
            }
            
            let rollback_manager = RollbackManager::new(/* ... */);
            rollback_manager.manual_rollback(&reason).await?;
            
            println!("✅ Rollback completed successfully");
        },
        
        Commands::Recover { start_time, end_time } => {
            let start = DateTime::parse_from_rfc3339(&start_time)?.with_timezone(&Utc);
            let end = DateTime::parse_from_rfc3339(&end_time)?.with_timezone(&Utc);
            
            let recovery_manager = DataRecoveryManager {
                postgres: Arc::new(postgres),
                cosmos: Arc::new(cosmos),
                backup_storage: Arc::new(/* backup storage implementation */),
            };
            
            let report = recovery_manager.recover_lost_data(TimeRange { start, end }).await?;
            
            println!("\n=== Recovery Report ===");
            println!("Missing items found: {}", report.total_missing);
            println!("Items recovered: {}", report.recovered_count);
            println!("Recovery failures: {}", report.failed_count);
            println!("Inconsistencies found: {}", report.total_inconsistencies);
            println!("Inconsistencies fixed: {}", report.fixed_count);
        },
    }
    
    Ok(())
}
```

### Environment Configuration for Migration

```bash
# .env.migration

# PostgreSQL Configuration
DATABASE_URL=postgresql://user:password@localhost/social_pulse

# Cosmos DB Configuration
COSMOS_DB_ACCOUNT=social-pulse-cosmos
COSMOS_DB_KEY=your-cosmos-primary-key
COSMOS_DB_DATABASE=social_pulse
COSMOS_DB_REGION=East US

# Partition Key Strategy (ALL containers use /pk)
COSMOS_DB_PARTITION_KEY_PATH=/pk

# Migration Settings
MIGRATION_BATCH_SIZE=1000
MIGRATION_DELAY_MS=100
MIGRATION_PARALLEL_WORKERS=10

# Container Throughput (RU/s)
COSMOS_DB_USERS_THROUGHPUT=400
COSMOS_DB_POSTS_THROUGHPUT=1000
COSMOS_DB_COMMENTS_THROUGHPUT=600
COSMOS_DB_VOTES_THROUGHPUT=400

# Feature Flags
COSMOS_DB_MIGRATION_ENABLED=true
DUAL_WRITE_ENABLED=false
COSMOS_DB_READS_PERCENTAGE=0

# Monitoring
METRICS_ENABLED=true
ALERT_WEBHOOK_URL=https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK

# Rollback Configuration
ROLLBACK_ERROR_RATE_THRESHOLD=0.05
ROLLBACK_LATENCY_THRESHOLD=0.50
ROLLBACK_CONSISTENCY_THRESHOLD=0.95

# Performance Tuning
COSMOS_DB_CONNECTION_TIMEOUT=30
COSMOS_DB_REQUEST_TIMEOUT=10
COSMOS_DB_MAX_RETRY_ATTEMPTS=3

# Validation Targets
SINGLE_PARTITION_QUERY_TARGET_MS=10
CROSS_PARTITION_QUERY_TARGET_MS=100
MAX_RU_CONSUMPTION_PERCENT=80
```

---

*This migration guide provides a comprehensive framework for transitioning from PostgreSQL to Azure Cosmos DB while maintaining data integrity, performance, and system reliability. The phased approach ensures minimal risk and maximum observability throughout the migration process.*