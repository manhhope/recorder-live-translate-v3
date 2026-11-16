import { test } from "node:test";
import assert from "node:assert/strict";
import { buildLatestJson, compareVersions, entryFor, installGate, isNewer, ManifestError, parseLatestJson, parseSemver, readAutoUpdate, AUTO_UPDATE_KEY } from "./update.ts";

const sig = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZw==";
const good = { version: "3.0.3", notes: "Fixes", pub_date: "2026-11-16T10:00:00Z", platforms: {
  "darwin-aarch64": { signature: sig, url: "https://github.com/o/r/releases/download/v3.0.3/app_aarch64.app.tar.gz" },
  "windows-x86_64": { signature: sig, url: "https://github.com/o/r/releases/download/v3.0.3/app_x64-setup.exe" },
} };

test("so sanh phien ban: nho, bang, lon", () => {
  assert.equal(compareVersions("3.0.2", "3.0.3"), -1);
  assert.equal(compareVersions("3.0.2", "3.0.2"), 0);
  assert.equal(compareVersions("3.1.0", "3.0.9"), 1);
  assert.equal(compareVersions("3.0.9", "3.0.10"), -1); // so, khong phai chuoi
  assert.equal(compareVersions("v3.0.2", "3.0.2"), 0);
});

test("so sanh phien ban: tien phat hanh va dinh dang sai", () => {
  assert.equal(compareVersions("3.0.3-beta.1", "3.0.3"), -1);
  assert.equal(compareVersions("3.0.3-beta.2", "3.0.3-beta.10"), -1);
  assert.equal(compareVersions("3.0.3-alpha", "3.0.3-beta"), -1);
  assert.equal(compareVersions("3.0.3+build5", "3.0.3"), 0);
  assert.equal(compareVersions("3.0", "3.0.3"), null);
  assert.equal(compareVersions("abc", "3.0.3"), null);
  assert.equal(parseSemver("1.2.x"), null);
});

test("isNewer: chi ban moi hon moi dung (AC-33.10: thap hon, bang, sai dinh dang deu khong)", () => {
  assert.equal(isNewer("3.0.2", "3.0.3"), true);
  assert.equal(isNewer("3.0.2", "3.0.1"), false);
  assert.equal(isNewer("3.0.2", "3.0.2"), false);
  assert.equal(isNewer("3.0.2", "3.x"), false);
});

test("parse latest.json hop le", () => {
  const m = parseLatestJson(JSON.stringify(good));
  assert.equal(m.version, "3.0.3");
  assert.equal(m.notes, "Fixes");
  assert.deepEqual(Object.keys(m.platforms).sort(), ["darwin-aarch64", "windows-x86_64"]);
  assert.equal(parseLatestJson(JSON.stringify({ ...good, version: "v3.0.3", pub_date: "khong phai ngay" })).pub_date, null);
});

test("parse latest.json: tu choi json hong, version sai, thieu chu ky, http, thieu platforms", () => {
  const bad = (o: unknown, code: RegExp | string) => assert.throws(() => parseLatestJson(typeof o === "string" ? o : JSON.stringify(o)), (e: unknown) => e instanceof ManifestError && new RegExp(code).test(e.message));
  bad("{khong phai json", "json");
  bad("123", "json");
  bad({ ...good, version: "3.x" }, "version");
  bad({ ...good, platforms: {} }, "platforms");
  bad({ ...good, platforms: [] }, "platforms");
  bad({ ...good, platforms: { "darwin-aarch64": { url: good.platforms["darwin-aarch64"].url, signature: " " } } }, "signature:darwin-aarch64");
  bad({ ...good, platforms: { "darwin-aarch64": { url: "http://x/y.tar.gz", signature: sig } } }, "https:darwin-aarch64");
  bad({ ...good, platforms: { "darwin-aarch64": { url: "not a url", signature: sig } } }, "url:darwin-aarch64");
});

test("buildLatestJson sinh ra chuoi parse lai duoc va giu nguyen noi dung", () => {
  const text = buildLatestJson("3.0.3", "Fixes", "2026-11-16T10:00:00Z", good.platforms);
  assert.deepEqual(parseLatestJson(text), { version: "3.0.3", notes: "Fixes", pub_date: "2026-11-16T10:00:00Z", platforms: good.platforms });
  assert.throws(() => buildLatestJson("3.0.3", "", "2026-11-16T10:00:00Z", {}), ManifestError);
});

test("entryFor: dung goi theo nen tang, chi khi manifest moi hon", () => {
  const m = parseLatestJson(JSON.stringify(good));
  assert.equal(entryFor(m, "darwin-aarch64", "3.0.2")?.url.endsWith("app_aarch64.app.tar.gz"), true);
  assert.equal(entryFor(m, "darwin-x86_64", "3.0.2"), null); // khong co goi cho nen tang nay
  assert.equal(entryFor(m, "darwin-aarch64", "3.0.3"), null);
  assert.equal(entryFor(m, "darwin-aarch64", "3.1.0"), null);
});

test("installGate: dang ghi thi hoan", () => {
  assert.equal(installGate(true), "recording");
  assert.equal(installGate(false), "ok");
});

test("readAutoUpdate: mac dinh bat theo data-dictionary, doc duoc 0 va 1", () => {
  const mk = (v: string | null) => ({ getItem: (k: string) => (k === AUTO_UPDATE_KEY ? v : null) });
  assert.equal(readAutoUpdate(mk(null)), true);
  assert.equal(readAutoUpdate(mk("1")), true);
  assert.equal(readAutoUpdate(mk("0")), false);
  assert.equal(readAutoUpdate(undefined), true);
});
