pub mod app;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    use leptos::prelude::hydrate_body;
    hydrate_body(App)
}
