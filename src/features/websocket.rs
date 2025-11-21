use leptos::prelude::*;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebSocket;
use crate::shared::Chat;
use crate::shared::api::{WsRequest, WsResponse};
use crate::shared::utils::format_timestamp;

thread_local! {
    pub static WS_REF: RefCell<Option<WebSocket>> = RefCell::new(None);
}

pub fn use_websocket(
    chats: RwSignal<Vec<Chat>>,
    ws_connected: RwSignal<bool>,
    is_loading_messages: RwSignal<bool>,
    last_message_count: RwSignal<std::collections::HashMap<i64, usize>>,
    last_update_time: RwSignal<std::collections::HashMap<i64, f64>>,
    selected_chat: RwSignal<Option<i64>>,
) {
    Effect::new(move |_| {
        let ws = match WebSocket::new("ws://127.0.0.1:3000/ws") {
            Ok(socket) => socket,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to create WebSocket: {:?}", e).into());
                return;
            }
        };

        WS_REF.with(|ws_ref| {
            *ws_ref.borrow_mut() = Some(ws.clone());
        });

        let ws_clone = ws.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
            web_sys::console::log_1(&"✅ WebSocket connection opened".into());
            ws_connected.set(true);
            let request = WsRequest::GetDialogs;
            match serde_json::to_string(&request) {
                Ok(json) => {
                    web_sys::console::log_1(
                        &format!("📤 Sending GetDialogs request: {}", json).into(),
                    );
                    if let Err(e) = ws_clone.send_with_str(&json) {
                        web_sys::console::error_1(
                            &format!("Failed to send message: {:?}", e).into(),
                        );
                    }
                }
                Err(e) => {
                    web_sys::console::error_1(&format!("JSON error: {:?}", e).into());
                }
            }
        }) as Box<dyn FnMut(_)>);

        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        let onmessage_callback = Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let text: String = text.into();
                web_sys::console::log_1(&format!("📥 Received message: {}", text).into());

                match serde_json::from_str::<WsResponse>(&text) {
                    Ok(response) => match response {
                        WsResponse::Dialogs { data } => {
                            web_sys::console::log_1(
                                &format!("✅ Received {} dialogs", data.len()).into(),
                            );
                            if data.is_empty() {
                                web_sys::console::warn_1(&"⚠️ No dialogs received - check if Telegram client is authorized".into());
                            }
                            chats.set(data);
                        }
                        WsResponse::Messages { chat_id, data } => {
                            web_sys::console::log_1(
                                &format!(
                                    "✅ Received {} messages for chat {}",
                                    data.len(),
                                    chat_id
                                )
                                .into(),
                            );
                            is_loading_messages.set(false);

                            let has_new_messages = last_message_count.with_untracked(|counts| {
                                counts
                                    .get(&chat_id)
                                    .map_or(true, |&old_count| data.len() > old_count)
                            });

                            if has_new_messages {
                                web_sys::console::log_1(&"📨 New messages detected!".into());
                            }

                            chats.update(|chats_list| {
                                if let Some(chat) = chats_list.iter_mut().find(|c| c.id == chat_id)
                                {
                                    chat.messages = data.clone();
                                }
                            });

                            last_message_count.update(|counts| {
                                counts.insert(chat_id, data.len());
                            });
                            last_update_time.update(|times| {
                                times.insert(chat_id, js_sys::Date::now());
                            });
                        }
                        WsResponse::MessageSent {
                            chat_id,
                            success,
                            message,
                        } => {
                            web_sys::console::log_1(
                                &format!(
                                    "📤 Message send result for chat {}: success={}, message={}",
                                    chat_id, success, message
                                )
                                .into(),
                            );
                            if success {
                                web_sys::console::log_1(&"✅ Message sent successfully".into());
                                let request = WsRequest::GetMessages { chat_id };
                                if let Ok(json) = serde_json::to_string(&request) {
                                    WS_REF.with(|ws_ref| {
                                        if let Some(ws) = ws_ref.borrow().as_ref() {
                                            let _ = ws.send_with_str(&json);
                                        }
                                    });
                                }
                            } else {
                                web_sys::console::error_1(
                                    &format!("❌ Failed to send message: {}", message).into(),
                                );
                            }
                        }
                        WsResponse::FileSent {
                            chat_id,
                            success,
                            message,
                        } => {
                            web_sys::console::log_1(
                                &format!(
                                    "📎 File send result for chat {}: success={}, message={}",
                                    chat_id, success, message
                                )
                                .into(),
                            );
                            if success {
                                web_sys::console::log_1(&"✅ File sent successfully".into());
                                let request = WsRequest::GetMessages { chat_id };
                                if let Ok(json) = serde_json::to_string(&request) {
                                    WS_REF.with(|ws_ref| {
                                        if let Some(ws) = ws_ref.borrow().as_ref() {
                                            let _ = ws.send_with_str(&json);
                                        }
                                    });
                                }
                            } else {
                                web_sys::console::error_1(
                                    &format!("❌ Failed to send file: {}", message).into(),
                                );
                            }
                        }
                        WsResponse::NewMessage { chat_id, message } => {
                            web_sys::console::log_1(
                                &format!("📨 Received new message for chat {}", chat_id).into(),
                            );
                            chats.update(|chats_list| {
                                if let Some(chat) = chats_list.iter_mut().find(|c| c.id == chat_id)
                                {
                                    chat.messages.push(message.clone());
                                    chat.last_message = message.text.clone();
                                    chat.time = format_timestamp(message.timestamp);
                                }
                            });
                        }
                    },
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("❌ Failed to parse response: {}", e).into(),
                        );
                    }
                }
            }
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        let ws_connected_clone = ws_connected.clone();
        let selected_chat_clone = selected_chat.clone();
        let last_activity = RwSignal::new(js_sys::Date::now());
        let is_loading_clone = is_loading_messages.clone();
        let update_time_clone = last_update_time.clone();

        wasm_bindgen_futures::spawn_local(async move {
            loop {
                let time_since_activity = js_sys::Date::now() - last_activity.get();
                let interval = if time_since_activity > 60000.0 {
                    30000
                } else if time_since_activity > 30000.0 {
                    10000
                } else if time_since_activity > 10000.0 {
                    5000
                } else {
                    2000
                };

                gloo_timers::future::TimeoutFuture::new(interval).await;

                if ws_connected_clone.get() && !is_loading_clone.get() {
                    if let Some(chat_id) = selected_chat_clone.get() {
                        let should_update = update_time_clone.with_untracked(|times| {
                            times
                                .get(&chat_id)
                                .map_or(true, |&last_time| js_sys::Date::now() - last_time > 1500.0)
                        });

                        if should_update {
                            WS_REF.with(|ws_ref| {
                                if let Some(ref ws) = *ws_ref.borrow() {
                                    let request = WsRequest::GetMessages { chat_id };
                                    if let Ok(json) = serde_json::to_string(&request) {
                                        is_loading_clone.set(true);
                                        let _ = ws.send_with_str(&json);
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });

        let activity_tracker = last_activity.clone();
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let activity_callback =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                activity_tracker.set(js_sys::Date::now());
            }) as Box<dyn FnMut(_)>);

        let _ = document
            .add_event_listener_with_callback("click", activity_callback.as_ref().unchecked_ref());
        let _ = document.add_event_listener_with_callback(
            "keypress",
            activity_callback.as_ref().unchecked_ref(),
        );
        activity_callback.forget();

        let onerror_callback = Closure::wrap(Box::new(move |e: web_sys::ErrorEvent| {
            web_sys::console::error_1(&format!("❌ WebSocket error: {:?}", e).into());
        }) as Box<dyn FnMut(web_sys::ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        let onclose_callback = Closure::wrap(Box::new(move |e: web_sys::CloseEvent| {
            web_sys::console::warn_1(
                &format!(
                    "⚠️ WebSocket closed: code={}, reason={}",
                    e.code(),
                    e.reason()
                )
                .into(),
            );
        }) as Box<dyn FnMut(web_sys::CloseEvent)>);
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();
    });
}

