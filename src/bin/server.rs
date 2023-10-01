use leptos::*;
use axum::{routing::post, Router};
use tower_http::services::{ServeFile, ServeDir};
use taskboard::*;

#[tokio::main]
async fn main() {
    let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr.clone();

    let pkg_service = ServeDir::new("./pkg");
    let style_service = ServeFile::new("style.css");

    let app = Router::new()
               .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
               .nest_service("/pkg", pkg_service)
               .nest_service("/style.css", style_service)
               .fallback(leptos_axum::render_app_to_stream(leptos_options, || view! { <Board /> }));

    println!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
       .serve(app.into_make_service())
       .await
       .unwrap();
}
