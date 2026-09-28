use leptos::prelude::*;

use crate::models::MeResponse;

fn profile_initials(current_user: &MeResponse) -> String {
    if let Some(name) = current_user.full_name.as_ref() {
        let initials: String = name
            .split_whitespace()
            .filter_map(|part| part.chars().next())
            .take(2)
            .collect();

        if !initials.is_empty() {
            return initials.to_ascii_uppercase();
        }
    }

    current_user
        .email
        .chars()
        .next()
        .map(|ch| ch.to_ascii_uppercase().to_string())
        .unwrap_or_else(|| "P".to_string())
}

#[component]
pub fn Layout(
    title: String,
    description: String,
    login_url: String,
    current_user: Option<MeResponse>,
    is_authenticated: bool,
    children: Children,
) -> impl IntoView {
    let navbar_user = current_user.clone();
    let navbar_initials = navbar_user
        .as_ref()
        .map(profile_initials)
        .unwrap_or_else(|| "P".to_string());
    let show_authenticated_nav = current_user.is_some() || is_authenticated;

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
                    "body{font-family:Georgia,serif;margin:0;background:#f5f1e8;color:#1f2a1f}a{color:#31523d;text-decoration:none}a:hover{text-decoration:underline}.navbar{position:fixed;top:0;left:0;right:0;z-index:1000;height:64px;display:flex;align-items:center;justify-content:flex-start;padding:0 16px;background:rgba(245,241,232,.92);backdrop-filter:blur(10px);border-bottom:1px solid rgba(49,82,61,.14);box-shadow:0 8px 24px rgba(31,42,31,.06)}.nav-actions{display:flex;align-items:center;gap:12px}.login-button,.logout-button{display:inline-flex;align-items:center;justify-content:center;padding:10px 16px;border-radius:999px;background:#31523d;color:#f5f1e8;font-weight:700;box-shadow:0 8px 20px rgba(49,82,61,.18);border:0;cursor:pointer;font:inherit}.login-button:hover,.logout-button:hover{text-decoration:none;background:#24412d}.profile-link{display:inline-flex;align-items:center;justify-content:center}.profile-avatar{display:inline-flex;align-items:center;justify-content:center;width:42px;height:42px;border-radius:999px;background:#dbe7d1;color:#1f2a1f;font-weight:700;border:1px solid rgba(49,82,61,.14)}.logout-form{margin:0}main{width:min(1280px,calc(100% - 32px));margin:0 auto;padding:96px 0 72px}.page{display:grid;gap:22px}.map-card,.card{background:#fff;border-radius:28px;box-shadow:0 18px 48px rgba(31,42,31,.08)}.map-card{padding:20px;position:relative;min-height:78vh}.map-frame{height:calc(78vh - 40px);border-radius:22px;border:1px solid rgba(49,82,61,.12);overflow:hidden}.overlay{position:absolute;top:36px;left:36px;max-width:360px;padding:18px 20px;border-radius:22px;background:rgba(245,241,232,.9);backdrop-filter:blur(8px);box-shadow:0 12px 32px rgba(31,42,31,.08)}h1{font-size:clamp(2.4rem,6vw,5rem);line-height:.94;margin:0 0 12px}h2{font-size:1.45rem;margin:0 0 8px}.eyebrow{font-size:.8rem;letter-spacing:.12em;text-transform:uppercase;color:#506b45;font-weight:700;margin-bottom:10px}p,li{font-size:1.02rem;line-height:1.6}.cards-section{display:grid;gap:18px}.cards-header{padding:0 6px}.cards-grid{display:grid;gap:18px;grid-template-columns:repeat(auto-fit,minmax(260px,1fr))}.card{padding:22px}.pills{display:flex;flex-wrap:wrap;gap:8px;margin-top:12px}.pill{display:inline-block;padding:6px 10px;border-radius:999px;background:#eef4e9;font-size:.88rem}.muted{color:#4d5c4f}.detail{display:grid;gap:20px;grid-template-columns:1.1fr .9fr}.stack{display:grid;gap:18px}.footer{padding:0 6px;font-size:.95rem;color:#516252}.profile-grid{display:grid;gap:18px;grid-template-columns:1fr 1fr}.profile-hero{display:flex;align-items:center;gap:18px}.profile-badge{display:inline-flex;align-items:center;justify-content:center;width:72px;height:72px;border-radius:999px;background:#dbe7d1;color:#1f2a1f;font-size:1.4rem;font-weight:700}.field-list{display:grid;gap:12px}.field{padding:14px 16px;border-radius:18px;background:#f8f5ef;border:1px solid rgba(49,82,61,.1)}.field-label{font-size:.78rem;letter-spacing:.1em;text-transform:uppercase;color:#506b45;font-weight:700;margin-bottom:6px}@media (max-width: 820px){main{width:min(100%,calc(100% - 24px));padding-top:88px}.map-card{min-height:66vh}.map-frame{height:calc(66vh - 32px)}.overlay{position:static;max-width:none;margin-bottom:16px}.detail,.profile-grid{grid-template-columns:1fr}}"
                </style>
            </head>
            <body>
                <nav class="navbar" aria-label="Primary">
                    {if show_authenticated_nav {
                        view! {
                            <div class="nav-actions">
                                <a class="profile-link" href="/profile" aria-label="Profile page">
                                    <span class="profile-avatar">{navbar_initials}</span>
                                </a>
                                <form class="logout-form" method="post" action="/logout">
                                    <button class="logout-button" type="submit">"Log out"</button>
                                </form>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! { <a class="login-button" href=login_url>"Login"</a> }.into_any()
                    }}
                </nav>
                <main>
                    {children()}
                </main>
            </body>
        </html>
    }
}
