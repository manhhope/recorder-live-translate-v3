import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { compareVersions, parseVersion, mergeNotes, decide, manualNotes, runWhenIdle, loadSeen, saveSeen, SEEN_KEY, type ChangelogEntry } from "./whats-new.ts";

const e = (version: string, n: string[], f: string[] = []): ChangelogEntry => ({
  version, date: "2026-11-15",
  groups: { new: { en: n, "pt-PT": n.map(x => x + " pt") }, ...(f.length ? { fixed: { en: f, "pt-PT": f.map(x => x + " pt") } } : {}) },
});
const LOG = [e("0.1.0", ["a"]), e("0.2.0", ["b"], ["fx2"]), e("0.3.0", ["c"]), e("1.0.0", ["d"], ["fx10"])];

test("parseVersion: x.y.z, tien to v, hau to, thieu phan, sai dinh dang", () => {
  assert.deepEqual(parseVersion("1.2.3"), [1, 2, 3]);
  assert.deepEqual(parseVersion("v1.2"), [1, 2, 0]);
  assert.deepEqual(parseVersion("1.2.3-beta.1+5"), [1, 2, 3]);
  for (const bad of ["", "abc", "1.2.3.4", "1..2", null, 5]) assert.equal(parseVersion(bad), null);
});

test("compareVersions so theo so, khong theo chuoi (0.10.0 > 0.9.0)", () => {
  assert.ok(compareVersions("0.10.0", "0.9.0")! > 0);
  assert.ok(compareVersions("0.1.0", "0.1.1")! < 0);
  assert.equal(compareVersions("1.0", "1.0.0"), 0);
  assert.equal(compareVersions("x", "1.0.0"), null);
});

test("mergeNotes gop cac muc moi hon ban da xem, toi ban hien tai, ban moi truoc, theo nhom", () => {
  const n = mergeNotes(LOG, "0.1.0", "0.3.0", "en");
  assert.deepEqual(n, [{ group: "new", items: ["c", "b"] }, { group: "fixed", items: ["fx2"] }]);
});

test("mergeNotes: cung mot cau o nhieu ban chi hien mot lan", () => {
  assert.deepEqual(mergeNotes([e("0.1.0", ["x", "y"]), e("0.2.0", ["y", "z"])], null, "0.2.0", "en"), [{ group: "new", items: ["y", "z", "x"] }]);
});

test("mergeNotes bo ban chua ra (> hien tai) va ban da xem; chon ngon ngu", () => {
  assert.deepEqual(mergeNotes(LOG, "0.2.0", "0.3.0", "pt-PT"), [{ group: "new", items: ["c pt"] }]);
  assert.deepEqual(mergeNotes(LOG, "0.3.0", "0.3.0", "en"), []);
});

test("decide: lan mo dau tien (chua co ban da xem) hien ghi chu cua CHI ban hien tai, chua ghi nho (Got it moi ghi)", () => {
  assert.deepEqual(decide(LOG, null, "0.3.0", "en"), { show: true, remember: null, notes: [{ group: "new", items: ["c"] }] });
  assert.equal(decide(LOG, "rac", "0.3.0", "en").show, true); // gia tri da xem hong = nhu chua co
});

test("decide: lan mo dau nhung changelog khong co muc cho ban hien tai thi khong hien, chi ghi nho", () => {
  assert.deepEqual(decide(LOG, null, "0.2.5", "en"), { show: true, remember: null, notes: [{ group: "new", items: ["b"] }, { group: "fixed", items: ["fx2"] }] });
  assert.deepEqual(decide(LOG, null, "0.0.1", "en"), { show: false, remember: "0.0.1", notes: [] });
});

test("runWhenIdle: cho hop thoai khac dong roi moi chay; qua so lan thi van chay", () => {
  let busy = true, ran = 0; const q: (() => void)[] = [];
  runWhenIdle(() => ran++, () => busy, f => q.push(f));
  assert.equal(ran, 0); q.shift()!(); assert.equal(ran, 0);
  busy = false; q.shift()!(); assert.equal(ran, 1); assert.equal(q.length, 0);
  let n = 0; const q2: (() => void)[] = [];
  runWhenIdle(() => n++, () => true, f => q2.push(f), 1, 3);
  while (q2.length) q2.shift()!();
  assert.equal(n, 1);
});

test("decide: doi phien ban thi hien, nhay nhieu ban thi gop", () => {
  const d = decide(LOG, "0.1.0", "1.0.0", "en");
  assert.equal(d.show, true);
  assert.deepEqual(d.notes.map(g => g.items), [["d", "c", "b"], ["fx10", "fx2"]]);
});

test("decide: cung ban, ha cap hoac ban hien tai khong hop le thi khong hien va khong ghi", () => {
  assert.deepEqual(decide(LOG, "0.3.0", "0.3.0", "en"), { show: false, remember: null, notes: [] });
  assert.equal(decide(LOG, "0.3.0", "0.2.0", "en").show, false);
  assert.equal(decide(LOG, "0.3.0", "oops", "en").remember, null);
});

test("decide: ban moi nhung changelog khong co muc nao thi khong hien, chi ghi nho", () => {
  assert.deepEqual(decide(LOG, "1.0.0", "1.0.1", "en"), { show: false, remember: "1.0.1", notes: [] });
});

test("manualNotes: ban hien tai; khong co thi ban gan nhat truoc do; khong co gi thi rong", () => {
  assert.deepEqual(manualNotes(LOG, "0.2.0", "en"), [{ group: "new", items: ["b"] }, { group: "fixed", items: ["fx2"] }]);
  assert.deepEqual(manualNotes(LOG, "0.2.5", "en")[0].items, ["b"]);
  assert.deepEqual(manualNotes(LOG, "0.0.1", "en"), []);
});

test("loadSeen/saveSeen: luu ben, bo gia tri hong, khong nem loi khi storage hong", () => {
  const m = new Map<string, string>();
  const s = { getItem: (k: string) => m.get(k) ?? null, setItem: (k: string, v: string) => void m.set(k, v) };
  assert.equal(loadSeen(s), null);
  saveSeen(s, "0.1.0");
  assert.equal(m.get(SEEN_KEY), "0.1.0");
  assert.equal(loadSeen(s), "0.1.0");
  m.set(SEEN_KEY, "hong");
  assert.equal(loadSeen(s), null);
  const bad = { getItem: () => { throw new Error("x"); }, setItem: () => { throw new Error("x"); } };
  assert.equal(loadSeen(bad), null);
  assert.doesNotThrow(() => saveSeen(bad, "1.0.0"));
});

const rd = (p: string) => JSON.parse(readFileSync(new URL(p, import.meta.url), "utf8"));
const tauri = rd("../../src-tauri/tauri.conf.json").version as string;

test("phien ban: tauri.conf.json (nguon duy nhat) khop package.json", () => {
  assert.equal(rd("../../package.json").version, tauri, "package.json lech tauri.conf.json");
});

test("changelog that: ban moi nhat khop phien ban ung dung, du en va pt-PT, khong tieng Viet", () => {
  const log = rd("../changelog.json").entries as ChangelogEntry[];
  assert.ok(log.length >= 1);
  const top = [...log].sort((a, b) => compareVersions(b.version, a.version)!)[0];
  assert.equal(top.version, tauri, "changelog thieu muc cho phien ban hien tai");
  const VI = /[\u0102\u0103\u01A0\u01A1\u01AF\u01B0\u0110\u0111\u1EA0-\u1EF9]/;
  for (const x of log) {
    assert.ok(parseVersion(x.version), x.version);
    assert.match(x.date, /^\d{4}-\d{2}-\d{2}$/);
    assert.ok(Object.keys(x.groups).length > 0);
    for (const [g, l] of Object.entries(x.groups)) {
      assert.ok(["new", "improved", "fixed"].includes(g), g);
      assert.ok(l!.en.length > 0 && l!.en.length === l!["pt-PT"].length, `${x.version}/${g}: en va pt-PT khong cung so cau`);
      for (const t of [...l!.en, ...l!["pt-PT"]]) { assert.ok(t.trim().length > 3); assert.ok(!VI.test(t), t); }
    }
  }
});
