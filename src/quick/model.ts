// Logic thuan cua Quick Translate (kiem duoc bang node --test). Gioi han tinh theo KY TU, khong theo byte.
export const MAX_CHARS = 5000;

/** Dem ky tu Unicode (emoji, chu co dau gop) thay vi don vi UTF-16. */
export function charCount(s: string): number {
  return Array.from(s).length;
}

export type InputState = "empty" | "ok" | "toolong";

export function inputState(s: string): InputState {
  if (s.trim() === "") return "empty";
  return charCount(s) > MAX_CHARS ? "toolong" : "ok";
}

/** Ma loi tu Rust -> khoa i18n. Ma la ma cua `TranslateError::code()`. */
export function errorKey(code: string): string {
  const known = ["empty", "toolong", "unauthorized", "quota", "timeout", "network", "provider", "cancelled", "notverified"];
  const c = known.find((k) => code === k || code.startsWith(`${k}`)) ?? "provider";
  return `quick.err.${c}`;
}

export const SOURCE_LANGS = ["auto", "en", "pt-PT", "es", "fr", "de", "ja", "ko", "ar", "hi", "id", "th"] as const;
export const TARGET_LANGS = SOURCE_LANGS.filter((l) => l !== "auto");

/** Khoa nhan ngon ngu: `pt-PT` -> `lang.pt`. */
export function langLabelKey(code: string): string {
  return `lang.${code.split("-")[0]}`;
}

/** Danh sach app khong theo doi: moi dong mot bundle id, bo dong rong va trung. */
export function parseExcluded(text: string): string[] {
  return [...new Set(text.split(/\r?\n/).map((l) => l.trim()).filter(Boolean))];
}
