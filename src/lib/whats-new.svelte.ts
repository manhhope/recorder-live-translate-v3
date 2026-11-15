// Trang thai modal What's New. Phien ban lay tu tauri.conf.json (vite define, nguon duy nhat); phien ban da xem luu cung noi
// luu cai dat giao dien (localStorage "rlt.ui.*").
import changelog from "../changelog.json";
import { currentLocale } from "../i18n/index.ts";
import { decide, loadSeen, manualNotes, saveSeen, type ChangelogEntry, type Lang, type Notes } from "./whats-new.ts";

declare const __APP_VERSION__: string;
export const APP_VERSION: string = __APP_VERSION__;
const ENTRIES = changelog.entries as ChangelogEntry[];

export const whatsNew = $state({ open: false, notes: [] as Notes, version: APP_VERSION });
const lang = (): Lang => (currentLocale() === "pt-PT" ? "pt-PT" : "en");

/** Goi mot lan khi giao dien da san sang. Cai moi chi ghi nho phien ban; doi phien ban thi hien modal mot lan. */
export function evaluateWhatsNew(): void {
  const d = decide(ENTRIES, loadSeen(localStorage), APP_VERSION, lang());
  if (d.remember) saveSeen(localStorage, d.remember);
  if (d.show) { whatsNew.notes = d.notes; whatsNew.version = APP_VERSION; whatsNew.open = true; }
}

/** Mo lai thu cong tu Cai dat > About. */
export function openWhatsNew(): void {
  whatsNew.notes = manualNotes(ENTRIES, APP_VERSION, lang());
  whatsNew.version = APP_VERSION;
  whatsNew.open = true;
}

/** Got it, X, Esc: dong va ghi phien ban hien tai la da xem. */
export function closeWhatsNew(): void {
  whatsNew.open = false;
  saveSeen(localStorage, APP_VERSION);
}

/** Moc cho co che cap nhat tu dong (US-33, hoan): sau khi cai ban moi va khoi dong lai thi goi ham nay; chua co noi goi. */
export function onUpdateInstalled(): void { evaluateWhatsNew(); }
