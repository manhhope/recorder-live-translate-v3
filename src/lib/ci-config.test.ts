// Dam bao CI luon chay kiem tra tinh capability (check:caps) va cac lenh bat buoc.
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const ci = readFileSync(new URL("../../.github/workflows/ci.yml", import.meta.url), "utf8");

for (const cmd of ["npm run check:caps", "npm run test:i18n", "npm run scan:i18n", "npm run build", "npx playwright test", "cargo test --workspace", "-D warnings"]) {
  test(`ci.yml chay: ${cmd}`, () => {
    assert.ok(ci.includes(cmd), `ci.yml thieu buoc: ${cmd}`);
  });
}

// US-57 (AC-57.12): moi request toi endpoint tuy chinh di tu Rust, nen CSP cua WebView khong duoc noi long.
const conf = JSON.parse(readFileSync(new URL("../../src-tauri/tauri.conf.json", import.meta.url), "utf8"));
for (const key of ["csp", "devCsp"]) {
  test(`CSP ${key} khong cho WebView goi mang ra ngoai (US-57)`, () => {
    const csp: string = conf.app.security[key];
    const connect = csp.split(";").map((s) => s.trim()).find((s) => s.startsWith("connect-src")) ?? "";
    assert.ok(!/https?:\/\/(?!ipc\.localhost|localhost:1420)/.test(connect), `connect-src bi noi: ${connect}`);
    assert.ok(!connect.includes("*"), `connect-src co ky tu *: ${connect}`);
  });
}

// US-33: khoa ky cap nhat chi di qua GitHub Secrets; tauri.conf.json chi giu khoa CONG KHAI.
const release = readFileSync(new URL("../../.github/workflows/release.yml", import.meta.url), "utf8");
test("release.yml: bien khoa ky chi lay tu secrets, khong giu gia tri that", () => {
  for (const m of release.matchAll(/^\s+(TAURI_SIGNING_PRIVATE_KEY(?:_PASSWORD)?):\s*(.+)$/gm)) {
    assert.match(m[2].trim(), /^\$\{\{ secrets\.TAURI_SIGNING_PRIVATE_KEY(_PASSWORD)? \}\}$/, `${m[1]} khong lay tu secrets`);
  }
  assert.ok(!/untrusted comment|BEGIN [A-Z ]*PRIVATE KEY|RWRT/.test(release), "release.yml chua noi dung giong khoa");
});
test("tauri.conf.json: updater chi co khoa cong khai, endpoint https, ky bat buoc", () => {
  const u = conf.plugins.updater;
  assert.ok(typeof u.pubkey === "string" && u.pubkey.length > 0);
  assert.ok(!/PRIVATE|secret key/i.test(u.pubkey));
  assert.ok(u.endpoints.every((e: string) => e.startsWith("https://")));
  assert.equal(u.requireSignedVersion, true);
  assert.equal(conf.bundle.createUpdaterArtifacts, true);
});
