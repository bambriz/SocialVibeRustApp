use axum::{
    extract::{State, Json},
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::{AppState, AppError, Result};
use crate::models::user::{CreateUserRequest, LoginRequest};

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: crate::models::user::UserResponse,
}

pub async fn register(
    State(app_state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<ResponseJson<Value>> {
    let user = app_state.user_service.create_user(request, &app_state.auth_service).await?;
    let token = app_state.auth_service.generate_token(
        user.id, 
        &user.username
    )?;
    
    Ok(ResponseJson(json!({
        "user": user,
        "token": token,
        "message": "User registered successfully"
    })))
}

pub async fn login(
    State(app_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<ResponseJson<Value>> {
    let (user, token) = app_state.user_service.authenticate_user(
        &request.email,
        &request.password,
        &app_state.auth_service
    ).await?;
    
    Ok(ResponseJson(json!({
        "user": user,
        "token": token,
        "message": "Login successful"
    })))
}