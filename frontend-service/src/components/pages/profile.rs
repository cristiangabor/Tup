use leptos::prelude::*;

use crate::models::MeResponse;

use super::super::layout::Layout;

#[component]
pub fn ProfilePage(current_user: Option<MeResponse>, login_url: String, is_authenticated: bool) -> impl IntoView {
    let profile_user = current_user.clone();
    let login_url_for_layout = login_url.clone();
    let initials = profile_user
        .as_ref()
        .map(profile_initials)
        .unwrap_or_else(|| "P".to_string());

    view! {
        <Layout
            title="Tup | Profile".to_string()
            description="Your Tup profile details from the active sign-in session.".to_string()
            login_url=login_url_for_layout
            current_user=current_user
            is_authenticated=is_authenticated
        >
            <section class="card stack">
                <div class="eyebrow">"Profile"</div>
                <h1>"Your account"</h1>
                {if let Some(user) = profile_user {
                    view! {
                        <section class="profile-grid">
                            <article class="card stack">
                                <div class="profile-hero">
                                    <span class="profile-badge">{initials.clone()}</span>
                                    <div>
                                        <h2>{user.full_name.clone().unwrap_or_else(|| user.email.clone())}</h2>
                                        <p class="muted">{format!("Signed in with {}", user.auth_provider)}</p>
                                    </div>
                                </div>
                                <div class="field-list">
                                    <div class="field">
                                        <div class="field-label">"Email"</div>
                                        <div>{user.email}</div>
                                    </div>
                                    <div class="field">
                                        <div class="field-label">"Full name"</div>
                                        <div>{user.full_name.unwrap_or_else(|| "Not provided by the identity provider".to_string())}</div>
                                    </div>
                                    <div class="field">
                                        <div class="field-label">"Authentication provider"</div>
                                        <div>{user.auth_provider}</div>
                                    </div>
                                    <div class="field">
                                        <div class="field-label">"User ID"</div>
                                        <div>{user.user_id}</div>
                                    </div>
                                </div>
                            </article>
                            <aside class="card stack">
                                <h2>"Session actions"</h2>
                                <p class="muted">"Your profile is populated from the current auth-service session and the Google SSO callback flow."</p>
                                <form class="logout-form" method="post" action="/logout">
                                    <button class="logout-button" type="submit">"Log out"</button>
                                </form>
                            </aside>
                        </section>
                    }
                        .into_any()
                } else {
                    view! {
                        <article class="card stack">
                            <p class="muted">"No active session was found. Sign in with Google to populate your profile."</p>
                            <a class="login-button" href=login_url>"Login with Google"</a>
                        </article>
                    }
                        .into_any()
                }}
            </section>
        </Layout>
    }
}

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
