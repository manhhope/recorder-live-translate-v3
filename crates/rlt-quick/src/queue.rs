//! Hang doi dich (AC-31.9, AC-30.8). Quy tac:
//! - Moi `Origin` (Quick, Inline, Ocr) chi co TOI DA MOT yeu cau dang cho: yeu cau moi thay yeu cau cho cu cua
//!   cung origin (bao `Superseded`), va huy yeu cau dang chay cua origin do (ket qua cu bi bo, khong hien nham).
//! - Esc / dong cua so goi `cancel_origin`: huy ca dang cho lan dang chay.
//! - Yeu cau vuot 5.000 ky tu hoac rong bi tu choi NGAY o `submit`, khong vao hang doi (0 yeu cau gui di).
//! - Dang chay xong ma da bi huy thi ket qua bi bo (`Cancelled`), du nha cung cap co tra ve.
//! - Loi giu nguyen chu nguoi dung da nhap (AC-31.6): `Failed` mang theo yeu cau goc de giao dien khong mat chu.

use crate::error::TranslateError;
use crate::translator::{TextTranslator, TranslateRequest, Translation};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

#[derive(Clone, Debug, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// Duong vao cua yeu cau; khop gia tri CHECK cua cot `quick_translations.origin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Origin {
    Quick,
    Inline,
    Ocr,
}

impl Origin {
    pub fn as_db(self) -> &'static str {
        match self {
            Origin::Quick => "quick_translate",
            Origin::Inline => "inline_translate",
            Origin::Ocr => "ocr_translate",
        }
    }
}

pub type JobId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobOutcome {
    Done { job: JobId, origin: Origin, request: TranslateRequest, translation: Translation },
    /// Loi nha cung cap; `request` con nguyen de khong mat chu (AC-31.6).
    Failed { job: JobId, origin: Origin, request: TranslateRequest, error: TranslateError },
    /// Bi huy luc dang cho hoac dang chay (Esc).
    Cancelled { job: JobId, origin: Origin },
    /// Bi yeu cau moi cung origin thay the khi con dang cho.
    Superseded { job: JobId, origin: Origin },
}

impl JobOutcome {
    pub fn job(&self) -> JobId {
        match self {
            JobOutcome::Done { job, .. }
            | JobOutcome::Failed { job, .. }
            | JobOutcome::Cancelled { job, .. }
            | JobOutcome::Superseded { job, .. } => *job,
        }
    }
}

struct Pending {
    job: JobId,
    origin: Origin,
    req: TranslateRequest,
    cancel: CancelToken,
}

#[derive(Default)]
struct State {
    next_id: JobId,
    pending: VecDeque<Pending>,
    running: Option<(JobId, Origin, CancelToken)>,
    /// Ket qua "bi thay the" chua ai nhan (tra qua `drain_dropped`).
    dropped: Vec<JobOutcome>,
    closed: bool,
}

#[derive(Default)]
pub struct TranslateQueue {
    state: Mutex<State>,
    cv: Condvar,
}

impl TranslateQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Nop mot yeu cau. Tra ve `JobId`, hoac loi dau vao (khong vao hang doi).
    pub fn submit(&self, origin: Origin, req: TranslateRequest) -> Result<JobId, TranslateError> {
        req.validate()?;
        let mut st = self.state.lock().unwrap();
        if st.closed {
            return Err(TranslateError::Cancelled);
        }
        // Thay yeu cau cho cu cung origin.
        let mut kept = VecDeque::new();
        let mut dropped = Vec::new();
        for p in st.pending.drain(..) {
            if p.origin == origin {
                p.cancel.cancel();
                dropped.push(JobOutcome::Superseded { job: p.job, origin });
            } else {
                kept.push_back(p);
            }
        }
        st.pending = kept;
        st.dropped.extend(dropped);
        // Huy yeu cau dang chay cung origin: ket qua cua no khong con dung.
        if let Some((_, o, tok)) = &st.running {
            if *o == origin {
                tok.cancel();
            }
        }
        st.next_id += 1;
        let job = st.next_id;
        st.pending.push_back(Pending { job, origin, req, cancel: CancelToken::new() });
        self.cv.notify_all();
        Ok(job)
    }

    /// Huy moi yeu cau cua mot origin (Esc, AC-31.9). Tra cac ket qua `Cancelled` cho yeu cau dang cho.
    pub fn cancel_origin(&self, origin: Origin) -> Vec<JobOutcome> {
        let mut st = self.state.lock().unwrap();
        let mut out = Vec::new();
        let mut kept = VecDeque::new();
        for p in st.pending.drain(..) {
            if p.origin == origin {
                p.cancel.cancel();
                out.push(JobOutcome::Cancelled { job: p.job, origin });
            } else {
                kept.push_back(p);
            }
        }
        st.pending = kept;
        if let Some((_, o, tok)) = &st.running {
            if *o == origin {
                tok.cancel();
            }
        }
        out
    }

    pub fn cancel_all(&self) -> Vec<JobOutcome> {
        let mut out = Vec::new();
        for o in [Origin::Quick, Origin::Inline, Origin::Ocr] {
            out.extend(self.cancel_origin(o));
        }
        out
    }

    /// Lay cac yeu cau da bi thay the (de bao ve giao dien, thuong bo qua).
    pub fn drain_dropped(&self) -> Vec<JobOutcome> {
        std::mem::take(&mut self.state.lock().unwrap().dropped)
    }

    pub fn pending_len(&self) -> usize {
        self.state.lock().unwrap().pending.len()
    }

    pub fn is_running(&self) -> bool {
        self.state.lock().unwrap().running.is_some()
    }

    /// Dong hang doi: moi `submit` sau do bi tu choi, `run_next_blocking` tra `None` khi het viec.
    pub fn close(&self) {
        let mut st = self.state.lock().unwrap();
        st.closed = true;
        for p in &st.pending {
            p.cancel.cancel();
        }
        if let Some((_, _, tok)) = &st.running {
            tok.cancel();
        }
        self.cv.notify_all();
    }

    fn take_next(&self, block: bool) -> Option<Pending> {
        let mut st = self.state.lock().unwrap();
        loop {
            if let Some(p) = st.pending.pop_front() {
                st.running = Some((p.job, p.origin, p.cancel.clone()));
                return Some(p);
            }
            if !block || st.closed {
                return None;
            }
            st = self.cv.wait(st).unwrap();
        }
    }

    fn run_one(&self, p: Pending, translator: &dyn TextTranslator) -> JobOutcome {
        let res = if p.cancel.is_cancelled() { Err(TranslateError::Cancelled) } else { translator.translate(&p.req, &p.cancel) };
        self.state.lock().unwrap().running = None;
        // Ket qua den sau khi bi huy thi bo.
        if p.cancel.is_cancelled() {
            return JobOutcome::Cancelled { job: p.job, origin: p.origin };
        }
        match res {
            Ok(translation) => JobOutcome::Done { job: p.job, origin: p.origin, request: p.req, translation },
            Err(TranslateError::Cancelled) => JobOutcome::Cancelled { job: p.job, origin: p.origin },
            Err(error) => JobOutcome::Failed { job: p.job, origin: p.origin, request: p.req, error },
        }
    }

    /// Chay MOT yeu cau dang cho (khong cho neu rong). Dung trong test va vong lap don gian.
    pub fn run_next(&self, translator: &dyn TextTranslator) -> Option<JobOutcome> {
        let p = self.take_next(false)?;
        Some(self.run_one(p, translator))
    }

    /// Cho den khi co viec hoac hang doi dong.
    pub fn run_next_blocking(&self, translator: &dyn TextTranslator) -> Option<JobOutcome> {
        let p = self.take_next(true)?;
        Some(self.run_one(p, translator))
    }
}

/// Luong nen: lap `run_next_blocking`, goi `on_outcome` cho moi ket qua. `stop()` dong hang doi va doi luong.
pub struct Worker {
    queue: Arc<TranslateQueue>,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn spawn<F>(queue: Arc<TranslateQueue>, translator: Arc<dyn TextTranslator>, on_outcome: F) -> Self
    where
        F: Fn(JobOutcome) + Send + 'static,
    {
        let q = queue.clone();
        let handle = std::thread::Builder::new()
            .name("rlt-quick-translate".into())
            .spawn(move || {
                while let Some(o) = q.run_next_blocking(translator.as_ref()) {
                    on_outcome(o);
                }
            })
            .expect("khong tao duoc luong dich");
        Self { queue, handle: Some(handle) }
    }

    pub fn stop(mut self) {
        self.queue.close();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockTranslator;
    use crate::translator::{TextTranslator, TranslatorStatus, MAX_CHARS};
    use std::sync::mpsc;
    use std::time::Duration;

    fn req(t: &str) -> TranslateRequest {
        TranslateRequest::new(t, Some("en"), "pt-PT")
    }

    #[test]
    fn chay_tuan_tu_cac_origin_khac_nhau_theo_thu_tu_nop() {
        let q = TranslateQueue::new();
        let m = MockTranslator::new();
        q.submit(Origin::Quick, req("a")).unwrap();
        q.submit(Origin::Inline, req("b")).unwrap();
        q.submit(Origin::Ocr, req("c")).unwrap();
        let mut got = Vec::new();
        while let Some(o) = q.run_next(&m) {
            if let JobOutcome::Done { translation, .. } = o {
                got.push(translation.text);
            }
        }
        assert_eq!(got, vec!["[pt-PT] a", "[pt-PT] b", "[pt-PT] c"]);
        assert!(q.run_next(&m).is_none());
    }

    #[test]
    fn yeu_cau_moi_cung_origin_thay_yeu_cau_dang_cho() {
        let q = TranslateQueue::new();
        let m = MockTranslator::new();
        let j1 = q.submit(Origin::Quick, req("cu")).unwrap();
        let j2 = q.submit(Origin::Quick, req("moi")).unwrap();
        assert_eq!(q.pending_len(), 1);
        assert_eq!(q.drain_dropped(), vec![JobOutcome::Superseded { job: j1, origin: Origin::Quick }]);
        match q.run_next(&m).unwrap() {
            JobOutcome::Done { job, translation, .. } => {
                assert_eq!(job, j2);
                assert_eq!(translation.text, "[pt-PT] moi");
            }
            o => panic!("{o:?}"),
        }
        assert_eq!(m.calls(), 1, "yeu cau cu khong duoc gui di");
    }

    #[test]
    fn esc_huy_yeu_cau_dang_cho_va_khong_goi_nha_cung_cap() {
        let q = TranslateQueue::new();
        let m = MockTranslator::new();
        let j = q.submit(Origin::Quick, req("x")).unwrap();
        let o = q.cancel_origin(Origin::Quick);
        assert_eq!(o, vec![JobOutcome::Cancelled { job: j, origin: Origin::Quick }]);
        assert!(q.run_next(&m).is_none());
        assert_eq!(m.calls(), 0);
    }

    /// Dich cham, cho phep huy giua chung.
    struct SlowT(Mutex<Option<(mpsc::Sender<()>, mpsc::Receiver<()>)>>);
    impl TextTranslator for SlowT {
        fn translate(&self, r: &TranslateRequest, _c: &CancelToken) -> Result<Translation, TranslateError> {
            let (tx, rx) = self.0.lock().unwrap().take().map(|s| (s.0, s.1)).unwrap();
            tx.send(()).unwrap(); // bao "da bat dau"
            rx.recv().unwrap(); // cho phep tiep
            Ok(Translation { text: r.text.clone(), detected_lang: None })
        }
        fn status(&self) -> TranslatorStatus {
            TranslatorStatus::Mock
        }
    }

    #[test]
    fn huy_luc_dang_chay_bo_ket_qua_ve_sau() {
        let q = Arc::new(TranslateQueue::new());
        let (started_tx, started_rx) = mpsc::channel();
        let (go_tx, go_rx) = mpsc::channel();
        let t = Arc::new(SlowT(Mutex::new(Some((started_tx, go_rx)))));
        let j = q.submit(Origin::Quick, req("dang chay")).unwrap();
        let (q2, t2) = (q.clone(), t.clone());
        let h = std::thread::spawn(move || q2.run_next(t2.as_ref()).unwrap());
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(q.is_running());
        q.cancel_origin(Origin::Quick);
        go_tx.send(()).unwrap();
        assert_eq!(h.join().unwrap(), JobOutcome::Cancelled { job: j, origin: Origin::Quick });
        assert!(!q.is_running());
    }

    #[test]
    fn nop_yeu_cau_moi_trong_luc_chay_huy_ket_qua_cu() {
        let q = Arc::new(TranslateQueue::new());
        let (started_tx, started_rx) = mpsc::channel();
        let (go_tx, go_rx) = mpsc::channel();
        let t = Arc::new(SlowT(Mutex::new(Some((started_tx, go_rx)))));
        let j1 = q.submit(Origin::Quick, req("cu")).unwrap();
        let (q2, t2) = (q.clone(), t.clone());
        let h = std::thread::spawn(move || q2.run_next(t2.as_ref()).unwrap());
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let j2 = q.submit(Origin::Quick, req("moi")).unwrap();
        go_tx.send(()).unwrap();
        assert_eq!(h.join().unwrap(), JobOutcome::Cancelled { job: j1, origin: Origin::Quick });
        // Yeu cau moi van con trong hang doi de chay tiep.
        let m = MockTranslator::new();
        assert_eq!(q.run_next(&m).unwrap().job(), j2);
    }

    #[test]
    fn loi_giu_nguyen_chu_da_nhap_va_dung_ma_loi() {
        for err in [TranslateError::Unauthorized, TranslateError::QuotaExceeded, TranslateError::Timeout, TranslateError::Network] {
            let q = TranslateQueue::new();
            let m = MockTranslator::failing(err.clone());
            q.submit(Origin::Quick, req("giu lai chu nay")).unwrap();
            match q.run_next(&m).unwrap() {
                JobOutcome::Failed { request, error, .. } => {
                    assert_eq!(request.text, "giu lai chu nay");
                    assert_eq!(error, err);
                }
                o => panic!("{o:?}"),
            }
        }
    }

    #[test]
    fn dau_vao_sai_bi_chan_o_cua_khong_vao_hang_doi() {
        let q = TranslateQueue::new();
        assert!(q.submit(Origin::Quick, req(&"a".repeat(MAX_CHARS))).is_ok());
        let over = q.submit(Origin::Inline, req(&"a".repeat(MAX_CHARS + 1)));
        assert_eq!(over, Err(TranslateError::TooLong { chars: MAX_CHARS + 1 }));
        assert_eq!(q.submit(Origin::Ocr, req("   ")), Err(TranslateError::Empty));
        assert_eq!(q.pending_len(), 1);
    }

    #[test]
    fn worker_nen_tra_ket_qua_va_dung_gon() {
        let q = Arc::new(TranslateQueue::new());
        let (tx, rx) = mpsc::channel();
        let w = Worker::spawn(q.clone(), Arc::new(MockTranslator::new()), move |o| {
            let _ = tx.send(o);
        });
        let j = q.submit(Origin::Quick, req("xin chao")).unwrap();
        let o = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(o.job(), j);
        assert!(matches!(o, JobOutcome::Done { .. }));
        w.stop();
        assert_eq!(q.submit(Origin::Quick, req("sau khi dong")), Err(TranslateError::Cancelled));
    }
}
