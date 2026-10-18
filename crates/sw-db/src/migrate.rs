//! Forward-only schema migrations tracked with `PRAGMA user_version`.
//!
//! All pending migrations run inside one transaction: either the database
//! reaches the latest version or it is left exactly as it was. Before
//! upgrading an existing database, a copy is written with `VACUUM INTO`.

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use sw_core::AppError;

use crate::db_err;

#[derive(Debug, Clone, Copy)]
pub struct Migration {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "initial",
    sql: include_str!("../migrations/0001_initial.sql"),
}];

/// Number of pre-migration backups kept; older ones are deleted.
const BACKUPS_KEPT: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub from: u32,
    pub to: u32,
    pub backup: Option<PathBuf>,
}

pub fn latest_version(migrations: &[Migration]) -> u32 {
    migrations.iter().map(|m| m.version).max().unwrap_or(0)
}

pub fn migrate(
    conn: &mut Connection,
    backup_dir: Option<&Path>,
    migrations: &[Migration],
) -> Result<MigrationReport, AppError> {
    check_ordering(migrations)?;
    let current: u32 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(db_err)?;
    let latest = latest_version(migrations);

    if current > latest {
        return Err(AppError::Database {
            detail: format!(
                "database schema v{current} is newer than this app supports (v{latest}); \
                 update Scribewell to open it"
            ),
        });
    }
    let pending: Vec<&Migration> = migrations.iter().filter(|m| m.version > current).collect();
    let Some(first) = pending.first() else {
        return Ok(MigrationReport {
            from: current,
            to: current,
            backup: None,
        });
    };

    let backup = match backup_dir {
        Some(dir) if current > 0 => Some(write_backup(conn, dir, first.version)?),
        _ => None,
    };

    let tx = conn.transaction().map_err(db_err)?;
    for m in &pending {
        tracing::info!(version = m.version, name = m.name, "applying migration");
        if let Err(err) = tx.execute_batch(m.sql) {
            // Dropping `tx` rolls back every pending migration.
            return Err(AppError::MigrationFailed {
                version: m.version,
                detail: err.to_string(),
                backup: backup.as_ref().map(|p| p.display().to_string()),
            });
        }
    }
    tx.pragma_update(None, "user_version", latest)
        .map_err(db_err)?;
    tx.commit().map_err(|err| AppError::MigrationFailed {
        version: latest,
        detail: err.to_string(),
        backup: backup.as_ref().map(|p| p.display().to_string()),
    })?;

    Ok(MigrationReport {
        from: current,
        to: latest,
        backup,
    })
}

fn check_ordering(migrations: &[Migration]) -> Result<(), AppError> {
    let mut prev = 0;
    for m in migrations {
        if m.version != prev + 1 {
            return Err(AppError::internal(format!(
                "migration versions must be consecutive from 1; found v{} after v{prev}",
                m.version
            )));
        }
        prev = m.version;
    }
    Ok(())
}

fn write_backup(conn: &Connection, dir: &Path, next_version: u32) -> Result<PathBuf, AppError> {
    std::fs::create_dir_all(dir).map_err(|source| AppError::DataDir { source })?;
    let path = dir.join(format!("pre-v{next_version}-{}.db", sw_core::now_ms()));
    conn.execute("VACUUM INTO ?1", [path.display().to_string()])
        .map_err(db_err)?;
    prune_backups(dir);
    tracing::info!(path = %path.display(), "database backed up before migration");
    Ok(path)
}

fn prune_backups(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut backups: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("pre-v") && n.ends_with(".db"))
        })
        .collect();
    // Names embed a millisecond timestamp, but sort by mtime to be robust.
    backups.sort_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    let excess = backups.len().saturating_sub(BACKUPS_KEPT);
    for old in backups.into_iter().take(excess) {
        if let Err(err) = std::fs::remove_file(&old) {
            tracing::warn!(path = %old.display(), %err, "could not delete old backup");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V1: Migration = Migration {
        version: 1,
        name: "a",
        sql: "CREATE TABLE a (id INTEGER PRIMARY KEY, v TEXT);",
    };
    const V2: Migration = Migration {
        version: 2,
        name: "b",
        sql: "CREATE TABLE b (id INTEGER PRIMARY KEY);",
    };
    const V3_BROKEN: Migration = Migration {
        version: 3,
        name: "broken",
        sql: "CREATE TABLE c (id INTEGER PRIMARY KEY); ALTER TABLE does_not_exist ADD x INTEGER;",
    };

    fn version(conn: &Connection) -> u32 {
        conn.pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap()
    }

    fn table_exists(conn: &Connection, name: &str) -> bool {
        conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [name],
            |r| r.get::<_, i64>(0),
        )
        .unwrap()
            == 1
    }

    #[test]
    fn real_schema_applies_from_scratch_and_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        let report = migrate(&mut conn, None, MIGRATIONS).unwrap();
        assert_eq!(report.from, 0);
        assert_eq!(report.to, latest_version(MIGRATIONS));
        for table in [
            "meetings",
            "transcript_segments",
            "documents",
            "settings",
            "search_index",
        ] {
            assert!(table_exists(&conn, table), "{table} missing");
        }
        let again = migrate(&mut conn, None, MIGRATIONS).unwrap();
        assert_eq!(again.from, again.to);
    }

    #[test]
    fn failed_migration_rolls_back_everything() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn, None, &[V1]).unwrap();
        conn.execute("INSERT INTO a (v) VALUES ('keep me')", [])
            .unwrap();

        let err = migrate(&mut conn, None, &[V1, V2, V3_BROKEN]).unwrap_err();
        assert_eq!(err.code(), "db.migration_failed");
        assert_eq!(version(&conn), 1, "version must not advance");
        assert!(!table_exists(&conn, "b"), "v2 must be rolled back with v3");
        assert!(!table_exists(&conn, "c"));
        let kept: String = conn.query_row("SELECT v FROM a", [], |r| r.get(0)).unwrap();
        assert_eq!(kept, "keep me");
    }

    #[test]
    fn upgrading_existing_database_writes_backup() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("db.sqlite");
        let backups = dir.path().join("backups");
        let mut conn = Connection::open(&db_path).unwrap();
        migrate(&mut conn, Some(&backups), &[V1]).unwrap();
        conn.execute("INSERT INTO a (v) VALUES ('before upgrade')", [])
            .unwrap();

        let report = migrate(&mut conn, Some(&backups), &[V1, V2]).unwrap();
        let backup = report.backup.expect("backup expected for existing db");
        assert!(
            backup
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("pre-v2-")
        );

        let restored = Connection::open(&backup).unwrap();
        assert_eq!(version(&restored), 1);
        let v: String = restored
            .query_row("SELECT v FROM a", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, "before upgrade");
    }

    #[test]
    fn fresh_database_needs_no_backup() {
        let dir = tempfile::tempdir().unwrap();
        let mut conn = Connection::open(dir.path().join("db.sqlite")).unwrap();
        let report = migrate(&mut conn, Some(&dir.path().join("b")), &[V1]).unwrap();
        assert!(report.backup.is_none());
    }

    #[test]
    fn newer_database_is_refused() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
        let err = migrate(&mut conn, None, &[V1]).unwrap_err();
        assert_eq!(err.code(), "db.error");
        assert!(err.detail().unwrap().contains("newer"));
    }

    #[test]
    fn gaps_in_versions_are_rejected() {
        let mut conn = Connection::open_in_memory().unwrap();
        let err = migrate(&mut conn, None, &[V1, V3_BROKEN]).unwrap_err();
        assert_eq!(err.code(), "internal");
    }

    #[test]
    fn old_backups_are_pruned() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..8 {
            std::fs::write(dir.path().join(format!("pre-v2-{i}.db")), b"x").unwrap();
        }
        std::fs::write(dir.path().join("unrelated.txt"), b"x").unwrap();
        prune_backups(dir.path());
        let remaining: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(remaining.len(), BACKUPS_KEPT + 1);
    }
}
