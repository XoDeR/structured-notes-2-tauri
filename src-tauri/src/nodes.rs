use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::db::{new_id, now};
use crate::error::{Error, Result};
use crate::media;
use crate::models::TreeNode;

pub fn list(conn: &Connection) -> Result<Vec<TreeNode>> {
    let mut stmt =
        conn.prepare("SELECT id, parent_id, title FROM nodes ORDER BY title COLLATE NOCASE")?;
    let nodes = stmt
        .query_map([], |row| {
            let parent_id: Option<String> = row.get(1)?;
            Ok(TreeNode {
                id: row.get(0)?,
                can_edit: parent_id.is_some(),
                parent_id,
                title: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(nodes)
}

pub fn create(conn: &Connection, parent_id: &str, title: &str) -> Result<String> {
    let title = valid_title(title)?;
    ensure_exists(conn, parent_id)?;

    let id = new_id();
    let now = now();
    conn.execute(
        "INSERT INTO nodes (id, parent_id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, parent_id, title, now],
    )?;
    Ok(id)
}

pub fn rename(conn: &Connection, id: &str, title: &str) -> Result<()> {
    let title = valid_title(title)?;
    ensure_editable(conn, id)?;

    conn.execute(
        "UPDATE nodes SET title = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, title, now()],
    )?;
    Ok(())
}

pub fn move_to(conn: &Connection, id: &str, parent_id: &str) -> Result<()> {
    ensure_editable(conn, id)?;
    ensure_exists(conn, parent_id)?;

    if subtree_ids(conn, id)?.iter().any(|n| n == parent_id) {
        return Err(Error::invalid(
            "A node cannot be moved into itself or one of its descendants.",
        ));
    }

    conn.execute(
        "UPDATE nodes SET parent_id = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, parent_id, now()],
    )?;
    Ok(())
}

/// Deletes the node with everything below it and returns the parent to select next.
///
/// Topics stored in the removed branch are deleted with their files; topics that are only
/// linked there are merely unlinked.
pub fn delete(conn: &Connection, files_dir: &Path, id: &str) -> Result<String> {
    let parent_id = ensure_editable(conn, id)?;
    let subtree = subtree_ids(conn, id)?;

    let tx = conn.unchecked_transaction()?;
    let mut doomed_media = Vec::new();
    {
        let mut owned = tx.prepare(
            "SELECT DISTINCT topic_id FROM node_topic WHERE node_id = ?1 AND is_link = 0",
        )?;
        let mut topics = Vec::new();
        for node_id in &subtree {
            let ids = owned.query_map([node_id], |row| row.get::<_, String>(0))?;
            for topic_id in ids {
                topics.push(topic_id?);
            }
        }
        for topic_id in topics {
            doomed_media.extend(media::ids_of_topic(&tx, &topic_id)?);
            tx.execute("DELETE FROM topics WHERE id = ?1", [&topic_id])?;
        }
    }
    tx.execute("DELETE FROM nodes WHERE id = ?1", [id])?;
    tx.commit()?;

    media::remove_files(files_dir, &doomed_media);
    Ok(parent_id)
}

pub fn ensure_exists(conn: &Connection, id: &str) -> Result<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM nodes WHERE id = ?1)",
        [id],
        |r| r.get(0),
    )?;
    exists.then_some(()).ok_or(Error::NotFound("node"))
}

/// Returns the parent id of a node that may be changed (everything but the root).
fn ensure_editable(conn: &Connection, id: &str) -> Result<String> {
    let parent: Option<Option<String>> = conn
        .query_row("SELECT parent_id FROM nodes WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?;

    match parent {
        None => Err(Error::NotFound("node")),
        Some(None) => Err(Error::invalid("The root node cannot be changed.")),
        Some(Some(parent_id)) => Ok(parent_id),
    }
}

/// Ids of the node and every node below it.
fn subtree_ids(conn: &Connection, id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "WITH RECURSIVE sub(id) AS (
             SELECT ?1
             UNION ALL
             SELECT n.id FROM nodes n JOIN sub ON n.parent_id = sub.id
         )
         SELECT id FROM sub",
    )?;
    let ids = stmt
        .query_map([id], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(ids)
}

fn valid_title(title: &str) -> Result<&str> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 255 {
        return Err(Error::invalid("The title must be 1–255 characters long."));
    }
    Ok(title)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_conn;

    fn root(conn: &Connection) -> String {
        list(conn)
            .unwrap()
            .into_iter()
            .find(|n| n.parent_id.is_none())
            .unwrap()
            .id
    }

    #[test]
    fn a_root_node_exists_and_is_protected() {
        let conn = test_conn();
        let root = root(&conn);

        assert!(rename(&conn, &root, "x").is_err());
        assert!(delete(&conn, Path::new("."), &root).is_err());
        assert!(!list(&conn).unwrap()[0].can_edit);
    }

    #[test]
    fn a_node_cannot_move_into_its_own_subtree() {
        let conn = test_conn();
        let root = root(&conn);
        let a = create(&conn, &root, "a").unwrap();
        let b = create(&conn, &a, "b").unwrap();

        assert!(move_to(&conn, &a, &b).is_err());
        assert!(move_to(&conn, &a, &a).is_err());
        move_to(&conn, &b, &root).unwrap();
    }

    #[test]
    fn rejects_blank_titles() {
        let conn = test_conn();
        let root = root(&conn);
        assert!(create(&conn, &root, "   ").is_err());
    }

    #[test]
    fn deleting_removes_the_branch_and_returns_the_parent() {
        let conn = test_conn();
        let root = root(&conn);
        let a = create(&conn, &root, "a").unwrap();
        create(&conn, &a, "b").unwrap();

        assert_eq!(delete(&conn, Path::new("."), &a).unwrap(), root);
        assert_eq!(list(&conn).unwrap().len(), 1);
    }
}
