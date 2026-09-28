use axum::{routing::{get, post}, Router};
use tower_http::services::ServeDir;

use crate::{
    handlers::{crop_detail, dashboard, farmer_detail, farmers, health, index, logout, profile},
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/farmers", get(farmers))
        .route("/farmers/{slug}", get(farmer_detail))
        .route("/crops/{crop}", get(crop_detail))
        .route("/dashboard", get(dashboard))
        .route("/profile", get(profile))
        .route("/logout", post(logout))
        .route("/health", get(health))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state)
}
