pub mod app;

#[cfg(any(feature = "hydrate", feature = "worker-hydrate"))]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(any(feature = "hydrate", feature = "worker-hydrate"))]
#[wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    use leptos::prelude::hydrate_body;
    hydrate_body(App)
}

#[cfg(feature = "worker")]
use axum::{
    http::{Response, StatusCode},
    Extension, Router,
};

#[cfg(feature = "worker")]
use worker::{event, Context, Env, HttpRequest, Result};

#[cfg(feature = "worker")]
fn router(env: Env) -> Router {
    use crate::app::{shell, App};
    use axum::{
        routing::{get, post},
        Router,
    };
    use leptos::context::provide_context;
    use leptos::prelude::LeptosOptions;
    use leptos::server_fn::axum::register_explicit;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    register_explicit::<app::GetBoardState>();
    register_explicit::<app::AddTask>();
    register_explicit::<app::ChangeStatus>();

    let leptos_options = LeptosOptions::builder()
        .output_name("taskboard")
        .env("DEV")
        .site_root("")
        .site_pkg_dir("pkg")
        .build();
    let routes = generate_route_list(App);

    // `provide_context` is needed to be call server API functions
    provide_context(env.clone());

    Router::new()
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .route("/pkg/:name", get(file_handler))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            let env = env.clone();
            move || {
                leptos::leptos_dom::logging::console_log("leptos_routes");
                // this closure need `provide_context` as well under dehydrating mode
                provide_context(env.clone());
                shell(leptos_options.clone())
            }
        })
        .fallback(default_fallback)
        .with_state(leptos_options)
        .layer(Extension(env))
}

#[cfg(feature = "worker")]
#[event(fetch)]
pub async fn fetch(
    req: HttpRequest,
    env: Env,
    _ctx: Context,
) -> Result<Response<axum::body::Body>> {
    use tower_service::Service;
    console_error_panic_hook::set_once();
    Ok(router(env).call(req).await?)
}

#[cfg(feature = "worker")]
async fn default_fallback(uri: axum::http::Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {uri}"))
}

#[cfg(feature = "worker")]
#[worker::send]
async fn file_handler(
    axum::extract::Path(name): axum::extract::Path<String>,
    Extension(env): Extension<Env>,
) -> Response<axum::body::Body> {
    use std::ffi::OsStr;
    use std::path::Path;

    let store = env.kv("__STATIC_CONTENT").expect("__STATIC_CONTENT");
    let list = store.list().execute().await.expect("list");

    let name = Path::new(&name);
    let Some(found) = list.keys.iter().map(|key| Path::new(&key.name)).find(|p| {
        p.file_stem().map(Path::new).unwrap().file_stem().unwrap() == name.file_stem().unwrap()
            && p.extension() == name.extension()
    }) else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(format!("No file found for {:?}", &name).into())
            .expect("404 error response");
    };

    let content_type = match found
        .extension()
        .map(OsStr::to_string_lossy)
        .unwrap_or_default()
        .as_ref()
    {
        "css" => "text/css",
        "js" => "text/javascript",
        "json" => "application/json",
        "txt" => "text/plain",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    };

    let Some(content) = store
        .get(&found.to_string_lossy())
        .bytes()
        .await
        .expect("byte")
    else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Bad Request".into())
            .expect("400 error response");
    };
    Response::builder()
        .header("content-type", content_type)
        .body(content.into())
        .expect("success response")
}
