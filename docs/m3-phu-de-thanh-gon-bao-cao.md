# M3: thanh điều khiển cửa sổ phụ đề gọn một dòng

## Đã làm
- `src/subtitle/SubtitleApp.svelte`: thanh còn MỘT dòng cao 24px sát mép trên (A-, cỡ chữ, A+, chọn chế độ hiển thị, 2 ô màu tròn, thanh trượt độ mờ 56px, nút ba chấm, nút đóng). Reset colours, Reset position, Hide from screen sharing nằm trong menu ba chấm. Thanh luôn có mặt (mờ còn 25% khi chuột ngoài cửa sổ) nên vẫn còn vùng kéo; bấm vào nút/ô chọn/menu không kích hoạt kéo. Vùng chữ phụ đề chiếm phần còn lại bên dưới (flex:1).
- Style theo phương án A: nút mini 22px viền trắng 35%, bo 5px, nền menu tối, chữ #D9D9DE.
- i18n: thêm `sub.more` ở `src/locales/en.json` ("More options") và `pt-PT.json` ("Mais opções"). Không chuỗi cứng.
- Không đụng Rust, hành vi NSPanel không kích hoạt giữ nguyên.

## Kiểm
- `npm run build`: built OK.
- `npm run test:i18n`: tests 12, pass 12, fail 0.
- `npm run scan:i18n`: "So chuoi cung tim thay: 0".
- `cargo test --workspace`: toàn bộ "test result: ok", 0 failed (output thật ở dưới).
- Playwright: chưa có spec cho trang phụ đề, không chạy. Chụp ảnh bằng chromium với mock Tauri.
- Ảnh sau: `docs/img/m3-phu-de-sau-mo.png` (mờ), `m3-phu-de-sau-hover.png`, `m3-phu-de-sau-menu.png`. Ảnh trước: không chụp lại, dùng ảnh phản hồi của chủ. Chưa kiểm trên cửa sổ Tauri thật (menu có thể bị cắt nếu cửa sổ thấp hơn ~120px).
- Lưu ý: không tìm thấy `docs/design-system-v3/phuong-an-a-quiet-native.html` trong v3, dùng bản ở `projects/recorder-live-translate/docs/design-system-v3/`.

## Diff thật (commit 14a23ea)
```diff
commit 14a23ea883a6f84111d8131b485dc9516eeb0b3e
Author: Nguyen Manh <nguyenmanh@Nguyens-Mac-mini.local>
Date:   Sat Nov 14 19:00:36 2026 +0200

    M3: thanh phu de gon mot dong, menu ba cham
    
    Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>

diff --git a/src/locales/en.json b/src/locales/en.json
index ef6b359..701efea 100644
--- a/src/locales/en.json
+++ b/src/locales/en.json
@@ -368,6 +368,7 @@
   "sub.font.increase": "Larger text",
   "sub.hideshare": "Hide from screen sharing",
   "sub.hideshare.note": "Best effort. Some sharing apps can still capture the window.",
+  "sub.more": "More options",
   "sub.mode.bilingual": "Both languages",
   "sub.mode.label": "Display mode",
   "sub.mode.source": "Original only",
diff --git a/src/locales/pt-PT.json b/src/locales/pt-PT.json
index 67c65e8..1778559 100644
--- a/src/locales/pt-PT.json
+++ b/src/locales/pt-PT.json
@@ -368,6 +368,7 @@
   "sub.font.increase": "Texto maior",
   "sub.hideshare": "Ocultar na partilha de ecrã",
   "sub.hideshare.note": "Melhor esforço. Algumas aplicações de partilha ainda podem captar a janela.",
+  "sub.more": "Mais opções",
   "sub.mode.bilingual": "Ambos os idiomas",
   "sub.mode.label": "Modo de apresentação",
   "sub.mode.source": "Só original",
diff --git a/src/subtitle/SubtitleApp.svelte b/src/subtitle/SubtitleApp.svelte
index 5726561..9fd727d 100644
--- a/src/subtitle/SubtitleApp.svelte
+++ b/src/subtitle/SubtitleApp.svelte
@@ -11,7 +11,8 @@
     opacity_percent: 0, hidden_from_capture: true,
   });
   let line = $state<Line | null>(null);
-  let toolsOpen = $state(false);
+  let hover = $state(false);
+  let menuOpen = $state(false);
   let err = $state("");
 
   const rows = $derived(rowsFor(s.display_mode, line));
@@ -35,29 +36,37 @@
     try { await invoke("subtitle_set_color", { which, hex: (e.target as HTMLInputElement).value }); }
     catch (code) { const c = colorError(String(code)); err = $_(c.key, { values: c.values }); }
   }
-  const startDrag = (e: MouseEvent) => { if (e.button === 0 && !(e.target as HTMLElement).closest(".tools")) getCurrentWindow().startDragging(); };
+  const startDrag = (e: MouseEvent) => { if (e.button === 0 && !(e.target as HTMLElement).closest("button, select, input, label, .menu")) getCurrentWindow().startDragging(); };
 </script>
 
 <!-- svelte-ignore a11y_no_static_element_interactions -->
 <div class="sub" style:background={backgroundCss(s.opacity_percent)} onmousedown={startDrag}
-     onmouseenter={() => (toolsOpen = true)} onmouseleave={() => (toolsOpen = false)} role="presentation">
-  {#if toolsOpen}
-    <div class="tools">
-      <button onclick={() => step(-1)} disabled={s.font_size_pt <= 12} title={$_("sub.font.decrease")}>{$_("sub.font.decrease.short")}</button>
-      <span>{s.font_size_pt}{$_("sub.font.unit")}</span>
-      <button onclick={() => step(1)} disabled={s.font_size_pt >= 36} title={$_("sub.font.increase")}>{$_("sub.font.increase.short")}</button>
-      <select aria-label={$_("sub.mode.label")} value={s.display_mode} onchange={e => setMode((e.target as HTMLSelectElement).value as DisplayMode)}>
-        {#each MODES as m}<option value={m.mode}>{$_(m.key)}</option>{/each}
-      </select>
-      <input type="color" aria-label={$_("sub.color.source")} title={$_("sub.color.source")} value={s.source_color} onchange={e => setColor("source", e)} />
-      <input type="color" aria-label={$_("sub.color.translation")} title={$_("sub.color.translation")} value={s.translation_color} onchange={e => setColor("translation", e)} />
-      <button onclick={() => invoke("subtitle_reset_colors")}>{$_("sub.color.reset")}</button>
-      <input type="range" min="0" max="80" step="5" aria-label={$_("sub.opacity")} title={$_("sub.opacity")} value={s.opacity_percent} onchange={setOpacity} />
-      <label title={$_("sub.hideshare.note")}><input type="checkbox" checked={s.hidden_from_capture} onchange={setHidden} /> {$_("sub.hideshare")}</label>
-      <button onclick={() => invoke("subtitle_reset_position")}>{$_("sub.resetposition")}</button>
-      <button onclick={() => invoke("subtitle_close")} aria-label={$_("sub.close")}>{$_("sub.close.short")}</button>
+     onmouseenter={() => (hover = true)} onmouseleave={() => { hover = false; menuOpen = false; }} role="presentation">
+  <div class="tools" class:dim={!hover && !menuOpen} data-drag>
+    <button class="mini" onclick={() => step(-1)} disabled={s.font_size_pt <= 12} title={$_("sub.font.decrease")} aria-label={$_("sub.font.decrease")}>{$_("sub.font.decrease.short")}</button>
+    <span class="pt">{s.font_size_pt}{$_("sub.font.unit")}</span>
+    <button class="mini" onclick={() => step(1)} disabled={s.font_size_pt >= 36} title={$_("sub.font.increase")} aria-label={$_("sub.font.increase")}>{$_("sub.font.increase.short")}</button>
+    <select aria-label={$_("sub.mode.label")} title={$_("sub.mode.label")} value={s.display_mode} onchange={e => setMode((e.target as HTMLSelectElement).value as DisplayMode)}>
+      {#each MODES as m}<option value={m.mode}>{$_(m.key)}</option>{/each}
+    </select>
+    <input class="sw" type="color" aria-label={$_("sub.color.source")} title={$_("sub.color.source")} value={s.source_color} onchange={e => setColor("source", e)} />
+    <input class="sw" type="color" aria-label={$_("sub.color.translation")} title={$_("sub.color.translation")} value={s.translation_color} onchange={e => setColor("translation", e)} />
+    <input class="rg" type="range" min="0" max="80" step="5" aria-label={$_("sub.opacity")} title={$_("sub.opacity")} value={s.opacity_percent} onchange={setOpacity} />
+    <span class="grow"></span>
+    <div class="more">
+      <button class="mini" onclick={() => (menuOpen = !menuOpen)} title={$_("sub.more")} aria-label={$_("sub.more")} aria-expanded={menuOpen}>
+        <svg class="i" viewBox="0 0 16 16"><circle cx="3" cy="8" r="1"/><circle cx="8" cy="8" r="1"/><circle cx="13" cy="8" r="1"/></svg>
+      </button>
+      {#if menuOpen}
+        <div class="menu" role="menu">
+          <button role="menuitem" onclick={() => { invoke("subtitle_reset_colors"); menuOpen = false; }}>{$_("sub.color.reset")}</button>
+          <button role="menuitem" onclick={() => { invoke("subtitle_reset_position"); menuOpen = false; }}>{$_("sub.resetposition")}</button>
+          <label title={$_("sub.hideshare.note")}><input type="checkbox" checked={s.hidden_from_capture} onchange={setHidden} /> {$_("sub.hideshare")}</label>
+        </div>
+      {/if}
     </div>
-  {/if}
+    <button class="mini" onclick={() => invoke("subtitle_close")} title={$_("sub.close")} aria-label={$_("sub.close")}>{$_("sub.close.short")}</button>
+  </div>
   {#if err}<p class="err" role="alert">{err}</p>{/if}
   <div class="text" style:font-size="{s.font_size_pt}px">
     {#each rows as r}
@@ -70,9 +79,22 @@
 
 <style>
   :global(html), :global(body) { margin: 0; background: transparent; overflow: hidden; font-family: -apple-system, system-ui, sans-serif; }
-  .sub { box-sizing: border-box; height: 100vh; padding: 10px 16px; border-radius: 10px; display: flex; flex-direction: column; justify-content: flex-end; user-select: none; }
-  .tools { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 12px; color: #d0d6dc; margin-bottom: 6px; }
-  .tools button, .tools select { font-size: 12px; }
+  .sub { box-sizing: border-box; height: 100vh; padding: 0 14px 8px; border-radius: 12px; display: flex; flex-direction: column; user-select: none; }
+  .tools { flex: none; height: 24px; display: flex; flex-wrap: nowrap; gap: 6px; align-items: center; font-size: 11px; color: #d9d9de; transition: opacity 0.15s; position: relative; }
+  .tools.dim { opacity: 0.25; }
+  .tools .pt { min-width: 28px; text-align: center; }
+  .tools .grow { flex: 1; min-width: 0; }
+  .mini { min-width: 22px; height: 22px; padding: 0 4px; display: inline-grid; place-items: center; font-size: 11px; color: #fff; background: transparent; border: 1px solid rgba(255, 255, 255, 0.35); border-radius: 5px; cursor: pointer; }
+  .mini:disabled { opacity: 0.4; cursor: not-allowed; }
+  .i { width: 14px; height: 14px; fill: currentColor; }
+  .tools select { font-size: 11px; height: 22px; max-width: 120px; }
+  .sw { width: 22px; height: 22px; padding: 0; border: 1px solid rgba(255, 255, 255, 0.35); border-radius: 50%; background: none; cursor: pointer; }
+  .rg { width: 56px; min-width: 0; }
+  .more { position: relative; }
+  .menu { position: absolute; right: 0; top: 26px; z-index: 5; display: flex; flex-direction: column; gap: 2px; padding: 4px; min-width: 190px; background: rgba(16, 16, 18, 0.96); border: 1px solid rgba(255, 255, 255, 0.12); border-radius: 8px; }
+  .menu button, .menu label { text-align: left; font-size: 12px; color: #fff; background: transparent; border: 0; padding: 5px 8px; border-radius: 5px; cursor: pointer; display: flex; gap: 6px; align-items: center; }
+  .menu button:hover, .menu label:hover { background: rgba(255, 255, 255, 0.12); }
+  .text { flex: 1; min-height: 0; display: flex; flex-direction: column; justify-content: flex-end; }
   .text p { margin: 0; line-height: 1.35; text-shadow: 0 1px 2px rgba(0, 0, 0, 0.6); }
   .wait { color: #8a939c; font-size: 14px; }
   .err { color: #ffb4a8; font-size: 12px; margin: 0 0 4px; }
```

## Output thật các lệnh kiểm (chạy lại ngày 2026-11-14, PATH có ~/.cargo/bin)

### npm run build (25 dòng cuối)
```
vite v8.3.8 building client environment for production...
transforming...
✓ 189 modules transformed.
rendering chunks...
computing gzip size...
dist/capture.html                    0.39 kB │ gzip:  0.25 kB
dist/index.html                      0.45 kB │ gzip:  0.27 kB
dist/subtitle.html                   0.46 kB │ gzip:  0.27 kB
dist/ask-ai.html                     0.50 kB │ gzip:  0.27 kB
dist/quick.html                      0.51 kB │ gzip:  0.28 kB
dist/assets/capture-CLPkpjn8.css     0.43 kB │ gzip:  0.28 kB
dist/assets/quick-aKWaxB_i.css       1.19 kB │ gzip:  0.49 kB
dist/assets/ask-ai-DcznQoGE.css      1.78 kB │ gzip:  0.61 kB
dist/assets/subtitle-DxRgPZ0Y.css    2.16 kB │ gzip:  0.73 kB
dist/assets/app-8TywOAV0.css        14.64 kB │ gzip:  3.45 kB
dist/assets/event-C6dsb0za.js        1.31 kB │ gzip:  0.62 kB
dist/assets/capture-CNggIVPf.js      1.89 kB │ gzip:  1.03 kB
dist/assets/api-BtvFmX2P.js          3.80 kB │ gzip:  1.65 kB
dist/assets/quick-BawLA5N-.js        7.60 kB │ gzip:  3.13 kB
dist/assets/ask-ai-DtDCmDen.js       9.43 kB │ gzip:  3.60 kB
dist/assets/subtitle-B-qRxE-8.js    19.21 kB │ gzip:  5.45 kB
dist/assets/main-BzVSJFcl.js        82.78 kB │ gzip: 24.52 kB
dist/assets/core-D8HG1jMO.js       160.75 kB │ gzip: 52.22 kB

✓ built in 277ms
```

### npm run test:i18n
```

> recorder-live-translate-v3@0.1.0 test:i18n
> node --test src/i18n/i18n.test.ts

✔ tap khoa en va pt-PT khop nhau (0 thieu, 0 thua) (0.86525ms)
✔ diffKeys bao loi khi thieu khoa va khi co khoa thua (1.300958ms)
✔ moi khoa deu hop le theo quy uoc dat khoa (0.248875ms)
✔ khong co chuoi gia, trong, placeholder trong en va pt-PT (0.650958ms)
✔ findFakeStrings bat duoc chuoi gia (0.078584ms)
✔ 0 ky tu hay chuoi tieng Viet trong en va pt-PT (0.721375ms)
✔ locale mac dinh la pt-PT bat ke gia tri luu khong hop le (0.057667ms)
✔ giao dien chay pt-PT mac dinh, doi sang en ngay, khong con chuoi en o pt-PT (9.102291ms)
✔ thieu khoa o pt-PT roi ve chu tieng Anh va ghi canh bao (AC-49.3) (0.773541ms)
✔ dinh dang ngay khac nhau dung theo locale (3.3225ms)
✔ dinh dang so va tien khac nhau dung theo locale (0.435917ms)
✔ script quet chuoi cung bat duoc chuoi cung va cho qua ma dung i18n (175.502709ms)
ℹ tests 12
ℹ suites 0
ℹ pass 12
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 345.834208
```

### npm run scan:i18n
```

> recorder-live-translate-v3@0.1.0 scan:i18n
> node scripts/scan-hardcoded.mjs src

Quet 43 tep trong src/ (bo qua locales, i18n)
So chuoi cung tim thay: 0
```

### cargo test --workspace (các dòng running/test result)
```
     Running unittests src/lib.rs (target/debug/build/rlt-ai/b2d18ddf666c4ceb/out/rlt_ai-b2d18ddf666c4ceb)
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/providers.rs (target/debug/build/rlt-ai/ceea3fe981c94d32/out/providers-ceea3fe981c94d32)
running 25 tests
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
     Running unittests src/main.rs (target/debug/build/rlt-app/a878c6db0620467e/out/rlt_app-a878c6db0620467e)
running 113 tests
test result: ok. 113 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.17s
     Running unittests src/lib.rs (target/debug/build/rlt-core/b93904f75e01b2f3/out/rlt_core-b93904f75e01b2f3)
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/session.rs (target/debug/build/rlt-core/463e85e176b480b8/out/session-463e85e176b480b8)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s
     Running unittests src/lib.rs (target/debug/build/rlt-features/51224881cf0717d3/out/rlt_features-51224881cf0717d3)
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/drive.rs (target/debug/build/rlt-features/3a65bfdf7a0290c5/out/drive-3a65bfdf7a0290c5)
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/drive_google.rs (target/debug/build/rlt-features/445cbccc083f58a2/out/drive_google-445cbccc083f58a2)
running 23 tests
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running tests/export.rs (target/debug/build/rlt-features/8c1d6c2700ec13e1/out/export-8c1d6c2700ec13e1)
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running tests/glossary.rs (target/debug/build/rlt-features/a8e3ea3e04203140/out/glossary-a8e3ea3e04203140)
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
     Running tests/highlights_search.rs (target/debug/build/rlt-features/2895249aa9c9b4ee/out/highlights_search-2895249aa9c9b4ee)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
     Running tests/import_dotnet.rs (target/debug/build/rlt-features/c91074324b11a7bf/out/import_dotnet-c91074324b11a7bf)
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.17s
     Running tests/usage.rs (target/debug/build/rlt-features/3cebdb2c8d21ae52/out/usage-3cebdb2c8d21ae52)
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/usage_local_date.rs (target/debug/build/rlt-features/4ee72c27b9096d7f/out/usage_local_date-4ee72c27b9096d7f)
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/usage_local_tz.rs (target/debug/build/rlt-features/ab6cf85e484fcad0/out/usage_local_tz-ab6cf85e484fcad0)
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/build/rlt-platform/d47b33503f430b0a/out/rlt_platform-d47b33503f430b0a)
running 51 tests
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running unittests src/bin/s1_capture.rs (target/debug/build/rlt-platform/d05a1cd3f83502d1/out/s1_capture-d05a1cd3f83502d1)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/build/rlt-quick/2938251c030ab334/out/rlt_quick-2938251c030ab334)
running 93 tests
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.23s
     Running unittests src/lib.rs (target/debug/build/rlt-secrets/5980eea0ac8f719c/out/rlt_secrets-5980eea0ac8f719c)
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running unittests src/lib.rs (target/debug/build/rlt-segment/da07951ce6cba38b/out/rlt_segment-da07951ce6cba38b)
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/golden_cases.rs (target/debug/build/rlt-segment/275a2350109529d6/out/golden_cases-275a2350109529d6)
running 158 tests
test result: ok. 158 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.61s
     Running unittests src/lib.rs (target/debug/build/rlt-soniox/0d61b15e037c359b/out/rlt_soniox-0d61b15e037c359b)
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/fake_server.rs (target/debug/build/rlt-soniox/267b9b48d133bf17/out/fake_server-267b9b48d133bf17)
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
     Running unittests src/lib.rs (target/debug/build/rlt-store/0b94d1db8b8e6db9/out/rlt_store-0b94d1db8b8e6db9)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/store.rs (target/debug/build/rlt-store/5db7ed22a0d27efe/out/store-5db7ed22a0d27efe)
running 17 tests
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
