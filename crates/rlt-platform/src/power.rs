//! US-54: giữ máy không ngủ khi đang ghi.
//!
//! macOS: IOKit `IOPMAssertionCreateWithName` với `PreventUserIdleSystemSleep` (ADR-7, rà soát 7.6).
//! Quy tắc của rà soát: tạo và giải phóng assertion từ CÙNG MỘT luồng sống lâu. `PowerKeeper` sở hữu
//! một luồng riêng; mọi `acquire`/`release` được gửi sang luồng đó và backend chỉ được chạm tới ở đó.
//! Assertion còn gắn với tiến trình: app thoát hoặc sập thì hệ điều hành tự gỡ (AC-54.3).
//!
//! Logic quản lý assertion nằm sau trait `PowerBackend` để test không cần IOKit.

use std::fmt;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{self, JoinHandle};

/// Lỗi của lớp giữ máy thức.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerError {
    /// Hệ điều hành từ chối tạo assertion (mã trả về của IOKit).
    Create(String),
    /// Hệ điều hành báo lỗi khi giải phóng.
    Release(String),
    /// Luồng giữ máy thức đã chết.
    KeeperGone,
}

impl fmt::Display for PowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PowerError::Create(m) => write!(f, "khong tao duoc assertion giu may thuc: {m}"),
            PowerError::Release(m) => write!(f, "khong giai phong duoc assertion: {m}"),
            PowerError::KeeperGone => write!(f, "luong giu may thuc da dung"),
        }
    }
}

impl std::error::Error for PowerError {}

/// Cổng tới API chặn ngủ của hệ điều hành. Không cần `Send`: backend được tạo và dùng trong một luồng.
pub trait PowerBackend {
    type Handle;
    fn create(&mut self, reason: &str) -> Result<Self::Handle, PowerError>;
    fn release(&mut self, handle: Self::Handle) -> Result<(), PowerError>;
}

enum Cmd {
    Acquire(String, Sender<Result<(), PowerError>>),
    Release(Sender<Result<(), PowerError>>),
    Query(Sender<bool>),
    Shutdown,
}

/// Giữ máy không ngủ. Idempotent: `acquire` khi đang giữ không tạo assertion thứ hai;
/// `release` khi không giữ không làm gì. `Drop` giải phóng rồi dừng luồng.
pub struct PowerKeeper {
    tx: Sender<Cmd>,
    thread: Option<JoinHandle<()>>,
}

impl PowerKeeper {
    /// `factory` chạy TRONG luồng giữ máy thức, nên backend không cần `Send`.
    pub fn spawn<B, F>(factory: F) -> Self
    where
        B: PowerBackend + 'static,
        F: FnOnce() -> B + Send + 'static,
    {
        let (tx, rx) = channel();
        let thread = thread::Builder::new()
            .name("rlt-power".into())
            .spawn(move || run_keeper(factory(), rx))
            .expect("khong tao duoc luong rlt-power");
        Self { tx, thread: Some(thread) }
    }

    /// Bắt đầu giữ máy thức (khi phiên chuyển sang Recording).
    pub fn acquire(&self, reason: &str) -> Result<(), PowerError> {
        let (rtx, rrx) = channel();
        self.tx.send(Cmd::Acquire(reason.to_string(), rtx)).map_err(|_| PowerError::KeeperGone)?;
        rrx.recv().map_err(|_| PowerError::KeeperGone)?
    }

    /// Thôi giữ (khi Dừng, hoặc khi người dùng tắt công tắc).
    pub fn release(&self) -> Result<(), PowerError> {
        let (rtx, rrx) = channel();
        self.tx.send(Cmd::Release(rtx)).map_err(|_| PowerError::KeeperGone)?;
        rrx.recv().map_err(|_| PowerError::KeeperGone)?
    }

    /// Đang giữ assertion hay không.
    pub fn is_held(&self) -> bool {
        let (rtx, rrx) = channel();
        if self.tx.send(Cmd::Query(rtx)).is_err() {
            return false;
        }
        rrx.recv().unwrap_or(false)
    }
}

impl Drop for PowerKeeper {
    fn drop(&mut self) {
        let _ = self.tx.send(Cmd::Shutdown);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn run_keeper<B: PowerBackend>(mut backend: B, rx: Receiver<Cmd>) {
    let mut held: Option<B::Handle> = None;
    while let Ok(cmd) = rx.recv() {
        match cmd {
            Cmd::Acquire(reason, reply) => {
                let res = if held.is_some() {
                    Ok(())
                } else {
                    backend.create(&reason).map(|h| {
                        held = Some(h);
                    })
                };
                let _ = reply.send(res);
            }
            Cmd::Release(reply) => {
                let res = match held.take() {
                    Some(h) => backend.release(h),
                    None => Ok(()),
                };
                let _ = reply.send(res);
            }
            Cmd::Query(reply) => {
                let _ = reply.send(held.is_some());
            }
            Cmd::Shutdown => break,
        }
    }
    // Thoát vòng (Shutdown hoặc mọi Sender đã rơi): giải phóng ngay trong chính luồng này.
    if let Some(h) = held.take() {
        let _ = backend.release(h);
    }
}

/// Backend của nền tảng hiện tại: IOKit trên macOS, không làm gì ở nơi khác (Windows có
/// `SetThreadExecutionState` ở giai đoạn Windows, không thuộc M1 macOS).
#[cfg(target_os = "macos")]
pub type SystemBackend = iokit::IoKitBackend;
#[cfg(not(target_os = "macos"))]
pub type SystemBackend = NoopBackend;

/// Backend rỗng cho nền tảng chưa cài.
#[derive(Default)]
pub struct NoopBackend;

impl PowerBackend for NoopBackend {
    type Handle = ();
    fn create(&mut self, _reason: &str) -> Result<(), PowerError> {
        Ok(())
    }
    fn release(&mut self, _handle: ()) -> Result<(), PowerError> {
        Ok(())
    }
}

/// `PowerKeeper` dùng backend thật của nền tảng.
pub fn system_keeper() -> PowerKeeper {
    PowerKeeper::spawn(SystemBackend::default)
}

#[cfg(target_os = "macos")]
pub mod iokit {
    //! FFI tay tới IOKit và CoreFoundation (không thêm crate). `IOPMAssertionID` là `u32`.
    use super::{PowerBackend, PowerError};
    use std::ffi::{c_char, c_void, CString};

    type CFStringRef = *const c_void;
    type IOReturn = i32;
    type IOPMAssertionID = u32;

    const K_IOPM_ASSERTION_LEVEL_ON: u32 = 255;
    const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
    /// Giữ HỆ THỐNG không ngủ vì không thao tác; KHÔNG chặn tắt màn hình (rà soát 7.6, R19).
    const ASSERTION_TYPE: &str = "PreventUserIdleSystemSleep";

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IOPMAssertionCreateWithName(
            assertion_type: CFStringRef,
            level: u32,
            name: CFStringRef,
            id: *mut IOPMAssertionID,
        ) -> IOReturn;
        fn IOPMAssertionRelease(id: IOPMAssertionID) -> IOReturn;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFStringCreateWithCString(alloc: *const c_void, c: *const c_char, enc: u32) -> CFStringRef;
        fn CFRelease(cf: *const c_void);
    }

    struct CfString(CFStringRef);

    impl CfString {
        fn new(s: &str) -> Option<Self> {
            let c = CString::new(s).ok()?;
            // SAFETY: `c` sống tới hết lời gọi; allocator null = mặc định; hàm sao chép chuỗi.
            let r = unsafe { CFStringCreateWithCString(std::ptr::null(), c.as_ptr(), K_CF_STRING_ENCODING_UTF8) };
            (!r.is_null()).then_some(Self(r))
        }
    }

    impl Drop for CfString {
        fn drop(&mut self) {
            // SAFETY: con trỏ do CFStringCreateWithCString trả về, chưa giải phóng.
            unsafe { CFRelease(self.0) }
        }
    }

    #[derive(Default)]
    pub struct IoKitBackend;

    /// Id assertion; không `Clone`/`Copy` để mỗi assertion chỉ được giải phóng một lần.
    pub struct AssertionId(IOPMAssertionID);

    impl PowerBackend for IoKitBackend {
        type Handle = AssertionId;

        fn create(&mut self, reason: &str) -> Result<AssertionId, PowerError> {
            let ty = CfString::new(ASSERTION_TYPE).ok_or_else(|| PowerError::Create("CFString".into()))?;
            let name = CfString::new(reason).ok_or_else(|| PowerError::Create("CFString".into()))?;
            let mut id: IOPMAssertionID = 0;
            // SAFETY: hai CFString hợp lệ trong suốt lời gọi, `id` là con trỏ ghi hợp lệ.
            let r = unsafe { IOPMAssertionCreateWithName(ty.0, K_IOPM_ASSERTION_LEVEL_ON, name.0, &mut id) };
            if r == 0 {
                Ok(AssertionId(id))
            } else {
                Err(PowerError::Create(format!("IOReturn {r:#x}")))
            }
        }

        fn release(&mut self, handle: AssertionId) -> Result<(), PowerError> {
            // SAFETY: `handle.0` do `create` cấp và bị tiêu thụ ở đây (chỉ giải phóng một lần).
            let r = unsafe { IOPMAssertionRelease(handle.0) };
            if r == 0 {
                Ok(())
            } else {
                Err(PowerError::Release(format!("IOReturn {r:#x}")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread::ThreadId;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Ev {
        Create(ThreadId, String),
        Release(ThreadId, u32),
    }

    #[derive(Clone, Default)]
    struct Log(Arc<Mutex<Vec<Ev>>>);
    impl Log {
        fn events(&self) -> Vec<Ev> {
            self.0.lock().unwrap().clone()
        }
    }

    struct Mock {
        log: Log,
        next: u32,
        fail_creates: u32,
    }

    impl PowerBackend for Mock {
        type Handle = u32;
        fn create(&mut self, reason: &str) -> Result<u32, PowerError> {
            if self.fail_creates > 0 {
                self.fail_creates -= 1;
                return Err(PowerError::Create("mock tu choi".into()));
            }
            self.next += 1;
            self.log.0.lock().unwrap().push(Ev::Create(thread::current().id(), reason.into()));
            Ok(self.next)
        }
        fn release(&mut self, h: u32) -> Result<(), PowerError> {
            self.log.0.lock().unwrap().push(Ev::Release(thread::current().id(), h));
            Ok(())
        }
    }

    fn keeper(fail_creates: u32) -> (PowerKeeper, Log) {
        let log = Log::default();
        let l = log.clone();
        (PowerKeeper::spawn(move || Mock { log: l, next: 0, fail_creates }), log)
    }

    #[test]
    fn create_and_release_run_on_the_same_dedicated_thread() {
        let (k, log) = keeper(0);
        k.acquire("ghi am").unwrap();
        assert!(k.is_held());
        k.release().unwrap();
        assert!(!k.is_held());
        let ev = log.events();
        assert_eq!(ev.len(), 2);
        let (Ev::Create(t1, _), Ev::Release(t2, h)) = (&ev[0], &ev[1]) else { panic!("{ev:?}") };
        assert_eq!(t1, t2, "tao va giai phong phai cung mot luong");
        assert_ne!(*t1, thread::current().id(), "phai la luong rieng, khong phai luong goi");
        assert_eq!(*h, 1);
    }

    #[test]
    fn calls_from_many_threads_still_hit_the_same_keeper_thread() {
        let (k, log) = keeper(0);
        let k = Arc::new(k);
        let k2 = k.clone();
        thread::spawn(move || k2.acquire("a").unwrap()).join().unwrap();
        let k3 = k.clone();
        thread::spawn(move || k3.release().unwrap()).join().unwrap();
        let ev = log.events();
        let (Ev::Create(t1, _), Ev::Release(t2, _)) = (&ev[0], &ev[1]) else { panic!() };
        assert_eq!(t1, t2);
    }

    #[test]
    fn acquire_is_idempotent_and_makes_one_assertion() {
        let (k, log) = keeper(0);
        k.acquire("a").unwrap();
        k.acquire("b").unwrap();
        k.acquire("c").unwrap();
        assert_eq!(log.events().len(), 1);
        k.release().unwrap();
        k.release().unwrap(); // lan hai khong lam gi
        assert_eq!(log.events().len(), 2);
    }

    #[test]
    fn release_without_acquire_is_a_noop() {
        let (k, log) = keeper(0);
        k.release().unwrap();
        assert!(log.events().is_empty());
        assert!(!k.is_held());
    }

    #[test]
    fn failed_create_is_reported_and_not_held_then_retry_works() {
        let (k, log) = keeper(1);
        assert!(matches!(k.acquire("x"), Err(PowerError::Create(_))));
        assert!(!k.is_held());
        k.acquire("x").unwrap();
        assert!(k.is_held());
        assert_eq!(log.events().len(), 1);
    }

    #[test]
    fn each_new_session_gets_a_fresh_assertion_id() {
        let (k, log) = keeper(0);
        for _ in 0..3 {
            k.acquire("phien").unwrap();
            k.release().unwrap();
        }
        let ids: Vec<u32> = log.events().iter().filter_map(|e| if let Ev::Release(_, h) = e { Some(*h) } else { None }).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn drop_releases_a_held_assertion_on_the_keeper_thread() {
        let (k, log) = keeper(0);
        k.acquire("a").unwrap();
        drop(k);
        let ev = log.events();
        assert_eq!(ev.len(), 2);
        let (Ev::Create(t1, _), Ev::Release(t2, _)) = (&ev[0], &ev[1]) else { panic!() };
        assert_eq!(t1, t2);
    }

    #[test]
    fn drop_without_hold_releases_nothing() {
        let (k, log) = keeper(0);
        drop(k);
        assert!(log.events().is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn iokit_backend_creates_and_releases_a_real_assertion() {
        // IOKit chạy được không cần GUI hay quyền; đây là assertion thật, giải phóng ngay.
        let k = PowerKeeper::spawn(iokit::IoKitBackend::default);
        k.acquire("rlt test: giu may thuc").unwrap();
        assert!(k.is_held());
        k.release().unwrap();
        assert!(!k.is_held());
    }
}
