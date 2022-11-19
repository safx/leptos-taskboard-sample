use leptos::*;

#[component]
pub fn Board(cx: Scope) -> Element {
    view ! { cx,
        <div>
            <section class="section">
                <div class="container">
                    <div class="columns">
                        <Column text="Open" />
                        <Column text="In progress" />
                        <Column text="Completed" />
                    </div>
                </div>
             </section>
        </div>
    }
}

#[component]
fn Column(cx: Scope, text: &'static str) -> Element {
    view ! { cx,
        <div class="column">
            <div class="tags has-addons">
                <span class="tag">{ text }</span>
                <span class="tag is-dark">{ 0 }</span>
            </div>
        </div>
    }
}
