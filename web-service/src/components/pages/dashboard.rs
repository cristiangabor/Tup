use leptos::prelude::*;

use crate::models::MeResponse;

use super::super::layout::Layout;

#[component]
pub fn DashboardPage(current_user: Option<MeResponse>) -> impl IntoView {
    view! {
        <Layout
            title="Tup | Farmer Dashboard".to_string()
            description="Farmer dashboard entry point for registration, inventory updates, and account tasks.".to_string()
        >
            <section class="card stack">
                <div class="eyebrow">"Private app area"</div>
                <h1>"Dashboard"</h1>
                <p>"This route can still be rendered from the same Leptos service, but SEO is not the priority here. It will typically consume auth-service cookies and call protected farmer APIs."</p>
                {current_user.map(|user| view! {
                    <article class="card">
                        <h2>"Authenticated user"</h2>
                        <p>{format!("{} via {}", user.email, user.auth_provider)}</p>
                        <p class="muted">{format!("User ID: {}", user.user_id)}</p>
                        <p class="muted">{user.full_name.unwrap_or_else(|| "No full name".to_string())}</p>
                    </article>
                })}
                <div class="pills">
                    <span class="pill">"Uses auth-service cookies"</span>
                    <span class="pill">"Protected farmer actions"</span>
                    <span class="pill">"Client-side forms and map editing"</span>
                </div>
            </section>
        </Layout>
    }
}
