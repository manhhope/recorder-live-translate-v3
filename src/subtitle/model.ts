// Logic thuan cua cua so phu de (US-13): chon dong nao hien theo che do, doi ma loi mau thanh khoa i18n.

export type DisplayMode = "bilingual" | "translation_only" | "source_only";

export interface SubtitleSettings {
  display_mode: DisplayMode;
  font_size_pt: number;
  source_color: string;
  translation_color: string;
  opacity_percent: number;
  hidden_from_capture: boolean;
}

export interface Line { source: string; translation: string }
/** Dong chu tu luong phien ghi (cung hinh dang voi `LineDto` cua cua so chinh). */
export interface FeedLine { captionLineId: number; sourceText: string; translatedText: string | null }

/** Khop `MAX_LINES` o Rust (subtitle/feed.rs): chi giu cac dong cuoi. */
export const MAX_LINES = 300;

/** Dong cung `captionLineId` thi thay tai cho (chu tam -> chu chot); dong moi them cuoi. Khong sua mang vao. */
export function upsertLine<T extends FeedLine>(lines: T[], line: T): T[] {
  const i = lines.findIndex(l => l.captionLineId === line.captionLineId);
  if (i >= 0) { const out = lines.slice(); out[i] = line; return out; }
  const out = [...lines, line];
  return out.length > MAX_LINES ? out.slice(out.length - MAX_LINES) : out;
}

/** Popup mo giua chung: ban chup tu Rust (thu tu cu -> moi) ghep voi cac dong da nhan qua su kien trong luc cho ban chup.
 *  Cung id thi dong da nhan qua su kien thang (moi hon); dong chi co o su kien dat sau ban chup. */
export function mergeSnapshot<T extends FeedLine>(current: T[], snapshot: T[]): T[] {
  const cur = new Map(current.map(l => [l.captionLineId, l]));
  const snapIds = new Set(snapshot.map(l => l.captionLineId));
  const out = [...snapshot.map(l => cur.get(l.captionLineId) ?? l), ...current.filter(l => !snapIds.has(l.captionLineId))];
  return out.length > MAX_LINES ? out.slice(out.length - MAX_LINES) : out;
}

/** Cac hang hien thi cua nhieu dong, theo che do hien tai. */
export function rowsForLines(mode: DisplayMode, lines: FeedLine[]): Row[] {
  return lines.flatMap(l => rowsFor(mode, { source: l.sourceText, translation: l.translatedText ?? "" }));
}

/** Thanh nut hien khi con tro nam trong popup (trang hoac Rust bao), khi tieu diem ban phim nam trong thanh, hoac menu dang mo. */
export const toolbarVisible = (pageHover: boolean, rustHover: boolean, focusWithin: boolean, menuOpen: boolean) =>
  pageHover || rustHover || focusWithin || menuOpen;
export interface Row { kind: "source" | "translation"; text: string }


/** Che do song ngu hien ca hai (goc truoc, dich sau); cac che do kia chi hien mot dong. Dong rong bi bo. */
export function rowsFor(mode: DisplayMode, line: Line | null): Row[] {
  if (!line) return [];
  const rows: Row[] = [];
  if (mode !== "translation_only" && line.source.trim()) rows.push({ kind: "source", text: line.source });
  if (mode !== "source_only" && line.translation.trim()) rows.push({ kind: "translation", text: line.translation });
  return rows;
}

export interface ColorErrorInfo { key: string; values?: Record<string, string> }

/** Backend tra "invalid" hoac "low_contrast:<ti le>"; doi sang khoa i18n. */
export function colorError(code: string): ColorErrorInfo {
  if (code.startsWith("low_contrast:")) {
    return { key: "sub.color.err.contrast", values: { ratio: code.slice("low_contrast:".length) } };
  }
  return { key: "sub.color.err.invalid" };
}

export const MODES: { mode: DisplayMode; key: string }[] = [
  { mode: "bilingual", key: "sub.mode.bilingual" },
  { mode: "translation_only", key: "sub.mode.translation" },
  { mode: "source_only", key: "sub.mode.source" },
];

/** Nen cua so: den 0..80% trong suot -> alpha = 1 - opacity. */
export function backgroundCss(opacityPercent: number): string {
  const p = Math.min(80, Math.max(0, Math.round(opacityPercent)));
  return `rgba(16, 20, 24, ${((100 - p) / 100).toFixed(2)})`;
}
