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
