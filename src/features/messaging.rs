use leptos::prelude::*;
use crate::shared::{Chat, Message};
use crate::shared::api::WsRequest;
use crate::shared::utils::get_current_time;
use crate::features::websocket::WS_REF;

pub fn use_messaging(
    chats: RwSignal<Vec<Chat>>,
    selected_chat: RwSignal<Option<i64>>,
    next_message_id: RwSignal<i32>,
) -> (impl Fn(String) + Clone, impl Fn() -> Vec<Message> + Clone) {
    let send_message = move |text: String| {
        if let Some(chat_id) = selected_chat.get() {
            let current_time = get_current_time();
            let msg_id = next_message_id.get();
            next_message_id.set(msg_id + 1);

            chats.update(|chats_list| {
                if let Some(chat) = chats_list.iter_mut().find(|c| c.id == chat_id as i64) {
                    chat.messages.push(Message {
                        id: msg_id,
                        sender_name: "You".to_string(),
                        text: text.clone(),
                        is_outgoing: true,
                        timestamp: js_sys::Date::now() as i64,
                        is_file: false,
                        file_name: None,
                    });
                    chat.last_message = text.clone();
                    chat.time = current_time;
                }
            });

            WS_REF.with(|ws_ref| {
                if let Some(ref ws) = *ws_ref.borrow() {
                    let request = WsRequest::SendMessage {
                        chat_id,
                        text: text.clone(),
                    };
                    match serde_json::to_string(&request) {
                        Ok(json) => {
                            web_sys::console::log_1(
                                &format!("📤 Sending message to chat {}: {}", chat_id, text).into(),
                            );
                            if let Err(e) = ws.send_with_str(&json) {
                                web_sys::console::error_1(
                                    &format!("Failed to send message via WebSocket: {:?}", e)
                                        .into(),
                                );
                            }
                        }
                        Err(e) => {
                            web_sys::console::error_1(
                                &format!("Failed to serialize SendMessage request: {:?}", e).into(),
                            );
                        }
                    }
                } else {
                    web_sys::console::error_1(&"WebSocket not connected".into());
                }
            });
        }
    };

    let get_messages = move || {
        if let Some(chat_id) = selected_chat.get() {
            chats.with(|chats_list| {
                chats_list
                    .iter()
                    .find(|c| c.id == chat_id as i64)
                    .map(|c| {
                        let mut messages = c.messages.clone();
                        messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                        messages
                    })
                    .unwrap_or_default()
            })
        } else {
            vec![]
        }
    };

    (send_message, get_messages)
}

