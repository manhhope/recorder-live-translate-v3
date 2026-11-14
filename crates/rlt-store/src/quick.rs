//! Lich su Quick Translate (ADR-11, US-31 AC-31.7). Bang `quick_translations` nam trong `schema-v3.sql`.
//! Tach han khoi bien ban phien: khong co khoa ngoai toi `sessions`. Noi dung la du lieu NHAY CAM: khong bao gio
//! dua vao nhat ky; `Debug` cua cac kieu o day che noi dung.

use crate::error::Result;
use crate::store::Store;
use rusqlite::params;
use std::fmt;

/// Tran so ky tu cua mot muc (AC-31.3, khop CHECK cua bang).
pub const QUICK_MAX_CHARS: usize = 5000;
/// Tuoi toi da cua mot muc, tinh bang ngay (AC-31.7).
pub const QUICK_MAX_AGE_DAYS: i64 = 30;

#[derive(Clone, PartialEq, Eq)]
pub struct NewQuickTranslation {
    /// `quick_translate`, `inline_translate` hoac `ocr_translate`.
    pub origin: String,
    pub source_text: String,
    pub translated_text: String,
    pub source_lang: Option<String>,
    pub detected_lang: Option<String>,
    pub target_lang: String,
}

impl fmt::Debug for NewQuickTranslation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewQuickTranslation")
            .field("origin", &self.origin)
            .field("source_text", &format_args!("<{} ky tu>", self.source_text.chars().count()))
            .field("translated_text", &format_args!("<{} ky tu>", self.translated_text.chars().count()))
            .field("target_lang", &self.target_lang)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct QuickTranslation {
    pub quick_translation_id: i64,
    pub origin: String,
    pub source_text: String,
    pub translated_text: String,
    pub source_lang: Option<String>,
    pub detected_lang: Option<String>,
    pub target_lang: String,
    pub created_at: String,
}

impl fmt::Debug for QuickTranslation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QuickTranslation")
            .field("id", &self.quick_translation_id)
            .field("origin", &self.origin)
            .field("text", &"<an>")
            .field("created_at", &self.created_at)
            .finish()
    }
}

impl Store {
    /// Them mot muc. Trigger `trg_quick_translations_cap` giu 50 muc gan nhat cho moi `origin`.
    /// Chu qua 5.000 ky tu bi CHECK tu choi (loi SQLite), khong cat ngam.
    pub fn add_quick_translation(&self, q: &NewQuickTranslation) -> Result<i64> {
        self.conn().execute(
            "INSERT INTO quick_translations (origin, source_text, translated_text, source_lang, detected_lang, target_lang, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            params![q.origin, q.source_text, q.translated_text, q.source_lang, q.detected_lang, q.target_lang],
        )?;
        Ok(self.conn().last_insert_rowid())
    }

    /// Moi nhat truoc.
    pub fn list_quick_translations(&self, limit: i64) -> Result<Vec<QuickTranslation>> {
        let mut st = self.conn().prepare(
            "SELECT quick_translation_id, origin, source_text, translated_text, source_lang, detected_lang, target_lang, created_at
             FROM quick_translations ORDER BY created_at DESC, quick_translation_id DESC LIMIT ?1",
        )?;
        let rows = st.query_map([limit], |r| {
            Ok(QuickTranslation {
                quick_translation_id: r.get(0)?,
                origin: r.get(1)?,
                source_text: r.get(2)?,
                translated_text: r.get(3)?,
                source_lang: r.get(4)?,
                detected_lang: r.get(5)?,
                target_lang: r.get(6)?,
                created_at: r.get(7)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// "Xoa lich su" (AC-31.7): xoa sach, tra so muc da xoa.
    pub fn clear_quick_translations(&self) -> Result<usize> {
        Ok(self.conn().execute("DELETE FROM quick_translations", [])?)
    }

    /// Xoa muc cu hon `cutoff` (chuoi ISO UTC, so sanh theo chuoi). Goi luc mo app va moi ngay.
    pub fn purge_quick_translations_before(&self, cutoff: &str) -> Result<usize> {
        Ok(self.conn().execute("DELETE FROM quick_translations WHERE created_at < ?1", [cutoff])?)
    }

    /// Xoa muc cu hon `days` ngay so voi bay gio (theo dong ho SQLite).
    pub fn purge_quick_translations_older_than_days(&self, days: i64) -> Result<usize> {
        Ok(self.conn().execute(
            "DELETE FROM quick_translations WHERE created_at < strftime('%Y-%m-%dT%H:%M:%fZ','now', ?1)",
            [format!("-{days} days")],
        )?)
    }
}
