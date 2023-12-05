use leptos::*;
use leptos_meta::{Stylesheet, provide_meta_context};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[cfg(feature = "ssr")]
use once_cell::sync::Lazy;

#[cfg(any(feature = "ssr", feature = "worker"))]
use std::sync::Mutex;

#[cfg(feature = "worker")]
use std::sync::Arc;

#[cfg(feature = "ssr")]
static BOARD: Lazy<Mutex<Tasks>> = Lazy::new(|| { Mutex::new(Tasks::new()) });

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
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
    #[cfg(feature = "ssr")]
    fn new() -> Self {
        Self(vec![
            Task::new("Task 1".to_string(), "🐱".to_string(), 3, 1),
            Task::new("Task 2".to_string(), "🐶".to_string(), 2, 1),
            Task::new("Task 3".to_string(), "🐱".to_string(), 1, 2),
            Task::new("Task 4".to_string(), "🐹".to_string(), 3, 3),
        ])
    }

    fn filtered(&self, status: i32) -> Vec<Task> {
        self.0
            .iter()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    #[cfg(feature = "ssr")]
    fn change_status(&mut self, id: Uuid, delta: i32) {
        if let Some(card) = self.0.iter_mut().find(|e| e.id == id) {
            let new_status =  card.status + delta;
            if 1 <= new_status && new_status <= 3 {
                card.status = new_status
            }
        }
    }

    #[cfg(feature = "ssr")]
    fn add_task(&mut self, name: &str, assignee: &str, mandays: u32) {
        self.0.push(Task::new(name.to_string(), assignee.to_string(), mandays, 1));
    }
}

impl Task {
    #[cfg(any(feature = "ssr", feature = "worker"))]
    fn new(name: String, assignee: String, mandays: u32, status: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            assignee,
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
    #[cfg(any(feature = "ssr"))]
    {
        let board = BOARD.lock().unwrap();
        Ok(board.clone())
    }

    #[cfg(feature = "worker")]
    {
        let tasks = use_context::<Arc<Mutex<worker::Env>>>().expect("context expected").lock().unwrap().d1("DB")?
            .prepare("SELECT * FROM tasks")
            .all().await?
            .results()?;
        Ok(Tasks(tasks))
    }
}

#[server]
pub async fn add_task(name: String, assignee: String, mandays: u32) -> Result<(), ServerFnError> {
    #[cfg(any(feature = "ssr"))]
    {
        let mut board = BOARD.lock().unwrap();
        board.add_task(&name, &assignee, mandays);
        Ok(())
    }

    #[cfg(feature = "worker")]
    {
        let task = Task::new(name, assignee, mandays, 1);
        worker::console_log!("{:?}", task);
        let _result = use_context::<Arc<Mutex<worker::Env>>>().expect("context expected").lock().unwrap().d1("DB")?
        .prepare("INSERT INTO tasks (id, name, assignee, mandays, status) VALUES (?1, ?2, ?3, ?4, ?5)")
        .bind(&[task.id.to_string().into(), task.name.into(), task.assignee.into(), task.mandays.into(), task.status.into()])?
        .run().await?;
        Ok(())
    }
}

#[server]
pub async fn change_status(id: Uuid, delta: i32) -> Result<Uuid, ServerFnError> {
    #[cfg(any(feature = "ssr"))]
    {
        let mut board = BOARD.lock().unwrap();
        board.change_status(id, delta);
        Ok(id)
    }

    #[cfg(feature = "worker")]
    {
        let Some(task) = use_context::<Arc<Mutex<worker::Env>>>().expect("context expected").lock().unwrap().d1("DB")?
            .prepare("SELECT * FROM tasks where id = ?1")
            .bind(&[id.to_string().into()])?
            .first::<Task>(None).await? else {
            return Err(ServerFnError::Args(id.to_string()));
        };

        let _result = use_context::<Arc<Mutex<worker::Env>>>().expect("context expected").lock().unwrap().d1("DB")?
            .prepare("UPDATE tasks set status = ?1 where id = ?2")
            .bind(&[(task.status + delta).into(), id.to_string().into()])?
            .run().await?;
        Ok(id)
    }
}

#[component]
pub fn Board() -> impl IntoView {
    let tasks = {
        let create_card: AddTaskAction = create_action(|input: &(String, String, u32)| add_task(input.0.clone(), input.1.clone(), input.2));
        let move_card: ChangeStatusAction = create_action(|input: &(Uuid, i32)| change_status(input.0, input.1));
        provide_context(create_card);
        provide_context(move_card);

        create_resource(
            move || (create_card.version().get(), move_card.version().get()),
            |_| get_board_state(),
        )
    };

    provide_meta_context();
    view ! {
       <>
           <Stylesheet href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css" />
           <Stylesheet href="/pkg/style.css" />
           <div class="container">
               <Control />
           </div>
           <Transition fallback=|| view! { "Loading..." }>
               {move || tasks.get().map(|tasks| match tasks {
                   Err(e) => view! { <div class="item-view">{format!("Error: {}", e)}</div> }.into_any(),
                   Ok(ts) => {
                       let t1 = ts.clone();
                       let t2 = ts.clone();
                       view! {
                           <section class="section">
                               <div class="container">
                                   <div class="columns">
                                       <Column text="Open"        tasks=Signal::derive(move || t1.filtered(1)) />
                                       <Column text="In progress" tasks=Signal::derive(move || t2.filtered(2)) />
                                       <Column text="Completed"   tasks=Signal::derive(move || ts.filtered(3)) />
                                   </div>
                               </div>
                            </section>
                       }.into_any()
                   }
               })}
           </Transition>
       </>
    }
}

#[component]
fn Control() -> impl IntoView {
    let (name, set_name) = create_signal("".to_string());
    let (assignee, set_assignee) = create_signal("🐱".to_string());
    let (mandays, set_mandays) = create_signal(0);

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
    let (move_dec, move_inc) = {
        let move_card = use_context::<ChangeStatusAction>().unwrap();
        let move_dec = move |_| move_card.dispatch((task.id, -1));
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
        .get_async("/", |_req, ctx| async move {
            let (bundle, runtime) =
                leptos::leptos_dom::ssr::render_to_stream_with_prefix_undisposed_with_context_and_block_replacement(
                    || Board().into_view(),
                    || generate_head_metadata_separated().1.into(),
                    || provide_context(Arc::new(Mutex::new(ctx.env))),
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
                return Response::error("Not found", 404);
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
                return Response::error("Bad Request (unknown api)", 400)
            };
            let runtime = create_runtime();
            provide_context(Arc::new(Mutex::new(ctx.env)));
            let body_ref = req.text().await?;
            let result = server_fn.call((), body_ref.as_bytes()).await;
            runtime.dispose();
            let serialized = result.map_err(|e| worker::Error::from(e.to_string()))?;

            match serialized {
                Payload::Url(data) => {
                    let mut response = Response::from_bytes(data.into())?;
                    let _ = response.headers_mut()
                        .set("Content-Type", "application/x-www-form-urlencoded");
                    Ok(response)
                }
                _ => Response::error("Bad Request (unknown payload)", 400)
            }
        })
        .run(req, env)
        .await
}
