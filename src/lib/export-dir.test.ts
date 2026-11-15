import { test } from "node:test";
import assert from "node:assert/strict";
import { loadExportPrefs, saveExportPrefs, applyPickedDir, pickErrorDetail, EXPORT_PREFS_KEY } from "./export-dir.ts";

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
test("huy hop chon (null/rong) giu thu muc cu; chon moi thi doi", () => {
  assert.equal(applyPickedDir("/old", null), "/old");
  assert.equal(applyPickedDir("/old", ""), "/old");
  assert.equal(applyPickedDir(null, null), null);
  assert.equal(applyPickedDir("/old", "/new"), "/new");
});
test("pickErrorDetail: ma ngan giu nguyen, thong diep dai bi cat, rong thi unknown", () => {
  assert.equal(pickErrorDetail("export_pick_failed"), "export_pick_failed");
  assert.equal(pickErrorDetail(new Error("x\n y")), "x y");
  assert.equal(pickErrorDetail("a".repeat(500)).length, 160);
  assert.equal(pickErrorDetail(""), "unknown");
  assert.equal(pickErrorDetail(undefined), "unknown");
});
