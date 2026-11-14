# M3: Usage Logs lấy số thật từ API usage của Soniox (US-32, AC-32.10 tới AC-32.12)

Task `t_fda5aa10d67c`, 2026-11-14. Không gọi Soniox thật (không dùng khóa); chỉ tải công khai `https://docs.soniox.com/openapi.yaml` (soniox.com/docs/openapi.yaml chuyển hướng tới đó) để đối chiếu lược đồ.

## THAY ĐỔI THEO CHỦ
- Bỏ cách chỉ đọc bảng `usage_logs` cục bộ; số liệu tuần lấy từ API usage của Soniox, cục bộ chỉ còn là dự phòng có nhãn.
- Tuần Thứ Hai tới Chủ Nhật theo **ngày UTC** (lược đồ Soniox), màn ghi rõ; ước tính cục bộ theo giờ máy, không bao giờ trộn trong một kết quả.
- Spec: `projects/recorder-live-translate/docs/spec-v3.md` thêm AC-32.10, 32.11, 32.12 và dòng v3.7-draft.

## Đã làm
- Client Rust `crates/rlt-features/src/soniox_usage.rs`: `GET /v1/usage-logs` (`start_time`, `end_time`, `limit` 1000, `sort`, `cursor`; lặp tới khi `next_page_cursor` rỗng, chặn cursor lặp và 100 trang) và `GET /v1/usage/summary` (`start_time`, `end_time`). Field theo `UsageLogEntry`, `GetUsageLogsResponse`, `UsageSummaryEntry`, `GetUsageSummaryResponse`; tiền là chuỗi 10 chữ số, cộng bằng số nguyên 1e-10 USD. Cửa sổ tuần `[Thứ Hai 00:00Z, Thứ Hai kế 00:00Z)`.
- Khóa đọc từ Keychain trong Rust (`SecretStore`), đi vào header `Authorization: Bearer`, không vào URL, log, lỗi, kết quả trả về.
- Tauri: `usage_week_source(weeksAgo 0..3)` và `usage_export_week_csv` (`src-tauri/src/session/usage_soniox.rs`, `mod.rs`, `main.rs`, `build.rs`, capability). Cache 60 giây; sau 429 tạm ngừng gọi 60 giây; mạng chạy ở luồng riêng.
- Dự phòng cục bộ khi thiếu khóa, 401, 403, 429, mạng, 5xx, phản hồi sai: kết quả mang `source` (`soniox` hoặc `local`), `basis` (`utc` hoặc `local`), `reason`.
- Giao diện `src/screens/UsageLogsTab.svelte`: nhãn nguồn, dòng "ngày UTC", lý do dự phòng, tổng chi phí, giờ âm thanh cộng từ usage-logs, theo ngày, theo model, tách API và Console playground, trạng thái trống nêu nguyên nhân, nút Xóa kèm chú thích "chỉ xóa cục bộ, không xóa bên Soniox". Chuỗi mới ở `src/locales/en.json` và `pt-PT.json` (`set.usage.*`), không chuỗi cứng.
- CSV: có cột `source` (`soniox` hoặc `local_estimate`); CSV cục bộ cũ (`usage::export_csv`) thêm cột thứ 7.

## Kết quả kiểm (chạy 2026-11-14)
- `cargo test --workspace`: 631 passed, 0 failed (test hồi quy SIGABRT cố ý in dòng "bad: ExitStatus" là hành vi mong đợi).
- `cargo test -p rlt-features --test soniox_usage`: 13 passed (phân trang bằng cursor, cùng `sort`, cursor lặp, toán tuần UTC, ghép tiền summary với giờ logs, ánh xạ 401/403/429/mạng, thiếu khóa không gọi mạng, `key_never_leaks_into_errors_or_results`, mã nguồn không log khóa). Máy chủ giả đọc fixture `fixtures/soniox/*.json` lấy từ ví dụ của lược đồ thật.
- `cargo test -p rlt-app usage_soniox`: 9 passed (nhãn UTC, cache gọi API 1 lần, thiếu khóa, lỗi API đúng lý do, 429 tạm ngừng rồi tiếp, cục bộ không trộn UTC, CSV có cột nguồn).
- `cargo clippy --workspace --all-targets`: không cảnh báo. `npm run build`: thành công. `npm run test:i18n`: 12 pass, 0 fail. `npm run scan:i18n`: 0 chuỗi cứng trong 43 tệp.

## Chưa đo với dữ liệu thật
- Chưa gọi Soniox thật: chưa biết trả lời thật khớp fixture đến đâu (nhất là cửa sổ tối đa, `num_requests`, mô hình có `model` rỗng), và `input_audio_duration_ms` có phản ánh đúng giờ âm thanh tính tiền không.
- Chưa kiểm 403 thật (khóa khác dự án) và 429 thật.
- Chưa chạy trong app Tauri thật trên Windows.

## Hướng dẫn chủ tự kiểm
1. Mở Cài đặt, tab Soniox/Usage Logs, lưu khóa Soniox. Màn phải hiện "Nguồn: Soniox" và "ngày UTC, Thứ Hai tới Chủ Nhật".
2. Đối chiếu tổng chi phí tuần và từng ngày với Soniox Console (theo ngày UTC). Kiểm tách API và Console playground, và các model.
3. Xóa khóa rồi Làm mới: phải hiện "Ước tính cục bộ" kèm lý do "chưa lưu khóa". Ngắt mạng: lý do "không kết nối".
4. Bấm Làm mới liên tục: số không đổi và có dòng "từ cache" (không gọi dồn).
5. Xuất CSV: cột cuối `source`. Bấm Xóa dữ liệu mức dùng: chỉ số cục bộ mất, số Soniox giữ nguyên.

## Ghi chú commit
Task chạy trong cây làm việc có thay đổi của tác vụ khác (Playwright: `.gitignore`, `package*.json`, `src/styles/app.css`, `test/e2e/*.spec.ts`, và `src-tauri/permissions/` là thư mục sinh tự động). Commit chỉ `git add` đúng các đường dẫn của task này. t_ef195a0074ce đã hoàn tất trước task này: commit 6440848 và dd646fd (báo cáo `docs/phien-chet-som-bao-cao.md`) đều là tổ tiên của e619d6b (kiểm bằng `git merge-base --is-ancestor`).


## Diff (commit e619d6b)

Các tệp mới lớn (`crates/rlt-features/src/soniox_usage.rs` 406 dòng, `src-tauri/src/session/usage_soniox.rs` 432 dòng, `crates/rlt-features/tests/soniox_usage.rs`, `fixtures/soniox/*.json`) xem bằng `git show e619d6b -- <đường dẫn>`. Dưới đây là thống kê và diff đầy đủ của các tệp được sửa.

### Thống kê

```
 crates/rlt-features/src/lib.rs            |   1 +
 crates/rlt-features/src/soniox_usage.rs   | 406 ++++++++++++++++++++++++++
 crates/rlt-features/src/usage.rs          |   8 +-
 crates/rlt-features/tests/soniox_usage.rs | 262 +++++++++++++++++
 crates/rlt-features/tests/usage.rs        |   8 +-
 docs/m3-usage-soniox-api-bao-cao.md       |  37 +++
 fixtures/soniox/usage-logs-page1.json     |  47 +++
 fixtures/soniox/usage-logs-page2.json     |  26 ++
 fixtures/soniox/usage-summary-week.json   | 460 ++++++++++++++++++++++++++++++
 src-tauri/build.rs                        |   2 +
 src-tauri/capabilities/main.json          |   2 +
 src-tauri/gen/schemas/acl-manifests.json  |   2 +-
 src-tauri/gen/schemas/capabilities.json   |   2 +-
 src-tauri/gen/schemas/desktop-schema.json |  24 ++
 src-tauri/gen/schemas/macOS-schema.json   |  24 ++
 src-tauri/src/main.rs                     |   2 +
 src-tauri/src/session/mod.rs              |  36 +++
 src-tauri/src/session/usage_soniox.rs     | 432 ++++++++++++++++++++++++++++
 src/lib/api.ts                            |  10 +
 src/locales/en.json                       |  31 +-
 src/locales/pt-PT.json                    |  31 +-
 src/screens/UsageLogsTab.svelte           |  81 ++++--
 test/e2e/tauri-mock.ts                    |  12 +
 23 files changed, 1913 insertions(+), 33 deletions(-)
```

### Diff các tệp sửa

```diff
diff --git a/crates/rlt-features/src/lib.rs b/crates/rlt-features/src/lib.rs
index 8fc1ee7..96e1c0a 100644
--- a/crates/rlt-features/src/lib.rs
+++ b/crates/rlt-features/src/lib.rs
@@ -11,6 +11,7 @@ pub mod glossary;
 pub mod highlights;
 pub mod import_dotnet;
 pub mod search;
+pub mod soniox_usage;
 pub mod usage;
 
 pub use error::{FeatureError, Result};
diff --git a/crates/rlt-features/src/usage.rs b/crates/rlt-features/src/usage.rs
index d0060c6..720d1d8 100644
--- a/crates/rlt-features/src/usage.rs
+++ b/crates/rlt-features/src/usage.rs
@@ -313,10 +313,10 @@ fn csv_field(s: &str) -> String {
     }
 }
 
-/// CSV (AC-32.6): thoi diem, nha cung cap, loai, giay am thanh, ky tu, chi phi uoc tinh (USD).
-/// 100% dong du 6 cot.
+/// CSV (AC-32.6, AC-32.10): thoi diem, nha cung cap, loai, giay am thanh, ky tu, chi phi uoc tinh (USD), nguon.
+/// Du lieu o day luon la `local_estimate` (bang `usage_logs` cuc bo). 100% dong du 7 cot.
 pub fn export_csv(store: &Store) -> Result<String> {
-    let mut out = String::from("occurred_at,provider,usage_kind,audio_seconds,chars_translated,est_cost_usd\n");
+    let mut out = String::from("occurred_at,provider,usage_kind,audio_seconds,chars_translated,est_cost_usd,source\n");
     let mut st = store.conn().prepare(
         "SELECT occurred_at, provider, usage_kind, audio_ms, chars_translated, est_cost_usd_micros
          FROM usage_logs ORDER BY occurred_at, usage_log_id",
@@ -327,7 +327,7 @@ pub fn export_csv(store: &Store) -> Result<String> {
     for row in rows {
         let (t, p, k, a, c, m) = row?;
         out.push_str(&format!(
-            "{},{},{},{}.{:03},{},{}.{:06}\n",
+            "{},{},{},{}.{:03},{},{}.{:06},local_estimate\n",
             csv_field(&t), csv_field(&p), csv_field(&k), a / 1000, a % 1000, c, m / 1_000_000, m % 1_000_000
         ));
     }
diff --git a/crates/rlt-features/tests/usage.rs b/crates/rlt-features/tests/usage.rs
index 3fe1d7a..5a18e51 100644
--- a/crates/rlt-features/tests/usage.rs
+++ b/crates/rlt-features/tests/usage.rs
@@ -93,15 +93,17 @@ fn tran_theo_ngay_tu_cai_dat() {
 }
 
 #[test]
-fn csv_6_cot_va_xoa_toan_bo() {
+fn csv_7_cot_co_nguon_va_xoa_toan_bo() {
     let s = new_store();
     record_usage(&s, &rec("2026-11-14", "2026-11-14T10:00:00.000Z", "soniox", "session", None, 1_234_567)).unwrap();
     record_usage(&s, &rec("2026-11-14", "2026-11-14T10:01:00.000Z", "gemini", "ai_assistant", None, 0)).unwrap();
     let csv = export_csv(&s).unwrap();
     let lines: Vec<&str> = csv.lines().collect();
     assert_eq!(lines.len(), 3);
-    assert!(lines.iter().all(|l| l.split(',').count() == 6), "{csv}");
-    assert!(lines[1].ends_with(",1.234567"), "{}", lines[1]);
+    assert!(lines.iter().all(|l| l.split(',').count() == 7), "{csv}");
+    assert!(lines[0].ends_with(",source"));
+    assert!(lines[1..].iter().all(|l| l.ends_with(",local_estimate")));
+    assert!(lines[1].ends_with(",1.234567,local_estimate"), "{}", lines[1]);
     assert!(lines[1].contains(",60.000,"));
     assert_eq!(clear_all(&s).unwrap(), 2);
     assert_eq!(export_csv(&s).unwrap().lines().count(), 1);
diff --git a/src-tauri/build.rs b/src-tauri/build.rs
index 428a94d..6485b0a 100644
--- a/src-tauri/build.rs
+++ b/src-tauri/build.rs
@@ -86,6 +86,8 @@ pub const APP_COMMANDS: &[&str] = &[
     "usage_week",
     "usage_export_csv",
     "usage_clear",
+    "usage_week_source",
+    "usage_export_week_csv",
     "drive_status",
     "drive_sync",
     "drive_connection_status",
diff --git a/src-tauri/capabilities/main.json b/src-tauri/capabilities/main.json
index e5ee6d6..a854c37 100644
--- a/src-tauri/capabilities/main.json
+++ b/src-tauri/capabilities/main.json
@@ -72,6 +72,8 @@
     "allow-usage-week",
     "allow-usage-export-csv",
     "allow-usage-clear",
+    "allow-usage-week-source",
+    "allow-usage-export-week-csv",
     "allow-drive-status",
     "allow-drive-sync",
     "allow-drive-connection-status",
diff --git a/src-tauri/src/main.rs b/src-tauri/src/main.rs
index 755d8db..6bc9e64 100644
--- a/src-tauri/src/main.rs
+++ b/src-tauri/src/main.rs
@@ -120,6 +120,8 @@ fn main() {
             session::usage_week,
             session::usage_export_csv,
             session::usage_clear,
+            session::usage_week_source,
+            session::usage_export_week_csv,
             session::drive_status,
             session::drive_sync,
             session::drive_connection_status,
diff --git a/src-tauri/src/session/mod.rs b/src-tauri/src/session/mod.rs
index 958b3b5..dacee96 100644
--- a/src-tauri/src/session/mod.rs
+++ b/src-tauri/src/session/mod.rs
@@ -5,6 +5,7 @@
 
 pub mod records;
 pub mod service;
+pub mod usage_soniox;
 
 #[cfg(test)]
 mod integration_tests;
@@ -48,6 +49,8 @@ pub const SESSION_COMMANDS: &[&str] = &[
     "usage_week",
     "usage_export_csv",
     "usage_clear",
+    "usage_week_source",
+    "usage_export_week_csv",
     "drive_status",
     "drive_sync",
     "drive_connection_status",
@@ -270,6 +273,39 @@ pub fn usage_export_csv(s: State<'_, SessionState>) -> Res<String> {
     s.with_store(usage_csv)
 }
 
+fn usage_cache() -> &'static usage_soniox::UsageCache {
+    static C: std::sync::OnceLock<usage_soniox::UsageCache> = std::sync::OnceLock::new();
+    C.get_or_init(Default::default)
+}
+
+/// So lieu tuan `weeks_ago` (0 = tuan nay) tu API usage cua Soniox (ngay UTC); loi/thieu khoa/mat mang thi roi ve
+/// bang `usage_logs` cuc bo, kem `source` va `reason`. Khoa doc trong Rust tu Keychain, khong ra frontend.
+/// Goi mang chay tren luong rieng de khong chan giao dien.
+#[tauri::command]
+pub async fn usage_week_source(s: State<'_, SessionState>, weeks_ago: i64) -> Res<usage_soniox::UsageSourceOut> {
+    if !(0..=3).contains(&weeks_ago) {
+        return Err("invalid_week".into());
+    }
+    let now = usage_soniox::now_secs();
+    let net = tauri::async_runtime::spawn_blocking(move || {
+        let client = rlt_features::soniox_usage::SonioxUsageClient::new(rlt_features::soniox_usage::DEFAULT_BASE_URL)?;
+        usage_soniox::soniox_week(usage_cache(), &client, crate::ai::secret_store().as_ref(), now, weeks_ago)
+    })
+    .await
+    .map_err(|_| "task_failed".to_string())?;
+    match net {
+        Ok(o) => Ok(o),
+        Err(e) => s.with_store(|st| usage_soniox::local_week(st, now, weeks_ago, e.code())),
+    }
+}
+
+/// CSV cua tuan dang xem, dung so lieu va nguon nhu `usage_week_source` (co cot `source`).
+#[tauri::command]
+pub async fn usage_export_week_csv(s: State<'_, SessionState>, weeks_ago: i64) -> Res<String> {
+    let w = usage_week_source(s, weeks_ago).await?;
+    Ok(usage_soniox::week_csv(&w))
+}
+
 #[tauri::command]
 pub fn usage_clear(s: State<'_, SessionState>) -> Res<usize> {
     s.with_store(records::usage_clear)
diff --git a/src/lib/api.ts b/src/lib/api.ts
index 70dcdf7..e04dbd3 100644
--- a/src/lib/api.ts
+++ b/src/lib/api.ts
@@ -74,6 +74,16 @@ export const highlightToggle = (sessionId: string, captionLineId: number, note:
 export const glossaryList = () => invoke<TermRow[]>("glossary_list");
 export const usageWeek = (date: string, provider: string | null = "soniox") => invoke<WeekOut>("usage_week", { date, provider });
 export const usageExportCsv = () => invoke<string>("usage_export_csv");
+export interface UsageDayOut { date: string; audioMs: number; costUsdMicros: number; apiCostUsdMicros: number; playgroundCostUsdMicros: number; requests: number }
+export interface UsageModelOut { model: string; costUsdMicros: number; audioMs: number; requests: number }
+/** `source`: soniox (API that, ngay UTC) | local (uoc tinh cuc bo, ngay gio may). `reason`: ly do roi ve cuc bo. */
+export interface UsageSourceOut {
+  source: "soniox" | "local"; basis: "utc" | "local"; reason: string | null; weekStart: string; weekEnd: string;
+  days: UsageDayOut[]; totalCostUsdMicros: number; totalAudioMs: number; apiCostUsdMicros: number; playgroundCostUsdMicros: number;
+  totalRequests: number; models: UsageModelOut[]; estimated: boolean; cached: boolean; fetchedAt: number; sessionsCount: number; failedStartCount: number;
+}
+export const usageWeekSource = (weeksAgo: number) => invoke<UsageSourceOut>("usage_week_source", { weeksAgo });
+export const usageExportWeekCsv = (weeksAgo: number) => invoke<string>("usage_export_week_csv", { weeksAgo });
 export const usageClear = () => invoke<number>("usage_clear");
 export const driveStatus = (sessionId: string) => invoke<[string, string | null]>("drive_status", { sessionId });
 export const driveSync = (sessionId: string) => invoke<DriveOut>("drive_sync", { sessionId });
diff --git a/src/locales/en.json b/src/locales/en.json
index 33ec046..f43a43d 100644
--- a/src/locales/en.json
+++ b/src/locales/en.json
@@ -637,5 +637,34 @@
   "term.dlg.title": "Add a glossary term",
   "term.dlg.hint": "The selected text is filled in for you.",
   "shell.tray.subtitle": "Open subtitles",
-  "shell.tray.capture": "Capture region"
+  "shell.tray.capture": "Capture region",
+  "set.usage.src.soniox": "Source: Soniox",
+  "set.usage.src.local": "Source: local estimate",
+  "set.usage.basis.utc": "Days are UTC days, Monday to Sunday (UTC), not your local time.",
+  "set.usage.basis.local": "Days follow this computer's time zone, Monday to Sunday.",
+  "set.usage.fallback": "Showing the local estimate instead. Reason: {reason}",
+  "set.usage.reason.nokey": "No Soniox API key is saved. Add it above to see the real figures.",
+  "set.usage.reason.unauthorized": "Soniox rejected the API key (401). Check or replace the key above.",
+  "set.usage.reason.forbidden": "Soniox denied access to usage data for this key (403). Use a key from the same project.",
+  "set.usage.reason.ratelimited": "Soniox is limiting requests (429). Wait a minute, then refresh.",
+  "set.usage.reason.network": "No connection to Soniox. Check your internet connection.",
+  "set.usage.reason.server": "Soniox had a server problem. Try again in a moment.",
+  "set.usage.reason.badresponse": "Soniox sent a response this app could not read. Try again later.",
+  "set.usage.reason.keychain": "The key could not be read from the system keychain.",
+  "set.usage.total": "Total cost this week",
+  "set.usage.real": "From Soniox",
+  "set.usage.hours": "Audio (sum of usage logs)",
+  "set.usage.byday": "By day (UTC)",
+  "set.usage.byday.local": "By day (local time)",
+  "set.usage.byscope": "By source of the request",
+  "set.usage.scope.api": "API",
+  "set.usage.scope.playground": "Console playground",
+  "set.usage.bymodel": "By model",
+  "set.usage.requests": "{n} request(s)",
+  "set.usage.cached": "Shown from a short cache (up to 1 minute) so repeated refreshes do not hit Soniox.",
+  "set.usage.note.soniox": "Figures come from your Soniox account. Days and weeks are UTC.",
+  "set.usage.none.soniox": "Soniox reports no usage for this week (UTC days).",
+  "set.usage.none.localreason": "No local estimate either: nothing was recorded on this computer this week.",
+  "set.usage.clear.note": "Deletes only the local estimate stored on this computer. It does not delete anything at Soniox, and the figures from Soniox stay the same.",
+  "set.usage.clear.done": "Local usage data deleted."
 }
diff --git a/src/locales/pt-PT.json b/src/locales/pt-PT.json
index a0c5d78..3ca2ed6 100644
--- a/src/locales/pt-PT.json
+++ b/src/locales/pt-PT.json
@@ -637,5 +637,34 @@
   "term.dlg.title": "Adicionar um termo ao glossário",
   "term.dlg.hint": "O texto selecionado já vem preenchido.",
   "shell.tray.subtitle": "Abrir legendas",
-  "shell.tray.capture": "Capturar região"
+  "shell.tray.capture": "Capturar região",
+  "set.usage.src.soniox": "Fonte: Soniox",
+  "set.usage.src.local": "Fonte: estimativa local",
+  "set.usage.basis.utc": "Os dias são dias UTC, de segunda a domingo (UTC), não a sua hora local.",
+  "set.usage.basis.local": "Os dias seguem o fuso horário deste computador, de segunda a domingo.",
+  "set.usage.fallback": "A mostrar a estimativa local. Motivo: {reason}",
+  "set.usage.reason.nokey": "Não há nenhuma chave API do Soniox guardada. Adicione-a acima para ver os valores reais.",
+  "set.usage.reason.unauthorized": "O Soniox rejeitou a chave API (401). Verifique ou substitua a chave acima.",
+  "set.usage.reason.forbidden": "O Soniox negou o acesso aos dados de utilização com esta chave (403). Use uma chave do mesmo projeto.",
+  "set.usage.reason.ratelimited": "O Soniox está a limitar os pedidos (429). Aguarde um minuto e atualize.",
+  "set.usage.reason.network": "Sem ligação ao Soniox. Verifique a sua ligação à internet.",
+  "set.usage.reason.server": "O Soniox teve um problema no servidor. Tente novamente daqui a pouco.",
+  "set.usage.reason.badresponse": "O Soniox enviou uma resposta que a aplicação não conseguiu ler. Tente mais tarde.",
+  "set.usage.reason.keychain": "Não foi possível ler a chave do porta-chaves do sistema.",
+  "set.usage.total": "Custo total esta semana",
+  "set.usage.real": "Do Soniox",
+  "set.usage.hours": "Áudio (soma dos registos de utilização)",
+  "set.usage.byday": "Por dia (UTC)",
+  "set.usage.byday.local": "Por dia (hora local)",
+  "set.usage.byscope": "Por origem do pedido",
+  "set.usage.scope.api": "API",
+  "set.usage.scope.playground": "Playground da consola",
+  "set.usage.bymodel": "Por modelo",
+  "set.usage.requests": "{n} pedido(s)",
+  "set.usage.cached": "Mostrado a partir de uma cache curta (até 1 minuto) para que atualizações repetidas não sobrecarreguem o Soniox.",
+  "set.usage.note.soniox": "Os valores vêm da sua conta Soniox. Dias e semanas são em UTC.",
+  "set.usage.none.soniox": "O Soniox não regista utilização esta semana (dias UTC).",
+  "set.usage.none.localreason": "Também não há estimativa local: nada foi registado neste computador esta semana.",
+  "set.usage.clear.note": "Elimina apenas a estimativa local guardada neste computador. Não elimina nada no Soniox, e os valores do Soniox mantêm-se.",
+  "set.usage.clear.done": "Dados locais de utilização eliminados."
 }
diff --git a/src/screens/UsageLogsTab.svelte b/src/screens/UsageLogsTab.svelte
index 5c75aa4..36ea72d 100644
--- a/src/screens/UsageLogsTab.svelte
+++ b/src/screens/UsageLogsTab.svelte
@@ -3,13 +3,13 @@
   import { _, formatCurrency, formatNumber, locale, currentLocale } from "../i18n/index.ts";
   import Icon from "../lib/Icon.svelte";
   import * as api from "../lib/api.ts";
-  import type { WeekOut } from "../lib/api.ts";
+  import type { UsageSourceOut } from "../lib/api.ts";
 
   const loc = $derived(($locale as "en" | "pt-PT") ?? currentLocale());
   const weekKeys = ["set.week.0", "set.week.1", "set.week.2", "set.week.3"];
   const dayKeys = ["day.mon", "day.tue", "day.wed", "day.thu", "day.fri", "day.sat", "day.sun"];
   let week = $state(0);
-  let data = $state<WeekOut | null>(null);
+  let data = $state<UsageSourceOut | null>(null);
   let loading = $state(true);
   let error = $state<string | null>(null);
   let hasKey = $state(false);
@@ -17,20 +17,26 @@
   let keyMsg = $state<{ ok: boolean; code: string } | null>(null);
   let busy = $state(false);
   let confirmClear = $state(false);
+  let cleared = $state(false);
 
   const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
   const max = $derived(Math.max(1, ...(data?.days.map(d => d.audioMs) ?? [0])));
+  const isLocal = $derived(data?.source === "local");
   const emptyWhyKey = $derived((data?.failedStartCount ?? 0) > 0 ? "set.usage.none.failed" : "set.usage.none.noaudio");
   const emptyWhy = $derived($_(emptyWhyKey, { values: { n: data?.sessionsCount ?? 0, f: data?.failedStartCount ?? 0 } }));
-  const empty = $derived(!data || data.totalAudioMs === 0);
+  const empty = $derived(!data || (isLocal ? data.totalAudioMs === 0 : data.totalRequests === 0 && data.totalCostUsdMicros === 0));
+  const reasonKey = (r: string) => "set.usage.reason." + r.replace(/_/g, "");
+  const fallbackText = (r: string) => $_("set.usage.fallback", { values: { reason: $_(reasonKey(r)) } });
+  const requestsText = (n: number) => $_("set.usage.requests", { values: { n: formatNumber(n, loc) } });
   const audioLabel = $derived.by(() => {
     const mins = Math.round((data?.totalAudioMs ?? 0) / 60000);
     return $_("set.usage.audio", { values: { h: formatNumber(Math.floor(mins / 60), loc), m: mins % 60 } });
   });
+  const money = (micros: number) => formatCurrency(api.usd(micros), loc);
 
   async function loadWeek() {
     loading = true; error = null;
-    try { data = await api.usageWeek(api.mondayOf(week)); } catch (e) { error = api.errCode(e); }
+    try { data = await api.usageWeekSource(week); } catch (e) { error = api.errCode(e); }
     loading = false;
   }
   onMount(async () => {
@@ -57,7 +63,7 @@
   }
   async function exportCsv() {
     try {
-      const csv = await api.usageExportCsv();
+      const csv = await api.usageExportWeekCsv(week);
       const url = URL.createObjectURL(new Blob([csv], { type: "text/csv" }));
       const a = document.createElement("a");
       a.href = url; a.download = "usage-logs.csv"; a.click();
@@ -65,7 +71,7 @@
     } catch (e) { error = api.errCode(e); }
   }
   async function clearAll() {
-    try { await api.usageClear(); confirmClear = false; await loadWeek(); } catch (e) { error = api.errCode(e); }
+    try { await api.usageClear(); confirmClear = false; cleared = true; await loadWeek(); } catch (e) { error = api.errCode(e); }
   }
 </script>
 
@@ -95,7 +101,7 @@
   <h4>{$_("set.tab.usagelogs")}</h4>
   <div class="tl2" style="justify-content:space-between;margin-bottom:8px">
     <span class="seg" role="group" aria-label={$_("set.week.pick")}>
-      {#each weekKeys as k, i (k)}<button type="button" class={week === i ? "on" : ""} aria-pressed={week === i} onclick={() => { week = i; void loadWeek(); }}>{$_(k)}</button>{/each}
+      {#each weekKeys as k, i (k)}<button type="button" class={week === i ? "on" : ""} aria-pressed={week === i} onclick={() => { week = i; cleared = false; void loadWeek(); }}>{$_(k)}</button>{/each}
     </span>
     <span class="rr">
       <button type="button" class="btn sm" onclick={loadWeek}><Icon name="refresh" />{$_("common.refresh")}</button>
@@ -107,23 +113,52 @@
       <button type="button" class="btn sm" onclick={loadWeek}><Icon name="refresh" />{$_("common.retry")}</button></div>
   {:else if loading}
     <div class="empty" style="padding:30px"><Icon name="refresh" /><span>{$_("common.loading")}</span></div>
-  {:else if !empty && data}
-    <div class="hint" style="margin:0">{$_("set.usage.week")}</div>
-    <div><b>{audioLabel}</b> {$_("set.usage.cost")} <b>{"≈"} {formatCurrency(api.usd(data.totalEstCostUsdMicros), loc)}</b> <span class="hint">{$_("common.estimate")}</span></div>
-    <div class="wkw">
-      <div class="wk">
-        {#each data.days as d, i (d.localDate)}
-          <div><div class="st" style="height:{(d.audioMs / max) * 85}%"></div><em>{$_(dayKeys[i])}</em></div>
-        {/each}
-      </div>
+  {:else if data}
+    <div class="tl2" style="gap:8px;flex-wrap:wrap">
+      <span class="bd {isLocal ? 'rec' : 'acc'}" data-testid="usage-source"><Icon name={isLocal ? "warn" : "check"} />{$_(isLocal ? "set.usage.src.local" : "set.usage.src.soniox")}</span>
+      <span class="hint" style="margin:0" data-testid="usage-basis">{$_(data.basis === "utc" ? "set.usage.basis.utc" : "set.usage.basis.local")}</span>
     </div>
-    <div class="hint">{$_("set.usage.note")}</div>
-  {:else}
-    <div class="empty" style="padding:30px"><Icon name="list" /><b>{$_("set.usage.none")}</b><span>{$_("set.usage.none.hint")}</span>
-      {#if data && data.sessionsCount > 0}
-        <span role="status">{emptyWhy}</span>
+    {#if isLocal && data.reason}
+      <div class="hint" role="status" data-testid="usage-fallback">{fallbackText(data.reason)}</div>
+    {/if}
+    {#if !empty}
+      <div class="hint" style="margin:0">{data.weekStart} → {data.weekEnd}</div>
+      <div><b>{audioLabel}</b> <span class="hint">{$_("set.usage.hours")}</span></div>
+      <div><b data-testid="usage-total">{data.estimated ? "≈ " : ""}{money(data.totalCostUsdMicros)}</b> <span class="hint">{$_("set.usage.total")}{data.estimated ? " " + $_("common.estimate") : ""}</span></div>
+      <h4 style="margin-top:10px">{$_(data.basis === "utc" ? "set.usage.byday" : "set.usage.byday.local")}</h4>
+      <div class="wkw">
+        <div class="wk">
+          {#each data.days as d, i (d.date)}
+            <div title="{d.date}: {money(d.costUsdMicros)}"><div class="st" style="height:{(d.audioMs / max) * 85}%"></div><em>{$_(dayKeys[i])}</em></div>
+          {/each}
+        </div>
+      </div>
+      {#if !isLocal}
+        <h4 style="margin-top:10px">{$_("set.usage.byscope")}</h4>
+        <div class="rw"><div>{$_("set.usage.scope.api")}</div><div class="rr"><b>{money(data.apiCostUsdMicros)}</b></div></div>
+        <div class="rw"><div>{$_("set.usage.scope.playground")}</div><div class="rr"><b>{money(data.playgroundCostUsdMicros)}</b></div></div>
+        {#if data.models.length > 0}
+          <h4 style="margin-top:10px">{$_("set.usage.bymodel")}</h4>
+          {#each data.models as m (m.model)}
+            <div class="rw"><div>{m.model}<small>{requestsText(m.requests)}</small></div><div class="rr"><b>{money(m.costUsdMicros)}</b></div></div>
+          {/each}
+        {/if}
+        <div class="hint">{$_("set.usage.note.soniox")}</div>
+        {#if data.cached}<div class="hint">{$_("set.usage.cached")}</div>{/if}
+      {:else}
+        <div class="hint">{$_("set.usage.note")}</div>
       {/if}
-    </div>
+    {:else}
+      <div class="empty" style="padding:30px"><Icon name="list" /><b>{$_("set.usage.none")}</b>
+        {#if isLocal}
+          <span>{$_("set.usage.none.localreason")}</span>
+          {#if data.sessionsCount > 0}<span role="status">{emptyWhy}</span>{/if}
+        {:else}
+          <span>{$_("set.usage.none.soniox")}</span>
+        {/if}
+        <span>{$_("set.usage.none.hint")}</span>
+      </div>
+    {/if}
   {/if}
   <div class="tl2" style="margin-top:10px">
     {#if confirmClear}
@@ -133,4 +168,6 @@
       <button type="button" class="btn sm ghost" onclick={() => (confirmClear = true)}><Icon name="trash" />{$_("set.usage.clear")}</button>
     {/if}
   </div>
+  <div class="hint" data-testid="usage-clear-note">{$_("set.usage.clear.note")}</div>
+  {#if cleared}<div class="hint" role="status">{$_("set.usage.clear.done")}</div>{/if}
 </div>
```
