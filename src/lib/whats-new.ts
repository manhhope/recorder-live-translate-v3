// What's New: logica thuan (khong phu thuoc giao dien) de test don vi. Phien ban dang x.y.z (so), bo hau to -pre/+build.
export type Lang = "en" | "pt-PT";
export type GroupId = "new" | "improved" | "fixed";
export const GROUP_ORDER: GroupId[] = ["new", "improved", "fixed"];
export type ChangelogEntry = {
  version: string;
  date: string;
  groups: Partial<Record<GroupId, Record<Lang, string[]>>>;
};
export type Notes = { group: GroupId; items: string[] }[];

/** "1.2.3-beta+5" -> [1,2,3]; sai dinh dang -> null. Thieu phan thi dem 0 ("1.2" = 1.2.0). */
export function parseVersion(v: unknown): number[] | null {
  if (typeof v !== "string") return null;
  const core = v.trim().replace(/^v/i, "").split(/[-+]/)[0];
  if (!/^\d+(\.\d+){0,2}$/.test(core)) return null;
  const p = core.split(".").map(Number);
  while (p.length < 3) p.push(0);
  return p;
}

/** <0 neu a<b, 0 neu bang, >0 neu a>b; null neu mot ben khong hop le. */
export function compareVersions(a: string, b: string): number | null {
  const x = parseVersion(a), y = parseVersion(b);
  if (!x || !y) return null;
  for (let i = 0; i < 3; i++) if (x[i] !== y[i]) return x[i] - y[i];
  return 0;
}

/** Gop cac muc co seen < version <= current (seen = null: tu dau) theo nhom, ban moi nhat truoc. */
export function mergeNotes(entries: ChangelogEntry[], seen: string | null, current: string, lang: Lang): Notes {
  const picked = entries
    .filter(e => {
      const hi = compareVersions(e.version, current);
      const lo = seen === null ? 1 : compareVersions(e.version, seen);
      return hi !== null && lo !== null && hi <= 0 && lo > 0;
    })
    .sort((a, b) => compareVersions(b.version, a.version) ?? 0);
  const out: Notes = [];
  for (const g of GROUP_ORDER) {
    const items = [...new Set(picked.flatMap(e => e.groups[g]?.[lang] ?? []))]; // cung cau o nhieu ban chi hien mot lan
    if (items.length) out.push({ group: g, items });
  }
  return out;
}

export type Decision = { show: boolean; remember: string | null; notes: Notes };

/** Quyet dinh luc mo app. remember = phien ban can ghi ngay (khong hien thi); show = hien modal. */
export function decide(entries: ChangelogEntry[], seen: string | null, current: string, lang: Lang): Decision {
  const none: Decision = { show: false, remember: null, notes: [] };
  if (!parseVersion(current)) return none;
  if (seen === null || !parseVersion(seen)) { // lan mo dau tien (chua co ban da xem): hien ghi chu cua ban hien tai; Got it moi ghi nho
    const notes = manualNotes(entries, current, lang);
    return notes.length ? { show: true, remember: null, notes } : { ...none, remember: current };
  }
  const c = compareVersions(current, seen);
  if (c === null || c <= 0) return none; // bang (da xem) hoac ha cap: giu nguyen, khong hien
  const notes = mergeNotes(entries, seen, current, lang);
  return notes.length ? { show: true, remember: null, notes } : { ...none, remember: current };
}

/** Mo thu cong: ghi chu cua ban hien tai; neu khong co thi ban moi nhat <= hien tai. */
export function manualNotes(entries: ChangelogEntry[], current: string, lang: Lang): Notes {
  const exact = entries.filter(e => compareVersions(e.version, current) === 0);
  if (exact.length) return mergeNotes(exact, null, current, lang);
  const older = entries.filter(e => (compareVersions(e.version, current) ?? 1) < 0).sort((a, b) => compareVersions(b.version, a.version) ?? 0);
  return older.length ? mergeNotes([older[0]], null, older[0].version, lang) : [];
}

export const SEEN_KEY = "rlt.ui.whatsNewSeen";
type Store = Pick<Storage, "getItem" | "setItem">;
export function loadSeen(s: Store): string | null {
  try { const v = s.getItem(SEEN_KEY); return v && parseVersion(v) ? v : null; } catch { return null; }
}
export function saveSeen(s: Store, v: string): void {
  try { s.setItem(SEEN_KEY, v); } catch { /* bo qua: lan sau hien lai */ }
}

/** Xep hang sau cac hop thoai khac: goi `run` khi `busy()` het (kiem tra moi `everyMs`, toi da `maxTries` lan, het luot thi van chay). */
export function runWhenIdle(run: () => void, busy: () => boolean, schedule: (f: () => void, ms: number) => unknown, everyMs = 400, maxTries = 25): void {
  let n = 0;
  const tick = () => { if (busy() && ++n < maxTries) schedule(tick, everyMs); else run(); };
  tick();
}

const VI_CHARS = /[\u0102\u0103\u01A0\u01A1\u01AF\u01B0\u0110\u0111\u1EA0-\u1EF9]/;
/** Kiem changelog: tra ve danh sach loi (rong = hop le). Muc moi nhat phai khop phien ban ung dung; en va pt-PT cung so cau; khong tieng Viet. */
export function validateChangelog(log: ChangelogEntry[], appVersion: string): string[] {
  const errs: string[] = [];
  if (!log.length) return ["changelog rong"];
  const valid = log.filter(x => parseVersion(x.version));
  const top = [...valid].sort((a, b) => compareVersions(b.version, a.version)!)[0];
  if (!top || top.version !== appVersion) errs.push(`changelog thieu muc cho phien ban hien tai ${appVersion}`);
  for (const x of log) {
    if (!parseVersion(x.version)) { errs.push(`phien ban sai dinh dang: ${x.version}`); continue; }
    if (!/^\d{4}-\d{2}-\d{2}$/.test(x.date)) errs.push(`${x.version}: ngay sai dinh dang`);
    const gs = Object.entries(x.groups);
    if (!gs.length) errs.push(`${x.version}: khong co nhom nao`);
    for (const [g, l] of gs) {
      if (!GROUP_ORDER.includes(g as GroupId)) errs.push(`${x.version}: nhom la ${g}`);
      if (!l || !l.en?.length || l.en.length !== l["pt-PT"]?.length) { errs.push(`${x.version}/${g}: en va pt-PT khong cung so cau`); continue; }
      for (const t of [...l.en, ...l["pt-PT"]]) {
        if (t.trim().length <= 3) errs.push(`${x.version}/${g}: cau qua ngan`);
        if (VI_CHARS.test(t)) errs.push(`${x.version}/${g}: co tieng Viet: ${t}`);
      }
    }
  }
  return errs;
}
