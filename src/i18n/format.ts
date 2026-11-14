// Dinh dang ngay, so, tien theo locale bang Intl (khong tu ghep chuoi). Ham thuan, khong phu thuoc svelte-i18n.
export type Locale = "en" | "pt-PT";

export function formatDate(value: Date | number, locale: Locale, opts?: Intl.DateTimeFormatOptions): string {
  const o: Intl.DateTimeFormatOptions = opts ?? { day: "2-digit", month: "2-digit", year: "numeric" };
  return new Intl.DateTimeFormat(locale, { timeZone: "UTC", ...o }).format(value);
}

export function formatTime(value: Date | number, locale: Locale, timeZone = "UTC"): string {
  return new Intl.DateTimeFormat(locale, { hour: "2-digit", minute: "2-digit", hourCycle: "h23", timeZone }).format(value);
}

export function formatNumber(value: number, locale: Locale, opts?: Intl.NumberFormatOptions): string {
  return new Intl.NumberFormat(locale, opts).format(value);
}

export function formatCurrency(value: number, locale: Locale, currency = "USD"): string {
  return new Intl.NumberFormat(locale, { style: "currency", currency }).format(value);
}
