use axum::{
    body::Body,
    extract::State,
    http::{Request, Uri},
    response::{IntoResponse, Response},
};
use leptos::{get_configuration, LeptosOptions, view};
use axum::Router;
use taskboard::Board;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tower::ServiceExt;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let conf = get_configuration(Some("Cargo.toml")).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(Board);

    let app = Router::new()
               .leptos_routes(&leptos_options, routes, Board)
               .fallback(file_handler)
               .with_state(leptos_options);

    println!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

pub async fn file_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> Response {
    let file_req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();

    match ServeDir::new(options.site_root.clone()).oneshot(file_req).await {
        Ok(res) => res.into_response(),
        Err(err) => {
            let handler =
                leptos_axum::render_app_to_stream(options.to_owned(), move || {
                    view! { <div>{err.to_string()}</div> }
                });
            handler(req).await.into_response()
        }
    }
}
