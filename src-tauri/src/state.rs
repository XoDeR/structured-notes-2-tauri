use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};

use rusqlite::Connection;

pub struct AppState {
    db: Mutex<Connection>,
    pub files_dir: PathBuf,
}

impl AppState {
    pub fn new(db: Connection, files_dir: PathBuf) -> Self {
        Self {
            db: Mutex::new(db),
            files_dir,
        }
    }

    /// A panic while holding the lock cannot leave the database half-written, so poisoning is ignored.
    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        self.db.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
