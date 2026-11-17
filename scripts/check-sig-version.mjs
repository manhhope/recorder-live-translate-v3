// Kiem moi .sig (minisign cua Tauri) co trusted comment mang `version:<VERSION>`.
// Updater cau hinh requireSignedVersion=true tu choi goi neu thieu (loi "failed its signature check"), nen phai chan truoc khi phat hanh.
// Dung: node scripts/check-sig-version.mjs <VERSION> <file.sig> [<file.sig> ...]   (thoat 1 neu co tep sai)
import fs from "node:fs";
import { pathToFileURL } from "node:url";

/** Tra ve { ok, reason, trusted } cho noi dung mot tep .sig (base64 cua van ban minisign). */
export function checkSigVersion(sigText, version) {
  let text;
  try { text = Buffer.from(sigText.trim(), "base64").toString("utf8"); } catch { return { ok: false, reason: "khong giai base64 duoc" }; }
  const line = text.split(/\r?\n/).find((l) => l.startsWith("trusted comment:"));
  if (!line) return { ok: false, reason: "khong co dong trusted comment" };
  const trusted = line.slice("trusted comment:".length).trim();
  // Cac truong cach nhau bang tab; so khop nguyen truong de `version:3.0.40` khong lot qua cho 3.0.4.
  const fields = trusted.split("\t").map((f) => f.trim());
  if (!fields.includes(`version:${version}`)) return { ok: false, reason: `trusted comment thieu version:${version}`, trusted };
  return { ok: true, reason: "", trusted };
}

function main(argv) {
  const [version, ...files] = argv;
  if (!version || !files.length) { console.error("Dung: check-sig-version.mjs <VERSION> <file.sig>..."); return 2; }
  let bad = 0;
  for (const f of files) {
    const r = checkSigVersion(fs.readFileSync(f, "utf8"), version.replace(/^v/, ""));
    console.log(`${r.ok ? "PASS" : "FAIL"} ${f}${r.ok ? "" : `: ${r.reason}`}${r.trusted ? `\n     trusted comment: ${r.trusted}` : ""}`);
    if (!r.ok) bad++;
  }
  return bad ? 1 : 0;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) process.exit(main(process.argv.slice(2)));
