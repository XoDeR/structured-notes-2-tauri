use std::collections::BTreeSet;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::db::{new_id, now};
use crate::error::{Error, Result};
use crate::models::{PasteMode, TopicDetail, TopicInput, TopicSummary};
use crate::{media, nodes, sanitize};

const MAX_CONTENT_BYTES: usize = 2_000_000;
const MAX_TAGS: usize = 50;

pub fn ensure_exists(conn: &Connection, id: &str) -> Result<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM topics WHERE id = ?1)",
        [id],
        |r| r.get(0),
    )?;
    exists.then_some(()).ok_or(Error::NotFound("topic"))
}

pub fn list_in_node(conn: &Connection, node_id: &str) -> Result<Vec<TopicSummary>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.title, nt.is_link, t.is_favorite
         FROM topics t JOIN node_topic nt ON nt.topic_id = t.id
         WHERE nt.node_id = ?1 ORDER BY t.title COLLATE NOCASE",
    )?;
    let topics = stmt
        .query_map([node_id], |row| {
            Ok(TopicSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                is_link: row.get(2)?,
                is_favorite: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(topics)
}

/// The topic as seen from `node_id`: a link there is read-only.
pub fn detail(conn: &Connection, id: &str, node_id: Option<&str>) -> Result<Option<TopicDetail>> {
    let row = conn
        .query_row(
            "SELECT title, content, updated_at FROM topics WHERE id = ?1",
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?;
    let Some((title, content, updated_at)) = row else {
        return Ok(None);
    };

    let is_link = match node_id {
        Some(node_id) => conn
            .query_row(
                "SELECT is_link FROM node_topic WHERE node_id = ?1 AND topic_id = ?2",
                [node_id, id],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(false),
        None => false,
    };

    let mut stmt = conn.prepare("SELECT value FROM tags WHERE topic_id = ?1 ORDER BY value")?;
    let tags = stmt
        .query_map([id], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;

    Ok(Some(TopicDetail {
        id: id.to_owned(),
        title,
        content,
        is_link,
        can_edit: !is_link,
        updated_at,
        tags,
        attachments: media::attachments_of(conn, id)?,
    }))
}

pub fn create(conn: &Connection, node_id: &str, input: &TopicInput) -> Result<String> {
    let input = validated(input)?;
    nodes::ensure_exists(conn, node_id)?;

    let id = new_id();
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO topics (id, title, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, input.title, input.content, now],
    )?;
    tx.execute(
        "INSERT INTO node_topic (node_id, topic_id, is_link) VALUES (?1, ?2, 0)",
        [node_id, &id],
    )?;
    sync_tags(&tx, &id, &input.tags)?;
    tx.commit()?;
    Ok(id)
}

pub fn update(conn: &Connection, id: &str, input: &TopicInput) -> Result<()> {
    let input = validated(input)?;

    let tx = conn.unchecked_transaction()?;
    let changed = tx.execute(
        "UPDATE topics SET title = ?2, content = ?3, updated_at = ?4 WHERE id = ?1",
        params![id, input.title, input.content, now()],
    )?;
    if changed == 0 {
        return Err(Error::NotFound("topic"));
    }
    sync_tags(&tx, id, &input.tags)?;
    tx.commit()?;
    Ok(())
}

/// Deletes the topic, or only detaches it from `node_id` when it is a link there.
pub fn delete(conn: &Connection, files_dir: &Path, id: &str, node_id: &str) -> Result<()> {
    let is_link: bool = conn
        .query_row(
            "SELECT is_link FROM node_topic WHERE node_id = ?1 AND topic_id = ?2",
            [node_id, id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or(Error::NotFound("topic"))?;

    if is_link {
        conn.execute(
            "DELETE FROM node_topic WHERE node_id = ?1 AND topic_id = ?2",
            [node_id, id],
        )?;
        return Ok(());
    }

    let files = media::ids_of_topic(conn, id)?;
    conn.execute("DELETE FROM topics WHERE id = ?1", [id])?;
    media::remove_files(files_dir, &files);
    Ok(())
}

pub fn toggle_favorite(conn: &Connection, id: &str) -> Result<()> {
    let changed = conn.execute(
        "UPDATE topics SET is_favorite = 1 - is_favorite WHERE id = ?1",
        [id],
    )?;
    (changed > 0).then_some(()).ok_or(Error::NotFound("topic"))
}

pub fn known_tags(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT DISTINCT value FROM tags ORDER BY value COLLATE NOCASE")?;
    let tags = stmt
        .query_map([], |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(tags)
}

/// Pastes a cut/copied topic into another node as a move, a copy or a link.
/// Returns the id of the topic to select afterwards.
pub fn paste(
    conn: &Connection,
    files_dir: &Path,
    id: &str,
    mode: PasteMode,
    node_id: &str,
    from_node_id: Option<&str>,
) -> Result<String> {
    ensure_exists(conn, id)?;
    nodes::ensure_exists(conn, node_id)?;

    match mode {
        PasteMode::Move => {
            let from =
                from_node_id.ok_or_else(|| Error::invalid("Moving needs the source node."))?;
            move_topic(conn, id, from, node_id)?;
            Ok(id.to_owned())
        }
        PasteMode::Link => {
            conn.execute(
                "INSERT INTO node_topic (node_id, topic_id, is_link) VALUES (?1, ?2, 1)
                 ON CONFLICT DO NOTHING",
                [node_id, id],
            )?;
            Ok(id.to_owned())
        }
        PasteMode::Copy => copy_topic(conn, files_dir, id, node_id, from_node_id),
    }
}

fn move_topic(conn: &Connection, id: &str, from: &str, to: &str) -> Result<()> {
    let is_link: bool = conn
        .query_row(
            "SELECT is_link FROM node_topic WHERE node_id = ?1 AND topic_id = ?2",
            [from, id],
            |r| r.get(0),
        )
        .optional()?
        .ok_or(Error::NotFound("topic"))?;

    if from == to {
        return Ok(());
    }

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM node_topic WHERE node_id = ?1 AND topic_id = ?2",
        [from, id],
    )?;
    // If the topic is already in the target, the real placement wins over a link.
    tx.execute(
        "INSERT INTO node_topic (node_id, topic_id, is_link) VALUES (?1, ?2, ?3)
         ON CONFLICT (node_id, topic_id) DO UPDATE SET is_link = MIN(is_link, excluded.is_link)",
        params![to, id, is_link],
    )?;
    tx.commit()?;
    Ok(())
}

fn copy_topic(
    conn: &Connection,
    files_dir: &Path,
    id: &str,
    node_id: &str,
    from_node_id: Option<&str>,
) -> Result<String> {
    let (title, content): (String, String) = conn.query_row(
        "SELECT title, content FROM topics WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let title = if from_node_id == Some(node_id) {
        format!("{title} (copy)")
    } else {
        title
    };

    let copy_id = new_id();
    let now = now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO topics (id, title, content, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        params![copy_id, title, content, now],
    )?;
    tx.execute(
        "INSERT INTO node_topic (node_id, topic_id, is_link) VALUES (?1, ?2, 0)",
        [node_id, &copy_id],
    )?;
    tx.execute(
        "INSERT INTO tags (topic_id, value) SELECT ?1, value FROM tags WHERE topic_id = ?2",
        [&copy_id, id],
    )?;

    // Copied files get new ids, so the urls embedded in the content have to follow them.
    let mut content = content;
    for (old, new) in media::copy_all(&tx, files_dir, id, &copy_id)? {
        content = content.replace(&format!("/media/{old}"), &format!("/media/{new}"));
    }
    tx.execute(
        "UPDATE topics SET content = ?2 WHERE id = ?1",
        [&copy_id, &content],
    )?;
    tx.commit()?;
    Ok(copy_id)
}

struct Valid<'a> {
    title: &'a str,
    content: String,
    tags: BTreeSet<String>,
}

fn validated(input: &TopicInput) -> Result<Valid<'_>> {
    let title = input.title.trim();
    if title.is_empty() || title.chars().count() > 255 {
        return Err(Error::invalid("The title must be 1–255 characters long."));
    }
    if input.content.len() > MAX_CONTENT_BYTES {
        return Err(Error::invalid("The content is too large."));
    }

    let tags: BTreeSet<String> = input
        .tags
        .iter()
        .map(|t| t.trim().to_owned())
        .filter(|t| !t.is_empty())
        .collect();
    if tags.len() > MAX_TAGS || tags.iter().any(|t| t.chars().count() > 100) {
        return Err(Error::invalid(
            "At most 50 tags of up to 100 characters are allowed.",
        ));
    }

    Ok(Valid {
        title,
        content: sanitize::clean(&input.content),
        tags,
    })
}

fn sync_tags(conn: &Connection, topic_id: &str, tags: &BTreeSet<String>) -> Result<()> {
    conn.execute("DELETE FROM tags WHERE topic_id = ?1", [topic_id])?;
    let mut insert = conn.prepare("INSERT INTO tags (topic_id, value) VALUES (?1, ?2)")?;
    for tag in tags {
        insert.execute([topic_id, tag])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;

    fn input(title: &str, content: &str, tags: &[&str]) -> TopicInput {
        TopicInput {
            title: title.into(),
            content: content.into(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
        }
    }

    fn root(conn: &Connection) -> String {
        nodes::list(conn).unwrap()[0].id.clone()
    }

    #[test]
    fn creates_a_topic_with_clean_content_and_unique_tags() {
        let conn = test_conn();
        let root = root(&conn);
        let id = create(
            &conn,
            &root,
            &input("T", "<p>a</p><script>x</script>", &["b", " a ", "b", ""]),
        )
        .unwrap();

        let topic = detail(&conn, &id, Some(&root)).unwrap().unwrap();
        assert_eq!(topic.content, "<p>a</p>");
        assert_eq!(topic.tags, ["a", "b"]);
        assert!(topic.can_edit);
    }

    #[test]
    fn links_are_read_only_and_only_detached_on_delete() {
        let conn = test_conn();
        let dir = Path::new(".");
        let root = root(&conn);
        let other = nodes::create(&conn, &root, "other").unwrap();
        let id = create(&conn, &root, &input("T", "", &[])).unwrap();

        paste(&conn, dir, &id, PasteMode::Link, &other, None).unwrap();
        assert!(detail(&conn, &id, Some(&other)).unwrap().unwrap().is_link);
        assert!(!detail(&conn, &id, Some(&other)).unwrap().unwrap().can_edit);

        delete(&conn, dir, &id, &other).unwrap();
        assert!(list_in_node(&conn, &other).unwrap().is_empty());
        assert_eq!(list_in_node(&conn, &root).unwrap().len(), 1);

        delete(&conn, dir, &id, &root).unwrap();
        assert!(detail(&conn, &id, None).unwrap().is_none());
    }

    #[test]
    fn move_relocates_and_copy_duplicates() {
        let conn = test_conn();
        let dir = Path::new(".");
        let root = root(&conn);
        let other = nodes::create(&conn, &root, "other").unwrap();
        let id = create(&conn, &root, &input("T", "body", &["x"])).unwrap();

        let copy = paste(&conn, dir, &id, PasteMode::Copy, &root, Some(&root)).unwrap();
        assert_ne!(copy, id);
        let titles: Vec<_> = list_in_node(&conn, &root)
            .unwrap()
            .into_iter()
            .map(|t| t.title)
            .collect();
        assert_eq!(titles, ["T", "T (copy)"]);
        assert_eq!(detail(&conn, &copy, None).unwrap().unwrap().tags, ["x"]);

        paste(&conn, dir, &id, PasteMode::Move, &other, Some(&root)).unwrap();
        assert_eq!(list_in_node(&conn, &other).unwrap()[0].id, id);
        assert_eq!(list_in_node(&conn, &root).unwrap().len(), 1);
    }

    #[test]
    fn deleting_a_node_deletes_its_topics_but_keeps_linked_ones() {
        let conn = test_conn();
        let dir = Path::new(".");
        let root = root(&conn);
        let branch = nodes::create(&conn, &root, "branch").unwrap();
        let owned = create(&conn, &branch, &input("owned", "", &[])).unwrap();
        let kept = create(&conn, &root, &input("kept", "", &[])).unwrap();
        paste(&conn, dir, &kept, PasteMode::Link, &branch, None).unwrap();

        nodes::delete(&conn, dir, &branch).unwrap();

        assert!(detail(&conn, &owned, None).unwrap().is_none());
        assert!(detail(&conn, &kept, Some(&root)).unwrap().is_some());
    }

    #[test]
    fn favorites_toggle() {
        let conn = test_conn();
        let root = root(&conn);
        let id = create(&conn, &root, &input("T", "", &[])).unwrap();

        toggle_favorite(&conn, &id).unwrap();
        assert!(list_in_node(&conn, &root).unwrap()[0].is_favorite);
        toggle_favorite(&conn, &id).unwrap();
        assert!(!list_in_node(&conn, &root).unwrap()[0].is_favorite);
    }
}
