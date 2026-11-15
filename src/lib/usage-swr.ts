// Logic stale-while-revalidate cua man Usage Logs, tach khoi giao dien de test don vi.
// Quy tac: so Soniox da co tren man KHONG BAO GIO bi xoa hay thay bang so rong/uoc tinh khi tai lai loi.
import type { UsageSourceOut } from "./api.ts";

/** Canh bao gon: `reason` = ly do Soniox khong dung duoc (khoa i18n set.usage.reason.*), `err` = ma loi cua lenh (err.*). */
export type UsageWarning = { kind: "reason" | "err"; code: string };
export interface UsageView { data: UsageSourceOut | null; warning: UsageWarning | null }

/** Man dang giu mot ban Soniox that (tu cache hoac vua tai) hay chua. */
export const hasSoniox = (d: UsageSourceOut | null): d is UsageSourceOut => d?.source === "soniox";

/** Ket qua cua `usage_week_source`. Soniox -> thay so moi, xoa canh bao. Du phong cuc bo (khong ket noi/het han khoa/Soniox loi)
 *  -> neu dang giu so Soniox thi GIU, kem canh bao; neu chua co thi hien so cuc bo kem canh bao. */
export function applyRefresh(cur: UsageSourceOut | null, res: UsageSourceOut): UsageView {
  if (res.source === "soniox") return { data: res, warning: null };
  const warning: UsageWarning = { kind: "reason", code: res.reason ?? "unknown" };
  return hasSoniox(cur) ? { data: cur, warning } : { data: res, warning };
}

/** Lenh nem loi (loi kho, task hong): giu nguyen so dang co, kem canh bao ma loi. */
export function applyFailure(cur: UsageSourceOut | null, code: string): UsageView {
  return { data: cur, warning: { kind: "err", code } };
}

/** Gio dia phuong HH:mm cua `fetchedAt` (epoch giay) cho nhan "Cap nhat luc HH:mm". */
export function updatedTime(fetchedAt: number, locale: string): string {
  return new Date(fetchedAt * 1000).toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit", hour12: false });
}

/** Nhan nguon hien tren man: ban Soniox (da luu hoac vua tai) co gio; so cuc bo thi khong co gio "cap nhat". */
export const showsUpdatedAt = (d: UsageSourceOut | null): boolean => hasSoniox(d) && d.fetchedAt > 0;

/** Thu trong tuan (0 = Thu Hai ... 6 = Chu Nhat) cua ngay lich "YYYY-MM-DD" (ngay dia phuong do backend gui, tinh theo lich, khong qua mui gio). */
export function weekdayIndex(date: string): number {
  const [y, m, d] = date.split("-").map(Number);
  return (new Date(Date.UTC(y, m - 1, d)).getUTCDay() + 6) % 7;
}

/** Ngay lich ngan "9 Nov" cua "YYYY-MM-DD" (lich thuan, khong doi mui gio). */
export function shortDate(date: string, locale: string): string {
  const [y, m, d] = date.split("-").map(Number);
  return new Intl.DateTimeFormat(locale === "en" ? "en-GB" : locale, { day: "numeric", month: "short", timeZone: "UTC" }).format(new Date(Date.UTC(y, m - 1, d)));
}
