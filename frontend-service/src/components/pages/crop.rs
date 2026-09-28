use leptos::prelude::*;

use crate::models::{Farmer, MeResponse};

use super::super::{cards::farmer_cards, layout::Layout};

#[component]
pub fn CropPage(
    crop: String,
    farmers: Vec<Farmer>,
    login_url: String,
    current_user: Option<MeResponse>,
    is_authenticated: bool,
) -> impl IntoView {
    let title = format!("Țup | {} farmers", crop);
    let description = format!("Find farmers cultivating {} and explore produce available near you.", crop);
    let cards = farmer_cards(farmers);
    view! {
        <Layout title=title description=description login_url=login_url current_user=current_user is_authenticated=is_authenticated>
            <section class="card stack">
                <div class="eyebrow">"Crop search"</div>
                <h1>{format!("Farmers growing {}", crop)}</h1>
                <p>"This page is server-rendered so search engines can index crop-specific landing pages and farmer links."</p>
                <div class="cards-grid">{cards}</div>
            </section>
        </Layout>
    }
}
