use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse,
    TokenUrl,
};
use cookie::time::Duration as CookieDuration;
use rand::RngCore;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use std::io::{Error as IoError, ErrorKind};
use std::net::SocketAddr;
use thiserror::Error;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Clone)]
struct AppState {
    db: PgPool,
    cfg: AppConfig,
    oauth: BasicClient,
}

#[derive(Clone)]
struct AppConfig {
    access_secret: String,
    refresh_secret: String,
    cookie_secure: bool,
    cookie_domain: Option<String>,
    cookie_same_site: SameSite,
    frontend_redirect_url: String,
    google_userinfo_url: String,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("token error: {0}")]
    Token(#[from] jsonwebtoken::errors::Error),
    #[error("oauth error: {0}")]
    OAuth(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::Validation(m) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", m),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Authentication required".to_string(),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "CSRF token is missing or invalid".to_string(),
            ),
            AppError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                e.to_string(),
            ),
            AppError::Token(e) => (
                StatusCode::UNAUTHORIZED,
                "TOKEN_ERROR",
                e.to_string(),
            ),
            AppError::OAuth(m) => (StatusCode::BAD_GATEWAY, "OAUTH_ERROR", m),
            AppError::Http(e) => (
                StatusCode::BAD_GATEWAY,
                "HTTP_ERROR",
                e.to_string(),
            ),
        };

        (
            status,
            Json(serde_json::json!({
                "error": code,
                "message": message,
            })),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    jti: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
#[derive(ToSchema)]
struct RegisterRequest {
    #[schema(example = "farmer@example.com")]
    email: String,
    #[schema(example = "StrongPassword123")]
    password: String,
    #[schema(example = "Farmer One")]
    full_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[derive(ToSchema)]
struct LoginRequest {
    #[schema(example = "farmer@example.com")]
    email: String,
    #[schema(example = "StrongPassword123")]
    password: String,
}

#[derive(Debug, Deserialize)]
#[derive(ToSchema)]
struct AuthorizeRequest {
    #[schema(example = "farmers:write")]
    permission: String,
}

#[derive(Debug, Deserialize)]
#[derive(ToSchema)]
struct GoogleCallbackQuery {
    code: String,
    state: String,
}

#[derive(Debug, Serialize)]
#[derive(ToSchema)]
struct AuthResponse {
    message: String,
    user_id: Uuid,
}

#[derive(Debug, Serialize)]
#[derive(ToSchema)]
struct MeResponse {
    user_id: Uuid,
    email: String,
    full_name: Option<String>,
    auth_provider: String,
}

#[derive(Debug, Serialize)]
#[derive(ToSchema)]
struct CsrfResponse {
    csrf_token: String,
}

#[derive(Debug, Serialize)]
#[derive(ToSchema)]
struct AuthorizeResponse {
    authorized: bool,
}

#[derive(Debug, Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
    message: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct LogoutResponse {
    message: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        csrf_token,
        register,
        login,
        refresh,
        logout,
        me,
        authorize,
        google_login,
        google_callback
    ),
    components(
        schemas(
            RegisterRequest,
            LoginRequest,
            AuthorizeRequest,
            GoogleCallbackQuery,
            AuthResponse,
            MeResponse,
            CsrfResponse,
            AuthorizeResponse,
            LogoutResponse,
            ErrorResponse
        )
    ),
    tags(
        (name = "auth", description = "Authentication and authorization endpoints")
    )
)]
struct ApiDoc;

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: Option<String>,
    full_name: Option<String>,
    auth_provider: String,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: String,
    name: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "auth_service=info,tower_http=info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/tup".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(3002);

    let cfg = AppConfig {
        access_secret: env::var("JWT_ACCESS_SECRET")
            .unwrap_or_else(|_| "change-me-access-secret".to_string()),
        refresh_secret: env::var("JWT_REFRESH_SECRET")
            .unwrap_or_else(|_| "change-me-refresh-secret".to_string()),
        cookie_secure: env::var("COOKIE_SECURE")
            .unwrap_or_else(|_| "true".to_string())
            .eq_ignore_ascii_case("true"),
        cookie_domain: env::var("COOKIE_DOMAIN").ok().filter(|v| !v.trim().is_empty()),
        cookie_same_site: parse_same_site(
            &env::var("COOKIE_SAMESITE").unwrap_or_else(|_| "Lax".to_string()),
        ),
        frontend_redirect_url: env::var("FRONTEND_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        google_userinfo_url: env::var("GOOGLE_USERINFO_URL")
            .unwrap_or_else(|_| "https://www.googleapis.com/oauth2/v3/userinfo".to_string()),
    };

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    run_migrations(&db).await?;

    let oauth = build_google_oauth_client()?;
    let state = AppState { db, cfg, oauth };

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth/csrf", get(csrf_token))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh).options(preflight_ok))
        .route("/auth/logout", post(logout).options(preflight_ok))
        .route("/auth/me", get(me))
        .route("/auth/authorize", post(authorize).options(preflight_ok))
        .route("/auth/google/login", get(google_login))
        .route("/auth/google/callback", get(google_callback))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("auth-service listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_google_oauth_client() -> Result<BasicClient, Box<dyn std::error::Error>> {
    let client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or_else(|_| "set-google-client-id".to_string());
    let client_secret =
        env::var("GOOGLE_CLIENT_SECRET").unwrap_or_else(|_| "set-google-client-secret".to_string());
    let redirect_url = env::var("GOOGLE_REDIRECT_URL")
        .unwrap_or_else(|_| "http://localhost:3002/auth/google/callback".to_string());

    validate_google_oauth_config(&client_id, &client_secret, &redirect_url)?;

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?;
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?;

    Ok(BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url)?))
}

fn validate_google_oauth_config(
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let invalid_client_id = client_id.trim().is_empty()
        || matches!(client_id, "replace-me" | "set-google-client-id");
    let invalid_client_secret = client_secret.trim().is_empty()
        || matches!(client_secret, "replace-me" | "set-google-client-secret");

    if invalid_client_id || invalid_client_secret {
        return Err(IoError::new(
            ErrorKind::InvalidInput,
            "Google SSO is misconfigured: set real GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET values before starting auth-service.",
        )
        .into());
    }

    if !redirect_url.contains("/auth/google/callback") {
        return Err(IoError::new(
            ErrorKind::InvalidInput,
            "Google SSO is misconfigured: GOOGLE_REDIRECT_URL must point to /auth/google/callback.",
        )
        .into());
    }

    Ok(())
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "auth",
    responses((status = 200, description = "Service health check", body = String))
)]
async fn health() -> &'static str {
    "ok"
}

async fn preflight_ok() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[utoipa::path(
    get,
    path = "/auth/csrf",
    tag = "auth",
    responses(
        (status = 200, description = "Returns and sets CSRF token", body = CsrfResponse)
    )
)]
async fn csrf_token(State(state): State<AppState>, jar: CookieJar) -> Result<(CookieJar, Json<CsrfResponse>), AppError> {
    let csrf = generate_token();
    let jar = jar.add(build_csrf_cookie(&state.cfg, &csrf));
    Ok((jar, Json(CsrfResponse { csrf_token: csrf })))
}

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered", body = AuthResponse),
        (status = 400, description = "Validation error", body = ErrorResponse)
    )
)]
async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    validate_email_password(&payload.email, &payload.password)?;
    let hash = hash_password(&payload.password)?;
    let user_id = Uuid::new_v4();

    let created = sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, full_name, auth_provider)
        VALUES ($1, $2, $3, $4, 'local')
        ON CONFLICT(email) DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(payload.email.to_lowercase())
    .bind(hash)
    .bind(payload.full_name)
    .execute(&state.db)
    .await?;

    if created.rows_affected() == 0 {
        return Err(AppError::Validation("email is already registered".to_string()));
    }

    let (jar, uid) = issue_auth_cookies(jar, &state, user_id).await?;
    Ok((
        StatusCode::CREATED,
        jar,
        Json(AuthResponse {
            message: "registration successful".to_string(),
            user_id: uid,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse)
    )
)]
async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let user = find_user_by_email(&state.db, &payload.email).await?;
    let user = user.ok_or(AppError::Unauthorized)?;
    let hash = user.password_hash.clone().ok_or(AppError::Unauthorized)?;

    if !verify_password(&payload.password, &hash)? {
        return Err(AppError::Unauthorized);
    }

    let (jar, uid) = issue_auth_cookies(jar, &state, user.id).await?;
    Ok((
        StatusCode::OK,
        jar,
        Json(AuthResponse {
            message: "login successful".to_string(),
            user_id: uid,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "auth",
    responses(
        (status = 200, description = "Token rotated", body = AuthResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "CSRF check failed", body = ErrorResponse)
    )
)]
async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    validate_csrf(&headers, &jar)?;
    let refresh_cookie = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let claims = decode_token(&refresh_cookie, &state.cfg.refresh_secret, "refresh")?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let jti = Uuid::parse_str(&claims.jti).map_err(|_| AppError::Unauthorized)?;

    let mut tx = state.db.begin().await?;

    let token_row = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(1) FROM refresh_tokens WHERE jti = $1 AND revoked_at IS NULL AND expires_at > NOW()",
    )
    .bind(jti)
    .fetch_one(&mut *tx)
    .await?;

    if token_row == 0 {
        return Err(AppError::Unauthorized);
    }

    sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE jti = $1")
        .bind(jti)
        .execute(&mut *tx)
        .await?;

    let (jar, uid, new_jti) = build_auth_cookies(jar, &state.cfg, user_id)?;

    sqlx::query("INSERT INTO refresh_tokens (jti, user_id, expires_at) VALUES ($1, $2, $3)")
        .bind(new_jti)
        .bind(uid)
        .bind(Utc::now() + Duration::days(7))
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok((
        StatusCode::OK,
        jar,
        Json(AuthResponse {
            message: "token refreshed".to_string(),
            user_id: uid,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Logout successful", body = LogoutResponse),
        (status = 403, description = "CSRF check failed", body = ErrorResponse)
    )
)]
async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    validate_csrf(&headers, &jar)?;
    if let Some(refresh_cookie) = jar.get("refresh_token") {
        if let Ok(claims) = decode_token(refresh_cookie.value(), &state.cfg.refresh_secret, "refresh") {
            if let Ok(jti) = Uuid::parse_str(&claims.jti) {
                let _ = sqlx::query("UPDATE refresh_tokens SET revoked_at = NOW() WHERE jti = $1")
                    .bind(jti)
                    .execute(&state.db)
                    .await;
            }
        }
    }

    let jar = clear_auth_cookies(jar, &state.cfg);
    Ok((
        StatusCode::OK,
        jar,
        Json(LogoutResponse {
            message: "logged out".to_string(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current user", body = MeResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
async fn me(State(state): State<AppState>, jar: CookieJar) -> Result<Json<MeResponse>, AppError> {
    let access = jar
        .get("access_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let claims = decode_token(&access, &state.cfg.access_secret, "access")?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, password_hash, full_name, auth_provider FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    Ok(Json(MeResponse {
        user_id: user.id,
        email: user.email,
        full_name: user.full_name,
        auth_provider: user.auth_provider,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/authorize",
    tag = "auth",
    request_body = AuthorizeRequest,
    responses(
        (status = 200, description = "Authorization result", body = AuthorizeResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "CSRF check failed", body = ErrorResponse)
    )
)]
async fn authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<AuthorizeRequest>,
) -> Result<Json<AuthorizeResponse>, AppError> {
    validate_csrf(&headers, &jar)?;
    let access = jar
        .get("access_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let claims = decode_token(&access, &state.cfg.access_secret, "access")?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let granted = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(1)
        FROM user_permissions up
        JOIN permissions p ON up.permission_id = p.id
        WHERE up.user_id = $1 AND p.name = $2
        "#,
    )
    .bind(user_id)
    .bind(payload.permission)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(AuthorizeResponse {
        authorized: granted > 0,
    }))
}

#[utoipa::path(
    get,
    path = "/auth/google/login",
    tag = "auth",
    responses((status = 307, description = "Redirects to Google OAuth consent"))
)]
async fn google_login(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, Redirect) {
    let (auth_url, csrf_state) = state
        .oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    let state_cookie = Cookie::build(("oauth_state", csrf_state.secret().to_string()))
        .path("/")
        .http_only(true)
        .secure(state.cfg.cookie_secure)
        .same_site(state.cfg.cookie_same_site)
        .max_age(CookieDuration::minutes(10))
        .build();

    let jar = jar.add(with_domain(state_cookie, &state.cfg));
    (jar, Redirect::to(auth_url.as_ref()))
}

#[utoipa::path(
    get,
    path = "/auth/google/callback",
    params(
        ("code" = String, Query, description = "Google authorization code"),
        ("state" = String, Query, description = "OAuth state for CSRF protection")
    ),
    tag = "auth",
    responses(
        (status = 307, description = "Redirects back to frontend after login"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 502, description = "Google exchange/userinfo failed", body = ErrorResponse)
    )
)]
async fn google_callback(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<GoogleCallbackQuery>,
) -> Result<(CookieJar, Redirect), AppError> {
    let stored_state = jar
        .get("oauth_state")
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    if stored_state != query.state {
        return Err(AppError::Unauthorized);
    }

    let token = state
        .oauth
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .map_err(|e| AppError::OAuth(e.to_string()))?;

    let access = token.access_token().secret();

    let user_info: GoogleUserInfo = reqwest::Client::new()
        .get(&state.cfg.google_userinfo_url)
        .header(AUTHORIZATION, format!("Bearer {access}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let user_email = user_info.email.to_lowercase();
    let user_name = user_info.name;

    let user = find_user_by_email(&state.db, &user_email).await?;
    let user_id = if let Some(existing) = user {
        existing.id
    } else {
        let uid = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, full_name, auth_provider) VALUES ($1, $2, $3, 'google')",
        )
        .bind(uid)
        .bind(user_email)
        .bind(user_name)
        .execute(&state.db)
        .await?;
        uid
    };

    let (jar, _, refresh_jti) = build_auth_cookies(jar, &state.cfg, user_id)?;
    sqlx::query("INSERT INTO refresh_tokens (jti, user_id, expires_at) VALUES ($1, $2, $3)")
        .bind(refresh_jti)
        .bind(user_id)
        .bind(Utc::now() + Duration::days(7))
        .execute(&state.db)
        .await?;

    let jar = jar.remove(Cookie::from("oauth_state"));
    Ok((jar, Redirect::to(&state.cfg.frontend_redirect_url)))
}

async fn issue_auth_cookies(
    jar: CookieJar,
    state: &AppState,
    user_id: Uuid,
) -> Result<(CookieJar, Uuid), AppError> {
    let mut tx = state.db.begin().await?;
    let (jar, uid, refresh_jti) = build_auth_cookies(jar, &state.cfg, user_id)?;

    sqlx::query("INSERT INTO refresh_tokens (jti, user_id, expires_at) VALUES ($1, $2, $3)")
        .bind(refresh_jti)
        .bind(uid)
        .bind(Utc::now() + Duration::days(7))
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok((jar, uid))
}

fn build_auth_cookies(jar: CookieJar, cfg: &AppConfig, user_id: Uuid) -> Result<(CookieJar, Uuid, Uuid), AppError> {
    let now = Utc::now();
    let refresh_jti = Uuid::new_v4();

    let access_claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::minutes(15)).timestamp() as usize,
        jti: Uuid::new_v4().to_string(),
        token_type: "access".to_string(),
    };

    let refresh_claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + Duration::days(7)).timestamp() as usize,
        jti: refresh_jti.to_string(),
        token_type: "refresh".to_string(),
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(cfg.access_secret.as_bytes()),
    )?;

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(cfg.refresh_secret.as_bytes()),
    )?;

    let csrf = generate_token();

    let jar = jar
        .add(build_access_cookie(cfg, &access_token))
        .add(build_refresh_cookie(cfg, &refresh_token))
        .add(build_csrf_cookie(cfg, &csrf));

    Ok((jar, user_id, refresh_jti))
}

fn clear_auth_cookies(jar: CookieJar, cfg: &AppConfig) -> CookieJar {
    let clear = |name: &str| {
        with_domain(
            Cookie::build((name.to_string(), "".to_string()))
                .path("/")
                .http_only(true)
                .secure(cfg.cookie_secure)
                .same_site(cfg.cookie_same_site)
                .max_age(CookieDuration::seconds(0))
                .build(),
            cfg,
        )
    };

    jar.remove(clear("access_token"))
        .remove(clear("refresh_token"))
        .remove(with_domain(
            Cookie::build(("csrf_token", ""))
                .path("/")
                .http_only(false)
                .secure(cfg.cookie_secure)
                .same_site(cfg.cookie_same_site)
                .max_age(CookieDuration::seconds(0))
                .build(),
            cfg,
        ))
}

fn build_access_cookie(cfg: &AppConfig, value: &str) -> Cookie<'static> {
    with_domain(
        Cookie::build(("access_token", value.to_string()))
            .path("/")
            .http_only(true)
            .secure(cfg.cookie_secure)
            .same_site(cfg.cookie_same_site)
            .max_age(CookieDuration::minutes(15))
            .build(),
        cfg,
    )
}

fn build_refresh_cookie(cfg: &AppConfig, value: &str) -> Cookie<'static> {
    with_domain(
        Cookie::build(("refresh_token", value.to_string()))
            .path("/")
            .http_only(true)
            .secure(cfg.cookie_secure)
            .same_site(cfg.cookie_same_site)
            .max_age(CookieDuration::days(7))
            .build(),
        cfg,
    )
}

fn build_csrf_cookie(cfg: &AppConfig, value: &str) -> Cookie<'static> {
    with_domain(
        Cookie::build(("csrf_token", value.to_string()))
            .path("/")
            .http_only(false)
            .secure(cfg.cookie_secure)
            .same_site(cfg.cookie_same_site)
            .max_age(CookieDuration::hours(12))
            .build(),
        cfg,
    )
}

fn with_domain(mut cookie: Cookie<'static>, cfg: &AppConfig) -> Cookie<'static> {
    if let Some(domain) = &cfg.cookie_domain {
        cookie.set_domain(domain.clone());
    }
    cookie
}

fn parse_same_site(input: &str) -> SameSite {
    match input.to_ascii_lowercase().as_str() {
        "strict" => SameSite::Strict,
        "none" => SameSite::None,
        _ => SameSite::Lax,
    }
}

fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex(bytes)
}

fn hex(bytes: [u8; 32]) -> String {
    const CHARS: &[u8] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for b in bytes {
        out.push(CHARS[(b >> 4) as usize] as char);
        out.push(CHARS[(b & 0x0f) as usize] as char);
    }
    out
}

fn decode_token(token: &str, secret: &str, expected_type: &str) -> Result<Claims, AppError> {
    let mut v = Validation::default();
    v.validate_exp = true;
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &v)?;
    if data.claims.token_type != expected_type {
        return Err(AppError::Unauthorized);
    }
    Ok(data.claims)
}

fn validate_csrf(headers: &HeaderMap, jar: &CookieJar) -> Result<(), AppError> {
    let header = headers
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let cookie = jar.get("csrf_token").map(|c| c.value().to_string());
    match (header, cookie) {
        (Some(h), Some(c)) if h == c => Ok(()),
        _ => Err(AppError::Forbidden),
    }
}

fn validate_email_password(email: &str, password: &str) -> Result<(), AppError> {
    if !email.contains('@') {
        return Err(AppError::Validation("email must be valid".to_string()));
    }
    if password.len() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters".to_string(),
        ));
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Validation(e.to_string()))?
        .to_string();
    Ok(hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash).map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

async fn find_user_by_email(db: &PgPool, email: &str) -> Result<Option<UserRow>, AppError> {
    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, password_hash, full_name, auth_provider FROM users WHERE email = $1",
    )
    .bind(email.to_lowercase())
    .fetch_optional(db)
    .await?;

    Ok(user)
}

async fn run_migrations(db: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(db).await
}
