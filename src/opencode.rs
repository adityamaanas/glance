//! OpenCode's message/part SQLite format, queried with a read-only connection.
use crate::harness::{opencode_export, Snapshot};
use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use serde_json::{json, Value};
use std::path::Path;

fn connection(path: &Path) -> Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .context("open OpenCode database read-only")?;
    conn.busy_timeout(std::time::Duration::from_millis(500))?;
    Ok(conn)
}

pub fn read(path: &Path, id: &str) -> Result<Snapshot> {
    let mut conn = connection(path)?;
    let tx = conn.transaction()?;
    let (directory, title): (String, String) = tx
        .query_row(
            "SELECT directory, title FROM session WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .context("find OpenCode session")?;
    let mut messages = Vec::new();
    {
        let mut statement = tx
            .prepare("SELECT id, data FROM message WHERE session_id = ?1 ORDER BY time_created, id")
            .context("unsupported OpenCode schema; use opencode export and --transcript")?;
        let rows = statement.query_map([id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        let mut parts = tx.prepare(
            "SELECT data FROM part WHERE message_id = ?1 AND session_id = ?2 ORDER BY id",
        )?;
        for row in rows {
            let (message_id, data) = row?;
            let info: Value = serde_json::from_str(&data).context("parse OpenCode message")?;
            let mut values = Vec::new();
            for part in parts.query_map([&message_id, id], |r| r.get::<_, String>(0))? {
                values.push(serde_json::from_str::<Value>(&part?).context("parse OpenCode part")?);
            }
            messages.push(json!({"info":info,"parts":values}));
        }
    }
    tx.commit()?;
    Ok(opencode_export(
        &json!({"info":{"id":id,"directory":directory,"title":title},"messages":messages}),
    ))
}

pub fn sessions(path: &Path) -> Result<Vec<(String, String, String, u64)>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let conn = connection(path)?;
    let mut query = conn.prepare("SELECT id, directory, title, time_updated FROM session WHERE parent_id IS NULL ORDER BY time_updated DESC")?;
    let rows = query.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, u64>(3)? / 1000))
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reads_committed_wal_rows_and_preserves_database_contents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("opencode.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE session(id TEXT, directory TEXT, title TEXT, parent_id TEXT, time_updated INTEGER); CREATE TABLE message(id TEXT, session_id TEXT, time_created INTEGER, data TEXT); CREATE TABLE part(id TEXT, message_id TEXT, session_id TEXT, data TEXT);").unwrap();
        conn.execute(
            "INSERT INTO session VALUES ('session','/project','SQLite test',NULL,1000)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO message VALUES ('m1','session',1,?1)",
            [json!({"role":"assistant"}).to_string()],
        )
        .unwrap();
        conn.execute("INSERT INTO part VALUES ('p1','m1','session',?1)", [json!({"type":"tool","tool":"test","state":{"status":"error","error":"assertion failed"}}).to_string()]).unwrap();
        let before = std::fs::read(&path).unwrap();
        let snapshot = read(&path, "session").unwrap();
        assert_eq!(snapshot.turns.len(), 1);
        assert!(
            matches!(&snapshot.turns[0], crate::transcript::Turn::Tool(t) if t.contains("assertion failed"))
        );
        assert_eq!(sessions(&path).unwrap().len(), 1);
        assert_eq!(before, std::fs::read(&path).unwrap());
        assert!(read(&path, "session' OR 1=1 --").is_err());
    }
}
