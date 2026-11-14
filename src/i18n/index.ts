// Khoi tao i18n bang svelte-i18n. Locale mac dinh pt-PT, du phong en (AC-49.3, AC-49.5).
import { addMessages, dictionary, init, locale, _, waitLocale } from "svelte-i18n";
import { get } from "svelte/store";
import en from "../locales/en.json" with { type: "json" };
import ptPT from "../locales/pt-PT.json" with { type: "json" };
import type { Locale } from "./format.ts";

export const SUPPORTED_LOCALES = ["en", "pt-PT"] as const;
export const DEFAULT_LOCALE: Locale = "pt-PT";
export const FALLBACK_LOCALE: Locale = "en";
export const STORAGE_KEY = "rlt.ui.locale";

export const missingKeys: string[] = [];

export function isLocale(v: unknown): v is Locale {
  return typeof v === "string" && (SUPPORTED_LOCALES as readonly string[]).includes(v);
}

// Lan dau chay (chua luu) luon la pt-PT, khong theo ngon ngu he dieu hanh (AC-49.5).
export function resolveInitialLocale(stored: string | null | undefined): Locale {
  return isLocale(stored) ? stored : DEFAULT_LOCALE;
}

function readStored(): string | null {
  try { if (typeof window === "undefined") return null; return globalThis.localStorage?.getItem(STORAGE_KEY) ?? null; } catch { return null; }
}

export function setupI18n(initial?: string | null, messages: { en: Record<string, string>; "pt-PT": Record<string, string> } = { en, "pt-PT": ptPT }) {
  dictionary.set({}); // addMessages gop sau, nen xoa de lan khoi tao sau thay the han
  addMessages("en", messages.en);
  addMessages("pt-PT", messages["pt-PT"]);
  missingKeys.length = 0;
  // svelte-i18n roi ve en trong im lang; AC-49.3 can 1 dong canh bao cho moi khoa thieu o pt-PT.
  for (const id of Object.keys(messages.en)) {
    const v = messages["pt-PT"][id];
    if (typeof v !== "string" || v === "") { missingKeys.push(id); console.warn(`[i18n] missing key in pt-PT: ${id}`); }
  }
  return init({
    fallbackLocale: FALLBACK_LOCALE,
    initialLocale: resolveInitialLocale(initial ?? readStored()),
    handleMissingMessage: ({ id }) => { missingKeys.push(id); console.warn(`[i18n] missing key: ${id}`); },
  });
}

export async function setLocale(next: Locale) {
  locale.set(next);
  try { if (typeof window !== "undefined") globalThis.localStorage?.setItem(STORAGE_KEY, next); } catch { /* bo qua */ }
  await waitLocale();
}

export const currentLocale = (): Locale => (get(locale) as Locale) ?? DEFAULT_LOCALE;
export { _, locale };
export * from "./format.ts";
export { isValidKey, KEY_PATTERN } from "./keys.ts";
