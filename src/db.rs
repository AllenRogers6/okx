use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::time::SystemTime;

const DB_FILE: &str = "okx.db";
const APP_DIR: &str = "okx";

const DB_VERSION: i64 = 1;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_DIR)
            .join(DB_FILE);

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;

        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version < DB_VERSION {
            conn.execute_batch(&format!(
                "CREATE TABLE IF NOT EXISTS files (
                    path          TEXT PRIMARY KEY,
                    score         REAL NOT NULL DEFAULT 0.0,
                    last_accessed INTEGER NOT NULL
                );
                -- meta stores lightweight key/value config, e.g. last decay timestamp.
                CREATE TABLE IF NOT EXISTS meta (
                    key   TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );
                PRAGMA user_version = {DB_VERSION};"
            ))?;
        }

        let db = Self { conn };
        db.maybe_decay()?;
        Ok(db)
    }

    fn maybe_decay(&self) -> Result<()> {
        let now = now_secs();
        let last: Option<i64> = self
            .conn
            .query_row("SELECT value FROM meta WHERE key = 'last_decay'", [], |r| {
                r.get::<_, String>(0)
            })
            .ok()
            .and_then(|v| v.parse().ok());

        let one_hour = 3600;
        if last.map_or(true, |t| now - t > one_hour) {
            self.conn
                .execute_batch("UPDATE files SET score = score * 0.95")?;
            self.conn.execute(
                "INSERT INTO meta (key, value) VALUES ('last_decay', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = ?1",
                params![now.to_string()],
            )?;
        }
        Ok(())
    }

    pub fn add(&self, path: &str) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO files (path, score, last_accessed) VALUES (?1, 1.0, ?2)
             ON CONFLICT(path) DO UPDATE SET
                score = score * 0.9 + 1.0,
                last_accessed = ?2",
        )?;
        stmt.execute(params![path, now_secs()])?;
        Ok(())
    }

    pub fn remove(&self, path: &str) -> Result<bool> {
        let n = self
            .conn
            .execute("DELETE FROM files WHERE path = ?1", params![path])?;
        Ok(n > 0)
    }

    pub fn clean(&self) -> Result<usize> {
        let paths: Vec<String> = {
            let mut stmt = self.conn.prepare_cached("SELECT path FROM files")?;
            stmt.query_map([], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect()
        };

        let mut removed = 0;
        for path in &paths {
            if !std::path::Path::new(path).exists() {
                self.conn
                    .execute("DELETE FROM files WHERE path = ?1", params![path])?;
                removed += 1;
            }
        }
        Ok(removed)
    }

    pub fn query(&self, pattern: &str) -> Result<Option<(String, f64)>> {
        Ok(self.query_many(pattern, 1)?.into_iter().next())
    }

    pub fn query_many(&self, pattern: &str, limit: usize) -> Result<Vec<(String, f64)>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT path, score FROM files ORDER BY score DESC LIMIT 100")?;

        let rows: Vec<(String, f64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            return Ok(vec![]);
        }

        let pattern = pattern.to_lowercase();
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

        let mut scored: Vec<(String, f64)> = rows
            .into_iter()
            .filter_map(|(path, frecency)| {
                let file_name = std::path::Path::new(&path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_lowercase())
                    .unwrap_or_else(|| path.to_lowercase());

                let fuzzy = matcher.fuzzy_match(&file_name, &pattern)?;
                let combined = frecency * (fuzzy as f64 + 1.0).ln();
                Some((path, combined))
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        Ok(scored)
    }

    pub fn list_all(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT path FROM files ORDER BY score DESC")?;
        Ok(stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect())
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system time before Unix epoch")
        .as_secs() as i64
}
