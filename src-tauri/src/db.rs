use std::path::Path;

use rusqlite::functions::FunctionFlags;
use rusqlite::{Connection, params};

use crate::error::Result;

const SCHEMA_V1: &str = "
CREATE TABLE nodes (
    id         TEXT PRIMARY KEY,
    parent_id  TEXT REFERENCES nodes(id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX nodes_parent_id ON nodes(parent_id);

CREATE TABLE topics (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    content     TEXT NOT NULL DEFAULT '',
    is_favorite INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE node_topic (
    node_id  TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    topic_id TEXT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    is_link  INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (node_id, topic_id)
);
CREATE INDEX node_topic_topic_id ON node_topic(topic_id);

CREATE TABLE tags (
    topic_id TEXT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    value    TEXT NOT NULL,
    PRIMARY KEY (topic_id, value)
);
CREATE INDEX tags_value ON tags(value);

CREATE TABLE media (
    id         TEXT PRIMARY KEY,
    topic_id   TEXT NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    collection TEXT NOT NULL CHECK (collection IN ('images', 'attachments')),
    file_name  TEXT NOT NULL,
    mime_type  TEXT NOT NULL,
    size       INTEGER NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX media_topic_id ON media(topic_id);
";

/// Ordered migrations; the index + 1 is the `user_version` they bring the database to.
const MIGRATIONS: &[&str] = &[SCHEMA_V1];

pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    init(&conn)?;
    Ok(conn)
}

pub fn init(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    register_functions(conn)?;
    migrate(conn)?;
    ensure_root(conn)?;
    Ok(())
}

/// SQLite's own LIKE is only case-insensitive for ASCII, so search uses a Unicode-aware helper.
fn register_functions(conn: &Connection) -> Result<()> {
    conn.create_scalar_function(
        "ci_contains",
        2,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let haystack: String = ctx.get(0)?;
            let needle: String = ctx.get(1)?;
            Ok(haystack.to_lowercase().contains(&needle.to_lowercase()))
        },
    )?;
    Ok(())
}

fn migrate(conn: &Connection) -> Result<()> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", index as i64 + 1)?;
        tx.commit()?;
    }

    Ok(())
}

fn ensure_root(conn: &Connection) -> Result<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM nodes WHERE parent_id IS NULL)",
        [],
        |row| row.get(0),
    )?;

    if !exists {
        let now = now();
        conn.execute(
            "INSERT INTO nodes (id, parent_id, title, created_at, updated_at) VALUES (?1, NULL, 'Notes', ?2, ?2)",
            params![new_id(), now],
        )?;
    }

    Ok(())
}

pub fn new_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

#[cfg(test)]
pub fn test_conn() -> Connection {
    let conn = Connection::open_in_memory().expect("in-memory database");
    init(&conn).expect("schema");
    conn
}
