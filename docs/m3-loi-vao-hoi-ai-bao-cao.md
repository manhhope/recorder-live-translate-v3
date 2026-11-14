# M3: các lối vào Hỏi AI - báo cáo

## Kết luận
Chủ không thấy chỗ hỏi AI vì thực tế chưa có lối nào chạy được: phím tắt Cmd+Shift+A chỉ phát sự kiện `shell://deeplink` nhưng **không có đoạn giao diện nào lắng nghe sự kiện đó**, và Rust không có cửa sổ `ask-ai` nào được tạo. Việc này làm màn Hỏi AI ngay trong cửa sổ chính và nối 5 lối vào vào nó.

## Bảng lối vào
| # | Lối vào | Ở đâu | Hành vi | Test Playwright (test/e2e/ask-ai.spec.ts) |
|---|---|---|---|---|
| 1 | Nút "Ask AI" / "Perguntar à IA" | Màn Record, thanh công cụ, hiện cả lúc nghỉ, đang ghi, sau khi ghi | Mở màn Hỏi AI với phiên đang chạy, hoặc phiên vừa ghi, hoặc phiên mới nhất | "nút Hỏi AI hiện lúc nghỉ và lúc đang ghi", "sau khi ghi xong nút vẫn còn và mở đúng phiên", "chưa từng ghi" |
| 2 | Nút "Ask AI" + khung chat | Sessions, chi tiết biên bản | Chat nhiều lượt gắn biên bản (cùng conversation_id), chip Explain, Summarise, Action items | "chi tiết biên bản: chat nhiều lượt ..." |
| 3 | Mục "Ask AI" kèm phím tắt | Thanh bên, hiện `⇧⌘A` lấy từ `shortcuts_status` thật (đổi phím ở Cài đặt thì đổi theo, tắt phím thì ẩn) | Mở màn Hỏi AI | "thanh bên: mục Hỏi AI có phím tắt cạnh nhãn" |
| 4 | Phím tắt Cmd+Shift+A / `rlt://ask-ai` | Toàn cục | Rust `AskAiOpen` đưa cửa sổ lên và phát `shell://deeplink {"action":"ask_ai"}`; `App.svelte` lắng nghe và mở màn Hỏi AI | "phím tắt: sự kiện shell://deeplink ..." |
| 5 | Chưa có khóa/mô hình AI | Mọi lối trên | Banner "Not configured" + nút "Open Settings: Translation and AI" nhảy tới Cài đặt, tab Translation and AI | "chưa có khóa AI ..." |

Chuỗi phím tắt tới giao diện được chứng minh ở ba mắt xích: (a) test Rust có sẵn `shortcuts::tests` kiểm `AskAi -> Dispatch::AskAiOpen`; (b) test Rust mới kiểm `DeepLinkAction::AskAi` serialize đúng `{"action":"ask_ai"}`; (c) test Playwright phát đúng payload đó qua lớp `listen` giả và thấy màn Hỏi AI mở (và payload `settings` thì bị bỏ qua). Chưa chạy được phím tắt toàn cục thật trên macOS trong môi trường headless (xem "Chưa kiểm").

## Thay đổi
- `src/screens/AskAi.svelte` (mới): màn Hỏi AI, chọn phiên, trạng thái "chưa có biên bản" có nút về Record.
- `src/lib/app.svelte.ts`: `Screen` thêm `ask`, `lastSession`, `askSession`, `settingsTab`, hàm `openAsk()` và `openAiSettings()`.
- `src/App.svelte`: mục thanh bên, nghe `shell://deeplink`, hiện phím tắt.
- `src/screens/Record.svelte`, `Sessions.svelte`: nút và khung chat; `Settings.svelte`: nhận tab từ `app.settingsTab`.
- `src/locales/en.json`, `pt-PT.json`: 5 khóa mới `ask.screen.*`, và sửa `ask.open.ai` (trước đây trỏ tới tab "AI Assistant" không còn tồn tại).
- `src-tauri/capabilities/main.json`: cấp cho cửa sổ main 7 lệnh chat (`ai_list_conversations`, `ai_list_messages`, `ai_ask`, `ai_retry`, `ai_cancel`, `ai_clear_history`, `ai_delete_conversation`). Không có lệnh đọc khóa API.
- Test Rust mới: `shell::tests::main_may_run_the_ask_ai_chat_commands`; assert mới trong `deeplink::tests::valid_links_parse`. Không thêm lệnh Tauri mới.
- `test/e2e/tauri-mock.ts`: thêm `plugin:event|listen`/`unlisten`, `emitEvent`, các lệnh `ai_*` chat, `shell_set_recording`, `shell_set_keep_awake`.

Diff đầy đủ ở cuối báo cáo.

## Output test thật
- `cargo test --workspace`: exit 0, tổng 632 test pass, 0 fail (có `shell::tests::main_may_run_the_ask_ai_chat_commands ... ok`, `shell::deeplink::tests::valid_links_parse ... ok`).
- `cargo clippy --workspace --all-targets`: exit 0, 0 cảnh báo (`Finished dev profile`).
- `npm run build`: `✓ built in 259ms`.
- `npm run test:i18n`: `ℹ tests 12, pass 12, fail 0`.
- `npm run scan:i18n`: `Quet 44 tep trong src/ ... So chuoi cung tim thay: 0`.
- Playwright `ask-ai.spec.ts` + `sessions.spec.ts` + `record.spec.ts` + `responsive.spec.ts`: `39 passed`. Riêng `ask-ai.spec.ts`: 9 pass.
- Toàn bộ Playwright: 63 pass, 19 fail. **19 fail đều không do việc này**: chúng bấm vào tab Cài đặt tên `set.tab.ai` và `set.tab.usagelogs` (settings.spec, known-bugs P-01/03/04, i18n, states 83/93, visual 45/53) mà Settings hiện không còn (đã gộp thành "Translation and AI" và "Soniox and Usage Logs"). Đó là bộ test của việc Playwright (t_b332962c022e) chưa cập nhật theo bố cục Cài đặt mới; tôi không sửa vì ngoài phạm vi.

## Ảnh chụp
- `docs/m3-loi-vao-hoi-ai-anh/01-record-nut-ask-ai.png`: nút Ask AI ở Record và mục thanh bên kèm `⇧⌘A`.
- `02-man-hoi-ai-thanh-ben.png`: màn Hỏi AI, một lượt Summarise.
- `03-chi-tiet-bien-ban-chat-chip.png`: chat trong chi tiết biên bản.
- `04-chua-co-khoa-pt-PT.png`: hướng dẫn chưa có khóa, tiếng pt-PT.

## Chưa kiểm / lưu ý
- Chưa chạy app Tauri thật bấm phím tắt toàn cục (headless, lớp Tauri là giả). Cần chủ thử trên máy: bấm Cmd+Shift+A, mong đợi cửa sổ chính lên và hiện màn Hỏi AI.
- Chưa gọi nhà cung cấp AI thật; lượt trả lời trong test là của lớp giả.
- Đã có sẵn từ trước (không sửa): khi đang ghi mà chuyển màn rồi quay lại Record, Record mới mount không nhận tiếp sự kiện dừng vì kênh sự kiện gắn với lần mount cũ. Nên ghi chú cho việc khác; nay có mục Ask AI nên người dùng dễ gặp hơn.
- Một việc khác đã hoàn nguyên `Record.svelte` giữa chừng (cây git chung); tôi đã áp lại và chạy lại test sau đó.

## Diff (git diff HEAD, chỉ các file của việc này; AskAi.svelte và ask-ai.spec.ts là file mới, xem trong repo)

```diff
diff --git a/src-tauri/capabilities/main.json b/src-tauri/capabilities/main.json
index a854c37..2eb7de5 100644
--- a/src-tauri/capabilities/main.json
+++ b/src-tauri/capabilities/main.json
@@ -43,6 +43,13 @@
     "allow-ai-clear-api-key",
     "allow-ai-accept-consent",
     "allow-ai-test-connection",
+    "allow-ai-list-conversations",
+    "allow-ai-list-messages",
+    "allow-ai-ask",
+    "allow-ai-retry",
+    "allow-ai-cancel",
+    "allow-ai-clear-history",
+    "allow-ai-delete-conversation",
     "allow-session-start",
     "allow-session-pause",
     "allow-session-resume",
diff --git a/src-tauri/gen/schemas/capabilities.json b/src-tauri/gen/schemas/capabilities.json
index d8fd036..b779182 100644
--- a/src-tauri/gen/schemas/capabilities.json
+++ b/src-tauri/gen/schemas/capabilities.json
@@ -1 +1 @@
-{"ask-ai-window":{"identifier":"ask-ai-window","description":"Cua so ask-ai (ADR-10): chat Hoi AI. Khong co lenh dat hay doc khoa API; khoa chi dat duoc tu cua so main.","local":true,"windows":["ask-ai"],"permissions":["core:event:allow-listen","core:event:allow-unlisten","allow-ai-get-settings","allow-ai-accept-consent","allow-ai-list-conversations","allow-ai-list-messages","allow-ai-ask","allow-ai-retry","allow-ai-cancel","allow-ai-clear-history","allow-ai-delete-conversation"]},"capture-overlay-window":{"identifier":"capture-overlay-window","description":"Lop phu chon vung chup (M3-04/05, label 'capture-overlay-N'): chi chot hoac huy vung, khong doc phien hay khoa.","local":true,"windows":["capture-overlay-*"],"permissions":["core:event:allow-listen","core:event:allow-unlisten","allow-capture-commit","allow-capture-cancel"]},"main-window":{"identifier":"main-window","description":"Cua so chinh (ADR-10): lenh vo ung dung va cai dat. Autostart va deep-link chi di qua lenh Rust nay, khong cap plugin JS.","local":true,"windows":["main"],"permissions":["core:default","allow-core-version","allow-quick-ax-status","allow-deepl-save-key","allow-deepl-has-key","allow-deepl-clear-key","allow-deepl-test-connection","allow-shell-set-recording","allow-shell-set-keep-awake","allow-shell-set-locale","allow-shortcuts-status","allow-shortcuts-list","allow-shortcut-reset","allow-shortcut-rebind","allow-shortcut-set-enabled","allow-autostart-get","allow-autostart-set","allow-quit-app","allow-subtitle-open","allow-subtitle-close","allow-subtitle-get","allow-subtitle-font-step","allow-subtitle-set-mode","allow-subtitle-set-opacity","allow-subtitle-reset-colors","allow-subtitle-set-color","allow-subtitle-set-hidden","allow-subtitle-reset-position","allow-capture-begin","allow-capture-cancel","allow-capture-commit","allow-ai-get-settings","allow-ai-save-settings","allow-ai-set-api-key","allow-ai-clear-api-key","allow-ai-accept-consent","allow-ai-test-connection","allow-session-start","allow-session-pause","allow-session-resume","allow-session-stop","allow-session-new","allow-session-state","allow-consent-get","allow-consent-accept","allow-soniox-save-api-key","allow-soniox-has-api-key","allow-soniox-clear-api-key","allow-soniox-test-connection","allow-permissions-status","allow-record-prefs-get","allow-record-prefs-set","allow-sessions-list","allow-session-get","allow-session-rename","allow-session-delete","allow-records-search","allow-session-export","allow-highlight-toggle","allow-glossary-list","allow-glossary-add","allow-glossary-delete","allow-glossary-import-csv","allow-usage-week","allow-usage-export-csv","allow-usage-clear","allow-usage-week-source","allow-usage-export-week-csv","allow-drive-status","allow-drive-sync","allow-drive-connection-status","allow-drive-save-client-config","allow-drive-connect","allow-drive-disconnect","allow-mic-permission-request","allow-import-dotnet-pick"]},"overlay-window":{"identifier":"overlay-window","description":"Cua so phu de (ADR-10, label 'subtitle'): hien chu nhan dang tu am thanh bat ky nen khong dung duoc phien, bien ban, khoa hay mang.","local":true,"windows":["subtitle"],"permissions":["core:event:allow-listen","core:event:allow-unlisten","core:window:allow-start-dragging","allow-subtitle-get","allow-subtitle-font-step","allow-subtitle-set-mode","allow-subtitle-set-opacity","allow-subtitle-reset-colors","allow-subtitle-set-color","allow-subtitle-set-hidden","allow-subtitle-reset-position","allow-subtitle-close"]},"quick-toolbar-window":{"identifier":"quick-toolbar-window","description":"Thanh nho khi boi chon chu (US-41): chi duoc bam Dich (inline) va dong.","local":true,"windows":["quick-toolbar"],"permissions":["allow-quick-inline-run"]},"quick-translate-window":{"identifier":"quick-translate-window","description":"Cua so quick-translate (ADR-10): dich nhanh, lich su, tuy chon. Khong co lenh doc khoa, phim tat hay thoat app.","local":true,"windows":["quick-translate"],"permissions":["core:event:allow-listen","core:event:allow-unlisten","allow-quick-translate","allow-quick-cancel","allow-quick-history","allow-quick-clear-history","allow-quick-hide","allow-quick-ax-status","allow-quick-prefs-get","allow-quick-prefs-set","allow-quick-ocr-translate"]},"region-capture-window":{"identifier":"region-capture-window","description":"Cua so region-capture (ADR-10): chua co lenh rieng (them o M3); tam thoi chi nghe su kien.","local":true,"windows":["region-capture"],"permissions":["core:event:allow-listen","core:event:allow-unlisten"]}}
\ No newline at end of file
+{"ask-ai-window":{"identifier":"ask-ai-window","description":"Cua so ask-ai (ADR-10): chat Hoi AI. Khong co lenh dat hay doc khoa API; khoa chi dat duoc tu cua so main.","local":true,"windows":["ask-ai"],"permissions":["core:event:allow-listen","core:event:allow-unlisten","allow-ai-get-settings","allow-ai-accept-consent","allow-ai-list-conversations","allow-ai-list-messages","allow-ai-ask","allow-ai-retry","allow-ai-cancel","allow-ai-clear-history","allow-ai-delete-conversation"]},"capture-overlay-window":{"identifier":"capture-overlay-window","description":"Lop phu chon vung chup (M3-04/05, label 'capture-overlay-N'): chi chot hoac huy vung, khong doc phien hay khoa.","local":true,"windows":["capture-overlay-*"],"permissions":["core:event:allow-listen","core:event:allow-unlisten","allow-capture-commit","allow-capture-cancel"]},"main-window":{"identifier":"main-window","description":"Cua so chinh (ADR-10): lenh vo ung dung va cai dat. Autostart va deep-link chi di qua lenh Rust nay, khong cap plugin JS.","local":true,"windows":["main"],"permissions":["core:default","allow-core-version","allow-quick-ax-status","allow-deepl-save-key","allow-deepl-has-key","allow-deepl-clear-key","allow-deepl-test-connection","allow-shell-set-recording","allow-shell-set-keep-awake","allow-shell-set-locale","allow-shortcuts-status","allow-shortcuts-list","allow-shortcut-reset","allow-shortcut-rebind","allow-shortcut-set-enabled","allow-autostart-get","allow-autostart-set","allow-quit-app","allow-subtitle-open","allow-subtitle-close","allow-subtitle-get","allow-subtitle-font-step","allow-subtitle-set-mode","allow-subtitle-set-opacity","allow-subtitle-reset-colors","allow-subtitle-set-color","allow-subtitle-set-hidden","allow-subtitle-reset-position","allow-capture-begin","allow-capture-cancel","allow-capture-commit","allow-ai-get-settings","allow-ai-save-settings","allow-ai-set-api-key","allow-ai-clear-api-key","allow-ai-accept-consent","allow-ai-test-connection","allow-ai-list-conversations","allow-ai-list-messages","allow-ai-ask","allow-ai-retry","allow-ai-cancel","allow-ai-clear-history","allow-ai-delete-conversation","allow-session-start","allow-session-pause","allow-session-resume","allow-session-stop","allow-session-new","allow-session-state","allow-consent-get","allow-consent-accept","allow-soniox-save-api-key","allow-soniox-has-api-key","allow-soniox-clear-api-key","allow-soniox-test-connection","allow-permissions-status","allow-record-prefs-get","allow-record-prefs-set","allow-sessions-list","allow-session-get","allow-session-rename","allow-session-delete","allow-records-search","allow-session-export","allow-highlight-toggle","allow-glossary-list","allow-glossary-add","allow-glossary-delete","allow-glossary-import-csv","allow-usage-week","allow-usage-export-csv","allow-usage-clear","allow-usage-week-source","allow-usage-export-week-csv","allow-drive-status","allow-drive-sync","allow-drive-connection-status","allow-drive-save-client-config","allow-drive-connect","allow-drive-disconnect","allow-mic-permission-request","allow-import-dotnet-pick"]},"overlay-window":{"identifier":"overlay-window","description":"Cua so phu de (ADR-10, label 'subtitle'): hien chu nhan dang tu am thanh bat ky nen khong dung duoc phien, bien ban, khoa hay mang.","local":true,"windows":["subtitle"],"permissions":["core:event:allow-listen","core:event:allow-unlisten","core:window:allow-start-dragging","allow-subtitle-get","allow-subtitle-font-step","allow-subtitle-set-mode","allow-subtitle-set-opacity","allow-subtitle-reset-colors","allow-subtitle-set-color","allow-subtitle-set-hidden","allow-subtitle-reset-position","allow-subtitle-close"]},"quick-toolbar-window":{"identifier":"quick-toolbar-window","description":"Thanh nho khi boi chon chu (US-41): chi duoc bam Dich (inline) va dong.","local":true,"windows":["quick-toolbar"],"permissions":["allow-quick-inline-run"]},"quick-translate-window":{"identifier":"quick-translate-window","description":"Cua so quick-translate (ADR-10): dich nhanh, lich su, tuy chon. Khong co lenh doc khoa, phim tat hay thoat app.","local":true,"windows":["quick-translate"],"permissions":["core:event:allow-listen","core:event:allow-unlisten","allow-quick-translate","allow-quick-cancel","allow-quick-history","allow-quick-clear-history","allow-quick-hide","allow-quick-ax-status","allow-quick-prefs-get","allow-quick-prefs-set","allow-quick-ocr-translate"]},"region-capture-window":{"identifier":"region-capture-window","description":"Cua so region-capture (ADR-10): chua co lenh rieng (them o M3); tam thoi chi nghe su kien.","local":true,"windows":["region-capture"],"permissions":["core:event:allow-listen","core:event:allow-unlisten"]}}
\ No newline at end of file
diff --git a/src-tauri/src/shell/deeplink.rs b/src-tauri/src/shell/deeplink.rs
index 8219aea..49d5675 100644
--- a/src-tauri/src/shell/deeplink.rs
+++ b/src-tauri/src/shell/deeplink.rs
@@ -121,6 +121,8 @@ mod tests {
         assert_eq!(parse("rlt://open"), Ok(DeepLinkAction::App));
         assert_eq!(parse("RLT://Open/"), Ok(DeepLinkAction::App));
         assert_eq!(parse("rlt://ask-ai"), Ok(DeepLinkAction::AskAi));
+        // Hợp đồng với giao diện: App.svelte lắng nghe `shell://deeplink` và so `payload.action === "ask_ai"`.
+        assert_eq!(serde_json::to_string(&DeepLinkAction::AskAi).unwrap(), r#"{"action":"ask_ai"}"#);
         assert_eq!(parse("rlt://session/abc-123_X"), Ok(DeepLinkAction::Session { id: "abc-123_X".into() }));
         assert_eq!(parse("rlt://settings/shortcuts"), Ok(DeepLinkAction::Settings { tab: "shortcuts".into() }));
         assert_eq!(parse("rlt://settings/SONIOX"), Ok(DeepLinkAction::Settings { tab: "soniox".into() }));
diff --git a/src-tauri/src/shell/mod.rs b/src-tauri/src/shell/mod.rs
index bd6783f..75b1fde 100644
--- a/src-tauri/src/shell/mod.rs
+++ b/src-tauri/src/shell/mod.rs
@@ -567,6 +567,17 @@ mod tests {
         }
     }
 
+    /// Hỏi AI chạy ngay trong cửa sổ chính (phím tắt/deeplink ask_ai -> màn Hỏi AI), nên `main` phải được gọi các lệnh chat.
+    #[test]
+    fn main_may_run_the_ask_ai_chat_commands() {
+        let main = perms("main");
+        for c in ["ai-list-conversations", "ai-list-messages", "ai-ask", "ai-retry", "ai-cancel", "ai-clear-history", "ai-delete-conversation", "ai-accept-consent", "ai-get-settings"] {
+            assert!(main.contains(&format!("allow-{c}")), "main thieu quyen allow-{c}");
+        }
+        // Khóa API vẫn chỉ đặt/xóa được, không có lệnh đọc lại.
+        assert!(!main.iter().any(|p| p.contains("ai-get-api-key")));
+    }
+
     #[test]
     fn csp_is_strict() {
         let conf: Value = serde_json::from_str(include_str!("../../tauri.conf.json")).unwrap();
diff --git a/src/App.svelte b/src/App.svelte
index c844e4e..00ce1ee 100644
--- a/src/App.svelte
+++ b/src/App.svelte
@@ -6,16 +6,32 @@
   import Record from "./screens/Record.svelte";
   import Sessions from "./screens/Sessions.svelte";
   import Settings from "./screens/Settings.svelte";
-  import { app, applyTheme, type Screen } from "./lib/app.svelte.ts";
+  import AskAi from "./screens/AskAi.svelte";
+  import { listen } from "@tauri-apps/api/event";
+  import { displayAccel } from "./lib/accel.ts";
+  import { app, applyTheme, openAsk, type Screen } from "./lib/app.svelte.ts";
 
   const items: { id: Screen; icon: string; key: string; kbd: string }[] = [
     { id: "rec", icon: "mic", key: "nav.record", kbd: "⌘1" },
     { id: "ses", icon: "clock", key: "nav.sessions", kbd: "⌘2" },
+    { id: "ask", icon: "sparkle", key: "nav.askai", kbd: "" },
   ];
+  // Phim tat Hoi AI that (nguoi dung doi duoc o Cai dat), hien canh nhan thanh ben; tat thi khong hien.
+  let askKbd = $state("");
+  async function loadAskKbd() {
+    try {
+      const a = (await api.shortcutsStatus()).find(s => s.action === "ask_ai");
+      askKbd = a?.enabled ? displayAccel(a.accelerator) : "";
+    } catch { askKbd = ""; }
+  }
+  $effect(() => { void app.screen; void loadAskKbd(); });
   // Dong bo khay he thong voi ngon ngu hien tai va ap dung tuy chon giu may thuc da luu (loi thi bo qua: vo chua san sang).
   onMount(() => {
     void api.shellSetLocale(currentLocale()).catch(() => {});
     if (localStorage.getItem(api.KEEP_AWAKE_KEY) === "1") void api.shellSetKeepAwake(true).catch(() => {});
+    // Phim tat toan cuc Hoi AI (Rust: shell/mod.rs AskAiOpen) phat shell://deeplink {action: ask_ai}; deeplink rlt://ask-ai cung vay.
+    const un = listen<{ action?: string }>("shell://deeplink", e => { if (e.payload?.action === "ask_ai") openAsk(); });
+    return () => { void un.then(f => f()).catch(() => {}); };
   });
   const isPt = $derived($locale === "pt-PT");
   const dark = $derived(app.theme === "dark" || (app.theme === "system" && matchMedia("(prefers-color-scheme: dark)").matches));
@@ -25,7 +41,7 @@
   <nav class="nav" aria-label={$_("nav.tools")}>
     {#each items as n (n.id)}
       <button type="button" class="ni {app.screen === n.id ? 'on' : ''}" aria-current={app.screen === n.id ? "page" : undefined} onclick={() => (app.screen = n.id)}>
-        <Icon name={n.icon} /><span>{$_(n.key)}</span><kbd>{n.kbd}</kbd>
+        <Icon name={n.icon} /><span>{$_(n.key)}</span>{#if (n.id === "ask" ? askKbd : n.kbd)}<kbd>{n.id === "ask" ? askKbd : n.kbd}</kbd>{/if}
       </button>
     {/each}
     <div class="nsp"></div>
@@ -43,6 +59,6 @@
     </div>
   </nav>
   <main class="main">
-    {#if app.screen === "rec"}<Record />{:else if app.screen === "ses"}<Sessions />{:else}<Settings />{/if}
+    {#if app.screen === "rec"}<Record />{:else if app.screen === "ses"}<Sessions />{:else if app.screen === "ask"}<AskAi />{:else}<Settings bind:tab={app.settingsTab} />{/if}
   </main>
 </div>
diff --git a/src/lib/app.svelte.ts b/src/lib/app.svelte.ts
index ac35a31..109405c 100644
--- a/src/lib/app.svelte.ts
+++ b/src/lib/app.svelte.ts
@@ -1,7 +1,8 @@
 // Trang thai giao dien dung chung.
 import { DEFAULT_DST, DEFAULT_SRC } from "./langs.ts";
 
-export type Screen = "rec" | "ses" | "set";
+export type Screen = "rec" | "ses" | "ask" | "set";
+export type SettingsTabId = "general" | "translate" | "shortcuts" | "glossary" | "subtitle" | "usage" | "drive" | "about";
 export type Level = "idle" | "quiet" | "talk" | "clip" | "none";
 
 export const app = $state({
@@ -14,8 +15,25 @@ export const app = $state({
   micOn: true,
   view: "both" as "both" | "dst" | "src",
   theme: "system" as "system" | "light" | "dark",
+  /** Phien ghi gan nhat (dang ghi hoac vua ghi xong); man Hoi AI dung no khi khong chon phien khac. */
+  lastSession: null as string | null,
+  /** Phien man Hoi AI dang hoi; null = tu chon phien moi nhat. */
+  askSession: null as string | null,
+  settingsTab: "general" as SettingsTabId,
 });
 
+/** Mo man Hoi AI (nut Record, thanh ben, phim tat/deeplink ask_ai). */
+export function openAsk(sessionId: string | null = null) {
+  app.askSession = sessionId ?? app.lastSession;
+  app.screen = "ask";
+}
+
+/** Nhay toi Cai dat, muc "Dich va AI" (noi nhap khoa va chon mo hinh). */
+export function openAiSettings() {
+  app.settingsTab = "translate";
+  app.screen = "set";
+}
+
 export function applyTheme(t: "system" | "light" | "dark") {
   app.theme = t;
   const el = document.documentElement;
diff --git a/src/locales/en.json b/src/locales/en.json
index f43a43d..40816fe 100644
--- a/src/locales/en.json
+++ b/src/locales/en.json
@@ -25,7 +25,7 @@
   "ask.model.title": "AI model (your key)",
   "ask.nocfg.hint": "No AI model has a key and Javis is not connected. Nothing has been sent.",
   "ask.nocfg.title": "Not configured",
-  "ask.open.ai": "Open the AI Assistant tab",
+  "ask.open.ai": "Open Settings: Translation and AI",
   "banner.mic.perm": "Microphone permission missing. The microphone has no signal (x mark), System audio still works.",
   "cfg.arrow.aria": "translate to",
   "cfg.arrow.tip": "Translate to",
@@ -666,5 +666,10 @@
   "set.usage.none.soniox": "Soniox reports no usage for this week (UTC days).",
   "set.usage.none.localreason": "No local estimate either: nothing was recorded on this computer this week.",
   "set.usage.clear.note": "Deletes only the local estimate stored on this computer. It does not delete anything at Soniox, and the figures from Soniox stay the same.",
-  "set.usage.clear.done": "Local usage data deleted."
+  "set.usage.clear.done": "Local usage data deleted.",
+  "ask.screen.pick": "Transcript",
+  "ask.screen.nosession.title": "Nothing to ask about yet",
+  "ask.screen.nosession.hint": "Record a session or open a saved transcript first, then ask the AI about it.",
+  "ask.screen.gorecord": "Go to Record",
+  "ask.screen.error": "Could not load your sessions."
 }
diff --git a/src/locales/pt-PT.json b/src/locales/pt-PT.json
index 3ca2ed6..052cf44 100644
--- a/src/locales/pt-PT.json
+++ b/src/locales/pt-PT.json
@@ -25,7 +25,7 @@
   "ask.model.title": "Modelo de IA (a sua chave)",
   "ask.nocfg.hint": "Nenhum modelo de IA tem chave e o Javis não está ligado. Nada foi enviado.",
   "ask.nocfg.title": "Não configurado",
-  "ask.open.ai": "Abrir o separador Assistente de IA",
+  "ask.open.ai": "Abrir Definições: Tradução e IA",
   "banner.mic.perm": "Falta a permissão do microfone. O microfone não tem sinal (sinal x), o áudio do sistema continua a funcionar.",
   "cfg.arrow.aria": "traduzir para",
   "cfg.arrow.tip": "Traduzir para",
@@ -666,5 +666,10 @@
   "set.usage.none.soniox": "O Soniox não regista utilização esta semana (dias UTC).",
   "set.usage.none.localreason": "Também não há estimativa local: nada foi registado neste computador esta semana.",
   "set.usage.clear.note": "Elimina apenas a estimativa local guardada neste computador. Não elimina nada no Soniox, e os valores do Soniox mantêm-se.",
-  "set.usage.clear.done": "Dados locais de utilização eliminados."
+  "set.usage.clear.done": "Dados locais de utilização eliminados.",
+  "ask.screen.pick": "Ata",
+  "ask.screen.nosession.title": "Ainda não há nada para perguntar",
+  "ask.screen.nosession.hint": "Grave uma sessão ou abra uma ata guardada e depois pergunte à IA sobre ela.",
+  "ask.screen.gorecord": "Ir para Gravar",
+  "ask.screen.error": "Não foi possível carregar as suas sessões."
 }
diff --git a/src/screens/Record.svelte b/src/screens/Record.svelte
index 82228d0..9e41e43 100644
--- a/src/screens/Record.svelte
+++ b/src/screens/Record.svelte
@@ -5,7 +5,7 @@
   import LangSelect from "../lib/LangSelect.svelte";
   import SourceButton from "../lib/SourceButton.svelte";
   import TermContext from "../lib/TermContext.svelte";
-  import { app, fmtClock, type Level } from "../lib/app.svelte.ts";
+  import { app, fmtClock, openAsk, type Level } from "../lib/app.svelte.ts";
   import { AUTO } from "../lib/langs.ts";
   import * as api from "../lib/api.ts";
   import type { LineDto, SessionStateName, StatusCode } from "../lib/api.ts";
@@ -131,6 +131,9 @@
     localStorage.setItem("rlt.capture.offset_ms", String(app.seconds * 1000));
     await act(api.captureBegin);
   }
+  // Hoi AI: dang ghi thi hoi ve phien dang chay, ghi xong thi ve phien vua ghi (hoac phien moi nhat neu chua co).
+  $effect(() => { if (sessionId) app.lastSession = sessionId; });
+  const askAi = () => openAsk(sessionId);
   const openSubtitle = () => act(api.subtitleOpen);
   function toggle() { if (active) void stop(); else void start(); }
   const stoppedLbl = (n: number) => $_("rec.stopped", { values: { n } });
@@ -151,6 +154,7 @@
     {/if}
   </div>
   <div class="tr2">
+    <button type="button" class="btn" onclick={askAi}><Icon name="sparkle" />{$_("nav.askai")}</button>
     <button type="button" class="btn" onclick={openSubtitle}><Icon name="caption" />{$_("rec.subtitle.open")}</button>
     <button type="button" class="btn" disabled={!active || !sessionId} title={active ? $_("rec.capture") : $_("rec.capture.needsession")} onclick={capture}><Icon name="crop" />{$_("rec.capture")}</button>
     <span class="bd {state === 'recording' ? 'rec' : ''}"><Icon name="dot" />{$_(state === "recording" ? "tool.badge.rec" : statusKey)}</span>
diff --git a/src/screens/Sessions.svelte b/src/screens/Sessions.svelte
index b25fe9a..a2128f8 100644
--- a/src/screens/Sessions.svelte
+++ b/src/screens/Sessions.svelte
@@ -2,9 +2,10 @@
   import { onMount } from "svelte";
   import { _, formatDate, currentLocale, locale } from "../i18n/index.ts";
   import Icon from "../lib/Icon.svelte";
+  import AskAiPopup from "../ai/AskAiPopup.svelte";
   import TermContext from "../lib/TermContext.svelte";
   import { DEFAULT_DST } from "../lib/langs.ts";
-  import { app, fmtClock } from "../lib/app.svelte.ts";
+  import { app, fmtClock, openAiSettings } from "../lib/app.svelte.ts";
   import * as api from "../lib/api.ts";
   import type { SessionRow, SessionDetail, SearchOut } from "../lib/api.ts";
 
@@ -18,6 +19,7 @@
   let drive = $state<api.DriveOut | null>(null);
   let renaming = $state<string | null>(null);
   let confirmDel = $state<string | null>(null);
+  let askOpen = $state(false);
 
   const loc = $derived(($locale as "en" | "pt-PT") ?? currentLocale());
   const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
@@ -49,7 +51,7 @@
   });
 
   async function open(id: string) {
-    error = null; notice = null; drive = null;
+    error = null; notice = null; drive = null; askOpen = false;
     try { detail = await api.sessionGet(id); } catch (e) { error = api.errCode(e); }
   }
   async function guard(fn: () => Promise<void>) {
@@ -94,6 +96,7 @@
       {#each ["md", "txt", "srt"] as const as f (f)}
         <button type="button" class="btn sm" onclick={() => exportAs(f)}><Icon name="download" />{f.toUpperCase()}</button>
       {/each}
+      <button type="button" class="btn sm" aria-pressed={askOpen} onclick={() => (askOpen = !askOpen)}><Icon name="sparkle" />{$_("nav.askai")}</button>
       <button type="button" class="btn sm" onclick={sync}><Icon name="refresh" />{$_("ses.drive.sync")}</button>
     </div>
   </div>
@@ -104,6 +107,13 @@
     {driveLbl(drive ? drive.status : d.session.syncStatus)}
     {#if !(drive?.oauthConnected)} {"·"} {$_("ses.drive.nooauth")}{/if}
   </div>
+  {#if askOpen}
+    <div class="askdock">
+      {#key d.session.sessionId}
+        <AskAiPopup sessionId={d.session.sessionId} onOpenSettings={openAiSettings} onClose={() => (askOpen = false)} />
+      {/key}
+    </div>
+  {/if}
   {#if d.lines.length}
     <TermContext dstLang={d.session.targetLang || DEFAULT_DST}>
     <div class="tr">
@@ -182,3 +192,8 @@
     <div class="empty"><Icon name="clock" /><span>{$_("ses.empty")}</span></div>
   {/if}
 {/if}
+
+<style>
+  .askdock{display:flex;justify-content:center;height:480px;flex:none;padding:8px 18px}
+  .askdock :global(.ask){width:min(720px,100%)}
+</style>
diff --git a/src/screens/Settings.svelte b/src/screens/Settings.svelte
index c8d0df6..ff6695f 100644
--- a/src/screens/Settings.svelte
+++ b/src/screens/Settings.svelte
@@ -9,8 +9,9 @@
   import UsageLogsTab from "./UsageLogsTab.svelte";
   import DriveTab from "./DriveTab.svelte";
   import AboutTab from "./AboutTab.svelte";
+  import type { SettingsTabId } from "../lib/app.svelte.ts";
 
-  type TabId = "general" | "translate" | "shortcuts" | "glossary" | "subtitle" | "usage" | "drive" | "about";
+  type TabId = SettingsTabId;
   let { tab = $bindable("general") }: { tab?: TabId } = $props();
   const tabs: { id: TabId; key: string; icon: string; group: string }[] = [
     { id: "general", key: "set.tab.general", icon: "gear", group: "set.tab.basic" },
diff --git a/test/e2e/tauri-mock.ts b/test/e2e/tauri-mock.ts
index dae7aaa..bfa12b6 100644
--- a/test/e2e/tauri-mock.ts
+++ b/test/e2e/tauri-mock.ts
@@ -43,6 +43,12 @@ export function installTauriMock(seed: MockSeed): void {
       chips: { summary: null, explain: null, action_items: null },
     },
     usageCleared: false,
+    aiConsent: {} as Record<string, boolean>,
+    convs: [] as any[],
+    msgs: {} as Record<number, any[]>,
+    listeners: [] as { event: string; id: number; cb: number }[],
+    nextConv: 1,
+    nextMsg: 1,
   };
   const rlt: any = { calls: [] as { cmd: string; args: any }[], unhandled: [] as string[], state, fail: { ...seed.fail } };
   w.__rlt = rlt;
@@ -54,6 +60,11 @@ export function installTauriMock(seed: MockSeed): void {
     if (!cb) throw new Error("khong co Channel dang mo");
     cb({ index: state.channelIndex++, message: event });
   };
+  // Su kien Tauri (listen): giong `plugin:event|listen`; test phat bang __rlt.emitEvent(ten, payload).
+  rlt.emitEvent = (event: string, payload: unknown) => {
+    for (const l of state.listeners.filter((x: any) => x.event === event)) callbacks.get(l.cb)?.({ event, id: l.id, payload });
+  };
+  w.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() { /* khong can */ } };
   const emitState = (s: string) => { state.run = s; rlt.emit({ kind: "state", state: s }); };
 
   const aiSettings = () => ({
@@ -62,7 +73,7 @@ export function installTauriMock(seed: MockSeed): void {
     context_max_turns: state.ai.context_max_turns,
     providers: (["gemini", "openai", "ollama"] as const).map(p => ({
       provider: p, model_name: state.ai.models[p], has_key: p === "ollama" ? false : !!state.aiKeys[p],
-      base_url: p === "ollama" ? "http://localhost:11434" : null, consented: false,
+      base_url: p === "ollama" ? "http://localhost:11434" : null, consented: !!state.aiConsent[p],
     })),
     chip_prompts: (["summary", "explain", "action_items"] as const).map(c => ({ chip: c, prompt: state.ai.chips[c] })),
   });
@@ -136,6 +147,8 @@ export function installTauriMock(seed: MockSeed): void {
     glossary_list: () => [],
     drive_status: () => ["not_connected", null],
     shell_set_locale: () => undefined,
+    shell_set_recording: () => undefined,
+    shell_set_keep_awake: () => undefined,
     autostart_get: () => false,
     core_version: () => "0.1.0",
     quick_ax_status: () => true,
@@ -183,6 +196,35 @@ export function installTauriMock(seed: MockSeed): void {
     ai_set_api_key: (a) => { state.aiKeys[a.provider] = true; },
     ai_clear_api_key: (a) => { state.aiKeys[a.provider] = false; },
     ai_test_connection: () => undefined,
+    "plugin:event|listen": (a) => { const id = state.listeners.length + 1; state.listeners.push({ event: a.event, id, cb: a.handler }); return id; },
+    "plugin:event|unlisten": (a) => { state.listeners = state.listeners.filter((l: any) => !(l.event === a.event && l.id === a.eventId)); },
+    ai_accept_consent: (a) => { state.aiConsent[a.provider] = true; },
+    ai_list_conversations: (a) => state.convs.filter((c: any) => c.session_id === a.sessionId).map(clone),
+    ai_list_messages: (a) => clone(state.msgs[a.conversationId] ?? []),
+    ai_clear_history: (a) => { const n = (state.msgs[a.conversationId] ?? []).length; state.msgs[a.conversationId] = []; return n; },
+    ai_delete_conversation: (a) => { state.convs = state.convs.filter((c: any) => c.conversation_id !== a.conversationId); return true; },
+    ai_cancel: () => undefined,
+    ai_ask: (a) => {
+      const p = a.params;
+      let conv = state.convs.find((c: any) => c.conversation_id === p.conversation_id);
+      if (!conv) {
+        conv = { conversation_id: state.nextConv++, session_id: p.session_id, provider: p.provider ?? "gemini", model_name: null, title: p.text ?? p.chip, updated_at: "2026-11-14T10:00:00Z" };
+        state.convs.push(conv); state.msgs[conv.conversation_id] = [];
+      }
+      const list = state.msgs[conv.conversation_id];
+      const mk = (role: string, content: string, chip: string | null, reply: number | null) => ({
+        message_id: state.nextMsg++, conversation_id: conv.conversation_id, role, reply_to_message_id: reply, chip_kind: chip,
+        content, status: "completed", model_name: null, error_code: null, context_truncated: null,
+      });
+      const u = mk("user", p.text ?? p.chip, p.chip, null);
+      const reply = "Reply to: " + (p.text ?? p.chip) + " (turn " + (list.filter((m: any) => m.role === "user").length + 1) + ")";
+      const ai = mk("assistant", reply, null, u.message_id);
+      list.push(u, ai);
+      rlt.aiAsks = (rlt.aiAsks ?? []).concat([{ session_id: p.session_id, text: p.text, chip: p.chip, conversation_id: p.conversation_id }]);
+      const ch = a.onUpdate?.id;
+      if (ch !== undefined) callbacks.get(ch)?.({ index: 0, message: { assistant_message_id: ai.message_id, text: reply } });
+      return { conversation_id: conv.conversation_id, user_message_id: u.message_id, assistant_message_id: ai.message_id, status: "completed", error_code: null, text: reply };
+    },
   };
 
   w.__TAURI_INTERNALS__ = {
```
