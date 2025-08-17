use rusqlite::Connection;
use std::sync::Mutex;
use once_cell::sync::Lazy;

pub static DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(include_str!("../schema.sql")).unwrap();
    Mutex::new(conn)
});

