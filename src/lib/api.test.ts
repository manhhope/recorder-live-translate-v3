import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { errKeyOf, driveKeyOf, mondayOf, usd, errCode } from "./api.ts";

const en: Record<string, string> = JSON.parse(fs.readFileSync(path.join(import.meta.dirname, "..", "locales", "en.json"), "utf8"));
const known = (k: string) => k in en;

test("ma loi cua Rust deu co khoa i18n, ma la thi roi ve err.generic", () => {
  for (const c of ["key_missing", "consent_required", "already_recording", "auth", "quota", "network", "timeout", "permission", "store_error", "session_active", "import_failed"]) {
    assert.notEqual(errKeyOf(c, known), "err.generic", c);
  }
  assert.equal(errKeyOf("khong_biet", known), "err.generic");
});

test("trang thai Drive co khoa i18n", () => {
  for (const s of ["not_connected", "synced", "pending", "syncing", "waiting_network", "error"]) assert.ok(known(driveKeyOf(s)), s);
});

test("errCode lay chuoi ma tu loi Tauri", () => {
  assert.equal(errCode("key_missing"), "key_missing");
  assert.equal(errCode(new Error("x")), "x");
});

test("mondayOf lui theo tuan (UTC) va usd doi micro", () => {
  assert.equal(mondayOf(0, new Date(Date.UTC(2026, 10, 14))), "2026-11-14");
  assert.equal(mondayOf(1, new Date(Date.UTC(2026, 10, 14))), "2026-11-07");
  assert.equal(usd(240_000), 0.24);
});
