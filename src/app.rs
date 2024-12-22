use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "csr")]
use std::sync::Arc;

#[cfg(any(feature = "ssr", feature = "worker"))]
use std::sync::{LazyLock, Mutex};

#[cfg(any(feature = "ssr", feature = "worker"))]
static BOARD: LazyLock<Mutex<Tasks>> = LazyLock::new(|| Mutex::new(Tasks::new()));

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

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css" />
                <link rel="stylesheet" href="/pkg/style.css" />
            </head>
            <body>
               <App />
            </body>
        </html>
    }
}

impl Tasks {
    #[cfg(not(feature = "hydrate"))]
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
            let new_status = card.status + delta;
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
    #[cfg(not(feature = "hydrate"))]
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

#[cfg(feature = "csr")]
type AddTaskAction = Box<dyn Fn(String, String, u32) -> () + Send + Sync>;
#[cfg(any(feature = "hydrate", feature = "ssr", feature = "worker"))]
type AddTaskAction = ServerAction<AddTask>;

#[cfg(feature = "csr")]
type ChangeStatusAction = Arc<dyn Fn(Uuid, i32) -> () + Send + Sync>;
#[cfg(any(feature = "hydrate", feature = "ssr", feature = "worker"))]
type ChangeStatusAction = ServerAction<ChangeStatus>;

#[server]
pub async fn get_board_state() -> Result<Tasks, ServerFnError> {
    let board = BOARD.lock().unwrap();
    Ok(board.clone())
}

#[server]
pub async fn add_task(name: String, assignee: String, mandays: u32) -> Result<(), ServerFnError> {
    let mut board = BOARD.lock().unwrap();
    board.add_task(&name, &assignee, mandays);
    Ok(())
}

#[server]
pub async fn change_status(id: Uuid, delta: i32) -> Result<Uuid, ServerFnError> {
    let mut board = BOARD.lock().unwrap();
    board.change_status(id, delta);
    Ok(id)
}

#[component]
pub fn App() -> impl IntoView {
    #[cfg(any(feature = "hydrate", feature = "ssr", feature = "worker"))]
    let (board, add_task_action) = {
        let add_task_action = ServerAction::<AddTask>::new();
        let change_status_action = ServerAction::<ChangeStatus>::new();

        let tasks = Resource::new(
            move || {
                (
                    add_task_action.version().get(),
                    change_status_action.version().get(),
                )
            },
            |_| get_board_state(),
        );

        let board = view! {
           <Transition fallback=|| view! { "Loading..." }>
               {move || tasks.get().map(|tasks| match tasks {
                   Err(e) => view! { <div class="item-view">{format!("Error: {}", e)}</div> }.into_any(),
                   Ok(ts) => view! { <Board tasks={ts} change_status=change_status_action /> }.into_any(),
               })}
           </Transition>
        }.into_any();

        (board, add_task_action)
    };

    #[cfg(feature = "csr")]
    let (board, add_task_action) = {
        let (tasks, set_tasks) = signal(Tasks::new());
        let add_task_action: AddTaskAction =
            Box::new(move |name: String, assignee: String, mandays: u32| {
                set_tasks.update(|v| v.add_task(&name, &assignee, mandays));
            });
        let change_status_action: ChangeStatusAction =
            Arc::new(move |id: Uuid, delta: i32| set_tasks.update(|v| v.change_status(id, delta)));
        let board = view! { <Board tasks={tasks} change_status=change_status_action /> }.into_any();
        (board, add_task_action)
    };

    view! {
       <>
         <div class="container">
           <Control add_task={add_task_action}/>
           {board}
         </div>
       </>
    }
}

#[component]
fn Board(#[prop(into)] tasks: Signal<Tasks>, change_status: ChangeStatusAction) -> impl IntoView {
    let filtered =
        move |status: i32| Memo::new(move |_| tasks.with(|tasks| tasks.filtered(status)));

    view! {
        <section class="section">
            <div class="container">
                <div class="columns">
                    <Column text="Open"        tasks=filtered(1) change_status=change_status.clone() />
                    <Column text="In progress" tasks=filtered(2) change_status=change_status.clone() />
                    <Column text="Completed"   tasks=filtered(3) change_status=change_status.clone() />
                </div>
            </div>
         </section>
    }
}

#[component]
fn Control(add_task: AddTaskAction) -> impl IntoView {
    let name = RwSignal::new("".to_string());
    let assignee = RwSignal::new("🐱".to_string());
    let (mandays, set_mandays) = signal(0);

    #[cfg(any(feature = "hydrate", feature = "ssr", feature = "worker"))]
    let handle_add = {
        move |_| {
            add_task.dispatch(AddTask {
                name: name.get(),
                assignee: assignee.get(),
                mandays: mandays.get(),
            });
        }
    };

    #[cfg(feature = "csr")]
    let handle_add = move |_| {
        add_task(
            name.get().to_string(),
            assignee.get().to_string(),
            mandays.get(),
        );
    };

    view! {
        <>
            <input bind:value=name />
            <select bind:value=assignee>
                <option prop:value="🐱">"🐱"</option>
                <option prop:value="🐶">"🐶"</option>
                <option prop:value="🐹">"🐹"</option>
            </select>
            <input prop:value=mandays.get() on:change=move |e| set_mandays.update(|v| *v = event_target_value(&e).parse::<u32>().unwrap()) />
            <button on:click=handle_add>{ "Add" }</button>
        </>
    }
}

#[component]
fn Column(
    #[prop(into)] tasks: Memo<Vec<Task>>,
    text: &'static str,
    change_status: ChangeStatusAction,
) -> impl IntoView {
    view! {
        <div class="column">
            <div class="tags has-addons">
                <span class="tag">{text}</span>
                <span class="tag is-dark">{move || tasks.read().len()}</span>
            </div>
            <For each=move || tasks.get()
                 key=|t| t.id
                 children=move |t| view! { <Card task=t change_status=change_status.clone() /> } />
        </div>
    }
}

#[component]
fn Card(task: Task, change_status: ChangeStatusAction) -> impl IntoView {
    #[cfg(any(feature = "hydrate", feature = "ssr", feature = "worker"))]
    let (move_dec, move_inc) = {
        let move_dec = move |_| {
            change_status.dispatch(ChangeStatus {
                id: task.id,
                delta: -1,
            });
        };
        let move_inc = move |_| {
            change_status.dispatch(ChangeStatus {
                id: task.id,
                delta: 1,
            });
        };
        (move_dec, move_inc)
    };

    #[cfg(feature = "csr")]
    let (move_dec, move_inc) = {
        let change = change_status.clone();
        let move_dec = move |_| change(task.id, -1);
        let move_inc = move |_| change_status(task.id, 1);
        (move_dec, move_inc)
    };

    view! {
        <div class="card">
            <div class="card-content">
              { task.name.clone() }
            </div>
            <footer class="card-footer">
                <div class="card-footer-item">
                    { task.assignee.clone() }
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
