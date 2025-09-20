pub mod api;
pub mod web;
pub mod users;
pub mod posts;
pub mod auth;
pub mod comments;
pub mod vote_routes;

use axum::Router;
use crate::AppState;

pub fn create_routes(app_state: &AppState) -> Router<AppState> {
    Router::new()
        .merge(web::routes())
        .nest("/api", api::routes(app_state))
}