//! SQLite storage for Scribewell. This is the only crate that talks to the
//! database; other crates go through the functions it exposes.

pub mod migrate;
pub mod settings;

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::Connection;
use sw_core::AppError;

pub use migrate::MigrationReport;

/// Converts a SQLite error into the IPC error type.
pub fn db_err(err: rusqlite::Error) -> AppError {
    AppError::Database {
        detail: err.to_string(),
    }
}

#[derive(Debug)]
pub struct Db {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
}

impl Db {
    /// Opens (creating if needed) the database file and applies pending
    /// migrations. Existing databases are backed up into `backup_dir` first.
    pub fn open(path: &Path, backup_dir: &Path) -> Result<(Self, MigrationReport), AppError> {
        let mut conn = Connection::open(path).map_err(db_err)?;
        configure(&conn)?;
        let report = migrate::migrate(&mut conn, Some(backup_dir), migrate::MIGRATIONS)?;
        Ok((
            Self {
                conn: Mutex::new(conn),
                path: Some(path.to_owned()),
            },
            report,
        ))
    }

    /// A fully migrated in-memory database, for tests.
    pub fn open_in_memory() -> Result<Self, AppError> {
        let mut conn = Connection::open_in_memory().map_err(db_err)?;
        configure(&conn)?;
        migrate::migrate(&mut conn, None, migrate::MIGRATIONS)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: None,
        })
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Runs `f` with exclusive access to the connection.
    pub fn with_conn<T>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let mut guard: MutexGuard<'_, Connection> = self
            .conn
            .lock()
            .map_err(|_| AppError::internal("database lock poisoned by an earlier panic"))?;
        f(&mut guard)
    }
}

fn configure(conn: &Connection) -> Result<(), AppError> {
    conn.busy_timeout(Duration::from_secs(5)).map_err(db_err)?;
    // WAL keeps readers unblocked during writes and survives crashes well.
    // In-memory databases report "memory" here, which is fine.
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(db_err)?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(db_err)?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(db_err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreign_keys_are_enforced() {
        let db = Db::open_in_memory().unwrap();
        let result = db.with_conn(|c| {
            c.execute(
                "INSERT INTO meeting_folders (meeting_id, folder_id) VALUES ('missing', 'missing')",
                [],
            )
            .map_err(db_err)
        });
        assert_eq!(result.unwrap_err().code(), "db.error");
    }

    #[test]
    fn fts5_search_index_works() {
        let db = Db::open_in_memory().unwrap();
        let hits: Vec<String> = db
            .with_conn(|c| {
                c.execute(
                    "INSERT INTO search_index (meeting_id, kind, ref_id, title, body)
                     VALUES ('m1', 'transcript', 's1', 'Pricing review', 'We agreed to move to annual billing')",
                    [],
                )
                .map_err(db_err)?;
                let mut stmt = c
                    .prepare("SELECT meeting_id FROM search_index WHERE search_index MATCH ?1")
                    .map_err(db_err)?;
                let rows = stmt
                    .query_map(["billing"], |r| r.get(0))
                    .map_err(db_err)?
                    .collect::<Result<Vec<String>, _>>()
                    .map_err(db_err)?;
                Ok(rows)
            })
            .unwrap();
        assert_eq!(hits, vec!["m1".to_owned()]);
    }

    #[test]
    fn file_database_uses_wal_and_reopens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("scribewell.db");
        let backups = dir.path().join("backups");
        {
            let (db, report) = Db::open(&path, &backups).unwrap();
            assert_eq!(report.from, 0);
            assert_eq!(report.to, migrate::latest_version(migrate::MIGRATIONS));
            let mode: String = db
                .with_conn(|c| {
                    c.pragma_query_value(None, "journal_mode", |r| r.get(0))
                        .map_err(db_err)
                })
                .unwrap();
            assert_eq!(mode.to_lowercase(), "wal");
        }
        let (_db, report) = Db::open(&path, &backups).unwrap();
        assert_eq!(report.from, report.to);
        assert!(report.backup.is_none(), "no backup when nothing to migrate");
    }
}
