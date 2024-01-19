use std::path::Path;
use rusqlite::Connection;
use crate::Entry;

pub trait Journal {
    fn get_past_entries(&self) -> Vec<Entry>;
    fn get_today_entry(&self) -> Option<Entry>;
    fn store_new_entry(&self, entry: Entry);
}

#[derive(Clone)]
pub struct SimpleSqliteJournal {
    path: String,
}

fn get_db_conn(path_to_db: &String) -> Connection {
    Connection::open(path_to_db).expect("Could not open db.")
}

fn create_db_if_not_exists(path_to_db: &str) {
    let exists = Path::new(path_to_db).exists();
    println!("The database does{} exist (at {}).", if exists { "" } else { " not" }, path_to_db);
    if exists { return; }
    let conn = Connection::open(path_to_db).unwrap();
    conn.execute(get_table_schema(), []).unwrap();
    println!("Created the database!");
}

unsafe impl Send for SimpleSqliteJournal {}
unsafe impl Sync for SimpleSqliteJournal {}

impl Journal for SimpleSqliteJournal {
    fn get_past_entries(&self) -> Vec<Entry> {
        let conn = get_db_conn(&self.path);
        let mut res = conn.prepare(query_for_past_entries()).unwrap();
        let res = res.query_map([], |row| Ok(Entry {
            title: row.get_unwrap(0),
            time: row.get_unwrap(1),
            text: row.get_unwrap(2),
        })).unwrap();
        let mut blogs = vec!();
        for row in res {
            blogs.push(row.unwrap());
        }
        blogs
    }

    fn get_today_entry(&self) -> Option<Entry> {
        let conn = get_db_conn(&self.path);
        match conn.query_row(query_for_today_entry(), [], |row| Ok(Entry {
            title: row.get_unwrap(0),
            time: row.get_unwrap(1),
            text: row.get_unwrap(2),
        })) {
            Ok(d) => Some(d),
            Err(_) => None
        }
    }

    fn store_new_entry(&self, entry: Entry) {
        let conn = get_db_conn(&self.path);
        conn.execute(
            "INSERT INTO blog_entries(title, time, text) VALUES(?, ?, ?)",
            (entry.title, entry.time, entry.text)).unwrap();
    }
}

impl SimpleSqliteJournal {
    pub fn new(path: String) -> SimpleSqliteJournal {
        let o = SimpleSqliteJournal { path };
        create_db_if_not_exists(&o.path);
        o
    }
}

fn get_table_schema() -> &'static str {
    "CREATE TABLE IF NOT EXISTS blog_entries(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    time INTEGER NOT NULL,
    text TEXT NOT NULL
);"
}

fn query_for_today_entry() -> &'static str {
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
