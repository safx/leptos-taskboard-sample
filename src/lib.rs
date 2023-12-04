use leptos::*;
use leptos_meta::{Stylesheet, provide_meta_context};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[cfg(any(feature = "ssr", feature = "worker"))]
use once_cell::sync::Lazy;

#[cfg(any(feature = "ssr", feature = "worker"))]
use std::sync::Mutex;

#[cfg(any(feature = "ssr", feature = "worker"))]
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

#[cfg(any(feature = "hydrate", feature = "ssr", feature = "worker"))]
type AddTaskAction = Action<(String, String, u32), Result<(), ServerFnError>>;
#[cfg(any(feature = "hydrate", feature = "ssr", feature = "worker"))]
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
pub fn Board() -> impl IntoView {
    #[cfg(any(feature = "hydrate", feature = "ssr", feature = "worker"))]
        let filtered_tasks = {
        let create_card: AddTaskAction = create_action(|input: &(String, String, u32)| add_task(input.0.clone(), input.1.clone(), input.2));
        let move_card: ChangeStatusAction = create_action(|input: &(Uuid, i32)| change_status(input.0, input.1));

        let tasks = Resource::new(
            move || (create_card.version().get(), move_card.version().get()),
            |_| get_board_state(),
        );
        provide_context(create_card);
        provide_context(move_card);

        move |status: i32| {
            #[cfg(feature = "hydrate")]
                let default_func = || Ok(Tasks::new());

            #[cfg(any(feature = "ssr", feature = "worker"))]
                let default_func = || Ok(BOARD.lock().unwrap().clone());

            tasks
                .get()
                .unwrap_or_else(default_func)
                .map(|tasks| tasks.filtered(status))
                .expect("none error")
        }
    };

    #[cfg(feature = "csr")]
        let filtered_tasks = {
        let (tasks, set_tasks) = create_signal(Tasks::new());
        provide_context(set_tasks);
        move |status: i32| tasks.with(|tasks| tasks.filtered(status))
    };

    provide_meta_context();
    view ! {
        <>
            <Stylesheet href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css" />
            <Stylesheet href="/pkg/style.css" />
            <div class="container">
                <Control />
            </div>
            <section class="section">
                <div class="container">
                    <div class="columns">
                        <Column text="Open"        tasks=move || filtered_tasks(1) />
                        <Column text="In progress" tasks=move || filtered_tasks(2) />
                        <Column text="Completed"   tasks=move || filtered_tasks(3) />
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

    #[cfg(any(feature = "hydrate", feature = "ssr", feature = "worker"))]
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
fn Column(#[prop(into)] tasks: Signal<Vec<Task>>, text: &'static str) -> impl IntoView {
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
    #[cfg(any(feature = "hydrate", feature = "ssr", feature = "worker"))]
        let (move_dec, move_inc) = {
        let move_card = use_context::<ChangeStatusAction>().unwrap();
        let move_dec = move |_| move_card.dispatch((task.id, -1));
        let move_inc = move |_| move_card.dispatch((task.id,  1));
        (move_dec, move_inc)
    };

    #[cfg(feature = "csr")]
        let (move_dec, move_inc) = {
        let set_tasks = use_context::<WriteSignal<Tasks>>().unwrap();
        let move_dec = move |_| set_tasks.update(|v| v.change_status(task.id, -1));
        let move_inc = move |_| set_tasks.update(|v| v.change_status(task.id,  1));
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

#[cfg(feature = "worker")]
use worker::event;

#[cfg(feature = "worker")]
#[event(fetch)]
async fn main(req: worker::Request, env: worker::Env, _ctx: worker::Context) -> worker::Result<worker::Response> {
    use std::ffi::OsStr;
    use std::path::Path;
    use worker::{Router, Response};

    use futures::StreamExt;
    use leptos_server::{Payload, server_fn_by_path};
    use leptos_meta::{MetaContext, generate_head_metadata_separated};

    let _ = GetBoardState::register_explicit();
    let _ = AddTask::register_explicit();
    let _ = ChangeStatus::register_explicit();

    console_error_panic_hook::set_once();

    let router = Router::new();
    router
        .get_async("/", |_req, _ctx| async move {
            let (bundle, runtime) =
                leptos::leptos_dom::ssr::render_to_stream_with_prefix_undisposed_with_context_and_block_replacement(
                    || Board().into_view(),
                    || generate_head_metadata_separated().1.into(),
                    || (),
                    true
                );

            let meta = use_context::<MetaContext>();
            let head_parts = meta
                .as_ref()
                .map(|meta| meta.dehydrate())
                .unwrap_or_default();

            let head = format!(r#"<!doctype html>
            <html lang="ja">
              <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                {head_parts}
                <link rel="modulepreload" href="/pkg/taskboard.js">
                <link rel="preload" href="/pkg/taskboard_bg.wasm" as="fetch" type="application/wasm" crossorigin="">
                    <script type="module">
                        function idle(c) {{
                            if ("requestIdleCallback" in window) {{
                                window.requestIdleCallback(c);
                            }} else {{
                                c();
                            }}
                        }}
                        idle(() => {{
                            import('/pkg/taskboard.js')
                                .then(mod => {{
                                    mod.default('/pkg/taskboard_bg.wasm').then(() => mod.hydrate());
                                }})
                        }});
                    </script>
              </head>
              "#);
            let tail = "</body></html>";

            let mut html: String = head;
            {
                let mut shell = Box::pin(bundle);
                while let Some(fragment) = shell.next().await {
                    html += &fragment;
                }
                runtime.dispose();
            }
            html += &tail;

            let mut response = Response::from_html(html)?;
            response.headers_mut()
                .set("Content-Type", "text/html")?;
            Ok(response)
        })
        .get_async("/pkg/:name", |_req, ctx| async move {
            let name = ctx.param("name").map(Path::new).expect("name expected");
            let store = ctx.env.kv("__STATIC_CONTENT")?;
            let list = store.list().execute().await?;
            let Some(found) = list.keys.iter().map(|key| Path::new(&key.name)).find(|p| p.file_stem().map(Path::new).unwrap().file_stem().unwrap() == name.file_stem().unwrap() && p.extension() == name.extension()) else {
                return Response::error("Bad Request", 400);
            };

            let content_type = match found.extension().map(OsStr::to_string_lossy).unwrap_or_default().as_ref() {
                "css" => "text/css",
                "js" => "text/javascript",
                "json" => "application/json",
                "txt" => "text/plain",
                "wasm" => "application/wasm",
                _ => "application/octet-stream"
            };

            let Some(content) = store.get(&found.to_string_lossy()).bytes().await? else {
                return Response::error("Bad Request", 400);
            };
            let mut response = Response::from_bytes(content)?;
            let _ = response.headers_mut()
                .set("Content-Type", content_type);
            Ok(response)
        })
        .post_async("/api/:name", |mut req, ctx| async move {
            let name = ctx.param("name").expect("name expected");
            let Some(server_fn) = server_fn_by_path(name.as_str()) else {
                return Response::error("Bad Request", 400)
            };
            let body_ref = req.text().await?;
            let serialized = server_fn.call((), body_ref.as_bytes()).await.map_err(|e| worker::Error::from(e.to_string()))?;
            match serialized {
                Payload::Url(data) => {
                    let mut response = Response::from_bytes(data.into())?;
                    let _ = response.headers_mut()
                        .set("Content-Type", "application/x-www-form-urlencoded");
                    Ok(response)
                }
                _ => Response::error("Bad Request", 400)
            }
        })
        .run(req, env)
        .await
}
