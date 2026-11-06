//! One SQLite writer. A transaction per resolved group. Restore is the last
//! snapshot plus the log that follows it.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use villmarks_sim::{Intent, World};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Record {
    Join { name: String, token: String },
    Step { intents: Vec<Intent> },
}

pub struct Store {
    conn: Connection,
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
            snapshot_every: snapshot_every.max(1) as i64,
        })
    }

    pub fn load_or_seed(&mut self, seed: World) -> Result<World, String> {
        let latest = self.latest_snapshot()?;
        let (mut world, from_seq) = match latest {
            Some((seq, json)) => {
                let world = serde_json::from_str(&json).map_err(|err| err.to_string())?;
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
            let record: Record = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
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

    fn next_seq(&self) -> Result<i64, String> {
        self.conn
            .query_row("SELECT COALESCE(MAX(seq), 0) + 1 FROM log", [], |row| {
                row.get(0)
            })
            .map_err(|err| err.to_string())
    }
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
