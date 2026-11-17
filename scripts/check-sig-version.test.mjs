import test from "node:test";
import assert from "node:assert/strict";
import { checkSigVersion } from "./check-sig-version.mjs";

const sig = (trusted) => Buffer.from(`untrusted comment: x\nAAAA\ntrusted comment: ${trusted}\nBBBB\n`).toString("base64");

test("co version:<VERSION> thi dat", () => {
  assert.ok(checkSigVersion(sig("timestamp:1\tfile:a.exe\tversion:3.0.4"), "3.0.4").ok);
});
test("thieu version thi hong (loi Mac 3.0.4)", () => {
  const r = checkSigVersion(sig("timestamp:1\tfile:Recorder Live Translate.app.tar.gz"), "3.0.4");
  assert.ok(!r.ok);
  assert.match(r.reason, /version:3\.0\.4/);
});
test("version khac hoac chi trung tien to thi hong", () => {
  assert.ok(!checkSigVersion(sig("timestamp:1\tversion:3.0.3"), "3.0.4").ok);
  assert.ok(!checkSigVersion(sig("timestamp:1\tversion:3.0.40"), "3.0.4").ok);
});
test("khong co trusted comment hoac khong phai base64 thi hong", () => {
  assert.ok(!checkSigVersion(Buffer.from("untrusted comment: x\nAAAA\n").toString("base64"), "3.0.4").ok);
  assert.ok(!checkSigVersion("", "3.0.4").ok);
});
