use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ssr")]
use std::sync::{LazyLock, Mutex};

#[cfg(feature = "ssr")]
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
                <link rel="stylesheet" href="/style.css" />
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

#[cfg(any(feature = "hydrate", feature = "ssr"))]
type AddTaskAction = Action<(String, String, u32), Result<(), ServerFnError>>;
#[cfg(any(feature = "hydrate", feature = "ssr"))]
type ChangeStatusAction = Action<(Uuid, i32), Result<Uuid, ServerFnError>>;

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
    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    let board = {
        let create_card: AddTaskAction = Action::new(|input: &(String, String, u32)| {
            add_task(input.0.clone(), input.1.clone(), input.2)
        });
        let move_card: ChangeStatusAction =
            Action::new(|input: &(Uuid, i32)| change_status(input.0, input.1));

        provide_context(create_card);
        provide_context(move_card);

        let tasks = Resource::new(
            move || (create_card.version().get(), move_card.version().get()),
            |_| get_board_state(),
        );

        view! {
           <Transition fallback=|| view! { "Loading..." }>
               {move || tasks.get().map(|tasks| match tasks {
                   Err(e) => view! { <div class="item-view">{format!("Error: {}", e)}</div> }.into_any(),
                   Ok(ts) => view! { <Board tasks={ts} /> }.into_any(),
               })}
           </Transition>
        }.into_any()
    };

    #[cfg(feature = "csr")]
    let board = {
        let (tasks, set_tasks) = signal(Tasks::new());
        provide_context(set_tasks);
        view! { <Board tasks={tasks} /> }.into_any()
    };

    view! {
       <>
         <div class="container">
           <Control />
           {board}
         </div>
       </>
    }
}

#[component]
fn Board(#[prop(into)] tasks: Signal<Tasks>) -> impl IntoView {
    let filtered =
        move |status: i32| Memo::new(move |_| tasks.with(|tasks| tasks.filtered(status)));

    view! {
        <section class="section">
            <div class="container">
                <div class="columns">
                    <Column text="Open"        tasks=filtered(1) />
                    <Column text="In progress" tasks=filtered(2) />
                    <Column text="Completed"   tasks=filtered(3) />
                </div>
            </div>
         </section>
    }
}

#[component]
fn Control() -> impl IntoView {
    let (name, set_name) = signal("".to_string());
    let (assignee, set_assignee) = signal("🐱".to_string());
    let (mandays, set_mandays) = signal(0);

    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    let add_task = {
        let create_card = use_context::<AddTaskAction>().unwrap();
        move |_| {
            create_card.dispatch((name.get(), assignee.get(), mandays.get()));
        }
    };

    #[cfg(feature = "csr")]
    let add_task = {
        let set_tasks = use_context::<WriteSignal<Tasks>>().unwrap();
        move |_| {
            set_tasks.update(|v| v.add_task(&name.get(), &assignee.get(), mandays.get()));
        }
    };

    view! {
        <>
            <input prop:value=name.get() on:change=move |e| set_name.update(|v| *v = event_target_value(&e)) />
            <select prop:value=assignee.get() on:change=move |e| set_assignee.update(|v| *v = event_target_value(&e)) >
                <option prop:value="🐱">"🐱"</option>
                <option prop:value="🐶">"🐶"</option>
                <option prop:value="🐹">"🐹"</option>
            </select>
            <input prop:value=mandays.get() on:change=move |e| set_mandays.update(|v| *v = event_target_value(&e).parse::<u32>().unwrap()) />
            <button on:click=add_task>{ "Add" }</button>
        </>
    }
}

#[component]
fn Column(#[prop(into)] tasks: Memo<Vec<Task>>, text: &'static str) -> impl IntoView {
    view! {
        <div class="column">
            <div class="tags has-addons">
                <span class="tag">{text}</span>
                <span class="tag is-dark">{move || tasks.read().len()}</span>
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
        let move_dec = move |_| {
            move_card.dispatch((task.id, -1));
        };
        let move_inc = move |_| {
            move_card.dispatch((task.id, 1));
        };
        (move_dec, move_inc)
    };

    #[cfg(feature = "csr")]
    let (move_dec, move_inc) = {
        let set_tasks = use_context::<WriteSignal<Tasks>>().unwrap();
        let move_dec = move |_| set_tasks.update(|v| v.change_status(task.id, -1));
        let move_inc = move |_| set_tasks.update(|v| v.change_status(task.id, 1));
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
