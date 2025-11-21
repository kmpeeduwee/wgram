use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::shared::Message;
use crate::shared::utils::format_timestamp;

#[component]
pub fn ChatWindow(
    #[allow(unused_variables)]
    chat_id: i64,
    chat_name: String,
    messages: Vec<Message>,
    input_value: RwSignal<String>,
    ws_connected: RwSignal<bool>,
    is_loading_messages: RwSignal<bool>,
    messages_end: NodeRef<leptos::html::Div>,
    #[prop(into)] on_send: Callback<String>,
    #[prop(into)] on_create_task: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="flex-1 flex flex-col" style="background: rgba(5,5,5,0.67)">
            <div class="px-5 py-4 flex items-center justify-between" style="background: #1f1d1d">
                <div class="flex items-center gap-3">
                    <div class="w-12 h-12 rounded-full flex items-center justify-center text-white font-semibold" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%)">
                        {chat_name.chars().next().unwrap_or('?')}
                    </div>
                    <div>
                        <div class="font-semibold text-white text-xl">{chat_name.clone()}</div>
                        <div class="text-xs flex items-center gap-1">
                            <div class=format!(
                                "w-2 h-2 rounded-full {}",
                                if ws_connected.get() { "bg-[#21ff5f]" } else { "bg-rose-500" }
                            )></div>
                            <span style="color: rgba(33,255,95,0.99)">
                                {if is_loading_messages.get() {
                                    "updating..."
                                } else if ws_connected.get() {
                                    "Online"
                                } else {
                                    "offline"
                                }}
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            <div class="flex-1 overflow-y-auto p-4" style="background: rgba(5,5,5,0.67)">
                <div class="max-w-4xl mx-auto space-y-3">
                    <div class="text-center mb-4">
                        <span class="text-white/60 text-sm px-4 py-1.5 rounded-full inline-block" style="background: rgba(255,255,255,0.1)">
                            "Today, 9:30 am"
                        </span>
                    </div>
                    <For
                        each=move || messages.clone()
                        key=|msg| msg.id
                        let:msg
                    >
                        <div class=if msg.is_outgoing { "flex justify-end items-start gap-2" } else { "flex justify-start items-start gap-2" }>
                            {if !msg.is_outgoing {
                                view! {
                                    <div class="w-10 h-10 rounded-full flex items-center justify-center text-white text-sm font-semibold flex-shrink-0" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%)">
                                        {chat_name.chars().next().unwrap_or('?')}
                                    </div>
                                }.into_any()
                            } else {
                                view! {}.into_any()
                            }}
                            <div class=format!(
                                "max-w-md px-4 py-2.5 rounded-3xl {}",
                                if msg.is_outgoing {
                                    "bg-[#312f2f] text-white"
                                } else {
                                    "bg-[#312f2f] text-white"
                                }
                            )>
                                {if msg.is_file {
                                    view! {
                                        <div class="flex items-center gap-2">
                                            <div class="p-2 rounded-lg bg-slate-100 dark:bg-slate-700">
                                                <svg class="w-4 h-4 text-slate-600 dark:text-slate-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                                                </svg>
                                            </div>
                                            <div class="flex-1 min-w-0">
                                                <div class="font-medium truncate">
                                                    {msg.file_name.clone().unwrap_or_else(|| "File".to_string())}
                                                </div>
                                                {if !msg.text.is_empty() {
                                                    view! {
                                                        <div class="text-sm opacity-75 mt-1">
                                                            {msg.text.clone()}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }}
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="break-words whitespace-pre-wrap">{msg.text.clone()}</div>
                                    }.into_any()
                                }}

                                <div class="flex items-center justify-end mt-1">
                                    <div class="text-[10px]" style="color: rgba(255,255,255,0.5)">
                                        {format_timestamp(msg.timestamp)}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </For>
                    <div node_ref=messages_end></div>
                </div>
            </div>

            <div class="p-4" style="background: rgba(5,5,5,0.67)">
                <div class="max-w-4xl mx-auto flex gap-3 items-center">
                    <div class="flex-1 flex items-center gap-3 px-4 py-3 rounded-3xl" style="background: #312f2f">
                        <input
                            type="text"
                            placeholder="Message..."
                            class="flex-1 bg-transparent text-white placeholder-white/40 outline-none text-xl"
                            prop:value=input_value
                            on:input=move |ev| input_value.set(event_target_value(&ev))
                            on:keydown=move |ev| {
                                if ev.key() == "Enter" && !ev.shift_key() && !input_value.get_untracked().is_empty() {
                                    ev.prevent_default();
                                    let text = input_value.get_untracked();
                                    on_send.run(text);
                                    input_value.set(String::new());
                                }
                            }
                        />
                        <button
                            class="text-white/70 hover:text-white transition text-xl flex-shrink-0"
                            on:click=move |_| {
                                let document = web_sys::window().unwrap().document().unwrap();
                                let input = document.create_element("input").unwrap();
                                input.set_attribute("type", "file").unwrap();
                                input.set_attribute("accept", "*/*").unwrap();

                                let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                                    web_sys::console::log_1(&"File selected".into());
                                }) as Box<dyn FnMut(_)>);

                                input.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()).unwrap();
                                closure.forget();

                                if let Ok(html_input) = input.dyn_into::<web_sys::HtmlInputElement>() {
                                    html_input.click();
                                }
                            }
                            title="Attach file"
                        >
                            "📎"
                        </button>
                    </div>
                    <button
                        class="w-14 h-14 rounded-full flex items-center justify-center text-white transition text-2xl flex-shrink-0"
                        style="background: rgba(255,255,255,0.1)"
                        on:click=move |_| {
                            let text = input_value.get_untracked();
                            if !text.is_empty() {
                                on_send.run(text);
                                input_value.set(String::new());
                            }
                        }
                        title="Send message"
                    >
                        "↑"
                    </button>
                    <button
                        class="px-4 py-2 rounded-full bg-emerald-600/80 hover:bg-emerald-600 text-white font-semibold transition text-sm"
                        on:click=move |_| {
                            let text = input_value.get_untracked();
                            if !text.is_empty() {
                                on_create_task.run(text);
                                input_value.set(String::new());
                            }
                        }
                        title="Create task"
                    >
                        "Task"
                    </button>
                </div>
            </div>
        </div>
    }
}

