use leptos::prelude::*;

use crate::models::{Farmer, MeResponse};

use super::super::{cards::farmer_cards, layout::Layout, map::LeafletMap};

#[component]
pub fn HomePage(
    farmers: Vec<Farmer>,
    login_url: String,
    current_user: Option<MeResponse>,
    is_authenticated: bool,
) -> impl IntoView {
    let cards = farmer_cards(farmers.clone());
    view! {
        <Layout
            title="Țup | Discover Local Farmers".to_string()
            description="Find local farmers, explore what they cultivate, and discover produce available near you.".to_string()
            login_url=login_url
            current_user=current_user
            is_authenticated=is_authenticated
        >
            <section class="page">
                <LeafletMap farmers=farmers.clone() />
                <section class="cards-section">
                    <div class="cards-header">
                        <div class="eyebrow">"Farmers"</div>
                        <h2>"Local farmer cards"</h2>
                        <p class="muted">"Scroll below the map to browse farmers, crops, and available produce."</p>
                    </div>
                    <div class="cards-grid">{cards}</div>
                    <p class="footer">"The map owns the first screen. The farmer list lives underneath so the page scrolls naturally."</p>
                </section>
            </section>
        </Layout>
    }
}
