use axum::http::HeaderMap;
use reqwest::header::{HeaderValue, SET_COOKIE};
use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::BTreeMap;

use crate::{
    models::{Farmer, MeResponse},
    state::AppState,
};

pub struct AuthStateResult {
    pub current_user: Option<MeResponse>,
    pub set_cookie_headers: Vec<HeaderValue>,
}

#[derive(Debug, Deserialize)]
struct CsrfResponse {
    csrf_token: String,
}

fn cookie_header(headers: &HeaderMap) -> Option<String> {
    headers.get("cookie")?.to_str().ok().map(str::to_string)
}

fn merge_cookie_header(base_cookie_header: &str, set_cookie_headers: &[HeaderValue]) -> String {
    let mut cookies = BTreeMap::new();

    for pair in base_cookie_header.split(';') {
        let trimmed = pair.trim();
        if let Some((name, value)) = trimmed.split_once('=') {
            cookies.insert(name.to_string(), value.to_string());
        }
    }

    for header in set_cookie_headers {
        let Ok(raw_header) = header.to_str() else {
            continue;
        };
        let Some(cookie_pair) = raw_header.split(';').next() else {
            continue;
        };
        let Some((name, value)) = cookie_pair.split_once('=') else {
            continue;
        };
        cookies.insert(name.trim().to_string(), value.trim().to_string());
    }

    cookies
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn extract_set_cookie_headers(response: &reqwest::Response) -> Vec<HeaderValue> {
    response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| HeaderValue::from_str(value.to_str().ok()?).ok())
        .collect()
}

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

pub async fn fetch_auth_state(state: &AppState, headers: &HeaderMap) -> AuthStateResult {
    let Some(base_cookie_header) = cookie_header(headers) else {
        return AuthStateResult {
            current_user: None,
            set_cookie_headers: Vec::new(),
        };
    };

    let me_url = format!("{}/auth/me", state.auth_service_url);
    let me_response = match state
        .client
        .get(&me_url)
        .header("cookie", &base_cookie_header)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return AuthStateResult {
                current_user: None,
                set_cookie_headers: Vec::new(),
            };
        }
    };

    if me_response.status().is_success() {
        return AuthStateResult {
            current_user: me_response.json::<MeResponse>().await.ok(),
            set_cookie_headers: Vec::new(),
        };
    }

    if me_response.status() != StatusCode::UNAUTHORIZED {
        return AuthStateResult {
            current_user: None,
            set_cookie_headers: Vec::new(),
        };
    }

    let csrf_response = match state
        .client
        .get(format!("{}/auth/csrf", state.auth_service_url))
        .header("cookie", &base_cookie_header)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return AuthStateResult {
                current_user: None,
                set_cookie_headers: Vec::new(),
            };
        }
    };

    let csrf_set_cookie_headers = extract_set_cookie_headers(&csrf_response);

    let csrf_payload = match csrf_response.json::<CsrfResponse>().await {
        Ok(payload) => payload,
        Err(_) => {
            return AuthStateResult {
                current_user: None,
                set_cookie_headers: Vec::new(),
            };
        }
    };

    let refresh_cookie_header = merge_cookie_header(&base_cookie_header, &csrf_set_cookie_headers);

    let refresh_response = match state
        .client
        .post(format!("{}/auth/refresh", state.auth_service_url))
        .header("cookie", refresh_cookie_header)
        .header("x-csrf-token", csrf_payload.csrf_token)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return AuthStateResult {
                current_user: None,
                set_cookie_headers: Vec::new(),
            };
        }
    };

    if !refresh_response.status().is_success() {
        return AuthStateResult {
            current_user: None,
            set_cookie_headers: Vec::new(),
        };
    }

    let refreshed_set_cookie_headers = extract_set_cookie_headers(&refresh_response);
    let refreshed_cookie_header = merge_cookie_header(&base_cookie_header, &refreshed_set_cookie_headers);

    let refreshed_me = match state
        .client
        .get(me_url)
        .header("cookie", refreshed_cookie_header)
        .send()
        .await
    {
        Ok(response) => match response.error_for_status() {
            Ok(response) => response.json::<MeResponse>().await.ok(),
            Err(_) => None,
        },
        Err(_) => None,
    };

    AuthStateResult {
        current_user: refreshed_me,
        set_cookie_headers: refreshed_set_cookie_headers,
    }
}

pub async fn logout_current_user(state: &AppState, headers: &HeaderMap) -> Option<Vec<HeaderValue>> {
    let cookie_header = match cookie_header(headers) {
        Some(value) => value,
        None => return Some(Vec::new()),
    };

    let csrf_response = state
        .client
        .get(format!("{}/auth/csrf", state.auth_service_url))
        .header("cookie", &cookie_header)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;

    let csrf_set_cookie_headers = extract_set_cookie_headers(&csrf_response);

    let csrf_payload = csrf_response.json::<CsrfResponse>().await.ok()?;
    let logout_cookie_header = merge_cookie_header(&cookie_header, &csrf_set_cookie_headers);

    let logout_response = state
        .client
        .post(format!("{}/auth/logout", state.auth_service_url))
        .header("cookie", logout_cookie_header)
        .header("x-csrf-token", csrf_payload.csrf_token)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;

    let set_cookie_headers = logout_response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| HeaderValue::from_str(value.to_str().ok()?).ok())
        .collect();

    Some(set_cookie_headers)
}
