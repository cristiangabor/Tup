use leptos::prelude::*;

use crate::models::{Farmer, MeResponse};

use super::super::{layout::Layout, map::LeafletMap};

#[component]
pub fn FarmerDetailPage(
    farmer: Farmer,
    login_url: String,
    current_user: Option<MeResponse>,
    is_authenticated: bool,
) -> impl IntoView {
    let farmer_for_map = farmer.clone();
    let cultivated_crops = farmer
        .cultivated_crops
        .clone()
        .into_iter()
        .map(|crop| view! { <span class="pill">{crop}</span> })
        .collect_view();
    let products_for_sale = farmer
        .products_for_sale
        .clone()
        .into_iter()
        .map(|item| view! { <li>{item}</li> })
        .collect_view();
    let title = format!("Tup | {}", farmer.business_name);
    let description = format!(
        "Discover {} in {}, {} and see which crops are cultivated and what products are available for sale.",
        farmer.business_name, farmer.city, farmer.region
    );
    view! {
        <Layout title=title description=description login_url=login_url current_user=current_user is_authenticated=is_authenticated>
            <section class="detail">
                <section class="card stack">
                    <div>
                        <div class="eyebrow">{farmer.region.clone()}</div>
                        <h1>{farmer.business_name.clone()}</h1>
                        <p>{format!("Located in {}, {}.", farmer.city, farmer.region)}</p>
                    </div>
                    <article class="card">
                        <h2>"Crops cultivated"</h2>
                        <div class="pills">{cultivated_crops}</div>
                    </article>
                    <article class="card">
                        <h2>"Products for sale"</h2>
                        <ul>{products_for_sale}</ul>
                    </article>
                </section>
                <LeafletMap farmers=vec![farmer_for_map] />
            </section>
        </Layout>
    }
}
