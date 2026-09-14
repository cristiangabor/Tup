use leptos::prelude::*;

#[component]
pub fn Layout(title: String, description: String, children: Children) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>{title}</title>
                <meta name="description" content={description} />
                <link
                    rel="stylesheet"
                    href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                    integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
                    crossorigin=""
                />
                <style>
                    "body{font-family:Georgia,serif;margin:0;background:#f5f1e8;color:#1f2a1f}main{width:min(1280px,calc(100% - 32px));margin:0 auto;padding:20px 0 72px}a{color:#31523d;text-decoration:none}a:hover{text-decoration:underline}.page{display:grid;gap:22px}.map-card,.card{background:#fff;border-radius:28px;box-shadow:0 18px 48px rgba(31,42,31,.08)}.map-card{padding:20px;position:relative;min-height:78vh}.map-frame{height:calc(78vh - 40px);border-radius:22px;border:1px solid rgba(49,82,61,.12);overflow:hidden}.overlay{position:absolute;top:36px;left:36px;max-width:360px;padding:18px 20px;border-radius:22px;background:rgba(245,241,232,.9);backdrop-filter:blur(8px);box-shadow:0 12px 32px rgba(31,42,31,.08)}h1{font-size:clamp(2.4rem,6vw,5rem);line-height:.94;margin:0 0 12px}h2{font-size:1.45rem;margin:0 0 8px}.eyebrow{font-size:.8rem;letter-spacing:.12em;text-transform:uppercase;color:#506b45;font-weight:700;margin-bottom:10px}p,li{font-size:1.02rem;line-height:1.6}.cards-section{display:grid;gap:18px}.cards-header{padding:0 6px}.cards-grid{display:grid;gap:18px;grid-template-columns:repeat(auto-fit,minmax(260px,1fr))}.card{padding:22px}.pills{display:flex;flex-wrap:wrap;gap:8px;margin-top:12px}.pill{display:inline-block;padding:6px 10px;border-radius:999px;background:#eef4e9;font-size:.88rem}.muted{color:#4d5c4f}.detail{display:grid;gap:20px;grid-template-columns:1.1fr .9fr}.stack{display:grid;gap:18px}.footer{padding:0 6px;font-size:.95rem;color:#516252}@media (max-width: 820px){main{width:min(100%,calc(100% - 24px));padding-top:12px}.map-card{min-height:66vh}.map-frame{height:calc(66vh - 32px)}.overlay{position:static;max-width:none;margin-bottom:16px}.detail{grid-template-columns:1fr}}"
                </style>
            </head>
            <body>
                <main>
                    {children()}
                </main>
            </body>
        </html>
    }
}
