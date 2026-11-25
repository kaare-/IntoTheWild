//! One SQLite writer. A transaction per resolved group. Restore is the last
//! snapshot plus the log that follows it.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use villmarks_sim::{Intent, World};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Record {
    Join { name: String, token: String },
    Step { intents: Vec<Intent> },
}

pub struct Store {
    conn: Connection,
    path: PathBuf,
    snapshot_every: i64,
}

impl Store {
    pub fn open(path: &Path, snapshot_every: u32) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
            }
        }
        let conn = Connection::open(path).map_err(|err| err.to_string())?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|err| err.to_string())?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|err| err.to_string())?;
        conn.busy_timeout(std::time::Duration::from_secs(3))
            .map_err(|err| err.to_string())?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS log (
                seq INTEGER PRIMARY KEY,
                payload TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS snapshots (
                seq INTEGER PRIMARY KEY,
                world_json TEXT NOT NULL
            );
            ",
        )
        .map_err(|err| err.to_string())?;
        Ok(Store {
            conn,
            path: path.to_path_buf(),
            snapshot_every: snapshot_every.max(1) as i64,
        })
    }

    pub fn load_or_seed(&mut self, seed: World) -> Result<World, String> {
        let latest = self.latest_snapshot()?;
        let (mut world, from_seq) = match latest {
            Some((seq, json)) => {
                let world = self.restore_world(&json)?;
                (world, seq)
            }
            None => {
                self.write_snapshot_in_place(0, &seed)?;
                (seed, 0)
            }
        };
        let mut statement = self
            .conn
            .prepare("SELECT payload FROM log WHERE seq > ?1 ORDER BY seq")
            .map_err(|err| err.to_string())?;
        let rows = statement
            .query_map(params![from_seq], |row| row.get::<_, String>(0))
            .map_err(|err| err.to_string())?;
        for row in rows {
            let payload = row.map_err(|err| err.to_string())?;
            let record: Record = serde_json::from_str(&payload)
                .map_err(|err| self.valley_file_message(&err.to_string()))?;
            apply(&mut world, record)?;
        }
        Ok(world)
    }

    pub fn commit(&mut self, world: &World, record: &Record) -> Result<(), String> {
        let seq = self.next_seq()?;
        let payload = serde_json::to_string(record).map_err(|err| err.to_string())?;
        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|err| err.to_string())?;
        tx.execute(
            "INSERT INTO log (seq, payload) VALUES (?1, ?2)",
            params![seq, payload],
        )
        .map_err(|err| err.to_string())?;
        if seq % self.snapshot_every == 0 {
            let json = serde_json::to_string(world).map_err(|err| err.to_string())?;
            tx.execute(
                "INSERT INTO snapshots (seq, world_json) VALUES (?1, ?2)",
                params![seq, json],
            )
            .map_err(|err| err.to_string())?;
        }
        tx.commit().map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn snapshot_now(&mut self, world: &World) -> Result<(), String> {
        let seq = self
            .conn
            .query_row("SELECT COALESCE(MAX(seq), 0) FROM log", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|err| err.to_string())?;
        let json = serde_json::to_string(world).map_err(|err| err.to_string())?;
        self.conn
            .execute(
                "INSERT INTO snapshots (seq, world_json) VALUES (?1, ?2)
                 ON CONFLICT(seq) DO UPDATE SET world_json = excluded.world_json",
                params![seq, json],
            )
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    fn latest_snapshot(&self) -> Result<Option<(i64, String)>, String> {
        self.conn
            .query_row(
                "SELECT seq, world_json FROM snapshots ORDER BY seq DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|err| err.to_string())
    }

    fn write_snapshot_in_place(&self, seq: i64, world: &World) -> Result<(), String> {
        let json = serde_json::to_string(world).map_err(|err| err.to_string())?;
        self.conn
            .execute(
                "INSERT INTO snapshots (seq, world_json) VALUES (?1, ?2)",
                params![seq, json],
            )
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    /// An older snapshot has sight and no hearing. Hearing is one cell past sight.
    fn restore_world(&self, json: &str) -> Result<World, String> {
        let json = fill_missing_hearing(json)
            .map_err(|err| self.valley_file_message(&err))?;
        serde_json::from_str(&json).map_err(|err| self.valley_file_message(&err.to_string()))
    }

    fn valley_file_message(&self, err: &str) -> String {
        let file = self.path.display();
        format!(
            "{err}. This valley file is from an older build. Delete {file}, \
             {file}-wal, and {file}-shm (the file named by --data, or \
             data/valley.sqlite if you did not pass one), then start again."
        )
    }

    fn next_seq(&self) -> Result<i64, String> {
        self.conn
            .query_row("SELECT COALESCE(MAX(seq), 0) + 1 FROM log", [], |row| {
                row.get(0)
            })
            .map_err(|err| err.to_string())
    }
}

/// Hearing arrived after the first valley files. It is one cell past sight.
fn fill_missing_hearing(json: &str) -> Result<String, String> {
    let mut value: Value = serde_json::from_str(json).map_err(|err| err.to_string())?;
    let Some(tuning) = value.get_mut("tuning").and_then(|v| v.as_object_mut()) else {
        return Ok(json.to_string());
    };
    if tuning.contains_key("hearing") {
        return Ok(json.to_string());
    }
    let Some(sight) = tuning.get("sight").and_then(|v| v.as_i64()) else {
        return Err(
            "missing field hearing, and the snapshot has no sight to set it from".into(),
        );
    };
    let sight = i32::try_from(sight).unwrap_or(i32::MAX);
    tuning.insert(
        "hearing".into(),
        Value::from(sight.saturating_add(1)),
    );
    serde_json::to_string(&value).map_err(|err| err.to_string())
}

fn apply(world: &mut World, record: Record) -> Result<(), String> {
    match record {
        Record::Join { name, token } => {
            world.join(&name, &token)?;
        }
        Record::Step { intents } => {
            world.step(&intents);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use villmarks_sim::Tuning;

    fn scratch() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("villmarks-hearing-{nanos}"));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn seed() -> World {
        World::valley(Tuning::evening(), 1)
    }

    #[test]
    fn an_old_snapshot_without_hearing_uses_one_cell_past_sight() {
        let db = scratch().join("valley.sqlite");
        {
            let mut store = Store::open(&db, 5).unwrap();
            store.load_or_seed(seed()).unwrap();
        }
        let conn = Connection::open(&db).unwrap();
        let json: String = conn
            .query_row(
                "SELECT world_json FROM snapshots ORDER BY seq DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let mut value: Value = serde_json::from_str(&json).unwrap();
        let tuning = value["tuning"].as_object_mut().unwrap();
        tuning.remove("hearing");
        let stripped = serde_json::to_string(&value).unwrap();
        let missing = serde_json::from_str::<World>(&stripped).unwrap_err();
        assert!(
            missing.to_string().contains("missing field `hearing`"),
            "{missing}"
        );
        conn.execute(
            "UPDATE snapshots SET world_json = ?1",
            params![stripped],
        )
        .unwrap();
        drop(conn);

        let mut store = Store::open(&db, 5).unwrap();
        let world = store.load_or_seed(seed()).unwrap();
        assert_eq!(world.tuning.hearing, world.tuning.sight.saturating_add(1));
        assert_eq!(world.tuning.hearing, seed().tuning.hearing);
    }

    #[test]
    fn a_valley_file_that_cannot_be_read_names_the_files_to_delete() {
        let db = scratch().join("data").join("valley.sqlite");
        std::fs::create_dir_all(db.parent().unwrap()).unwrap();
        {
            let mut store = Store::open(&db, 5).unwrap();
            store.load_or_seed(seed()).unwrap();
        }
        let conn = Connection::open(&db).unwrap();
        conn.execute("UPDATE snapshots SET world_json = ?1", params!["{"])
            .unwrap();
        drop(conn);

        let mut store = Store::open(&db, 5).unwrap();
        let err = store.load_or_seed(seed()).unwrap_err();
        let file = db.display().to_string();
        assert!(err.contains(&file), "{err}");
        assert!(err.contains(&format!("{file}-wal")), "{err}");
        assert!(err.contains(&format!("{file}-shm")), "{err}");
        assert!(err.contains("--data"), "{err}");
        assert!(err.contains("data/valley.sqlite"), "{err}");
    }
}
