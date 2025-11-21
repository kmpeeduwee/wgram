use leptos::prelude::*;
use crate::shared::ViewMode;

#[component]
pub fn Sidebar(view_mode: RwSignal<ViewMode>) -> impl IntoView {
    view! {
        <div class="w-[68px] flex flex-col items-center py-6 gap-4" style="background: #1f1d1d; border-right: 7px solid black">
            <div class="mb-4">
                <svg width="44" height="44" viewBox="0 0 44 44" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <circle cx="22" cy="22" r="20" fill="#48736f"/>
                    <path d="M22 10 L22 34 M10 22 L34 22" stroke="white" stroke-width="3" stroke-linecap="round"/>
                </svg>
            </div>

            <button
                class=move || format!(
                    "w-[35px] h-[35px] rounded-lg flex items-center justify-center transition text-xl {}",
                    if view_mode.get() == ViewMode::Chats {
                        "text-[#48736f]"
                    } else {
                        "text-white/70 hover:text-white"
                    }
                )
                on:click=move |_| view_mode.set(ViewMode::Chats)
                title="Home"
            >
                "🏠"
            </button>
            <div class="text-[12px] font-bold" style=move || format!(
                "color: {}",
                if view_mode.get() == ViewMode::Chats { "#48736f" } else { "white" }
            )>
                "Home"
            </div>

            <button
                class=move || format!(
                    "w-[35px] h-[35px] rounded-lg flex items-center justify-center transition text-xl mt-4 {}",
                    if view_mode.get() == ViewMode::Tasks {
                        "text-[#48736f]"
                    } else {
                        "text-white/70 hover:text-white"
                    }
                )
                on:click=move |_| view_mode.set(ViewMode::Tasks)
                title="Tasks"
            >
                "💾"
            </button>
            <div class="text-[12px]" style=move || format!(
                "color: {}",
                if view_mode.get() == ViewMode::Tasks { "#48736f" } else { "white" }
            )>
                "Tasks"
            </div>

            <div class="flex-1"></div>
        </div>
    }
}

