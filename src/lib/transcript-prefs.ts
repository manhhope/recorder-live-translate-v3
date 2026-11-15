// Tuy chinh hien thi transcript o man Ghi (co chu, mau chu nguon/dich, hien ban nguon). Logic thuan, khong phu thuoc giao dien,
// de test don vi. Chi ap cho man chinh; cua so phu de co cai dat rieng (Cai dat > Phu de, luu o backend) va khong bi anh huong.
// Luu o localStorage (cung cach voi tuy chon giao dien khac, vd KEEP_AWAKE_KEY). Luu ma mau (id), khong luu hex,
// de doi sang/toi van dat tuong phan AA.

export const TRANSCRIPT_PREFS_KEY = "rlt.ui.transcript";

/** Chu dich (px) theo muc; chu nguon nho hon 2px. Muc mac dinh = 15px, bang --rlt-fdst cua token. */
export const SIZES = [12, 13, 14, 15, 17, 19, 22] as const;
export const DEFAULT_SIZE = 3;
export const SRC_OFFSET_PX = 2;

export type View = "both" | "dst" | "src";
export type ColorId = "default" | "blue" | "green" | "amber" | "red" | "purple";
export const COLOR_IDS: ColorId[] = ["default", "blue", "green", "amber", "red", "purple"];

/** Hex theo nen sang / toi. `default` = null: dung token he thong (--rlt-text, --rlt-text2). */
export const PALETTE: Record<Exclude<ColorId, "default">, { light: string; dark: string }> = {
  blue: { light: "#0A5BC4", dark: "#6AA6FF" },
  green: { light: "#1A7436", dark: "#5EC977" },
  amber: { light: "#8A5A00", dark: "#F2B84B" },
  red: { light: "#C5221A", dark: "#FF6B63" },
  purple: { light: "#6B3FA0", dark: "#B794F4" },
};
/** Nen cua vung transcript (token --rlt-bg va --rlt-card cua tokens-v3.css), dung de do tuong phan. */
export const SURFACES = { light: ["#F5F5F7", "#FFFFFF"], dark: ["#1C1C1E", "#2C2C2E"] } as const;

export interface TranscriptPrefs { size: number; srcColor: ColorId; dstColor: ColorId; view: View }
export const DEFAULTS: TranscriptPrefs = { size: DEFAULT_SIZE, srcColor: "default", dstColor: "default", view: "both" };

const isColor = (v: unknown): v is ColorId => typeof v === "string" && (COLOR_IDS as string[]).includes(v);
const isView = (v: unknown): v is View => v === "both" || v === "dst" || v === "src";

/** Doc gia tri da luu; tung truong hong/la thi ve mac dinh, khong lam hong ca bo. */
export function parsePrefs(raw: string | null | undefined): TranscriptPrefs {
  let o: Record<string, unknown> = {};
  try { const j = raw ? JSON.parse(raw) : null; if (j && typeof j === "object") o = j as Record<string, unknown>; } catch { /* mac dinh */ }
  const size = typeof o.size === "number" && Number.isInteger(o.size) && o.size >= 0 && o.size < SIZES.length ? o.size : DEFAULT_SIZE;
  return {
    size,
    srcColor: isColor(o.srcColor) ? o.srcColor : "default",
    dstColor: isColor(o.dstColor) ? o.dstColor : "default",
    view: isView(o.view) ? o.view : "both",
  };
}

type Store = Pick<Storage, "getItem" | "setItem">;
const defaultStore = (): Store | null => { try { return globalThis.localStorage ?? null; } catch { return null; } };

export function loadPrefs(store: Store | null = defaultStore()): TranscriptPrefs {
  try { return parsePrefs(store?.getItem(TRANSCRIPT_PREFS_KEY)); } catch { return { ...DEFAULTS }; }
}
export function savePrefs(p: TranscriptPrefs, store: Store | null = defaultStore()): void {
  try { store?.setItem(TRANSCRIPT_PREFS_KEY, JSON.stringify(p)); } catch { /* bo qua: khong luu duoc thi van ap trong phien */ }
}

/** Doi muc co chu theo buoc, giu trong [0, SIZES.length - 1]. */
export const stepSize = (p: TranscriptPrefs, delta: number): TranscriptPrefs =>
  ({ ...p, size: Math.min(SIZES.length - 1, Math.max(0, p.size + delta)) });

/** Cong tat "Hien ban nguon": bat = both, tat = dst. Che do "Chi nguon" (src) tinh la dang hien ban nguon. */
export const showsSource = (v: View): boolean => v !== "dst";
export const withShowSource = (v: View, on: boolean): View => (on ? (v === "src" ? "src" : "both") : "dst");

/** Mau that cho mot ma; `default` tra null (de CSS dung token). */
export function resolveColor(id: ColorId, dark: boolean): string | null {
  return id === "default" ? null : PALETTE[id][dark ? "dark" : "light"];
}

/** Bien CSS ap len vung transcript. Mau mac dinh khong dat bien (CSS dung fallback theo token). */
export function cssVars(p: TranscriptPrefs, dark: boolean): string {
  const dst = SIZES[p.size];
  const v = [`--rlt-fdst:${dst}px`, `--rlt-fsrc:${dst - SRC_OFFSET_PX}px`];
  const s = resolveColor(p.srcColor, dark), d = resolveColor(p.dstColor, dark);
  if (s) v.push(`--rlt-usrc:${s}`);
  if (d) v.push(`--rlt-udst:${d}`);
  return v.join(";");
}

function lum(hex: string): number {
  const c = [1, 3, 5].map(i => parseInt(hex.slice(i, i + 2), 16) / 255).map(x => (x <= 0.03928 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
}
/** Ti le tuong phan WCAG giua hai mau `#RRGGBB`. */
export function contrast(a: string, b: string): number {
  const [x, y] = [lum(a), lum(b)].sort((m, n) => n - m);
  return (x + 0.05) / (y + 0.05);
}
