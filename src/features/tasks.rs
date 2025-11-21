use leptos::prelude::*;
use crate::shared::{Chat, Task, ViewMode};
use crate::shared::utils::get_current_time;

pub fn use_tasks(
    chats: RwSignal<Vec<Chat>>,
    selected_chat: RwSignal<Option<i64>>,
    tasks: RwSignal<Vec<Task>>,
    next_task_id: RwSignal<usize>,
    view_mode: RwSignal<ViewMode>,
) -> impl Fn(String) + Clone {
    move |text: String| {
        if let Some(chat_id) = selected_chat.get() {
            let user_name = chats.with(|chats_list| {
                chats_list
                    .iter()
                    .find(|c| c.id == chat_id as i64)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            });

            let task_id = next_task_id.get();
            next_task_id.set(task_id + 1);

            tasks.update(|tasks_list| {
                tasks_list.push(Task {
                    id: task_id,
                    user_name,
                    text,
                    created_at: get_current_time(),
                    completed: false,
                });
            });

            view_mode.set(ViewMode::Tasks);
        }
    }
}

