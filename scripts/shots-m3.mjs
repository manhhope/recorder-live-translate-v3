// Chup anh man hinh M3 (loi goi giao dien) bang Playwright chromium, doi chieu voi mockup phuong an A.
// Lop invoke cua Tauri duoc gia ngay trong file nay (chi de chup), khong dung vao test/e2e cua T2.
// Chay: npm run build && node scripts/shots-m3.mjs   (can `npx playwright install chromium` mot lan)
import { chromium } from "@playwright/test";
import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const out = path.join(root, "docs", "anh", "m3-giao-dien");
mkdirSync(out, { recursive: true });
const PORT = 4174;
const mockup = path.resolve(root, "..", "recorder-live-translate", "docs", "design-system-v3", "phuong-an-a-quiet-native.html");

const srv = spawn("npx", ["vite", "preview", "--outDir", "dist", "--host", "127.0.0.1", "--port", String(PORT), "--strictPort"], { cwd: root, stdio: "ignore" });
await new Promise(r => setTimeout(r, 2500));

function mock(locale) {
  const w = window;
  localStorage.setItem("rlt.ui.locale", locale);
  const shortcuts = [
    ["quick_translate", "CommandOrControl+Shift+T"], ["inline_translate", "CommandOrControl+Shift+I"], ["ask_ai", "CommandOrControl+Shift+A"], ["region_capture", "CommandOrControl+Shift+C"],
  ].map(([action, accelerator]) => ({ action, accelerator, enabled: true, registered: true, error_key: null }));
  const sub = { display_mode: "bilingual", font_size_pt: 28, opacity_percent: 40, source_color: "#FFFFFF", translation_color: "#FFD60A", hidden_from_capture: true };
  const terms = [
    { id: 1, termSource: "Soniox", termTarget: null, keepOriginal: true, sourceLang: "en", targetLang: "pt-PT" },
    { id: 2, termSource: "latency", termTarget: "latencia", keepOriginal: false, sourceLang: "en", targetLang: "pt-PT" },
  ];
  const h = {
    consent_get: () => true, soniox_has_api_key: () => true, session_state: () => ["recording", "s-1"],
    record_prefs_get: () => ({ targetLang: "pt-PT", listenLangs: ["en"], mode: "mixed", uiLanguage: locale }),
    permissions_status: () => ({ mic: "granted", screen: "denied" }), quick_ax_status: () => false, core_version: () => "0.1.0",
    shell_set_locale: () => null, shell_set_keep_awake: () => null, autostart_get: () => false, autostart_set: () => null,
    shortcuts_status: () => shortcuts, subtitle_get: () => sub, glossary_list: () => terms,
    deepl_has_key: () => true, deepl_test_connection: () => ({ character_count: 12345, character_limit: 500000 }),
    ai_get_settings: () => ({ default_provider: null, context_max_chars: 60000, context_max_turns: 10, providers: ["gemini", "openai", "ollama"].map(p => ({ provider: p, model_name: "m", has_key: p === "gemini", base_url: null, consented: false })), chip_prompts: [] }),
  };
  const cbs = new Map(); let n = 1;
  w.__TAURI_INTERNALS__ = {
    transformCallback(cb) { const id = n++; cbs.set(id, cb); return id; }, unregisterCallback(id) { cbs.delete(id); },
    async invoke(cmd, args) {
      const f = h[cmd]; if (!f) throw "mock_unhandled:" + cmd;
      await Promise.resolve();
      return f(args);
    },
  };
}

const browser = await chromium.launch({ headless: true });
const ctx = await browser.newContext({ viewport: { width: 1100, height: 760 } });
const shot = async (page, name) => { await page.waitForTimeout(300); await page.screenshot({ path: path.join(out, name + ".png") }); console.log("shot", name); };

try {
  for (const loc of ["en", "pt-PT"]) {
    const page = await ctx.newPage();
    await page.addInitScript(mock, loc);
    await page.goto(`http://127.0.0.1:${PORT}/index.html`);
    await page.waitForSelector(".app");
    await shot(page, `record-${loc}`);
    await page.click(".nav .ni >> nth=2");
    const tabs = await page.$$(".snav .sn");
    const names = ["general", "translate", "shortcuts", "glossary", "subtitle", "usage", "drive", "about"];
    const pick = loc === "en" ? names.map((_, i) => i) : [0, 7];
    for (const i of pick) { if (tabs[i]) { await tabs[i].click(); await shot(page, `settings-${names[i]}-${loc}`); } }
    await page.close();
  }
  const mk = await ctx.newPage();
  await mk.goto("file://" + mockup);
  await mk.waitForTimeout(500);
  await mk.screenshot({ path: path.join(out, "mockup-A.png") });
  for (const id of ["M-17", "M-20", "M-01"]) {
    const el = mk.locator(`[id="${id}"], [data-id="${id}"], section:has(.badge:text-is("${id}"))`).first();
    if (await el.count()) { await el.screenshot({ path: path.join(out, `mockup-A-${id}.png`) }).catch(() => {}); }
  }
} finally {
  await browser.close();
  srv.kill();
}
