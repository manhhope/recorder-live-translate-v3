use crate::error::{Result, StoreError};
use rusqlite::Connection;

/// Noi dung schema, nhung luc bien dich.
pub const SCHEMA_SQL: &str = include_str!("../../../docs/schema-v3.sql");

/// Cat file theo cap dau `-- @@MIGRATION <n> BEGIN` / `END`. Tra ve (n, sql) tang dan.
pub fn migrations(sql: &str) -> Result<Vec<(i64, String)>> {
    let mut out = Vec::new();
    let mut current: Option<(i64, String)> = None;
    for line in sql.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("-- @@MIGRATION ") {
            let mut it = rest.split_whitespace();
            let n: i64 = it
                .next()
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| StoreError::BadMigration(t.to_string()))?;
            match it.next() {
                Some("BEGIN") => {
                    if current.is_some() {
                        return Err(StoreError::BadMigration(format!("BEGIN long nhau o {n}")));
                    }
                    current = Some((n, String::new()));
                }
                Some("END") => match current.take() {
                    Some((m, body)) if m == n => out.push((n, body)),
                    _ => return Err(StoreError::BadMigration(format!("END {n} khong khop BEGIN"))),
                },
                _ => return Err(StoreError::BadMigration(t.to_string())),
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if current.is_some() {
        return Err(StoreError::BadMigration("thieu END".into()));
    }
    out.sort_by_key(|(n, _)| *n);
    Ok(out)
}

/// Chay moi khoi n > user_version, moi khoi trong mot transaction.
/// Khoi ket thuc bang `PRAGMA user_version = n;` nen khong dat lai o day.
///
/// Khoi dung lai bang (doi CHECK) can `foreign_keys = OFF` va `legacy_alter_table = ON`, ma hai
/// PRAGMA nay khong co tac dung ben trong transaction. Nen tat o day, chay, roi
/// `foreign_key_check` truoc khi tra lai gia tri cu (quy trinh 12 buoc cua SQLite).
pub fn run(conn: &mut Connection) -> Result<i64> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let pending: Vec<(i64, String)> =
        migrations(SCHEMA_SQL)?.into_iter().filter(|(n, _)| *n > current).collect();
    if pending.is_empty() {
        return Ok(current);
    }
    let fk: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?;
    let legacy: i64 = conn.query_row("PRAGMA legacy_alter_table", [], |r| r.get(0))?;
    conn.execute_batch("PRAGMA foreign_keys = OFF; PRAGMA legacy_alter_table = ON;")?;
    let result = apply(conn, &pending);
    let restore = format!(
        "PRAGMA legacy_alter_table = {}; PRAGMA foreign_keys = {};",
        if legacy == 1 { "ON" } else { "OFF" },
        if fk == 1 { "ON" } else { "OFF" }
    );
    let restored = conn.execute_batch(&restore);
    result?;
    restored?;
    Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
}

fn apply(conn: &mut Connection, pending: &[(i64, String)]) -> Result<()> {
    for (_, body) in pending {
        let tx = conn.transaction()?;
        tx.execute_batch(body)?;
        let broken: i64 =
            tx.query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r.get(0))?;
        if broken > 0 {
            return Err(StoreError::BadMigration(format!("foreign_key_check: {broken} dong hong")));
        }
        tx.commit()?;
    }
    Ok(())
}
