// Cau noi giua giao dien Quick Translate va lenh Tauri (M3-01). Khong co ham nao cham khoa API.
import { invoke } from "@tauri-apps/api/core";

export interface QuickOut { text: string; detected_lang: string | null }
export interface QuickErr { code: string; key: string }
export interface HistoryItem { id: number; origin: string; source_text: string; translated_text: string; target_lang: string; created_at: string }
export interface QuickPrefs { toolbar_enabled: boolean; excluded_apps: string[]; source_lang: string | null; target_lang: string }
export interface PrefsResult { prefs: QuickPrefs; needs_accessibility: boolean }

export const translate = (text: string, sourceLang: string | null, targetLang: string) =>
  invoke<QuickOut>("quick_translate", { text, sourceLang, targetLang });
export const cancel = () => invoke<void>("quick_cancel");
export const history = (limit = 50) => invoke<HistoryItem[]>("quick_history", { limit });
export const clearHistory = () => invoke<number>("quick_clear_history");
export const hide = () => invoke<void>("quick_hide");
export const axStatus = () => invoke<boolean>("quick_ax_status");
export const getPrefs = () => invoke<QuickPrefs>("quick_prefs_get");
export const setPrefs = (prefs: QuickPrefs) => invoke<PrefsResult>("quick_prefs_set", { prefs });
export const inlineRun = () => invoke<void>("quick_inline_run");
export const ocrTranslate = (image: number[], sourceLang: string | null, targetLang: string) =>
  invoke<{ recognized: string; text: string }>("quick_ocr_translate", { image, sourceLang, targetLang });
