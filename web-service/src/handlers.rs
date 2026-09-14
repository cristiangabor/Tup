use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::Html,
};
use leptos::prelude::*;

use crate::{
    api::{fetch_current_user, fetch_farmer, fetch_farmers},
    components::{CropPage, DashboardPage, FarmerDetailPage, FarmersPage, HomePage},
    models::Farmer,
    state::AppState,
};

pub async fn index(State(state): State<AppState>) -> Html<String> {
    let farmers = fetch_farmers(&state).await.unwrap_or_default();
    let html = view! { <HomePage farmers=farmers /> }.to_html();
    Html(html)
}

pub async fn farmers(State(state): State<AppState>) -> Html<String> {
    let farmers = fetch_farmers(&state).await.unwrap_or_default();
    let html = view! { <FarmersPage farmers=farmers /> }.to_html();
    Html(html)
}

pub async fn farmer_detail(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Html<String> {
    let farmer = fetch_farmer(&state, &slug).await.unwrap_or_else(|| Farmer {
        slug,
        id: String::new(),
        business_name: "Farmer not found".to_string(),
        farmer_name: String::new(),
        email: String::new(),
        city: String::new(),
        region: String::new(),
        country: String::new(),
        latitude: 0.0,
        longitude: 0.0,
        cultivated_crops: Vec::new(),
        products_for_sale: Vec::new(),
    });
    let html = view! { <FarmerDetailPage farmer=farmer /> }.to_html();
    Html(html)
}

pub async fn crop_detail(State(state): State<AppState>, Path(crop): Path<String>) -> Html<String> {
    let farmers = fetch_farmers(&state).await.unwrap_or_default();
    let filtered: Vec<Farmer> = farmers
        .into_iter()
        .filter(|farmer| {
            farmer
                .cultivated_crops
                .iter()
                .any(|item| item.eq_ignore_ascii_case(&crop))
        })
        .collect();
    let html = view! { <CropPage crop=crop.clone() farmers=filtered /> }.to_html();
    Html(html)
}

pub async fn dashboard(State(state): State<AppState>, headers: HeaderMap) -> Html<String> {
    let current_user = fetch_current_user(&state, &headers).await;
    let html = view! { <DashboardPage current_user=current_user /> }.to_html();
    Html(html)
}

pub async fn health() -> &'static str {
    "ok"
}
