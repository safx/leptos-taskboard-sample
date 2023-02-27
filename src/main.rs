use std::net::SocketAddr;
use axum::{routing::get, Router, response::Html};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    async fn root() -> Html<&'static str> {
        Html(
         r#"<!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="utf-8"/>
            </head>
            <body>
            </body>
            </html>"#)
    }

    let app = Router::new()
                  .route("/", get(root));

    println!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
