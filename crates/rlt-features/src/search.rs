//! Tim kiem FTS5 (US-08): bien ban, ghi chu highlight, thuat ngu. Khong phan biet dau.

use crate::{glossary, Result};
use rlt_store::{Highlight, SearchHit, Store};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResults {
    pub lines: Vec<SearchHit>,
    pub notes: Vec<Highlight>,
    pub terms: Vec<glossary::GlossaryTerm>,
}

impl SearchResults {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && self.notes.is_empty() && self.terms.is_empty()
    }
}

/// Tim trong dong bien ban (chu goc va chu dich), ghi chu, thuat ngu.
/// `session_id = Some(..)` gioi han bien ban trong mot phien (ghi chu va thuat ngu van tim chung).
/// Chuoi nguoi dung go duoc dua ve cum trong ngoac kep nen cac ky tu FTS (`AND`, `*`, `-`, `"`) khong gay loi.
pub fn search_all(store: &Store, query: &str, session_id: Option<&str>, limit: i64) -> Result<SearchResults> {
    Ok(SearchResults {
        lines: store.search_caption_lines(query, session_id, limit)?,
        notes: store
            .search_notes(query, limit)?
            .into_iter()
            .filter(|h| session_id.is_none_or(|s| h.session_id == s))
            .collect(),
        terms: glossary::search_terms(store, query, limit)?,
    })
}

/// Chi tim dong bien ban.
pub fn search_lines(store: &Store, query: &str, session_id: Option<&str>, limit: i64) -> Result<Vec<SearchHit>> {
    Ok(store.search_caption_lines(query, session_id, limit)?)
}
