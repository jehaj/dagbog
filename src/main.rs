use std::fs::read_to_string;
use axum::{
    routing::get,
    Router,
    response::{Html, Response}
};
use handlebars::Handlebars;
use serde_json::json;

async fn get_index() -> Html<String> {
    let mut handlebar = Handlebars::new();
    let data = json!({"date": "i dag"});
    // let utf8lossy = String::from_utf8_lossy(include_bytes!("../website/index.html"));
    // let index_file: &str = utf8lossy.as_ref();
    // handlebar.register_template_string("index", index_file).unwrap();
    handlebar.register_template_file("index", "website/index.html").unwrap();
    Html(handlebar.render("index", &data).unwrap())
}

async fn get_style() -> Response<String> {
    let style = read_to_string("website/style.css").unwrap();
    Response::builder().status(200).header("content","text/css").body(style).unwrap()
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(get_index))
        .route("/style.css", get(get_style));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
