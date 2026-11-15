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
