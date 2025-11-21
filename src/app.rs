use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::features::{use_messaging, use_tasks, use_websocket};
use crate::shared::{Chat, Task, ViewMode};
use crate::widgets::{ChatList, ChatWindow, Sidebar, TaskList};

#[component]
pub fn App() -> impl IntoView {
    let selected_chat = RwSignal::new(None::<i64>);
    let input_value = RwSignal::new(String::new());
    let search_query = RwSignal::new(String::new());
    let sidebar_width = RwSignal::new(384);
    let is_resizing = RwSignal::new(false);
    let view_mode = RwSignal::new(ViewMode::Chats);
    let show_archived = RwSignal::new(false);

    let messages_end = NodeRef::<leptos::html::Div>::new();

    let tasks = RwSignal::new(Vec::<Task>::new());
    let next_task_id = RwSignal::new(1);

    let chats = RwSignal::new(Vec::<Chat>::new());
    let ws_connected = RwSignal::new(false);
    let is_loading_messages = RwSignal::new(false);
    let next_message_id = RwSignal::new(100);
    let last_message_count = RwSignal::new(std::collections::HashMap::<i64, usize>::new());
    let last_update_time = RwSignal::new(std::collections::HashMap::<i64, f64>::new());

    use_websocket(
        chats,
        ws_connected,
        is_loading_messages,
        last_message_count,
        last_update_time,
        selected_chat,
    );

    let (send_message, get_messages) = use_messaging(chats, selected_chat, next_message_id);
    let get_messages_for_effect = get_messages.clone();

    let create_task = use_tasks(chats, selected_chat, tasks, next_task_id, view_mode);

    Effect::new(move |_| {
        get_messages_for_effect();
        if let Some(el) = messages_end.get() {
            let _ = el.scroll_into_view();
        }
    });

    let handle_mouse_move = move |e: web_sys::MouseEvent| {
        if is_resizing.get() {
            let new_width = e.client_x();
            if new_width >= 280 && new_width <= 600 {
                sidebar_width.set(new_width);
            }
        }
    };

    let handle_mouse_up = move |_: web_sys::MouseEvent| {
        is_resizing.set(false);
    };

    Effect::new(move |_| {
        let window = web_sys::window().expect("no global window");
        let document = window.document().expect("no document");

        let move_closure = Closure::wrap(Box::new(handle_mouse_move) as Box<dyn FnMut(_)>);
        let up_closure = Closure::wrap(Box::new(handle_mouse_up) as Box<dyn FnMut(_)>);

        document
            .add_event_listener_with_callback("mousemove", move_closure.as_ref().unchecked_ref())
            .unwrap();
        document
            .add_event_listener_with_callback("mouseup", up_closure.as_ref().unchecked_ref())
            .unwrap();

        move_closure.forget();
        up_closure.forget();
    });

    view! {
        <div class="flex h-screen antialiased select-none" style="background: #1f1d1d">
            <Sidebar view_mode />

            <Show
                when=move || view_mode.get() == ViewMode::Chats
                fallback=move || view! {
                    <div
                        class="flex flex-col"
                        style=move || format!("width: {}px; background: #1f1d1d", sidebar_width.get())
                    >
                        <div class="p-4" style="background: #1f1d1d">
                            <h1 class="text-white font-semibold text-xl mb-3">"Tasks"</h1>
                        </div>
                        <div class="flex-1 overflow-y-auto" style="background: #1f1d1d">
                            <TaskList tasks />
                        </div>
                    </div>
                }
            >
                <ChatList
                    chats
                    selected_chat
                    search_query
                    show_archived
                    sidebar_width
                    view_mode
                    ws_connected
                    is_loading_messages
                />
            </Show>

            <div
                class="w-1 cursor-col-resize transition-colors relative group"
                style="background: rgba(0,0,0,0.3)"
                on:mousedown=move |_| {
                    is_resizing.set(true);
                }
            >
                <div class="absolute inset-y-0 -left-1 -right-1"></div>
            </div>

            <Show
                when=move || selected_chat.get().is_some()
                fallback=|| view! {
                    <div class="flex-1 flex items-center justify-center text-white/50 text-xl" style="background: rgba(5,5,5,0.67)">
                        "Select a chat to start messaging"
                    </div>
                }
            >
                {
                    let get_messages_clone = get_messages.clone();
                    let send_message_clone = send_message.clone();
                    let create_task_clone = create_task.clone();

                    move || {
                        let chat_id = selected_chat.get().unwrap();
                        let chat_name = chats.with(|chats_list| {
                            chats_list
                                .iter()
                                .find(|c| c.id == chat_id as i64)
                                .map(|c| c.name.clone())
                                .unwrap_or_else(|| "Chat".to_string())
                        });
                        let messages = get_messages_clone();

                        let send_msg = send_message_clone.clone();
                        let create_task = create_task_clone.clone();

                        view! {
                            <ChatWindow
                                chat_id
                                chat_name
                                messages
                                input_value
                                ws_connected
                                is_loading_messages
                                messages_end
                                on_send=Callback::new(move |text| send_msg(text))
                                on_create_task=Callback::new(move |text| create_task(text))
                            />
                        }
                    }
                }
            </Show>
        </div>
    }
}
