use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Farmer {
    pub slug: String,
    pub id: String,
    pub business_name: String,
    pub farmer_name: String,
    pub email: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub cultivated_crops: Vec<String>,
    pub products_for_sale: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MeResponse {
    pub user_id: String,
    pub email: String,
    pub full_name: Option<String>,
    pub auth_provider: String,
}
