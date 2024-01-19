mod journal;

use std::time::{SystemTime, UNIX_EPOCH};
use axum::{routing::{get, post}, Router, extract::Json, response::{Html, Response}, http::StatusCode};
use axum::extract::State;
use handlebars::Handlebars;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize, Serializer};
use chrono::{Datelike, DateTime, Local, Month};
use num_traits::cast::FromPrimitive;
use journal::Journal;
use journal::SimpleSqliteJournal;

#[derive(Deserialize, Serialize)]
struct Entry {
    title: String,
    #[serde(default = "get_unix_timestamp", serialize_with = "time_to_text")]
    time: u64,
    text: String,
}

#[derive(Serialize)]
struct Reply {
    #[serde(flatten)]
    current_entry: Entry,
    entries: Vec<Entry>,
}

async fn get_index(State(journal): State<SimpleSqliteJournal>) -> Html<String> {
    let mut handlebar = Handlebars::new();
    let blogs = journal.get_past_entries();
    let res = journal.get_today_entry();
    let data = get_current_entry_if_exist(blogs, res);
    let index_file = include_str!("../website/index.html");
    handlebar.register_template_string("index", index_file).unwrap();
    // handlebar.register_template_file("index", "website/index.html").unwrap();
    Html(handlebar.render("index", &data).unwrap())
}

async fn new_blog_entry(State(journal): State<SimpleSqliteJournal>, Json(entry): Json<Entry>) -> StatusCode {
    journal.store_new_entry(entry);
    StatusCode::CREATED
}

async fn get_style() -> Response<String> {
    let style = include_str!("../website/style.css");
    get_file(style, true)
}

async fn get_script() -> Response<String> {
    let script = include_str!("../website/script.js");
    get_file(script, true)
}

async fn get_favicon() -> Response<String> {
    let favicon = include_str!("../website/favicon.svg");
    get_file(favicon, true)
}

fn get_file(body_string: &str, cached: bool) -> Response<String> {
    let mut builder = Response::builder();
    if cached {
        builder = builder.header("Cache-Control", "max-age=86400");
    }
    builder.status(StatusCode::OK).body(body_string.to_string()).unwrap()
}

#[tokio::main]
async fn main() {
    let app = app(SimpleSqliteJournal::new("db.sqlite3".to_string()));
    let port = 3000;
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Visit http://{}/", &addr);
    axum::serve(listener, app).await.unwrap();
}

fn app(journal: SimpleSqliteJournal) -> Router {
    let journal = journal;
    Router::new().route("/", get(get_index)).route("/new_entry", post(new_blog_entry)).with_state(journal).route("/script.js", get(get_script)).route("/favicon.svg", get(get_favicon)).route("/style.css", get(get_style))
}

fn get_current_entry_if_exist(blogs: Vec<Entry>, res: Option<Entry>) -> Value {
    match res {
        Some(entry) => json!(Reply {
            current_entry: entry,
            entries: blogs
        }),
        None => json!({
            "time": get_time_string(get_unix_timestamp()),
            "random_title": "Der var engang...",
            "random_text": "Nu skal I høre en fantastisk fortælling: Der var engang to brødre...",
            "entries": blogs
        })
    }
}

// Returner Onsdag d. 10. januar 2024 for 1704843226.
fn time_to_text<S>(time: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer, {
    let format = get_time_string(*time);
    serializer.serialize_str(format.as_str())
}

fn get_time_string(time: u64) -> String {
    let datetime: DateTime<Local> = DateTime::from(DateTime::from_timestamp(time as i64, 0).unwrap());
    let weekday = datetime.weekday().to_string();
    let day = datetime.day();
    let month_name = Month::from_u32(datetime.month()).unwrap().name();
    let year = datetime.year();
    let format = format!("{} d. {}. {} {}", weekday, day, month_name, year);
    format
}

fn get_unix_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use super::*;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_get_root() {
        let response = app(SimpleSqliteJournal::new("test.sqlite3".to_string())).oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_unix_to_date_string_1() {
        let date_string = get_time_string(1705615764);
        assert_eq!(date_string, "Thu d. 18. January 2024".to_string());
    }

    #[tokio::test]
    async fn test_unix_to_date_string_2() {
        let date_string = get_time_string(1705652313);
        assert_eq!(date_string, "Fri d. 19. January 2024".to_string());
    }
}
