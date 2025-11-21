use leptos::prelude::*;
use crate::shared::Task;

#[component]
pub fn TaskList(tasks: RwSignal<Vec<Task>>) -> impl IntoView {
    view! {
        <For
            each=move || tasks.get()
            key=|task| task.id
            let:task
        >
            <div class="px-4 py-3">
                <div class="flex items-start gap-3 p-3 rounded-lg" style="background: #312f2f">
                    <input
                        type="checkbox"
                        checked=task.completed
                        class="mt-1 w-5 h-5"
                        on:change=move |_| {
                            let task_id = task.id;
                            tasks.update(|tasks_list| {
                                if let Some(t) = tasks_list.iter_mut().find(|t| t.id == task_id) {
                                    t.completed = !t.completed;
                                }
                            });
                        }
                    />
                    <div class="flex-1 min-w-0">
                        <div class="font-semibold text-sm text-white">{task.user_name.clone()}</div>
                        <div class=format!(
                            "text-white mt-1 text-xs {}",
                            if task.completed { "line-through opacity-50" } else { "" }
                        )>{task.text.clone()}</div>
                        <div class="text-xs mt-1" style="color: #767876">{task.created_at.clone()}</div>
                    </div>
                </div>
            </div>
        </For>
    }
}

