use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, net::SocketAddr};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Debug, Deserialize)]
struct FarmerRegistrationRequest {
    business_name: String,
    farmer_name: String,
    email: String,
    phone: String,
    address_line1: String,
    city: String,
    region: String,
    postal_code: Option<String>,
    country: String,
    latitude: f64,
    longitude: f64,
    service_radius_km: Option<i32>,
    cultivated_crops: Vec<String>,
    products_for_sale: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FarmerRegistrationResponse {
    id: Uuid,
    business_name: String,
    farmer_name: String,
    email: String,
    city: String,
    region: String,
    country: String,
    cultivated_crops: Vec<String>,
    products_for_sale: Vec<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

enum AppError {
    Validation(String),
    Database(sqlx::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::Validation(message) => {
                (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": message }))).into_response()
            }
            Self::Database(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("database error: {error}") })),
            )
                .into_response(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/tup".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    ensure_schema(&db).await?;

    let state = AppState { db };
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/farmers/register", post(register_farmer))
        .with_state(state);

    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(address).await?;

    println!("listening on http://{address}");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn register_farmer(
    State(state): State<AppState>,
    Json(payload): Json<FarmerRegistrationRequest>,
) -> Result<(StatusCode, Json<FarmerRegistrationResponse>), AppError> {
    validate_registration(&payload)?;

    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO farmers (
            id,
            business_name,
            farmer_name,
            email,
            phone,
            address_line1,
            city,
            region,
            postal_code,
            country,
            latitude,
            longitude,
            service_radius_km,
            cultivated_crops,
            products_for_sale
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
        )
        "#,
    )
    .bind(id)
    .bind(&payload.business_name)
    .bind(&payload.farmer_name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.address_line1)
    .bind(&payload.city)
    .bind(&payload.region)
    .bind(&payload.postal_code)
    .bind(&payload.country)
    .bind(payload.latitude)
    .bind(payload.longitude)
    .bind(payload.service_radius_km)
    .bind(&payload.cultivated_crops)
    .bind(&payload.products_for_sale)
    .execute(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(FarmerRegistrationResponse {
            id,
            business_name: payload.business_name,
            farmer_name: payload.farmer_name,
            email: payload.email,
            city: payload.city,
            region: payload.region,
            country: payload.country,
            cultivated_crops: payload.cultivated_crops,
            products_for_sale: payload.products_for_sale,
        }),
    ))
}

fn validate_registration(payload: &FarmerRegistrationRequest) -> Result<(), AppError> {
    if payload.business_name.trim().is_empty() {
        return Err(AppError::Validation("business_name is required".to_string()));
    }
    if payload.farmer_name.trim().is_empty() {
        return Err(AppError::Validation("farmer_name is required".to_string()));
    }
    if payload.address_line1.trim().is_empty()
        || payload.city.trim().is_empty()
        || payload.region.trim().is_empty()
        || payload.country.trim().is_empty()
    {
        return Err(AppError::Validation("complete location fields are required".to_string()));
    }
    if payload.cultivated_crops.is_empty() {
        return Err(AppError::Validation("at least one cultivated crop is required".to_string()));
    }
    if payload.products_for_sale.is_empty() {
        return Err(AppError::Validation("at least one product for sale is required".to_string()));
    }
    if !(-90.0..=90.0).contains(&payload.latitude) {
        return Err(AppError::Validation("latitude must be between -90 and 90".to_string()));
    }
    if !(-180.0..=180.0).contains(&payload.longitude) {
        return Err(AppError::Validation("longitude must be between -180 and 180".to_string()));
    }

    Ok(())
}

async fn ensure_schema(db: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS farmers (
            id UUID PRIMARY KEY,
            business_name TEXT NOT NULL,
            farmer_name TEXT NOT NULL,
            email TEXT NOT NULL,
            phone TEXT NOT NULL,
            address_line1 TEXT NOT NULL,
            city TEXT NOT NULL,
            region TEXT NOT NULL,
            postal_code TEXT,
            country TEXT NOT NULL,
            latitude DOUBLE PRECISION NOT NULL,
            longitude DOUBLE PRECISION NOT NULL,
            service_radius_km INTEGER,
            cultivated_crops TEXT[] NOT NULL,
            products_for_sale TEXT[] NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(db)
    .await?;

    Ok(())
}
