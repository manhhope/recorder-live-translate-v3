// Sinh latest.json cho Tauri updater (US-33) tu cac goi da ky. Dung lai buildLatestJson (src/lib/update.ts, co test) de khuon dang
// va kiem hop le dung mot cho: version semver, chu ky khong rong, url https.
// Dung: node scripts/make-latest-json.mjs --version 3.1.0 [--notes-file notes.md] [--out latest.json] \
//         --platform darwin-aarch64=<file.sig>=<https url goi> --platform windows-x86_64=<file.sig>=<https url goi>
import fs from "node:fs";
import { buildLatestJson, PLATFORM_KEYS } from "../src/lib/update.ts";

const args = process.argv.slice(2);
const get = (k) => { const i = args.indexOf(k); return i >= 0 ? args[i + 1] : undefined; };
const version = get("--version");
if (!version) { console.error("thieu --version"); process.exit(2); }
const notes = get("--notes-file") ? fs.readFileSync(get("--notes-file"), "utf8").trim() : (get("--notes") ?? "");
const out = get("--out") ?? "latest.json";
const platforms = {};
for (let i = 0; i < args.length; i++) {
  if (args[i] !== "--platform") continue;
  const [key, sigFile, ...rest] = args[i + 1].split("=");
  const url = rest.join("=");
  if (!PLATFORM_KEYS.includes(key)) { console.error(`nen tang la: ${key} (hop le: ${PLATFORM_KEYS.join(", ")})`); process.exit(2); }
  platforms[key] = { signature: fs.readFileSync(sigFile, "utf8").trim(), url };
}
if (!Object.keys(platforms).length) { console.error("can it nhat mot --platform"); process.exit(2); }
const pubDate = get("--pub-date") ?? new Date().toISOString();
fs.writeFileSync(out, buildLatestJson(version.replace(/^v/, ""), notes, pubDate, platforms));
console.log(`Da ghi ${out}: ${version}, nen tang: ${Object.keys(platforms).join(", ")}`);
