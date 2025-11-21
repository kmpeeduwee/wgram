mod auth;

use auth::*;
use leptos::prelude::*;
use wgram_ui::App;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        let session_id = RwSignal::new(None::<String>);

        view! {
            <Show
                when=move || session_id.get().is_some()
                fallback=move || view! {
                    <AuthForm on_authenticated=move |sid| session_id.set(Some(sid)) />
                }
            >
                <App />
            </Show>
        }
    })
}
