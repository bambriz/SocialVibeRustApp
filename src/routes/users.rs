use axum::{
    extract::{State, Json},
    response::Json as ResponseJson,
};
use serde_json::{json, Value};
use crate::{AppState, Result};
use crate::models::user::CreateUserRequest;

pub async fn create_user(
    State(app_state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<ResponseJson<Value>> {
    let user = app_state.user_service.create_user(request, &app_state.auth_service).await?;
    
    Ok(ResponseJson(json!({
        "user": user,
        "message": "User created successfully"
    })))
}

