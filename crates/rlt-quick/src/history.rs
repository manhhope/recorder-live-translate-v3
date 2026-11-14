//! Ghi lich su Quick Translate vao `rlt-store` (ADR-11, AC-31.7). Chi ghi khi dich THANH CONG.
//! Khong bao gio ghi chu ra nhat ky.

use crate::queue::{JobOutcome, Origin};
use rlt_store::{NewQuickTranslation, Store, QUICK_MAX_AGE_DAYS};

/// Ghi mot ket qua `Done` vao lich su; cac ket qua khac bo qua. Tra `Ok(true)` neu da ghi.
pub fn record_outcome(store: &Store, outcome: &JobOutcome) -> rlt_store::Result<bool> {
    let JobOutcome::Done { origin, request, translation, .. } = outcome else {
        return Ok(false);
    };
    store.add_quick_translation(&NewQuickTranslation {
        origin: origin.as_db().to_string(),
        source_text: request.text.clone(),
        translated_text: translation.text.clone(),
        source_lang: request.source_lang.clone(),
        detected_lang: translation.detected_lang.clone(),
        target_lang: request.target_lang.clone(),
    })?;
    Ok(true)
}

/// Don muc qua 30 ngay (goi luc mo app va moi ngay, AC-31.7). Tra so muc da xoa.
pub fn purge_expired(store: &Store) -> rlt_store::Result<usize> {
    store.purge_quick_translations_older_than_days(QUICK_MAX_AGE_DAYS)
}

/// Nut "Xoa lich su".
pub fn clear(store: &Store) -> rlt_store::Result<usize> {
    store.clear_quick_translations()
}

/// Muc moi nhat truoc, loc theo origin neu can.
pub fn recent(store: &Store, origin: Option<Origin>, limit: i64) -> rlt_store::Result<Vec<rlt_store::QuickTranslation>> {
    let all = store.list_quick_translations(limit.max(1) * 3)?;
    Ok(all.into_iter().filter(|q| origin.map(|o| q.origin == o.as_db()).unwrap_or(true)).take(limit as usize).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockTranslator;
    use crate::queue::TranslateQueue;
    use crate::translator::TranslateRequest;
    use crate::TranslateError;

    #[test]
    fn chi_ghi_khi_thanh_cong_va_dung_origin() {
        let store = Store::open_in_memory().unwrap();
        let q = TranslateQueue::new();
        q.submit(Origin::Quick, TranslateRequest::new("hello", None, "pt-PT")).unwrap();
        q.submit(Origin::Ocr, TranslateRequest::new("menu", Some("en"), "pt-PT")).unwrap();
        let m = MockTranslator::new();
        while let Some(o) = q.run_next(&m) {
            assert!(record_outcome(&store, &o).unwrap());
        }
        let l = recent(&store, None, 10).unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(recent(&store, Some(Origin::Ocr), 10).unwrap().len(), 1);
        let quick = recent(&store, Some(Origin::Quick), 10).unwrap();
        assert_eq!(quick[0].source_text, "hello");
        assert_eq!(quick[0].translated_text, "[pt-PT] hello");
        assert_eq!(quick[0].detected_lang.as_deref(), Some("en"), "nguon Tu nhan luu ngon ngu nhan duoc");
    }

    #[test]
    fn loi_va_huy_khong_vao_lich_su() {
        let store = Store::open_in_memory().unwrap();
        let q = TranslateQueue::new();
        q.submit(Origin::Quick, TranslateRequest::new("x", None, "en")).unwrap();
        let m = MockTranslator::failing(TranslateError::Network);
        let o = q.run_next(&m).unwrap();
        assert!(!record_outcome(&store, &o).unwrap());
        assert!(!record_outcome(&store, &JobOutcome::Cancelled { job: 1, origin: Origin::Quick }).unwrap());
        assert!(recent(&store, None, 10).unwrap().is_empty());
    }

    #[test]
    fn xoa_lich_su_sach() {
        let store = Store::open_in_memory().unwrap();
        let q = TranslateQueue::new();
        for i in 0..5 {
            q.submit(Origin::Quick, TranslateRequest::new(format!("t{i}"), None, "en")).unwrap();
            let o = q.run_next(&MockTranslator::new()).unwrap();
            record_outcome(&store, &o).unwrap();
        }
        assert_eq!(clear(&store).unwrap(), 5);
        assert!(recent(&store, None, 10).unwrap().is_empty());
        assert_eq!(purge_expired(&store).unwrap(), 0);
    }
}
