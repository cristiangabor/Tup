use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub farmer_api_url: String,
    pub auth_service_url: String,
}
