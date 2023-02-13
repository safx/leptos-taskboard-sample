use leptos::*;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[cfg(feature = "ssr")]
use once_cell::sync::Lazy;

#[cfg(feature = "ssr")]
use std::sync::Mutex;

#[cfg(feature = "ssr")]
static BOARD: Lazy<Mutex<Tasks>> = Lazy::new(|| { Mutex::new(Tasks::new()) });

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tasks(Vec<Task>);

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct Task {
    id: Uuid,
    name: String,
    assignee: String,
    mandays: u32,
    status: i32,
}

impl Tasks {
    fn new() -> Self {
        Self(vec![
            Task::new("Task 1", "🐱", 3, 1),
            Task::new("Task 2", "🐶", 2, 1),
            Task::new("Task 3", "🐱", 1, 2),
            Task::new("Task 4", "🐹", 3, 3),
        ])
    }

    fn filtered(&self, status: i32) -> Vec<Task> {
        self.0
            .iter()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    fn change_status(&mut self, id: Uuid, delta: i32) {
        if let Some(card) = self.0.iter_mut().find(|e| e.id == id) {
            let new_status =  card.status + delta;
            if 1 <= new_status && new_status <= 3 {
                card.status = new_status
            }
        }
    }

    fn add_task(&mut self, name: &str, assignee: &str, mandays: u32) {
        self.0.push(Task::new(name, assignee, mandays, 1));
    }
}

impl Task {
    fn new(name: &str, assignee: &str, mandays: u32, status: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            assignee: assignee.to_string(),
            mandays,
            status,
        }
    }
}

#[cfg(feature = "ssr")]
pub fn register_server_functions() -> Result<(), ServerFnError> {
    GetBoardState::register()?;
    AddTask::register()?;
    ChangeStatus::register()?;
    Ok(())
}

#[server(GetBoardState, "/api")]
pub async fn get_board_state() -> Result<Tasks, ServerFnError> {
    let board = BOARD.lock().unwrap();
    Ok(board.clone())
}

#[server(AddTask, "/api")]
pub async fn add_task(name: String, assignee: String, mandays: u32) -> Result<(), ServerFnError> {
    let mut board = BOARD.lock().unwrap();
    board.add_task(&name, &assignee, mandays);
    Ok(())
}

#[server(ChangeStatus, "/api")]
pub async fn change_status(id: Uuid, delta: i32) -> Result<Uuid, ServerFnError> {
    let mut board = BOARD.lock().unwrap();
    board.change_status(id, delta);
    Ok(id)
}

#[component]
pub fn Board(cx: Scope) -> impl IntoView {
    let (tasks, set_tasks) = create_signal(cx, Tasks::new());
    provide_context(cx, set_tasks);
    let filtered_tasks = move |status: i32| tasks.with(|tasks| tasks.filtered(status));

    view ! { cx,
        <>
            <div class="container">
                <Control />
            </div>
            <section class="section">
                <div class="container">
                    <div class="columns">
                        <Column text="Open"        tasks=Signal::derive(cx, move || filtered_tasks(1)) />
                        <Column text="In progress" tasks=Signal::derive(cx, move || filtered_tasks(2)) />
                        <Column text="Completed"   tasks=Signal::derive(cx, move || filtered_tasks(3)) />
                    </div>
                </div>
             </section>
        </>
    }
}

#[component]
fn Control(cx: Scope) -> impl IntoView {
    let (name, set_name) = create_signal(cx, "".to_string());
    let (assignee, set_assignee) = create_signal(cx, "🐱".to_string());
    let (mandays, set_mandays) = create_signal(cx, 0);

    let set_tasks = use_context::<WriteSignal<Tasks>>(cx).unwrap();
    let add_task = move |_| {
        set_tasks.update(|v| v.add_task(&name.get(), &assignee.get(), mandays.get()));
    };

    view! { cx,
        <>
            <input value=name.get() on:change=move |e| set_name.update(|v| *v = event_target_value(&e)) />
            <select value=assignee.get() on:change=move |e| set_assignee.update(|v| *v = event_target_value(&e)) >
                <option value="🐱">"🐱"</option>
                <option value="🐶">"🐶"</option>
                <option value="🐹">"🐹"</option>
            </select>
            <input value=mandays.get() on:change=move |e| set_mandays.update(|v| *v = event_target_value(&e).parse::<u32>().unwrap()) />
            <button on:click=add_task>{ "Add" }</button>
        </>
    }
}

#[component]
fn Column(cx: Scope, text: &'static str, tasks: Signal<Vec<Task>>) -> impl IntoView {
    view ! { cx,
        <div class="column">
            <div class="tags has-addons">
                <span class="tag">{text}</span>
                <span class="tag is-dark">{move || tasks.get().len()}</span>
            </div>
            <For each=move || tasks.get()
                 key=|t| t.id
                 view=move |t| view! { cx, <Card task=t/> } />
        </div>
    }
}

#[component]
fn Card(cx: Scope, task: Task) -> impl IntoView {
    let set_tasks = use_context::<WriteSignal<Tasks>>(cx).unwrap();
    let move_dec = move |_| set_tasks.update(|v| v.change_status(task.id, -1));
    let move_inc = move |_| set_tasks.update(|v| v.change_status(task.id,  1));

    view ! { cx,
        <div class="card">
            <div class="card-content">
                { &task.name }
            </div>
            <footer class="card-footer">
                <div class="card-footer-item">
                    { &task.assignee }
                </div>
                <div class="card-footer-item">
                    { format!("💪 {}", &task.mandays) }
                </div>
            </footer>
            <footer class="card-footer">
                <button on:click=move_dec class="button card-footer-item">{ "◀︎" }</button>
                <button on:click=move_inc class="button card-footer-item">{ "▶︎︎" }</button>
            </footer>
          </div>
    }
}

#[cfg(feature = "csr")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "csr")]
#[wasm_bindgen]
pub fn main() {
    mount_to_body(|cx| view! { cx, <Board /> })
}
