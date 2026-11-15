// Logic thu muc xuat (tach khoi giao dien de test don vi): luu/nho lua chon (dinh dang, noi dung, thu muc luu lan truoc).
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

/** Thu muc chua file vua luu (dau tach / hoac \\); khong tach duoc thi giu thu muc cu. Hop thoai luu lan sau mo o day. */
export function rememberDir(current: string | null, savedPath: string | null | undefined): string | null {
  if (typeof savedPath !== "string") return current;
  const i = Math.max(savedPath.lastIndexOf("/"), savedPath.lastIndexOf("\\"));
  return i > 0 ? savedPath.slice(0, i) : i === 0 && savedPath.length > 1 ? savedPath.slice(0, 1) : current;
}
