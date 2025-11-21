use leptos::prelude::*;
use crate::shared::{Chat, ViewMode};
use crate::shared::api::WsRequest;
use crate::features::websocket::WS_REF;

#[component]
pub fn ChatList(
    chats: RwSignal<Vec<Chat>>,
    selected_chat: RwSignal<Option<i64>>,
    search_query: RwSignal<String>,
    show_archived: RwSignal<bool>,
    sidebar_width: RwSignal<i32>,
    view_mode: RwSignal<ViewMode>,
    ws_connected: RwSignal<bool>,
    is_loading_messages: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div
            class="flex flex-col"
            style=move || format!("width: {}px; background: #1f1d1d", sidebar_width.get())
        >
            <div class="p-4" style="background: #1f1d1d">
                <Show when=move || view_mode.get() == ViewMode::Chats>
                    <div class="mb-4">
                        <div class="relative">
                            <input
                                type="text"
                                placeholder="Search..."
                                class="w-full px-10 py-2 rounded-full text-white placeholder-white/60 outline-none text-xs"
                                style="background: rgba(84,54,57,0.48)"
                                prop:value=search_query
                                on:input=move |ev| search_query.set(event_target_value(&ev))
                            />
                            <div class="absolute left-3 top-1/2 -translate-y-1/2 text-white/60 text-lg">
                                "🔍"
                            </div>
                        </div>
                    </div>
                </Show>

                <h1 class="text-white font-semibold text-xl mb-3">
                    {move || if view_mode.get() == ViewMode::Chats { "Message" } else { "Tasks" }}
                </h1>
            </div>

            <div class="flex-1 overflow-y-auto" style="background: #1f1d1d">
                {move || {
                    let archived_count = chats.get().iter().filter(|c| c.is_archived).count();
                    if archived_count > 0 {
                        view! {
                            <div
                                class="mx-2 mb-2 p-3 rounded-lg cursor-pointer flex gap-3 transition-colors"
                                style=move || if show_archived.get() {
                                    "background: #312f2f"
                                } else {
                                    "background: #1f1d1d"
                                }
                                on:click=move |_| show_archived.update(|v| *v = !*v)
                            >
                                <div class="w-12 h-12 rounded-full flex items-center justify-center text-white text-lg font-semibold flex-shrink-0" style="background: rgba(255,255,255,0.1)">
                                    "📦"
                                </div>
                                <div class="flex-1 min-w-0">
                                    <div class="flex justify-between items-baseline mb-1">
                                        <div class="font-semibold text-white truncate text-xs">"Архив"</div>
                                    </div>
                                    <div class="text-xs truncate" style="color: #767876">
                                        {format!("{} архивных чатов", archived_count)}
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}

                <For
                    each=move || {
                        let query = search_query.get().to_lowercase();
                        let show_arch = show_archived.get();
                        chats.get().into_iter().filter(move |chat| {
                            let matches_search = query.is_empty() || chat.name.to_lowercase().contains(&query);
                            let matches_folder = if show_arch {
                                chat.is_archived
                            } else {
                                !chat.is_archived
                            };
                            matches_search && matches_folder
                        }).collect::<Vec<_>>()
                    }
                    key=|chat| chat.id
                    let:chat
                >
                    <div
                        class="mx-2 mb-2 p-3 rounded-lg cursor-pointer flex gap-3 transition-colors"
                        style=move || if selected_chat.get() == Some(chat.id as i64) {
                            "background: #312f2f"
                        } else {
                            "background: #1f1d1d"
                        }
                        on:click=move |_| {
                            let chat_id = chat.id;
                            selected_chat.set(Some(chat_id));

                            chats.update(|chats_list| {
                                if let Some(chat) = chats_list.iter_mut().find(|c| c.id == chat_id) {
                                    chat.unread_count = 0;
                                }
                            });

                            if ws_connected.get() {
                                let request = WsRequest::GetMessages { chat_id };
                                if let Ok(json) = serde_json::to_string(&request) {
                                    web_sys::console::log_1(&format!("📤 Requesting messages for chat {}", chat_id).into());
                                    is_loading_messages.set(true);
                                    WS_REF.with(|ws_ref| {
                                        if let Some(ws) = ws_ref.borrow().as_ref() {
                                            let _ = ws.send_with_str(&json);
                                        }
                                    });
                                }
                            }
                        }
                    >
                        <div class="relative flex-shrink-0">
                            <div class="w-12 h-12 rounded-full flex items-center justify-center text-white text-sm font-semibold" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%)">
                                {chat.name.chars().next().unwrap_or('?')}
                            </div>
                            <div class="absolute bottom-0 right-0 w-3.5 h-3.5 rounded-full border-2" style="background: #21ff5f; border-color: #1f1d1d"></div>
                        </div>

                        <div class="flex-1 min-w-0">
                            <div class="flex justify-between items-baseline mb-1">
                                <div class="font-semibold text-white truncate text-xs">{chat.name.clone()}</div>
                                <div class="text-xs ml-2 flex-shrink-0" style="color: #767876">{chat.time.clone()}</div>
                            </div>
                            <div class="flex justify-between items-center gap-2">
                                <div class="text-xs truncate flex-1" style="color: rgba(33,255,95,0.93)">{chat.last_message.clone()}</div>
                                {if chat.unread_count > 0 {
                                    view! {
                                        <span class="text-black text-[10px] rounded-full min-w-[20px] h-5 flex items-center justify-center px-1.5 font-semibold" style="background: #21ff5f">
                                            {chat.unread_count}
                                        </span>
                                    }.into_any()
                                } else {
                                    view! {}.into_any()
                                }}
                            </div>
                        </div>
                    </div>
                </For>
            </div>
        </div>
    }
}

