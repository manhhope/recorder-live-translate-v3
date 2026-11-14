use crate::error::{Result, StoreError};
use rusqlite::Connection;

/// Noi dung schema, nhung luc bien dich.
pub const SCHEMA_SQL: &str = include_str!("../migrations/schema-v3.sql");

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
pub fn run(conn: &mut Connection) -> Result<i64> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    for (n, body) in migrations(SCHEMA_SQL)? {
        if n > current {
            let tx = conn.transaction()?;
            tx.execute_batch(&body)?;
            tx.commit()?;
        }
    }
    Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
}
