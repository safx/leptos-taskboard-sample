use std::net::SocketAddr;
use leptos::*;
use leptos_axum::*;
use std::sync::Arc;
use axum::{routing::{get, post}, Router, response::Html};
use axum::error_handling::HandleError;
use tower_http::services::{ServeFile, ServeDir};
use http::StatusCode;
use taskboard::*;

#[tokio::main]
async fn main() {
    register_server_functions().unwrap();

    let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_address.clone();

    let pkg_service = HandleError::new(ServeDir::new("./pkg"), handle_file_error);
    let style_service = HandleError::new(ServeFile::new("style.css"), handle_file_error);

    async fn handle_file_error(err: std::io::Error) -> (StatusCode, String) {
        (StatusCode::NOT_FOUND, format!("File Not Found: {}", err))
    }

    let app = Router::new()
               .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
               .nest_service("/pkg", pkg_service)
               .nest_service("/style.css", style_service)
               .fallback(leptos_axum::render_app_to_stream(leptos_options, |cx| view! { cx, <Board /> }));

    println!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
       .serve(app.into_make_service())
       .await
       .unwrap();
}
