use std::fs::read_to_string;
use axum::{
    routing::get,
    Router,
    response::Html
};
use handlebars::Handlebars;
use serde_json::json;

async fn get_index() -> Html<String> {
    let mut handlebar = Handlebars::new();
    let data = json!({"name": "Nikolaj"});
    handlebar.register_template_file("index", "website/index.html").unwrap();
    Html(handlebar.render("index", &data).unwrap())
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(get_index));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
