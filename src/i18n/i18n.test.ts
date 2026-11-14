import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { spawnSync } from "node:child_process";
import { get } from "svelte/store";
import { setupI18n, setLocale, _, locale, DEFAULT_LOCALE, missingKeys, resolveInitialLocale, isValidKey } from "./index.ts";
import { formatDate, formatNumber, formatCurrency, formatTime } from "./format.ts";
import { diffKeys, findFakeStrings, hasVietnamese } from "./checks.ts";

const dir = path.join(import.meta.dirname, "..", "locales");
const en: Record<string, string> = JSON.parse(fs.readFileSync(path.join(dir, "en.json"), "utf8"));
const pt: Record<string, string> = JSON.parse(fs.readFileSync(path.join(dir, "pt-PT.json"), "utf8"));

test("tap khoa en va pt-PT khop nhau (0 thieu, 0 thua)", () => {
  const d = diffKeys(en, pt);
  assert.deepEqual(d, { missingInB: [], extraInB: [] });
  assert.ok(Object.keys(en).length > 300);
});

test("diffKeys bao loi khi thieu khoa va khi co khoa thua", () => {
  const broken = { ...pt }; delete broken["nav.record"]; broken["x.extra"] = "x";
  assert.deepEqual(diffKeys(en, broken), { missingInB: ["nav.record"], extraInB: ["x.extra"] });
});

test("moi khoa deu hop le theo quy uoc dat khoa", () => {
  for (const k of Object.keys(en)) assert.ok(isValidKey(k), k);
  assert.equal(isValidKey("Bad Key"), false);
});

test("khong co chuoi gia, trong, placeholder trong en va pt-PT", () => {
  assert.deepEqual(findFakeStrings(en), []);
  assert.deepEqual(findFakeStrings(pt), []);
});

test("findFakeStrings bat duoc chuoi gia", () => {
  const bad = { "a.b": "", "a.c": "TODO", "a.d": "lorem ipsum dolor", "a.e": "a.e", "a.f": "[[a.f]]", "a.g": "TBD translate", "a.h": "   ", "a.i": "Ok" };
  assert.deepEqual(findFakeStrings(bad).map(f => f.key), ["a.b", "a.c", "a.d", "a.e", "a.f", "a.g", "a.h"]);
});

test("0 ky tu hay chuoi tieng Viet trong en va pt-PT", () => {
  for (const [k, v] of [...Object.entries(en), ...Object.entries(pt)]) assert.equal(hasVietnamese(v), false, `${k}: ${v}`);
  assert.equal(hasVietnamese("Tiếng Việt"), true);
  assert.equal(hasVietnamese("Português, coração, não"), false);
});

test("locale mac dinh la pt-PT bat ke gia tri luu khong hop le", () => {
  assert.equal(DEFAULT_LOCALE, "pt-PT");
  assert.equal(resolveInitialLocale(null), "pt-PT");
  assert.equal(resolveInitialLocale("vi"), "pt-PT");
  assert.equal(resolveInitialLocale("en"), "en");
});

test("giao dien chay pt-PT mac dinh, doi sang en ngay, khong con chuoi en o pt-PT", async () => {
  await setupI18n(null);
  assert.equal(get(locale), "pt-PT");
  assert.equal(get(_)("nav.record"), "Gravar");
  await setLocale("en");
  assert.equal(get(_)("nav.record"), "Record");
  await setLocale("pt-PT");
  assert.equal(get(_)("nav.record"), "Gravar");
});

test("thieu khoa o pt-PT roi ve chu tieng Anh va ghi canh bao (AC-49.3)", async () => {
  const gone = ["nav.record", "nav.tools", "tool.start", "tool.stop", "common.cancel"];
  const broken = { ...pt }; for (const k of gone) delete broken[k];
  const warn = console.warn; const logs: string[] = []; console.warn = (m: string) => { logs.push(m); };
  try {
    await setupI18n("pt-PT", { en, "pt-PT": broken });
    for (const k of gone) assert.equal(get(_)(k), en[k]);
  } finally { console.warn = warn; }
  assert.deepEqual([...missingKeys].sort(), [...gone].sort());
  assert.equal(logs.length, 5);
  await setupI18n(null);
});

test("dinh dang ngay khac nhau dung theo locale", () => {
  const d = new Date(Date.UTC(2026, 10, 14, 15, 30));
  assert.equal(formatDate(d, "pt-PT"), "14/11/2026");
  assert.equal(formatDate(d, "en"), "11/14/2026");
  assert.equal(formatTime(d, "pt-PT"), "15:30");
  assert.equal(formatTime(d, "en"), "15:30");
  assert.notEqual(formatDate(d, "pt-PT"), formatDate(d, "en"));
});

test("dinh dang so va tien khac nhau dung theo locale", () => {
  assert.equal(formatNumber(1234.5, "en"), "1,234.5");
  // CLDR pt-PT: so 4 chu so khong nhom hang nghin ("1234,5"); tu 5 chu so tro len dung dau cach.
  assert.equal(formatNumber(1234.5, "pt-PT"), "1234,5");
  assert.equal(formatNumber(1.2, "pt-PT"), "1,2");
  assert.equal(formatNumber(41200, "pt-PT").replace(/\s/gu, " "), "41 200");
  assert.equal(formatCurrency(12.5, "en"), "$12.50");
  assert.match(formatCurrency(12.5, "pt-PT"), /^12,50\s(US\$|USD)$/u);
});

test("script quet chuoi cung bat duoc chuoi cung va cho qua ma dung i18n", () => {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "scan-"));
  const run = (content: string) => {
    fs.writeFileSync(path.join(tmp, "A.svelte"), content);
    return spawnSync(process.execPath, [path.join(import.meta.dirname, "..", "..", "scripts", "scan-hardcoded.mjs"), tmp], { encoding: "utf8" });
  };
  const bad = run(`<script>let n=1;</script>\n<button title="Save">Guardar</button>\n<p>{$_('a.b')}</p>`);
  assert.equal(bad.status, 1);
  assert.match(bad.stdout, /So chuoi cung tim thay: 2/);
  const good = run(`<script>let n=1;</script>\n<button title={$_('a.t')}>{$_('a.b')}</button>\n<span>{n}</span>`);
  assert.equal(good.status, 0, good.stdout);
  assert.match(good.stdout, /So chuoi cung tim thay: 0/);
  fs.rmSync(tmp, { recursive: true });
});
