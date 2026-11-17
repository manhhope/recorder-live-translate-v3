// US-33: logic thuan cua cap nhat tu dong (khong dung Tauri, kiem duoc bang node --test).
// - so sanh phien ban (semver) de giao dien tu chan ban khong moi hon (phong thu them, AC-33.10; plugin cung chan o Rust);
// - doc/kiem/sinh `latest.json` (dinh dang manifest tinh cua Tauri updater), dung chung voi scripts/make-latest-json.mjs;
// - quyet dinh co duoc cai luc nay khong (khong cai khi dang ghi, AC-33.1).

export const AUTO_UPDATE_KEY = "rlt.ui.autoUpdate";
/** Mac dinh theo data-dictionary-v3: settings.auto_update_enabled = 1 (AC-33.1, cho chu xac nhan). */
export const AUTO_UPDATE_DEFAULT = true;

const SEMVER = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/;

export interface Semver { major: number; minor: number; patch: number; pre: string[] }

export function parseSemver(v: string): Semver | null {
  const m = SEMVER.exec(v.trim());
  if (!m) return null;
  return { major: +m[1], minor: +m[2], patch: +m[3], pre: m[4] ? m[4].split(".") : [] };
}

/** -1 | 0 | 1; `null` khi mot ben khong phai semver (dinh dang sai: khong tai, khong cai). */
export function compareVersions(a: string, b: string): -1 | 0 | 1 | null {
  const x = parseSemver(a), y = parseSemver(b);
  if (!x || !y) return null;
  for (const k of ["major", "minor", "patch"] as const) if (x[k] !== y[k]) return x[k] < y[k] ? -1 : 1;
  if (!x.pre.length && !y.pre.length) return 0;
  if (!x.pre.length) return 1; // ban chinh thuc > ban tien phat hanh
  if (!y.pre.length) return -1;
  for (let i = 0; i < Math.max(x.pre.length, y.pre.length); i++) {
    const p = x.pre[i], q = y.pre[i];
    if (p === undefined) return -1;
    if (q === undefined) return 1;
    const pn = /^\d+$/.test(p), qn = /^\d+$/.test(q);
    if (pn && qn) { if (+p !== +q) return +p < +q ? -1 : 1; }
    else if (pn !== qn) return pn ? -1 : 1;
    else if (p !== q) return p < q ? -1 : 1;
  }
  return 0;
}

/** Chi `true` khi `candidate` la semver hop le va lon hon han `current` (bang hoac thap hon hoac sai dinh dang: false). */
export const isNewer = (current: string, candidate: string): boolean => compareVersions(current, candidate) === -1;

// ---- latest.json -----------------------------------------------------------------------------------------

export interface PlatformEntry { signature: string; url: string }
export interface Manifest { version: string; notes: string; pub_date: string | null; platforms: Record<string, PlatformEntry> }

/** Khoa nen tang ma plugin tra cuu: `{os}-{arch}` (darwin|windows) + (x86_64|aarch64). */
export const PLATFORM_KEYS = ["darwin-aarch64", "darwin-x86_64", "windows-x86_64"] as const;

export class ManifestError extends Error {}

export function parseLatestJson(text: string): Manifest {
  let raw: unknown;
  try { raw = JSON.parse(text); } catch { throw new ManifestError("json"); }
  if (!raw || typeof raw !== "object") throw new ManifestError("json");
  const o = raw as Record<string, unknown>;
  if (typeof o.version !== "string" || !parseSemver(o.version)) throw new ManifestError("version");
  if (!o.platforms || typeof o.platforms !== "object" || Array.isArray(o.platforms)) throw new ManifestError("platforms");
  const platforms: Record<string, PlatformEntry> = {};
  for (const [k, v] of Object.entries(o.platforms as Record<string, unknown>)) {
    const e = v as Partial<PlatformEntry> | null;
    if (!e || typeof e.url !== "string" || typeof e.signature !== "string" || !e.signature.trim()) throw new ManifestError("signature:" + k);
    let u: URL;
    try { u = new URL(e.url); } catch { throw new ManifestError("url:" + k); }
    if (u.protocol !== "https:") throw new ManifestError("https:" + k);
    platforms[k] = { url: e.url, signature: e.signature };
  }
  if (!Object.keys(platforms).length) throw new ManifestError("platforms");
  const pub = typeof o.pub_date === "string" && !Number.isNaN(Date.parse(o.pub_date)) ? o.pub_date : null;
  return { version: o.version.replace(/^v/, ""), notes: typeof o.notes === "string" ? o.notes : "", pub_date: pub, platforms };
}

export function buildLatestJson(version: string, notes: string, pubDate: string, platforms: Record<string, PlatformEntry>): string {
  const m = parseLatestJson(JSON.stringify({ version, notes, pub_date: pubDate, platforms }));
  return JSON.stringify({ version: m.version, notes: m.notes, pub_date: m.pub_date, platforms: m.platforms }, null, 2) + "\n";
}

/** Manifest dung cho may `key` (vd `darwin-aarch64`) va moi hon `current`; nguoc lai `null`. */
export function entryFor(m: Manifest, key: string, current: string): PlatformEntry | null {
  return isNewer(current, m.version) ? m.platforms[key] ?? null : null;
}

// ---- Khi nao duoc cai --------------------------------------------------------------------------------------

export type InstallGate = "ok" | "recording";
/** Dang ghi (ke ca tam dung) thi KHONG cai: hoan toi khi phien Dung (AC-33.1). */
export const installGate = (recording: boolean): InstallGate => (recording ? "recording" : "ok");

export const readAutoUpdate = (store: Pick<Storage, "getItem"> | undefined): boolean => {
  try {
    const v = store?.getItem(AUTO_UPDATE_KEY);
    return v == null ? AUTO_UPDATE_DEFAULT : v === "1";
  } catch { return AUTO_UPDATE_DEFAULT; }
};
