use leptos::*;

#[component]
pub fn Board(cx: Scope) -> Element {
    view ! { cx,
        <div>
            <section class="section">
                <div class="container">
                    <div class="columns">
                        <div class="column">
                            <div class="tags has-addons">
                                <span class="tag">"Open"</span>
                                <span class="tag is-dark">0</span>
                            </div>
                        </div>
                    </div>
                </div>
             </section>
        </div>
    }
}
