use axum::{
    extract::{Path, State},
    http::{header::SET_COOKIE, HeaderMap},
    response::{Html, IntoResponse, Redirect, Response},
};
use leptos::prelude::*;

use crate::{
    api::{fetch_auth_state, fetch_farmer, fetch_farmers, logout_current_user},
    components::{CropPage, DashboardPage, FarmerDetailPage, FarmersPage, HomePage, ProfilePage},
    models::Farmer,
    state::AppState,
};

pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let auth_state = fetch_auth_state(&state, &headers).await;
    let current_user = auth_state.current_user;
    let is_authenticated = has_auth_cookie(&headers, current_user.is_some());
    let farmers = fetch_farmers(&state).await.unwrap_or_default();
    let html = view! { <HomePage farmers=farmers login_url=login_url(&state) current_user=current_user is_authenticated=is_authenticated /> }.to_html();
    html_response(html, auth_state.set_cookie_headers)
}

pub async fn farmers(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let auth_state = fetch_auth_state(&state, &headers).await;
    let current_user = auth_state.current_user;
    let is_authenticated = has_auth_cookie(&headers, current_user.is_some());
    let farmers = fetch_farmers(&state).await.unwrap_or_default();
    let html = view! { <FarmersPage farmers=farmers login_url=login_url(&state) current_user=current_user is_authenticated=is_authenticated /> }.to_html();
    html_response(html, auth_state.set_cookie_headers)
}

pub async fn farmer_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Response {
    let auth_state = fetch_auth_state(&state, &headers).await;
    let current_user = auth_state.current_user;
    let is_authenticated = has_auth_cookie(&headers, current_user.is_some());
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
    let html = view! { <FarmerDetailPage farmer=farmer login_url=login_url(&state) current_user=current_user is_authenticated=is_authenticated /> }.to_html();
    html_response(html, auth_state.set_cookie_headers)
}

pub async fn crop_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(crop): Path<String>,
) -> Response {
    let auth_state = fetch_auth_state(&state, &headers).await;
    let current_user = auth_state.current_user;
    let is_authenticated = has_auth_cookie(&headers, current_user.is_some());
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
    let html = view! { <CropPage crop=crop.clone() farmers=filtered login_url=login_url(&state) current_user=current_user is_authenticated=is_authenticated /> }.to_html();
    html_response(html, auth_state.set_cookie_headers)
}

pub async fn dashboard(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let auth_state = fetch_auth_state(&state, &headers).await;
    let current_user = auth_state.current_user;
    let is_authenticated = has_auth_cookie(&headers, current_user.is_some());
    let html = view! { <DashboardPage current_user=current_user login_url=login_url(&state) is_authenticated=is_authenticated /> }.to_html();
    html_response(html, auth_state.set_cookie_headers)
}

pub async fn profile(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let auth_state = fetch_auth_state(&state, &headers).await;
    let current_user = auth_state.current_user;
    let is_authenticated = has_auth_cookie(&headers, current_user.is_some());
    let html = view! { <ProfilePage current_user=current_user login_url=login_url(&state) is_authenticated=is_authenticated /> }.to_html();
    html_response(html, auth_state.set_cookie_headers)
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let set_cookie_headers = logout_current_user(&state, &headers).await.unwrap_or_default();
    let mut response = Redirect::to("/").into_response();

    for header_value in set_cookie_headers {
        response.headers_mut().append(SET_COOKIE, header_value);
    }

    response
}

pub async fn health() -> &'static str {
    "ok"
}

fn login_url(state: &AppState) -> String {
    format!("{}/auth/google/login", state.auth_public_url)
}

fn html_response(html: String, set_cookie_headers: Vec<axum::http::HeaderValue>) -> Response {
    let mut response = Html(html).into_response();
    for header_value in set_cookie_headers {
        response.headers_mut().append(SET_COOKIE, header_value);
    }
    response
}

fn has_auth_cookie(headers: &HeaderMap, current_user_present: bool) -> bool {
    current_user_present
        || headers
            .get("cookie")
            .and_then(|value| value.to_str().ok())
            .map(|cookie| cookie.contains("access_token=") || cookie.contains("refresh_token="))
            .unwrap_or(false)
}
