# Phien chet som (0,1 giay): nguyen nhan, cach sua, kiem chung

Task t_ef195a0074ce. Ngay 2026-11-14.

## 1. Nguyen nhan that (co bang chung)

Bang chung tu DB that `~/Library/Application Support/com.mmola.recorderlivetranslate/rlt.db`:

| session_id (rut gon) | status | started_at | ended_at | duration_ms | mode |
|---|---|---|---|---|---|
| 5b4cac91 | interrupted | 16:54:55.098 | 16:54:55.236 | 0 | mixed |
| 8533ff5c | interrupted | 16:54:03.248 | 16:54:03.366 | 0 | mixed |
| a162d641 | interrupted | 16:53:45.378 | 16:53:45.502 | 0 | mixed |
| 1686118c | interrupted | 16:53:44.828 | 16:53:44.939 | 0 | mixed |
| 7c0c51a7 | interrupted | 16:53:43.832 | 16:53:43.947 | 0 | mixed |
| e0cc2e08 | interrupted | 16:53:30.479 | 16:53:30.719 | 0 | mixed |

`select count(*) from usage_logs` = 0, `sessions` = 6. Ca 6 phien deu o che do `mixed`, chet sau 0,11 den 0,24 giay.

### 1a. Nguyen nhan DA CHUNG MINH (cap nhat luot nghiem thu)

Nguyen nhan that: **may chay app (Mac mini) khong co thiet bi micro nao**. Che do `mixed` mo nguon he thong truoc (xong sau khoang 0,13 giay) roi mo micro, micro that bai ngay (0 ms), `MixedSource::start` dung nguon he thong va tra loi, `start_session` ghi `interrupted`, duration 0. Khong phai quyen Ghi man hinh, khong phai crash.

Bang chung do truc tiep tren chinh may nay (14/11/2026), bang chuong trinh tham do tam goi dung `PlatformSources.open(..).start(..)` (da xoa sau khi chay, khong nam trong diff):

```
mic_permission = NotDetermined
System: start -> Ok(()) after 135 ms
Microphone: start -> Err(Device("An error unknown to the coreaudio-rs API occurred, OSStatus: 560947818")) after 0 ms
```

- `system_profiler SPAudioDataType` liet ke chi thiet bi OUTPUT (DELL U2424HE, RTK, Mac mini Speakers); dem tu khoa "Input" = 0. May khong co thiet bi vao.
- OSStatus 560947818 = 0x216F626A = `'!obj'` (kAudioHardwareBadObjectError): CoreAudio khong co doi tuong thiet bi vao hop le. Loi tra o `device.default_input_config()` (`mic.rs`).
- Thoi gian khop DB: system ~135 ms + mic 0 ms ~ 0,11 den 0,24 giay `ended_at - started_at` cua 6 phien.
- Nguon he thong (ScreenCaptureKit) mo THANH CONG tren may nay (135 ms, Ok), nen quyen Ghi man hinh KHONG phai nguyen nhan; ket luan truoc ("nghi quyen Ghi man hinh") sai. `captions.log` cu cung cho thay `tokens_system` co du lieu ngay 14/11 07:44.
- Cac file crash `.ips` (`rlt-app-2026-11-14-1854xx...1907xx`, SIGABRT trong `tao::...control_flow_end_handler` / `rust_panic_cleanup`) la panic cua vong lap su kien giao dien (viec t_301edccbd6f9), khong lien quan: khong cung co che, khong nam trong `start_session`.
- Loi cu bi gan nhan chung "audio_device"/`permission` nen nguoi dung khong biet. Sau sua: `mic.rs` dat thong diep `khong co micro mac dinh dung duoc: ...` nen `audio_start_code` tra `no_mic`, giao dien hien ro "khong co micro".

Gioi han: probe chay tu terminal (quyen TCC cua terminal, `mic_permission = NotDetermined`), khong phai tu goi `.app`. Viec "khong co thiet bi vao" khong phu thuoc quyen nen ket luan van dung cho app; con thong bao tren giao dien Tauri thi CHUA DO (muc 5).

### 1b. Phan tich code (tu luot truoc)

Duong code (`crates/rlt-core/src/session.rs`, `start_session`): ham tao hang `sessions`, roi goi `source.start(sink)`. Neu nguon am loi, no ghi `finish_session(id, "interrupted", now, 0)` va tra `Err`. Khoang cach started_at/ended_at 0,1 giay chinh la thoi gian `MixedSource::start` mo nguon he thong (`SCShareableContent::get`, ScreenCaptureKit) roi that bai. Mac dinh `system.rs` gan tien to `permission:` cho loi nay (thieu quyen Ghi man hinh), nhung probe 1a cho thay buoc that bai that la MICRO, khong phai he thong. `rlt.db` khong luu ma loi va luc do app chua ghi log loi, nen 1a moi la bang chung.

Phan biet hai loai `interrupted`:

- Loi luc bat dau: `duration_ms = 0`, `ended_at` cach `started_at` ~0,1 giay, sinh ra tu nhanh `source.start` o tren. Day la truong hop cua 6 phien tren.
- Bi cat giua phien (`Ending` khac `Stopped`, `session.rs:541`), co `duration_ms > 0` (loi Soniox khong thu lai duoc, het quota...).
- Khoi dong lai sau crash: da grep `interrupted` trong toan bo `crates`, `src-tauri/src`, `src`: KHONG co doan quet khoi dong nao danh dau phien dang do la `interrupted`. Nghia la hien khong co co che "interrupted do crash"; phien bi crash giua chung se ket o trang thai khong ket thuc, khong phai `interrupted`. Vi vay 6 dong tren khong lien quan crash t_301edccbd6f9. `docs/crash-mo-app-bao-cao.md` khong ton tai trong repo; `crash.log` khong co trong thu muc du lieu app luc dieu tra.

Tai sao nguoi dung thay "lang le ve Idle" va khong ro nguyen nhan:

1. `classify_audio` tra `Permission` cho MOI loi audio (ca `_ =>`), nen loi micro, khong co micro, thiet bi ban deu hien cung mot cau "thieu quyen".
2. Khong co log loi nao o `start`.
3. Man Usage Logs khi tuan khong co audio chi hien "No data this week", khong noi co N phien da chet luc bat dau.
4. `let _ = record_usage(..)` nuot loi ghi usage.

Dong usage trong: khong phai loi ghi usage. `pump` chi ghi usage khi `audio_ms > 0`; phien chet luc bat dau co `audio_ms = 0` nen dung la khong co dong nao. Phien co audio that: kiem bang test (muc 4), local_date lay theo mui gio may (`local_date_now`), da co test `usage_row_uses_local_date_not_utc` tu truoc.

## 2. Thay doi

- `session/service.rs`: `audio_start_code` tra ma cu the `permission_screen`, `permission_mic`, `no_mic`, `audio_unsupported`, `audio_device` thay vi luon la `permission`; `start()` tra ma do cho giao dien (Record.svelte da hien `errCode` qua `err.<ma>`). `log_event` ghi log co cau truc `ts= event= key="value"` ra stderr va `session-errors.log` (cat khi > 256 KB), chi chua ma loi va chuoi da qua `scrub_log` (che chuoi >= 20 ky tu giong khoa, cat 200 ky tu). Cac su kien: `session_start_failed`, `session_failed`, `store_error`, `usage_recorded`, `usage_record_failed`. Het `let _ = record_usage(..)`: loi duoc ghi log.
- `session/records.rs`: `WeekOut` them `sessionsCount` va `failedStartCount` (phien `interrupted` + duration 0 trong tuan, theo ngay may).
- `src/screens/UsageLogsTab.svelte`, `src/lib/api.ts`: khi tuan co phien nhung khong co audio, hien nguyen nhan ro rang.
- `src/locales/en.json`, `pt-PT.json`: 7 khoa moi moi ngon ngu (5 ma loi + 2 chuoi trang thai Usage). Du an chi co hai ngon ngu nay.
- Test hoi quy moi: `source_failing_at_start_returns_specific_code_and_marks_interrupted_zero_duration` (5 loai loi, kiem ma tra ve, phien `interrupted` duration 0, khong con active, `failedStartCount = 1`), `session_with_real_audio_writes_usage_row_visible_in_current_week` (audio_ms > 0 -> dong usage, local_date dung, hien o tuan hien tai), `audio_start_codes_are_specific`, `scrub_log_masks_key_like_tokens`; `api.test.ts` them 5 ma loi.

## 3. Diff

```
 src-tauri/src/session/integration_tests.rs |  78 +++++++++++++++++++
 src-tauri/src/session/records.rs           |  16 ++++
 src-tauri/src/session/service.rs           | 117 ++++++++++++++++++++++++++++-
 src/lib/api.test.ts                        |   2 +-
 src/lib/api.ts                             |   2 +-
 src/locales/en.json                        |   7 ++
 src/locales/pt-PT.json                     |   7 ++
 src/screens/UsageLogsTab.svelte            |   8 +-
 8 files changed, 230 insertions(+), 7 deletions(-)

```

Chi tiet (rut gon cac file chinh):

```diff
diff --git a/src-tauri/src/session/records.rs b/src-tauri/src/session/records.rs
index 9020071..42ec2ff 100644
--- a/src-tauri/src/session/records.rs
+++ b/src-tauri/src/session/records.rs
@@ -300,11 +300,27 @@ pub struct WeekOut {
     pub total_est_cost_usd_micros: i64,
     /// Con so luon la UOC TINH (AC-32.3).
     pub estimated: bool,
+    /// So phien bat dau trong tuan (theo ngay may), de man Usage Logs giai thich khi usage trong.
+    pub sessions_count: i64,
+    /// Trong do: phien chet luc bat dau (`interrupted`, duration 0) nen khong co audio de tinh.
+    pub failed_start_count: i64,
 }
 
 pub fn usage_week(store: &Store, any_date: &str, provider: Option<&str>) -> Res<WeekOut> {
     let (days, total) = usage::week_chart(store, any_date, provider).map_err(err)?;
+    let (first, last) = (days.first().map(|d| d.0.clone()).unwrap_or_default(), days.last().map(|d| d.0.clone()).unwrap_or_default());
+    let (sessions_count, failed_start_count): (i64, i64) = store
+        .conn()
+        .query_row(
+            "SELECT COUNT(*), COALESCE(SUM(status = 'interrupted' AND COALESCE(duration_ms, 0) = 0), 0) FROM sessions \
+             WHERE date(started_at, 'localtime') BETWEEN ?1 AND ?2",
+            [&first, &last],
+            |r| Ok((r.get(0)?, r.get(1)?)),
+        )
+        .map_err(|e| e.to_string())?;
     Ok(WeekOut {
+        sessions_count,
+        failed_start_count,
         days: days
             .into_iter()
             .map(|(d, t)| DayRow { local_date: d, audio_ms: t.audio_ms, est_cost_usd_micros: t.est_cost_usd_micros, calls: t.call_count })
diff --git a/src-tauri/src/session/service.rs b/src-tauri/src/session/service.rs
index f52176e..0506b62 100644
--- a/src-tauri/src/session/service.rs
+++ b/src-tauri/src/session/service.rs
@@ -144,6 +144,85 @@ pub fn classify_soniox(e: &SonioxError) -> ErrKind {
     }
 }
 
+/// Ma loi cu the khi KHONG MO DUOC nguon am luc bat dau phien (tra ve giao dien, khong ve Idle lang le).
+/// `system.rs` gan tien to `permission:` cho loi quyen Ghi man hinh; micro thi nhan dang qua noi dung loi.
+pub fn audio_start_code(e: &AudioError) -> &'static str {
+    match e {
+        AudioError::Unsupported(_) => "audio_unsupported",
+        AudioError::Device(m) => {
+            let l = m.to_lowercase();
+            if l.starts_with("permission") || l.contains("screen") && l.contains("permission") {
+                "permission_screen"
+            } else if l.contains("khong co micro") {
+                "no_mic"
+            } else if l.contains("denied") || l.contains("not authori") || l.contains("permission") {
+                "permission_mic"
+            } else {
+                "audio_device"
+            }
+        }
+    }
+}
+
+/// Che moi chuoi dai giong khoa (>= 20 ky tu chu/so/-/_) va cat ngan: log khong bao gio chua khoa hay bi mat.
+pub fn scrub_log(s: &str) -> String {
+    let mut out = String::new();
+    let mut tok = String::new();
+    let flush = |tok: &mut String, out: &mut String| {
+        if tok.chars().count() >= 20 {
+            out.push_str("[che]");
+        } else {
+            out.push_str(tok);
+        }
+        tok.clear();
+    };
+    for c in s.chars() {
+        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
+            tok.push(c);
+        } else {
+            flush(&mut tok, &mut out);
+            out.push(if c.is_control() { ' ' } else { c });
+        }
+    }
+    flush(&mut tok, &mut out);
+    out.chars().take(200).collect()
+}
+
+fn log_dir() -> Option<std::path::PathBuf> {
+    let home = std::path::PathBuf::from(std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?);
+    const ID: &str = "com.mmola.recorderlivetranslate";
+    if cfg!(target_os = "macos") {
+        Some(home.join("Library/Application Support").join(ID))
+    } else if cfg!(windows) {
+        std::env::var_os("APPDATA").map(|a| std::path::PathBuf::from(a).join(ID))
+    } else {
+        Some(home.join(".local/share").join(ID))
+    }
+}
+
+/// Mot dong log co cau truc `ts=.. event=.. k=v ...` ra stderr va `session-errors.log` (cat khi > 256 KB).
+/// Chi nhan ma loi va chuoi da `scrub_log`; khong bao gio nhan khoa Soniox.
+pub fn log_event(event: &str, fields: &[(&str, String)]) {
+    let ts = rlt_core::session::iso_now();
+    let mut line = format!("ts={ts} event={event}");
+    for (k, v) in fields {
+        line.push_str(&format!(" {k}=\"{}\"", scrub_log(v).replace('"', "'")));
+    }
+    eprintln!("{line}");
+    if cfg!(test) {
+        return; // test khong ghi vao thu muc du lieu that cua nguoi dung
+    }
+    let Some(dir) = log_dir() else { return };
+    let path = dir.join("session-errors.log");
+    if std::fs::metadata(&path).map(|m| m.len() > 256 * 1024).unwrap_or(false) {
+        let _ = std::fs::rename(&path, dir.join("session-errors.log.1"));
+    }
+    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
+        use std::io::Write;
+        let _ = writeln!(f, "{line}");
+    }
+}
+
 fn classify_audio(e: &AudioError) -> ErrKind {
     match e {
         AudioError::Device(m) if m.contains("permission") => ErrKind::Permission,
@@ -355,7 +434,11 @@ impl SessionService {
         let auth = self.read_key()?;
         let level_sink = sink.clone();
         let on_level: LevelFn = Arc::new(move |l| level_sink(UiEvent::Level { level: l }));
-        let (source, kind, label, activity) = self.build_source(p.mode, on_level).map_err(|e| classify_audio(&e).code().to_string())?;
+        let (source, kind, label, activity) = self.build_source(p.mode, on_level).map_err(|e| {
+            let code = audio_start_code(&e);
+            log_event("session_start_failed", &[("stage", "open_source".into()), ("mode", format!("{:?}", p.mode)), ("code", code.into()), ("detail", e.to_string())]);
+            code.to_string()
+        })?;
         let id = new_uuid();
         let title = p.title.unwrap_or_else(|| format!("Session {}", rlt_core::session::iso_now()[..16].replace('T', " ")));
         let mut cfg = SessionConfig::new(&id, title, &p.target_lang, self.soniox_settings(&p.target_lang, &p.listen_langs, self.glossary_context(&p.target_lang, &p.listen_langs)), auth);
@@ -367,7 +450,15 @@ impl SessionService {
         let (tx, rx) = mpsc::channel(64);
         let handle = match start_session(cfg, source, self.store.clone(), tx).await {
             Ok(h) => h,
-            Err(e) => return Err(classify_core(&e).code().to_string()),
+            Err(e) => {
+                // Nguon am mo loi sau khi dong phien da tao: phien nay ghi `interrupted`, duration 0 (khong phai do crash).
+                let code = match &e {
+                    CoreError::Audio(a) => audio_start_code(a),
+                    other => classify_core(other).code(),
+                };
+                log_event("session_start_failed", &[("stage", "start_session".into()), ("session", id.clone()), ("mode", format!("{:?}", p.mode)), ("code", code.into()), ("detail", e.to_string())]);
+                return Err(code.to_string());
+            }
         };
         *self.active.lock().unwrap_or_else(|e| e.into_inner()) = Some(Active { handle, session_id: id.clone() });
         tokio::spawn(pump(rx, sink, id.clone(), self.store.clone(), self.active.clone()));
@@ -442,6 +533,7 @@ async fn pump(
             }
             CoreEvent::Device(DeviceEvent::DeviceChanged) => {}
             CoreEvent::Failed(e) => {
+                log_event("session_failed", &[("session", session_id.clone()), ("code", classify_core(&e).code().into()), ("detail", e.to_string())]);
                 errs.raise(classify_core(&e));
                 update(&sink, &mut shown, errs.current());
                 finished = true;
@@ -449,7 +541,7 @@ async fn pump(
                 *active.lock().unwrap_or_else(|e| e.into_inner()) = None;
                 sink(UiEvent::State { state: "idle" });
             }
-            CoreEvent::StoreError(_) => {}
+            CoreEvent::StoreError(m) => log_event("store_error", &[("session", session_id.clone()), ("detail", m)]),
             CoreEvent::Stopped { lines, duration_ms, drained } => {
                 audio_ms = duration_ms;
                 finished = true;
@@ -478,7 +570,10 @@ async fn pump(
             price_table_date: None,
         };
         let s = store.lock().unwrap_or_else(|e| e.into_inner());
-        let _ = record_usage(&s, &rec);
+        match record_usage(&s, &rec) {
+            Ok(_) => log_event("usage_recorded", &[("session", rec.session_id.clone().unwrap_or_default()), ("audio_ms", audio_ms.to_string()), ("local_date", rec.local_date.clone())]),
+            Err(e) => log_event("usage_record_failed", &[("session", rec.session_id.clone().unwrap_or_default()), ("audio_ms", audio_ms.to_string()), ("detail", e.to_string())]),
+        }
     }
 }
 
@@ -513,6 +608,20 @@ mod tests {
         assert_eq!(classify_soniox(&SonioxError::Network("x".into())), ErrKind::Network);
     }
 
+    #[test]
+    fn audio_start_codes_are_specific() {
+        assert_eq!(audio_start_code(&AudioError::Device("permission: x".into())), "permission_screen");
+        assert_eq!(audio_start_code(&AudioError::Device("khong co micro mac dinh".into())), "no_mic");
+        assert_eq!(audio_start_code(&AudioError::Unsupported("x".into())), "audio_unsupported");
+    }
+
+    #[test]
+    fn scrub_log_masks_key_like_tokens() {
+        let r = scrub_log("auth failed key=abcd1234abcd1234abcd1234abcd1234 status 401");
+        assert!(!r.contains("abcd1234abcd"), "{r}");
+        assert!(r.contains("[che]") && r.contains("status 401"));
+    }
+
     #[test]
     fn uuid_is_36_chars_v4() {
         let a = new_uuid();
diff --git a/src/lib/api.test.ts b/src/lib/api.test.ts
index 4100ec3..8d1a629 100644
--- a/src/lib/api.test.ts
+++ b/src/lib/api.test.ts
@@ -8,7 +8,7 @@ const en: Record<string, string> = JSON.parse(fs.readFileSync(path.join(import.m
 const known = (k: string) => k in en;
 
 test("ma loi cua Rust deu co khoa i18n, ma la thi roi ve err.generic", () => {
-  for (const c of ["key_missing", "consent_required", "already_recording", "auth", "quota", "network", "timeout", "permission", "store_error", "session_active", "import_failed"]) {
+  for (const c of ["key_missing", "consent_required", "already_recording", "auth", "quota", "network", "timeout", "permission", "permission_screen", "permission_mic", "no_mic", "audio_unsupported", "audio_device", "store_error", "session_active", "import_failed"]) {
     assert.notEqual(errKeyOf(c, known), "err.generic", c);
   }
   assert.equal(errKeyOf("khong_biet", known), "err.generic");
diff --git a/src/lib/api.ts b/src/lib/api.ts
index 68d1e2b..70dcdf7 100644
--- a/src/lib/api.ts
+++ b/src/lib/api.ts
@@ -36,7 +36,7 @@ export interface NoteRow { sessionId: string; captionLineId: number | null; note
 export interface TermRow { id: number; termSource: string; termTarget: string | null; keepOriginal: boolean; sourceLang: string; targetLang: string }
 export interface SearchOut { lines: HitRow[]; notes: NoteRow[]; terms: TermRow[] }
 export interface DayRow { localDate: string; audioMs: number; estCostUsdMicros: number; calls: number }
-export interface WeekOut { days: DayRow[]; totalAudioMs: number; totalEstCostUsdMicros: number; estimated: boolean }
+export interface WeekOut { days: DayRow[]; totalAudioMs: number; totalEstCostUsdMicros: number; estimated: boolean; sessionsCount: number; failedStartCount: number }
 export interface DriveOut { status: string; attempts: number; oauthConnected: boolean }
 export interface ImportOut { totalRows: number; tables: [string, number][] }
 
diff --git a/src/screens/UsageLogsTab.svelte b/src/screens/UsageLogsTab.svelte
index d5e0921..5c75aa4 100644
--- a/src/screens/UsageLogsTab.svelte
+++ b/src/screens/UsageLogsTab.svelte
@@ -20,6 +20,8 @@
 
   const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
   const max = $derived(Math.max(1, ...(data?.days.map(d => d.audioMs) ?? [0])));
+  const emptyWhyKey = $derived((data?.failedStartCount ?? 0) > 0 ? "set.usage.none.failed" : "set.usage.none.noaudio");
+  const emptyWhy = $derived($_(emptyWhyKey, { values: { n: data?.sessionsCount ?? 0, f: data?.failedStartCount ?? 0 } }));
   const empty = $derived(!data || data.totalAudioMs === 0);
   const audioLabel = $derived.by(() => {
     const mins = Math.round((data?.totalAudioMs ?? 0) / 60000);
@@ -117,7 +119,11 @@
     </div>
     <div class="hint">{$_("set.usage.note")}</div>
   {:else}
-    <div class="empty" style="padding:30px"><Icon name="list" /><b>{$_("set.usage.none")}</b><span>{$_("set.usage.none.hint")}</span></div>
+    <div class="empty" style="padding:30px"><Icon name="list" /><b>{$_("set.usage.none")}</b><span>{$_("set.usage.none.hint")}</span>
+      {#if data && data.sessionsCount > 0}
+        <span role="status">{emptyWhy}</span>
+      {/if}
+    </div>
   {/if}
   <div class="tl2" style="margin-top:10px">
     {#if confirmClear}

```

## 4. Output kiem thu (that)

- Chay lai sau khi sua `mic.rs` va them assert thong diep that vao `audio_start_codes_are_specific` (lan 2): `cargo test --workspace` moi nhom `ok`, 0 failed (nhom lon nhat 158 passed); `clippy --workspace --all-targets` Finished khong canh bao; `npm run build` built in 278ms; `test:i18n` fail 0; `scan:i18n` 0 chuoi cung.
- Lan 1: `cargo test --workspace`: tat ca ket qua `ok`, tong cong 599 passed, 0 failed, gom 4 test moi o tren.
- `cargo clippy --workspace --all-targets`: `Finished`, khong co warning/error.
- `npm run build`: `built in 230ms`.
- `npm run test:i18n`: pass 12, fail 0.
- `npm run scan:i18n`: khong con canh bao (sau khi chuyen chuoi vao `$derived`).
- `npm run test:api`: pass 6, fail 0.

## 5. CHUA DO (can may that, co giao dien)

- Nguyen nhan "khong co micro" DA do bang probe (muc 1a). Con CHUA DO: chay app that, bam Ghi o che do mixed, xac nhan thong bao `no_mic` tren man Ghi; doc, bam Ghi o che do mixed, doc `~/Library/Application Support/com.mmola.recorderlivetranslate/session-errors.log` (dong `event=session_start_failed code=...`) va thong bao tren man Ghi.
- Hien thi that cua cac thong bao moi va man Usage Logs tren cua so Tauri: chi kiem bang test va build, chua nhin bang mat.
- Hanh vi cua micro khi bi tu choi quyen tren macOS: cpal co the khong tra loi (chi cho tin hieu im); ma `permission_mic` chi bat duoc khi he dieu hanh tra loi chua "denied/not authorized". CHUA DO.

## 6. De xuat ngoai pham vi (khong tu sua)

- Che do mixed nen tu ha xuong chi-he-thong (kem canh bao) khi may khong co micro, thay vi chet ca phien.
- Them quet khoi dong danh dau phien dang do (khong co `ended_at`) la `interrupted` sau crash, de phan biet ro voi loi luc bat dau (hien khong co).
- Khong tao hang `sessions` truoc khi mo nguon thanh cong (hien moi lan bam Ghi that bai de lai mot dong bien ban rong).
- Dung `permissions_status` kiem quyen truoc khi `session_start` va dua nut mo System Settings.
