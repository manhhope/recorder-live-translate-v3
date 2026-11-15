import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { validateChangelog, compareVersions, parseVersion, mergeNotes, decide, manualNotes, runWhenIdle, loadSeen, saveSeen, SEEN_KEY, type ChangelogEntry } from "./whats-new.ts";

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

test("changelog that: hop le, ban moi nhat khop phien ban ung dung", () => {
  const log = rd("../changelog.json").entries as ChangelogEntry[];
  assert.deepEqual(validateChangelog(log, tauri), []);
  assert.equal(log[0].version, tauri);
});

test("changelog gia: validateChangelog bat dung tung loai lech", () => {
  const ok = (): ChangelogEntry[] => [e("1.0.1", ["fresh item"]), e("1.0.0", ["old item"])];
  assert.deepEqual(validateChangelog(ok(), "1.0.1"), []);
  assert.ok(validateChangelog(ok(), "1.0.2").some(x => x.includes("thieu muc")), "thieu muc cho phien ban");
  assert.ok(validateChangelog([], "1.0.1").length > 0, "rong");
  const lech = ok(); lech[0].groups.new!["pt-PT"].pop();
  assert.ok(validateChangelog(lech, "1.0.1").some(x => x.includes("cung so cau")), "lech so cau en/pt-PT");
  const vi = ok(); vi[0].groups.new!.en[0] = "Xin ch\u00e0o b\u1ea1n \u0111\u00e3";
  assert.ok(validateChangelog(vi, "1.0.1").some(x => x.includes("tieng Viet")), "tieng Viet");
  const badVer = ok(); badVer[1].version = "abc";
  assert.ok(validateChangelog(badVer, "1.0.1").some(x => x.includes("sai dinh dang")), "phien ban sai");
  const badDate = ok(); badDate[0].date = "15/11/2026";
  assert.ok(validateChangelog(badDate, "1.0.1").some(x => x.includes("ngay")), "ngay sai");
  const badGroup = ok(); (badGroup[0].groups as any).misc = badGroup[0].groups.new;
  assert.ok(validateChangelog(badGroup, "1.0.1").some(x => x.includes("nhom la")), "nhom la");
});

test("changelog that: mergeNotes tu ban da xem 3.0.0 len 3.0.1 chi gop muc 3.0.1, du hai ngon ngu", () => {
  const log = rd("../changelog.json").entries as ChangelogEntry[];
  for (const lang of ["en", "pt-PT"] as const) {
    const n = mergeNotes(log, "3.0.0", "3.0.1", lang);
    const want = Object.entries(log.find(x => x.version === "3.0.1")!.groups).flatMap(([, l]) => l![lang]);
    assert.deepEqual(n.flatMap(g => g.items).sort(), [...want].sort());
    assert.ok(n.length >= 2);
    const d = decide(log, "3.0.0", "3.0.1", lang);
    assert.equal(d.show, true);
    assert.deepEqual(d.notes, n);
    assert.equal(decide(log, "3.0.1", "3.0.1", lang).show, false);
  }
  // lan mo dau tien: chi ghi chu cua 3.0.1
  assert.equal(decide(log, null, "3.0.1", "en").show, true);
});
