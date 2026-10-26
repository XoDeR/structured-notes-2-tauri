use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};

use crate::db::{new_id, now};
use crate::error::{Error, Result};
use crate::models::Attachment;
use crate::topics;

pub const IMAGES: &str = "images";
pub const ATTACHMENTS: &str = "attachments";

const IMAGE_TYPES: [&str; 4] = ["image/png", "image/jpeg", "image/gif", "image/webp"];

pub struct MediaFile {
    pub path: PathBuf,
    pub mime_type: String,
    pub collection: String,
}

/// Files live next to the database, named after their media id.
pub fn file_path(files_dir: &Path, id: &str) -> PathBuf {
    files_dir.join(id)
}

pub fn ids_of_topic(conn: &Connection, topic_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM media WHERE topic_id = ?1")?;
    let ids = stmt
        .query_map([topic_id], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(ids)
}

/// Best effort: a file that is already gone must not fail the operation that orphaned it.
pub fn remove_files(files_dir: &Path, ids: &[String]) {
    for id in ids {
        let _ = fs::remove_file(file_path(files_dir, id));
    }
}

pub fn attachments_of(conn: &Connection, topic_id: &str) -> Result<Vec<Attachment>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_name, size FROM media
         WHERE topic_id = ?1 AND collection = 'attachments' ORDER BY created_at, id",
    )?;
    let items = stmt
        .query_map([topic_id], |row| {
            Ok(Attachment {
                id: row.get(0)?,
                name: row.get(1)?,
                size: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(items)
}

pub fn add_attachment(
    conn: &Connection,
    files_dir: &Path,
    topic_id: &str,
    source: &Path,
) -> Result<Attachment> {
    topics::ensure_exists(conn, topic_id)?;

    let name = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::invalid("The file has no name."))?;
    let id = new_id();
    let size = fs::copy(source, file_path(files_dir, &id))?;

    let mime = mime_guess::from_path(source).first_or_octet_stream();
    if let Err(e) = insert(
        conn,
        &id,
        topic_id,
        ATTACHMENTS,
        name,
        mime.as_ref(),
        size as i64,
    ) {
        remove_files(files_dir, &[id]);
        return Err(e);
    }

    Ok(Attachment {
        id,
        name: name.to_owned(),
        size: size as i64,
    })
}

/// Stores an image embedded in a topic and returns its media id.
pub fn add_image(
    conn: &Connection,
    files_dir: &Path,
    topic_id: &str,
    file_name: &str,
    bytes: &[u8],
) -> Result<String> {
    topics::ensure_exists(conn, topic_id)?;

    let mime = mime_guess::from_path(file_name).first_or_octet_stream();
    if !IMAGE_TYPES.contains(&mime.as_ref()) {
        return Err(Error::invalid(
            "Only PNG, JPEG, GIF and WebP images can be added.",
        ));
    }

    let id = new_id();
    fs::write(file_path(files_dir, &id), bytes)?;
    if let Err(e) = insert(
        conn,
        &id,
        topic_id,
        IMAGES,
        file_name,
        mime.as_ref(),
        bytes.len() as i64,
    ) {
        remove_files(files_dir, &[id]);
        return Err(e);
    }
    Ok(id)
}

pub fn delete(conn: &Connection, files_dir: &Path, id: &str) -> Result<()> {
    let deleted = conn.execute("DELETE FROM media WHERE id = ?1", [id])?;
    if deleted == 0 {
        return Err(Error::NotFound("file"));
    }
    remove_files(files_dir, &[id.to_owned()]);
    Ok(())
}

pub fn find(conn: &Connection, files_dir: &Path, id: &str) -> Result<MediaFile> {
    conn.query_row(
        "SELECT mime_type, collection FROM media WHERE id = ?1",
        [id],
        |row| {
            Ok(MediaFile {
                path: file_path(files_dir, id),
                mime_type: row.get(0)?,
                collection: row.get(1)?,
            })
        },
    )
    .optional()?
    .ok_or(Error::NotFound("file"))
}

/// Writes a copy of a stored file to a location chosen by the user.
pub fn export(conn: &Connection, files_dir: &Path, id: &str, destination: &Path) -> Result<()> {
    let file = find(conn, files_dir, id)?;
    fs::copy(file.path, destination)?;
    Ok(())
}

/// Duplicates every file of a topic for its copy; returns `(old id, new id)` pairs.
pub fn copy_all(
    conn: &Connection,
    files_dir: &Path,
    from_topic: &str,
    to_topic: &str,
) -> Result<Vec<(String, String)>> {
    let rows: Vec<(String, String, String, String, i64)> = {
        let mut stmt = conn.prepare(
            "SELECT id, collection, file_name, mime_type, size FROM media WHERE topic_id = ?1",
        )?;
        stmt.query_map([from_topic], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })?
        .collect::<rusqlite::Result<_>>()?
    };

    let mut copied = Vec::with_capacity(rows.len());
    for (old, collection, name, mime, size) in rows {
        let new = new_id();
        fs::copy(file_path(files_dir, &old), file_path(files_dir, &new))?;
        insert(conn, &new, to_topic, &collection, &name, &mime, size)?;
        copied.push((old, new));
    }
    Ok(copied)
}

fn insert(
    conn: &Connection,
    id: &str,
    topic_id: &str,
    collection: &str,
    file_name: &str,
    mime: &str,
    size: i64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO media (id, topic_id, collection, file_name, mime_type, size, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, topic_id, collection, file_name, mime, size, now()],
    )?;
    Ok(())
}
