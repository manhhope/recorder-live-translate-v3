// Logic thuan cua lop chon vung (US-48): doi diem keo trong cua so sang toa do logic GLOBAL va ma loi sang khoa i18n.

export interface Pt { x: number; y: number }

/** Toa do global (diem logic) = goc cua so tren man hinh + toa do trong cua so. */
export function toGlobal(win: Pt, local: Pt): Pt {
  return { x: win.x + local.x, y: win.y + local.y };
}

/** Khung ve tren lop phu tu hai diem keo bat ky (keo nguoc van ra khung duong). */
export function dragBox(a: Pt, b: Pt): { x: number; y: number; w: number; h: number } {
  return { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), w: Math.abs(a.x - b.x), h: Math.abs(a.y - b.y) };
}

const KNOWN = new Set(["no_permission", "no_monitor", "empty", "too_small", "all_black", "limit_reached"]);

/** Ma loi backend ("too_small", "capture_failed:...", ...) -> khoa i18n; ma la -> cap.err.unknown. */
export function errorKey(code: string): string {
  return KNOWN.has(code) ? `cap.err.${code.replace(/_/g, "")}` : "cap.err.unknown";
}
