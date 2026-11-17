import test from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, cpSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { isSemver, compareSemver, run } from "./bump-version.mjs";

function fixture(ver = "3.0.4") {
  const d = mkdtempSync(join(tmpdir(), "bump-"));
  mkdirSync(join(d, "src-tauri"));
  writeFileSync(join(d, "package.json"), `{\n  "name": "x",\n  "version": "${ver}",\n  "dependencies": { "a": "1.0.0" }\n}\n`);
  writeFileSync(join(d, "package-lock.json"), JSON.stringify({ name: "x", version: ver, lockfileVersion: 3, packages: { "": { name: "x", version: ver }, "node_modules/a": { version: "1.0.0" } } }, null, 2) + "\n");
  writeFileSync(join(d, "src-tauri/tauri.conf.json"), `{\n  "productName": "X",\n  "version": "${ver}",\n  "identifier": "a.b"\n}\n`);
  writeFileSync(join(d, "Cargo.toml"), `[workspace]\nmembers = ["a"]\n\n[workspace.package]\nversion = "${ver}"\nedition = "2021"\n\n[workspace.dependencies]\nserde = { version = "1" }\n`);
  writeFileSync(join(d, "Cargo.lock"), `[[package]]\nname = "serde"\nversion = "1.0.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n\n[[package]]\nname = "rlt-a"\nversion = "${ver}"\ndependencies = ["serde"]\n\n[[package]]\nname = "rlt-b"\nversion = "${ver}"\n`);
  return d;
}
const quiet = () => {};

test("semver hop le / khong hop le", () => {
  for (const v of ["3.0.5", "0.0.1", "3.1.0-beta.1", "10.20.30"]) assert.ok(isSemver(v), v);
  for (const v of ["v3.0.5", "3.0", "3.0.05", "3.0.5.1", "", "abc", "3.0.5+build", "3.0.5-"]) assert.ok(!isSemver(v), v);
});

test("so sanh semver", () => {
  assert.ok(compareSemver("3.0.10", "3.0.9") > 0);
  assert.ok(compareSemver("3.1.0-beta.1", "3.1.0") < 0);
  assert.ok(compareSemver("3.1.0-beta.2", "3.1.0-beta.10") < 0);
  assert.equal(compareSemver("3.0.4", "3.0.4"), 0);
});

test("sua dong loat 5 file, giu nguyen phan con lai", () => {
  const d = fixture();
  const r = run(["3.0.5", "--root", d], quiet);
  assert.deepEqual(r.files.sort(), ["Cargo.lock", "Cargo.toml", "package-lock.json", "package.json", "src-tauri/tauri.conf.json"]);
  assert.equal(JSON.parse(readFileSync(join(d, "package.json"), "utf8")).version, "3.0.5");
  const lock = JSON.parse(readFileSync(join(d, "package-lock.json"), "utf8"));
  assert.equal(lock.version, "3.0.5");
  assert.equal(lock.packages[""].version, "3.0.5");
  assert.equal(lock.packages["node_modules/a"].version, "1.0.0");
  assert.equal(JSON.parse(readFileSync(join(d, "src-tauri/tauri.conf.json"), "utf8")).version, "3.0.5");
  const cargo = readFileSync(join(d, "Cargo.toml"), "utf8");
  assert.match(cargo, /\[workspace\.package\]\nversion = "3\.0\.5"/);
  assert.match(cargo, /serde = \{ version = "1" \}/);
  const cl = readFileSync(join(d, "Cargo.lock"), "utf8");
  assert.equal((cl.match(/version = "3\.0\.5"/g) ?? []).length, 2);
  assert.match(cl, /name = "serde"\nversion = "1\.0\.0"/);
});

test("--dry-run khong ghi file nhung in diff", () => {
  const d = fixture();
  const out = [];
  run(["3.0.5", "--dry-run", "--root", d], (s) => out.push(s));
  assert.equal(JSON.parse(readFileSync(join(d, "package.json"), "utf8")).version, "3.0.4");
  assert.ok(out.join("\n").includes('+ "version": "3.0.5",'));
  assert.ok(out.join("\n").includes("DRY RUN"));
});

test("tu choi semver sai, version khong tang, thieu doi so", () => {
  const d = fixture();
  assert.throws(() => run(["v3.0.5", "--root", d], quiet), /semver/);
  assert.throws(() => run(["3.0.4", "--root", d], quiet), /lon hon/);
  assert.throws(() => run(["3.0.3", "--root", d], quiet), /lon hon/);
  assert.throws(() => run(["--root", d], quiet), /Cach dung/);
  assert.doesNotThrow(() => run(["3.0.3", "--force", "--dry-run", "--root", d], quiet));
});

test("tu choi khi cac file dang lech version voi nhau", () => {
  const d = fixture();
  writeFileSync(join(d, "Cargo.lock"), readFileSync(join(d, "Cargo.lock"), "utf8").replace(/rlt-b"\nversion = "3.0.4"/, 'rlt-b"\nversion = "3.0.1"'));
  assert.throws(() => run(["3.0.5", "--root", d], quiet), /Cargo\.lock/);
  assert.equal(JSON.parse(readFileSync(join(d, "package.json"), "utf8")).version, "3.0.4", "khong ghi dở");
});

test("chay that tren repo hien tai o che do --dry-run khong loi", () => {
  const r = run(["99.0.0", "--dry-run"], quiet);
  assert.equal(r.files.length, 5);
});
