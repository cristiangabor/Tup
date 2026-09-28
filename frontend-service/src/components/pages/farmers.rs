use leptos::prelude::*;

use crate::models::{Farmer, MeResponse};

use super::super::{cards::farmer_cards, layout::Layout, map::LeafletMap};

#[component]
pub fn FarmersPage(
    farmers: Vec<Farmer>,
    login_url: String,
    current_user: Option<MeResponse>,
    is_authenticated: bool,
) -> impl IntoView {
    let farmers_for_map = farmers.clone();
    let cards = farmer_cards(farmers);
    view! {
        <Layout
            title="Tup | Local Farmers".to_string()
            description="Browse local farmers by region, crop, and produce currently available for sale.".to_string()
            login_url=login_url
            current_user=current_user
            is_authenticated=is_authenticated
        >
            <section class="page">
                <LeafletMap farmers=farmers_for_map />
                <section class="cards-section">
                    <div class="cards-header">
                        <div class="eyebrow">"Directory"</div>
                        <h2>"Nearby farmers"</h2>
                        <p class="muted">"Each card stays below the map so the interface reads like a map-first directory."</p>
                    </div>
                    <div class="cards-grid">{cards}</div>
                </section>
            </section>
        </Layout>
    }
}
