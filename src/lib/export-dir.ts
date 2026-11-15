// Logic thu muc xuat (tach khoi giao dien de test don vi): luu/nho lua chon va dung chi tiet loi.
import type { ExportFormat, ExportContent } from "./api.ts";
export interface ExportPrefs { format: ExportFormat; content: ExportContent; dir: string | null }
export const EXPORT_PREFS_KEY = "rlt.export.prefs";
const FORMATS: ExportFormat[] = ["md", "txt", "srt", "docx"];
const CONTENTS: ExportContent[] = ["both", "translation", "source"];
type Store = Pick<Storage, "getItem" | "setItem">;

export function loadExportPrefs(store: Store): ExportPrefs {
  const dflt: ExportPrefs = { format: "md", content: "both", dir: null };
  try {
    const o = JSON.parse(store.getItem(EXPORT_PREFS_KEY) ?? "{}");
    return {
      format: FORMATS.includes(o.format) ? o.format : dflt.format,
      content: CONTENTS.includes(o.content) ? o.content : dflt.content,
      dir: typeof o.dir === "string" && o.dir ? o.dir : null,
    };
  } catch { return dflt; }
}

/** Luu lua chon; tra false neu khong luu duoc (khong nem loi). */
export function saveExportPrefs(store: Store, p: ExportPrefs): boolean {
  try { store.setItem(EXPORT_PREFS_KEY, JSON.stringify(p)); return true; } catch { return false; }
}

/** Ket qua hop chon thu muc: null (huy) hoac chuoi rong thi giu thu muc cu; con lai thi thu muc moi. */
export function applyPickedDir(current: string | null, picked: string | null | undefined): string | null {
  return typeof picked === "string" && picked ? picked : current;
}

/** Chi tiet loi de hien cho nguoi dung: ma ngan cua Rust, hoac thong diep Tauri (vd ACL) cat con 160 ky tu. */
export function pickErrorDetail(e: unknown): string {
  const raw = typeof e === "string" ? e : (e as { message?: string })?.message ?? "unknown";
  const one = raw.replace(/\s+/g, " ").trim();
  return one.length > 160 ? one.slice(0, 157) + "..." : one || "unknown";
}
