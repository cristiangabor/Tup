use leptos::prelude::*;

use crate::models::Farmer;

pub fn farmer_cards(farmers: Vec<Farmer>) -> impl IntoView {
    farmers
        .into_iter()
        .map(|farmer| {
            let crops = farmer
                .cultivated_crops
                .clone()
                .into_iter()
                .map(|crop| view! { <span class="pill">{crop}</span> })
                .collect_view();
            let products = farmer
                .products_for_sale
                .iter()
                .take(3)
                .map(|product| view! { <li>{product.clone()}</li> })
                .collect_view();
            view! {
                <a class="card" href=format!("/farmers/{}", farmer.slug)>
                    <div class="eyebrow">{farmer.region.clone()}</div>
                    <h2>{farmer.business_name.clone()}</h2>
                    <p class="muted">{format!("{}, {}", farmer.city, farmer.country)}</p>
                    <div class="pills">{crops}</div>
                    <ul>{products}</ul>
                </a>
            }
        })
        .collect_view()
}
