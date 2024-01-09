use std::fs::read_to_string;
use std::path::Path;
use axum::{
    routing::{get, post},
    Router,
    extract::Json,
    response::{Html, Response},
};
use axum::http::StatusCode;
use handlebars::Handlebars;
use serde_json::json;
use serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};

const PATH_TO_DB: &str = "./db.db";

#[derive(Deserialize, Serialize, Debug)]
struct Entry {
    title: String,
    #[serde(default = "get_unix_timestamp")]
    time: u64,
    text: String,
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
    println!("{:?}", blogs);
    let res = conn.query_row(query_for_todays_entry(), [], |row| Result::Ok(Entry {
        title: row.get_unwrap(0),
        time: row.get_unwrap(1),
        text: row.get_unwrap(2)
    }));
    let data = match res {
        Ok(entry) => json!({
            "date": entry.time,
            "random_title": entry.title,
            "random_text": entry.text,
            "entry": blogs
        }),
        Err(_) => json!({
            "date": "i dag",
            "random_title": "Der var engang...",
            "random_text": "Nu skal i høre den fantastiske fortælling: Der var engang to brødre..."
        })
    };
    // let utf8lossy = String::from_utf8_lossy(include_bytes!("../website/index.html"));
    // let index_file: &str = utf8lossy.as_ref();
    // handlebar.register_template_string("index", index_file).unwrap();
    handlebar.register_template_file("index", "website/index.html").unwrap();
    Html(handlebar.render("index", &data).unwrap())
}

fn query_for_todays_entry() -> &'static str {
    "SELECT title, time, text
FROM blog_entries
WHERE date(\"time\", 'unixepoch') = date(\"now\")
ORDER BY \"time\" DESC
LIMIT 1;"
}

fn query_for_past_entries() -> &'static str {
    "SELECT title, max(\"time\") as \"time\", text
FROM blog_entries
WHERE date(\"now\") > date(\"time\", 'unixepoch')
GROUP BY
date(\"time\",'unixepoch');"
}

async fn get_style() -> Response<String> {
    get_file("website/style.css", false)
}

async fn get_script() -> Response<String> {
    get_file("website/script.js", false)
}

async fn new_blog_entry(Json(entry): Json<Entry>) -> StatusCode {
    let conn = Connection::open(PATH_TO_DB).unwrap();
    conn.execute("INSERT INTO blog_entries(title, time, text) VALUES(?, ?, ?)", (entry.title, entry.time, entry.text)).unwrap();
    StatusCode::CREATED
}

fn get_file(path_var: &str, cached: bool) -> Response<String> {
    let script = read_to_string(path_var).unwrap();
    let mut builder = Response::builder();
    if cached {
        builder = builder.header("Cache-Control", "max-age=86400");
    }
    builder.status(StatusCode::OK).body(script).unwrap()
}

#[tokio::main]
async fn main() {
    create_db_if_not_exists();
    let app = Router::new()
        .route("/", get(get_index))
        .route("/new_entry", post(new_blog_entry))
        .route("/script.js", get(get_script))
        .route("/style.css", get(get_style));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
