#!/usr/bin/env node
// Nang version dong loat cho ban phat hanh: npm run release -- X.Y.Z [--dry-run] [--force] [--root <dir>]
// Sua: package.json, package-lock.json, src-tauri/tauri.conf.json, Cargo.toml (workspace.package.version),
// Cargo.lock (cac crate noi bo = package khong co dong `source`). Chi sua dung chuoi version, giu nguyen dinh dang.
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { join, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SEMVER = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*))*))?$/;

export function isSemver(v) {
  return typeof v === "string" && SEMVER.test(v);
}

// So sanh theo semver (khong xet build metadata). Tra ve <0, 0, >0.
export function compareSemver(a, b) {
  const pa = SEMVER.exec(a), pb = SEMVER.exec(b);
  for (let i = 1; i <= 3; i++) {
    const d = Number(pa[i]) - Number(pb[i]);
    if (d !== 0) return d;
  }
  if (!pa[4] && !pb[4]) return 0;
  if (!pa[4]) return 1;
  if (!pb[4]) return -1;
  const ia = pa[4].split("."), ib = pb[4].split(".");
  for (let i = 0; i < Math.max(ia.length, ib.length); i++) {
    if (ia[i] === undefined) return -1;
    if (ib[i] === undefined) return 1;
    const na = /^\d+$/.test(ia[i]), nb = /^\d+$/.test(ib[i]);
    if (na && nb) { const d = Number(ia[i]) - Number(ib[i]); if (d) return d; }
    else if (na) return -1;
    else if (nb) return 1;
    else if (ia[i] !== ib[i]) return ia[i] < ib[i] ? -1 : 1;
  }
  return 0;
}

const read = (root, f) => readFileSync(join(root, f), "utf8");

// Moi ham tra ve { file, before, after } voi noi dung moi; ham nem loi neu khong tim thay dung 1 cho de sua.
function replaceOnce(text, re, repl, file) {
  const m = text.match(re);
  if (!m) throw new Error(`${file}: khong tim thay dong version can sua`);
  return text.replace(re, repl);
}

export function currentVersion(root) {
  return JSON.parse(read(root, "src-tauri/tauri.conf.json")).version;
}

export function planChanges(root, next) {
  const cur = currentVersion(root);
  const out = [];

  // package.json
  let t = read(root, "package.json");
  out.push({ file: "package.json", before: t, after: replaceOnce(t, /^(\s*"version":\s*)"[^"]+"/m, `$1"${next}"`, "package.json") });

  // package-lock.json: version goc + packages[""].version (hai cho dau tien khop ten goi)
  t = read(root, "package-lock.json");
  const lock = JSON.parse(t);
  if (lock.version !== cur || lock.packages?.[""]?.version !== cur) {
    throw new Error(`package-lock.json: version (${lock.version}, ${lock.packages?.[""]?.version}) khong khop ${cur} trong tauri.conf.json`);
  }
  let n = 0;
  const lockAfter = t.replace(/^(\s*"version":\s*)"[^"]+"/gm, (m, p) => (n++ < 2 ? `${p}"${next}"` : m));
  out.push({ file: "package-lock.json", before: t, after: lockAfter });

  // tauri.conf.json
  t = read(root, "src-tauri/tauri.conf.json");
  out.push({ file: "src-tauri/tauri.conf.json", before: t, after: replaceOnce(t, /^(\s*"version":\s*)"[^"]+"/m, `$1"${next}"`, "tauri.conf.json") });

  // Cargo.toml: [workspace.package] version
  t = read(root, "Cargo.toml");
  out.push({
    file: "Cargo.toml", before: t,
    after: replaceOnce(t, /(\[workspace\.package\][^[]*?^version\s*=\s*)"[^"]+"/ms, `$1"${next}"`, "Cargo.toml"),
  });

  // Cargo.lock: chi cac package noi bo (khong co `source =`)
  if (existsSync(join(root, "Cargo.lock"))) {
    t = read(root, "Cargo.lock");
    let changed = 0;
    const after = t.replace(/\[\[package\]\]\nname = "[^"]+"\nversion = "([^"]+)"\n(?!source =)/g, (m, v) => {
      if (v !== cur) throw new Error(`Cargo.lock: crate noi bo co version ${v}, khac ${cur}`);
      changed++;
      return m.replace(`version = "${v}"`, `version = "${next}"`);
    });
    if (!changed) throw new Error("Cargo.lock: khong tim thay crate noi bo nao");
    out.push({ file: "Cargo.lock", before: t, after });
  }
  return { cur, changes: out.filter((c) => c.before !== c.after) };
}

// Diff dong thay doi gon (khong can git): in cac dong khac nhau kem so dong.
export function diffLines(file, before, after) {
  const a = before.split("\n"), b = after.split("\n"), lines = [`--- ${file}`];
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) lines.push(`@@ dong ${i + 1} @@`, `- ${a[i].trim()}`, `+ ${b[i].trim()}`);
  }
  return lines.join("\n");
}

export function run(argv, log = console.log) {
  const args = argv.filter((a) => !a.startsWith("--"));
  const flag = (f) => argv.includes(f);
  const ri = argv.indexOf("--root");
  const root = ri >= 0 ? resolve(argv[ri + 1]) : resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const rootArg = ri >= 0 ? argv[ri + 1] : null;
  const positional = args.filter((a) => a !== rootArg);
  const next = positional[0];
  if (!next) throw new Error("Cach dung: npm run release -- X.Y.Z [--dry-run] [--force]");
  if (!isSemver(next)) throw new Error(`"${next}" khong phai semver hop le (vd 3.1.0 hoac 3.1.0-beta.1, khong co tien to v)`);
  const { cur, changes } = planChanges(root, next);
  if (compareSemver(next, cur) <= 0 && !flag("--force")) {
    throw new Error(`Version moi ${next} phai lon hon version hien tai ${cur} (dung --force neu that su can)`);
  }
  log(`Nang version ${cur} -> ${next}${flag("--dry-run") ? " (DRY RUN, khong ghi file)" : ""}`);
  for (const c of changes) log(diffLines(c.file, c.before, c.after));
  if (!flag("--dry-run")) {
    for (const c of changes) writeFileSync(join(root, c.file), c.after);
    log(`Da sua ${changes.length} file. Buoc tiep: git switch -c release-${next}, commit "release ${next}", mo PR vao main.`);
  }
  return { cur, next, files: changes.map((c) => c.file) };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { run(process.argv.slice(2)); } catch (e) { console.error(`LOI: ${e.message}`); process.exit(1); }
}
