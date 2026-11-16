// Kiem tra tinh (CI): moi lenh Tauri tu dinh nghia trong manifest ma src/lib/api.ts goi phai co `allow-<lenh>`
// o it nhat mot file trong src-tauri/capabilities/ (lenh cua so phu de nam o overlay.json, con lai o main.json). Thieu quyen thi Tauri ACL tu choi luc chay, giao dien chi thay loi chung.
import fs from "node:fs";
const build = fs.readFileSync("src-tauri/build.rs", "utf8");
const manifest = new Set([...build.slice(build.indexOf("APP_COMMANDS"), build.indexOf("];")).matchAll(/"([a-z0-9_]+)"/g)].map(m => m[1]));
const sessionSrc = fs.readFileSync("src-tauri/src/session/mod.rs", "utf8");
const api = fs.readFileSync("src/lib/api.ts", "utf8");
const called = new Set([...api.matchAll(/invoke(?:<[^>]*>)?\(\s*"([a-z0-9_]+)"/g)].map(m => m[1]));
const main = fs.readdirSync("src-tauri/capabilities").filter(f => f.endsWith(".json"))
  .flatMap(f => JSON.parse(fs.readFileSync("src-tauri/capabilities/" + f, "utf8")).permissions);
const missing = [...called].filter(c => manifest.has(c) && !main.includes("allow-" + c.replace(/_/g, "-")));
const unregistered = [...called].filter(c => !manifest.has(c) && !sessionSrc.includes(`fn ${c}(`) && !/^plugin:/.test(c));
if (missing.length) { console.error("capabilities thieu quyen cho lenh:", missing.join(", ")); process.exit(1); }
console.log(`OK: ${called.size} lenh goi tu api.ts, ${[...called].filter(c => manifest.has(c)).length} nam trong manifest va deu duoc cap quyen`);
if (unregistered.length) console.log("Luu y (khong nam trong manifest, ACL khong kiem):", unregistered.join(", "));

// US-33: quyen cap nhat chi duoc o cua so `main` (khong cua so phu, khong quyen `updater:*` cua plugin, khong `process:*`).
const bad = [];
for (const f of fs.readdirSync("src-tauri/capabilities").filter(x => x.endsWith(".json"))) {
  const cap = JSON.parse(fs.readFileSync("src-tauri/capabilities/" + f, "utf8"));
  const perms = cap.permissions.map(p => (typeof p === "string" ? p : p.identifier));
  const upd = perms.filter(p => /^allow-update-/.test(p) || /^updater:/.test(p) || /^process:/.test(p));
  const isMain = JSON.stringify(cap.windows) === JSON.stringify(["main"]);
  if (upd.length && !(f === "main.json" && isMain && upd.every(p => /^allow-update-/.test(p)))) bad.push(`${f}: ${upd.join(", ")}`);
}
if (bad.length) { console.error("quyen cap nhat nam ngoai cua so main hoac dung quyen plugin:", bad.join(" | ")); process.exit(1); }
console.log("OK: quyen cap nhat chi o capabilities/main.json (cua so main), khong cap updater:* / process:*");
