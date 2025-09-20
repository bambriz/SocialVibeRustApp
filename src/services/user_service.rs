use crate::models::{User};
use crate::models::user::{CreateUserRequest, UserResponse};
use crate::db::repository::UserRepository;
use crate::{AppError, Result};
use uuid::Uuid;
use std::sync::Arc;
use chrono::Utc;

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }


    pub async fn get_user(&self, _user_id: Uuid) -> Result<Option<UserResponse>> {
        Ok(None)
    }

    pub async fn create_user(&self, request: CreateUserRequest, auth_service: &crate::auth::AuthService) -> Result<UserResponse> {
        // Hash the password before storing
        let password_hash = auth_service.hash_password(&request.password)?;
        
        let user = User {
            id: uuid::Uuid::new_v4(),
            username: request.username,
            email: request.email,
            password_hash,
            display_name: None,
            bio: None,
            avatar_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
        };
        
        let created_user = self.user_repo.create_user(&user).await?;
        Ok(UserResponse::from(created_user))
    }

    pub async fn authenticate_user(&self, email: &str, password: &str, auth_service: &crate::auth::AuthService) -> Result<(UserResponse, String)> {
        let user = self.user_repo.get_user_by_email(email).await?
            .ok_or_else(|| AppError::AuthError("Invalid email or password".to_string()))?;

        if !auth_service.verify_password(password, &user.password_hash)? {
            return Err(AppError::AuthError("Invalid email or password".to_string()));
        }

        let token = auth_service.generate_token(user.id, &user.username)?;
        Ok((UserResponse::from(user), token))
    }
}