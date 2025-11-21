use leptos::prelude::*;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AuthRequest {
    phone: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct VerifyRequest {
    phone: String,
    code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AuthResponse {
    success: bool,
    message: String,
    session_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AuthStep {
    Phone,
    Code,
    Authenticated,
}

#[component]
pub fn AuthForm(on_authenticated: impl Fn(String) + 'static + Copy + Send + Sync) -> impl IntoView {
    let phone = RwSignal::new(String::new());
    let code = RwSignal::new(String::new());
    let step = RwSignal::new(AuthStep::Phone);
    let error = RwSignal::new(None::<String>);
    let loading = RwSignal::new(false);

    let request_code = move || {
        spawn_local(async move {
            loading.set(true);
            error.set(None);

            let phone_val = phone.get();

            let body = serde_json::to_string(&AuthRequest {
                phone: phone_val.clone(),
            })
            .unwrap();

            let result = Request::post("http://127.0.0.1:3000/auth/request-code")
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await;

            loading.set(false);

            match result {
                Ok(response) => {
                    if response.ok() {
                        step.set(AuthStep::Code);
                    } else {
                        let text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Error".to_string());
                        error.set(Some(text));
                    }
                }
                Err(e) => {
                    error.set(Some(format!("Network error: {}", e)));
                }
            }
        });
    };

    let verify_code = move || {
        spawn_local(async move {
            loading.set(true);
            error.set(None);

            let phone_val = phone.get();
            let code_val = code.get();

            let body = serde_json::to_string(&VerifyRequest {
                phone: phone_val.clone(),
                code: code_val.clone(),
            })
            .unwrap();

            let result = Request::post("http://127.0.0.1:3000/auth/verify-code")
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await;

            loading.set(false);

            match result {
                Ok(response) => {
                    if response.ok() {
                        let auth_response: AuthResponse = response.json().await.unwrap();
                        if let Some(session_id) = auth_response.session_id {
                            step.set(AuthStep::Authenticated);
                            on_authenticated(session_id);
                        }
                    } else {
                        let text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Invalid code".to_string());
                        error.set(Some(text));
                    }
                }
                Err(e) => {
                    error.set(Some(format!("Network error: {}", e)));
                }
            }
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-100 dark:bg-gray-900">
            <div class="bg-white dark:bg-gray-800 p-8 rounded-2xl shadow-xl w-full max-w-md">
                <div class="text-center mb-8">
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">Wgram</h1>
                    <p class="text-gray-600 dark:text-gray-400">Sign in to your Telegram account</p>
                </div>

                <Show when=move || error.get().is_some()>
                    <div class="mb-4 p-4 bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 rounded-lg">
                        <p class="text-red-700 dark:text-red-200 text-sm">{move || error.get().unwrap_or_default()}</p>
                    </div>
                </Show>

                <Show
                    when=move || step.get() == AuthStep::Phone
                    fallback=move || view! {
                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                    Verification Code
                                </label>
                                <input
                                    type="text"
                                    placeholder="12345"
                                    class="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white outline-none focus:border-blue-500 transition"
                                    prop:value=code
                                    on:input=move |ev| code.set(event_target_value(&ev))
                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" && !loading.get() {
                                            verify_code();
                                        }
                                    }
                                />
                                <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">
                                    Check your SMS or email for the code
                                </p>
                            </div>

                            <button
                                class="w-full px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-semibold rounded-lg transition shadow-lg"
                                on:click=move |_| verify_code()
                                disabled=move || loading.get()
                            >
                                {move || if loading.get() { "Verifying..." } else { "Sign In" }}
                            </button>

                            <button
                                class="w-full px-6 py-3 text-blue-600 dark:text-blue-400 font-semibold"
                                on:click=move |_| step.set(AuthStep::Phone)
                            >
                                "Change phone number"
                            </button>
                        </div>
                    }
                >
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                Phone Number
                            </label>
                            <input
                                type="tel"
                                placeholder="+1234567890"
                                class="w-full px-4 py-3 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white outline-none focus:border-blue-500 transition"
                                prop:value=phone
                                on:input=move |ev| phone.set(event_target_value(&ev))
                                on:keydown=move |ev| {
                                    if ev.key() == "Enter" && !loading.get() {
                                        request_code();
                                    }
                                }
                            />
                        </div>

                        <button
                            class="w-full px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-semibold rounded-lg transition shadow-lg"
                            on:click=move |_| request_code()
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() { "Sending..." } else { "Send Code" }}
                        </button>
                    </div>
                </Show>
            </div>
        </div>
    }
}
