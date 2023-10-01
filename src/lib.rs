use leptos::*;
use leptos_meta::*;
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

    #[cfg(not(feature = "hydrate"))]
    fn change_status(&mut self, id: Uuid, delta: i32) {
        if let Some(card) = self.0.iter_mut().find(|e| e.id == id) {
            let new_status =  card.status + delta;
            if 1 <= new_status && new_status <= 3 {
                card.status = new_status
            }
        }
    }

    #[cfg(not(feature = "hydrate"))]
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

type AddTaskAction = Action<(String, String, u32), Result<(), ServerFnError>>;
type ChangeStatusAction = Action<(Uuid, i32), Result<Uuid, ServerFnError>>;

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
pub fn Board() -> impl IntoView {
    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    let filtered_tasks = {
        let create_card: AddTaskAction = create_action(|input: &(String, String, u32)| add_task(input.0.clone(), input.1.clone(), input.2));
        let move_card: ChangeStatusAction = create_action(|input: &(Uuid, i32)| change_status(input.0, input.1));

        let tasks = create_resource(
            move || (create_card.version().get(), move_card.version().get()),
            |_| get_board_state(),
        );

        #[cfg(feature = "hydrate")]
        let filtered = move |status: i32| tasks
            .get()
            .unwrap_or(Ok(Tasks::new()))
            .map(|tasks| tasks.filtered(status))
            .expect("none error");

        #[cfg(feature = "ssr")]
        let filtered = move |status: i32| tasks
            .get()
            .unwrap_or(Ok(BOARD.lock().unwrap().clone()))
            .map(|tasks| tasks.filtered(status))
            .expect("none error");

        provide_context(create_card);
        provide_context(move_card);

        filtered
    };

    provide_meta_context();
    view ! {
        <>
            <Stylesheet href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css" />
            <Stylesheet href="/style.css" />
            <div class="container">
                <Control />
            </div>
            <section class="section">
                <div class="container">
                    <div class="columns">
                        <Column text="Open"        tasks=Signal::derive(move || filtered_tasks(1)) />
                        <Column text="In progress" tasks=Signal::derive(move || filtered_tasks(2)) />
                        <Column text="Completed"   tasks=Signal::derive(move || filtered_tasks(3)) />
                    </div>
                </div>
             </section>
        </>
    }
}

#[component]
fn Control() -> impl IntoView {
    let (name, set_name) = create_signal("".to_string());
    let (assignee, set_assignee) = create_signal("🐱".to_string());
    let (mandays, set_mandays) = create_signal(0);

    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    let add_task = {
        let create_card = use_context::<AddTaskAction>().unwrap();
        move |_| {
            create_card.dispatch((name.get(), assignee.get(), mandays.get()));
        }
    };

    view! {
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
fn Column(text: &'static str, tasks: Signal<Vec<Task>>) -> impl IntoView {
    view ! {
        <div class="column">
            <div class="tags has-addons">
                <span class="tag">{text}</span>
                <span class="tag is-dark">{move || tasks.get().len()}</span>
            </div>
            <For each=move || tasks.get()
                 key=|t| t.id
                 children=move |t| view! { <Card task=t/> } />
        </div>
    }
}

#[component]
fn Card(task: Task) -> impl IntoView {
    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    let (move_dec, move_inc) = {
        let move_card = use_context::<ChangeStatusAction>().unwrap();
        let move_dec = move |_| move_card.dispatch((task.id, -1));
        let move_card = use_context::<ChangeStatusAction>().unwrap();
        let move_inc = move |_| move_card.dispatch((task.id,  1));
        (move_dec, move_inc)
    };

    view ! {
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

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    mount_to_body(|| view! { <Board /> })
}
