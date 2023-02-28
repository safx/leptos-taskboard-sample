use std::net::SocketAddr;
use axum::{routing::{get, post}, Router, response::Sse, response::sse::Event};
use axum::error_handling::HandleError;
use axum::handler::HandlerWithoutStateExt;
use tower_http::services::{ServeFile, ServeDir};
use http::StatusCode;
use leptos::*;
use leptos_axum::*;
use std::sync::Arc;
use taskboard::*;
use std::convert::Infallible;
use futures::stream::{self, Stream, StreamExt};

#[tokio::main]
async fn main() {
    register_server_functions().unwrap();

    let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr.clone();

    let pkg_service = ServeDir::new("pkg").not_found_service(handle_file_error.into_service());
    let style_service = ServeFile::new("style.css");

    async fn handle_file_error() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "File Not Found")
    }

    async fn event_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        let board = BOARD.lock().unwrap();
        let stream = stream::iter(vec![board.clone()])
            .chain(BOARD_CHANNEL.clone())
            .map(|value| Ok(Event::default().event("update").json_data(value).unwrap()));
        Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
    }

    let app = Router::new()
                  .route("/api/events", get(event_handler))
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
