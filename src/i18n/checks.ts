// Ham kiem tra bo chuoi dich, dung trong test va CI (AC-49.4, AC-49.10, AC-49.11).
type Dict = Record<string, string>;

export function diffKeys(a: Dict, b: Dict) {
  const ka = new Set(Object.keys(a)), kb = new Set(Object.keys(b));
  return { missingInB: [...ka].filter(k => !kb.has(k)).sort(), extraInB: [...kb].filter(k => !ka.has(k)).sort() };
}

const FAKE = [/^\s*$/, /^(todo|tbd|fixme|xxx|n\/a|placeholder)\b/i, /lorem ipsum/i, /^\[\[.*\]\]$/, /^\{\{.*\}\}$/, /\b(tbd|todo) translate/i];

export function findFakeStrings(d: Dict): { key: string; value: string }[] {
  return Object.entries(d)
    .filter(([k, v]) => typeof v !== "string" || v === k || FAKE.some(r => r.test(v)))
    .map(([key, value]) => ({ key, value }));
}

// Ky tu rieng cua tieng Viet (khong nam trong tieng Bo Dao Nha hay Anh) va mot so tu khoa.
const VI_CHARS = /[ĂăƠơƯưĐđẠ-ỹ]/u;
const VI_WORDS = /\b(tiếng|việt|không|của|được|những|cài đặt)\b/iu;
export function hasVietnamese(s: string): boolean {
  return VI_CHARS.test(s) || VI_WORDS.test(s);
}
