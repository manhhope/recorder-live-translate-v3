import { test } from "node:test";
import assert from "node:assert/strict";
import { loadExportPrefs, saveExportPrefs, rememberDir, EXPORT_PREFS_KEY } from "./export-dir.ts";

const mem = () => { const m = new Map<string, string>(); return { getItem: (k: string) => m.get(k) ?? null, setItem: (k: string, v: string) => void m.set(k, v), m }; };

test("chua luu gi: mac dinh md/both va dir null (giao dien hien Documents)", () => {
  assert.deepEqual(loadExportPrefs(mem()), { format: "md", content: "both", dir: null });
});
test("luu roi doc lai: nho thu muc da chon", () => {
  const s = mem();
  assert.equal(saveExportPrefs(s, { format: "srt", content: "source", dir: "/Users/a/Exports" }), true);
  assert.deepEqual(loadExportPrefs(s), { format: "srt", content: "source", dir: "/Users/a/Exports" });
});
test("du lieu hong hoac gia tri la bi bo qua", () => {
  const s = mem(); s.setItem(EXPORT_PREFS_KEY, "{khong phai json");
  assert.equal(loadExportPrefs(s).dir, null);
  s.setItem(EXPORT_PREFS_KEY, JSON.stringify({ format: "pdf", content: 3, dir: "" }));
  assert.deepEqual(loadExportPrefs(s), { format: "md", content: "both", dir: null });
});
test("kho day hoac bi cam: saveExportPrefs tra false, khong nem", () => {
  const bad = { getItem: () => null, setItem: () => { throw new Error("quota"); } };
  assert.equal(saveExportPrefs(bad, { format: "md", content: "both", dir: null }), false);
});
test("rememberDir: thu muc cua file vua luu; khong tach duoc hoac huy (null) thi giu thu muc cu", () => {
  assert.equal(rememberDir("/old", "/Users/a/Exports/Bao cao.md"), "/Users/a/Exports");
  assert.equal(rememberDir(null, "/Users/a/Bao cao (2).docx"), "/Users/a");
  assert.equal(rememberDir("/old", "C:\\Users\\a\\x.md"), "C:\\Users\\a");
  assert.equal(rememberDir(null, "/x.md"), "/");
  assert.equal(rememberDir("/old", "x.md"), "/old");
  assert.equal(rememberDir("/old", null), "/old");
  assert.equal(rememberDir(null, undefined), null);
});
