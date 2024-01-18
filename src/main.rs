use std::path::Path;
use axum::{
    routing::{get, post},
    Router,
    extract::Json,
    response::{Html, Response},
};
use axum::http::StatusCode;
use handlebars::Handlebars;
use serde_json::{json, Value};
use serde::{Deserialize, Serialize, Serializer};
use rusqlite::{Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{Datelike, DateTime, Local, Month};
use num_traits::cast::FromPrimitive;

const PATH_TO_DB: &str = "db.sqlite3";

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
    entries: Vec<Entry>
}

// Returner Onsdag d. 10. januar 2024 for 1704843226.
fn time_to_text<S>(time: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
{
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

async fn get_index() -> Html<String> {
    let mut handlebar = Handlebars::new();
    let conn = Connection::open(PATH_TO_DB).unwrap();
    let mut res = conn.prepare(query_for_past_entries()).unwrap();
    let res = res.query_map([], |row| Result::Ok(Entry {
        title: row.get_unwrap(0),
        time: row.get_unwrap(1),
        text: row.get_unwrap(2)
    })).unwrap();
    let mut blogs = vec!();
    for row in res {
        blogs.push(row.unwrap());
    }
    let res = conn.query_row(query_for_todays_entry(), [], |row| Result::Ok(Entry {
        title: row.get_unwrap(0),
        time: row.get_unwrap(1),
        text: row.get_unwrap(2)
    }));
    let data = get_current_entry_if_exist(blogs, res);
    let index_file = include_str!("../website/index.html");
    handlebar.register_template_string("index", index_file).unwrap();
    // handlebar.register_template_file("index", "website/index.html").unwrap();
    Html(handlebar.render("index", &data).unwrap())
}

fn get_current_entry_if_exist(blogs: Vec<Entry>, res: Result<Entry>) -> Value {
    match res {
        Ok(entry) => json!(Reply {
            current_entry: entry,
            entries: blogs
        }),
        Err(_) => json!({
            "time": get_time_string(get_unix_timestamp()),
            "random_title": "Der var engang...",
            "random_text": "Nu skal I høre en fantastisk fortælling: Der var engang to brødre...",
            "entries": blogs
        })
    }
}

fn query_for_todays_entry() -> &'static str {
    "SELECT title, time, text
FROM blog_entries
WHERE date(\"time\", 'unixepoch', 'localtime') = date(\"now\", 'localtime')
ORDER BY \"time\" DESC
LIMIT 1;"
}

fn query_for_past_entries() -> &'static str {
    "SELECT title, max(\"time\") as \"time\", text
FROM blog_entries
WHERE date(\"now\", 'localtime') > date(\"time\", 'unixepoch', 'localtime')
GROUP BY
date(\"time\",'unixepoch')
ORDER BY
\"time\" DESC;"
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

async fn new_blog_entry(Json(entry): Json<Entry>) -> StatusCode {
    let conn = Connection::open(PATH_TO_DB).unwrap();
    conn.execute("INSERT INTO blog_entries(title, time, text) VALUES(?, ?, ?)", (entry.title, entry.time, entry.text)).unwrap();
    StatusCode::CREATED
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
    create_db_if_not_exists();
    let app = app();
    let port = 3000;
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Visit http://{}/", &addr);
    axum::serve(listener, app).await.unwrap();
}

fn app() -> Router {
    Router::new()
        .route("/", get(get_index))
        .route("/new_entry", post(new_blog_entry))
        .route("/script.js", get(get_script))
        .route("/favicon.svg", get(get_favicon))
        .route("/style.css", get(get_style))
}

fn create_db_if_not_exists() {
    let exists = Path::new(PATH_TO_DB).exists();
    println!("The database does{} exist (at {}).", if exists { "" } else {" not"}, PATH_TO_DB);
    let conn = Connection::open(PATH_TO_DB).unwrap();
    if exists { return; }
    conn.execute(table_schema(), []).unwrap();
    println!("Created the database!");
}

fn table_schema() -> &'static str {
    "CREATE TABLE IF NOT EXISTS blog_entries(
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        time INTEGER NOT NULL,
        text TEXT NOT NULL
    );"
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use super::*;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_get_root() {
        let response = app()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
