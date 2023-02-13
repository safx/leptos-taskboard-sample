use leptos::*;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Tasks(Vec<Task>);

#[derive(Clone, Eq, PartialEq, Debug)]
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

#[component]
pub fn Board(cx: Scope) -> Element {
    let (tasks, set_tasks) = create_signal(cx, Tasks::new());
    let filtered_tasks = move |status: i32| tasks.with(|tasks| tasks.filtered(status));

    let filtered_tasks1 = create_memo(cx, move |_| filtered_tasks(1));
    let filtered_tasks2 = create_memo(cx, move |_| filtered_tasks(2));
    let filtered_tasks3 = create_memo(cx, move |_| filtered_tasks(3));

    view ! { cx,
        <div>
            <section class="section">
                <div class="container">
                    <div class="columns">
                        <Column text="Open"        tasks=filtered_tasks1 />
                        <Column text="In progress" tasks=filtered_tasks2 />
                        <Column text="Completed"   tasks=filtered_tasks3 />
                    </div>
                </div>
             </section>
        </div>
    }
}

#[component]
fn Column(cx: Scope, text: &'static str, tasks: Memo<Vec<Task>>) -> Element {
    view ! { cx,
        <div class="column">
            <div class="tags has-addons">
                <span class="tag">{text}</span>
                <span class="tag is-dark">{move || tasks.get().len()}</span>
            </div>
            <div>
                <For each=move ||tasks.get() key=|e| e.name.clone()>
                    { move |cx, t: &Task| view! { cx, <Card task=t.clone() /> } }
                </For>
            </div>
        </div>
    }
}

#[component]
fn Card(cx: Scope, task: Task) -> Element {
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
                <button class="button card-footer-item">{ "◀︎" }</button>
                <button class="button card-footer-item">{ "▶︎︎" }</button>
            </footer>
          </div>
    }
}
