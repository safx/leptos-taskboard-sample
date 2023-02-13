use std::net::SocketAddr;
use axum::{routing::{get, post}, Router, response::Html};
use axum::error_handling::HandleError;
use tower_http::services::{ServeFile, ServeDir};
use http::StatusCode;
use taskboard::*;

#[tokio::main]
async fn main() {
    register_server_functions().unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));


    async fn root() -> Html<&'static str> {
        Html(
         r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <link rel="stylesheet" href="/style.css">
                <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css">
            </head>
            <script type="module">import init, { main } from './pkg/taskboard.js'; init().then(main);</script>
            <body>
            </body>
            </html>"#)
    }

    let pkg_service = HandleError::new(ServeDir::new("./pkg"), handle_file_error);
    let style_service = HandleError::new(ServeFile::new("style.css"), handle_file_error);

    async fn handle_file_error(err: std::io::Error) -> (StatusCode, String) {
        (StatusCode::NOT_FOUND, format!("File Not Found: {}", err))
    }

    let app = Router::new()
               .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
               .nest_service("/pkg", pkg_service)
               .nest_service("/style.css", style_service)
               .route("/", get(root));


    println!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
       .serve(app.into_make_service())
       .await
       .unwrap();
}
