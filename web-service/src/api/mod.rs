use axum::http::HeaderMap;

use crate::{
    models::{Farmer, MeResponse},
    state::AppState,
};

pub async fn fetch_farmers(state: &AppState) -> Option<Vec<Farmer>> {
    state
        .client
        .get(format!("{}/farmers", state.farmer_api_url))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<Vec<Farmer>>()
        .await
        .ok()
}

pub async fn fetch_farmer(state: &AppState, slug: &str) -> Option<Farmer> {
    state
        .client
        .get(format!("{}/farmers/{}", state.farmer_api_url, slug))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<Farmer>()
        .await
        .ok()
}

pub async fn fetch_current_user(state: &AppState, headers: &HeaderMap) -> Option<MeResponse> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?.to_string();
    state
        .client
        .get(format!("{}/auth/me", state.auth_service_url))
        .header("cookie", cookie_header)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json::<MeResponse>()
        .await
        .ok()
}
