pub mod api;
pub mod web;

use axum::Router;
use crate::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .merge(web::routes())
        .nest("/api/v1", api::routes())
}