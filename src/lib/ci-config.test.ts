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
