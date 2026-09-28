use leptos::prelude::*;
use leptos_leaflet::prelude::*;

use crate::models::Farmer;

#[component]
pub fn LeafletMap(farmers: Vec<Farmer>) -> impl IntoView {
    let markers_json = serde_json::to_string(&farmers).unwrap_or_else(|_| "[]".to_string());

    let center = farmers
        .first()
        .map(|farmer| Position::new(farmer.latitude, farmer.longitude))
        .unwrap_or_else(|| Position::new(40.7128, -74.0060));

    let marker_nodes = farmers
        .into_iter()
        .map(|farmer| {
            let popup_line = format!("{}, {}", farmer.city, farmer.country);
            view! {
                <Marker position=Position::new(farmer.latitude, farmer.longitude)>
                    <Popup>
                        <strong>{farmer.business_name}</strong>
                        <br />
                        <span>{popup_line}</span>
                    </Popup>
                </Marker>
            }
        })
        .collect_view();

    view! {
        <section class="map-card">
            <div class="overlay">
                <div class="eyebrow">"Leaflet map"</div>
                <h1>"Find nearby farmers"</h1>
                <p>"The map takes the lead. Farmer cards sit below it so the page scrolls down into the directory."</p>
            </div>
            {if !cfg!(target_arch = "wasm32") {
                view! {
                    <div id="leaflet-map" class="map-frame" aria-label="Interactive map" data-markers=markers_json></div>
                    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js" integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo=" crossorigin=""></script>
                    <script>
                        {r#"
                            (() => {
                                const mapEl = document.getElementById('leaflet-map');
                                if (!mapEl || typeof L === 'undefined') return;
                                if (mapEl.dataset.initialized === 'true') return;
                                mapEl.dataset.initialized = 'true';

                                let farmers = [];
                                try {
                                    farmers = JSON.parse(mapEl.dataset.markers || '[]');
                                } catch (_) {
                                    farmers = [];
                                }

                                const first = farmers[0];
                                const center = first
                                    ? [first.latitude, first.longitude]
                                    : [40.7128, -74.0060];

                                const map = L.map(mapEl).setView(center, 5);
                                L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
                                    attribution: '&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors'
                                }).addTo(map);

                                farmers.forEach((farmer) => {
                                    if (typeof farmer.latitude !== 'number' || typeof farmer.longitude !== 'number') return;
                                    const marker = L.marker([farmer.latitude, farmer.longitude]).addTo(map);
                                    const name = farmer.business_name || 'Farmer';
                                    const city = farmer.city || '';
                                    const country = farmer.country || '';
                                    marker.bindPopup(`<strong>${name}</strong><br/>${city}, ${country}`);
                                });
                            })();
                        "#}
                    </script>
                }
                    .into_any()
            } else {
                view! {
                    <script src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js" integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV1lvTlZBo=" crossorigin=""></script>
                    <MapContainer class="map-frame" center=center zoom=5.0 set_view=true>
                        <TileLayer
                            url="https://tile.openstreetmap.org/{z}/{x}/{y}.png"
                            attribution="&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors"
                        />
                        {marker_nodes}
                    </MapContainer>
                }
                    .into_any()
            }}
        </section>
    }
}
