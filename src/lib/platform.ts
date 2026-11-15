// Nen tang chay giao dien. Windows: Ctrl thay Cmd, "Windows" thay "macOS".
export function isWindows(ua?: string): boolean {
  const s = ua ?? (typeof navigator !== "undefined" ? navigator.userAgent : "");
  return /Windows/i.test(s);
}

/** Doi chu rieng cua macOS trong chuoi giao dien sang Windows. Khong sua file JSON locale. */
export function platformize(text: string, windows: boolean = isWindows()): string {
  if (!windows) return text;
  return text
    .replace(/⌘⇧([A-Za-z0-9])/g, (_m, k: string) => `Ctrl+Shift+${k}`)
    .replace(/⌘([A-Za-z0-9,.])/g, (_m, k: string) => `Ctrl+${k}`)
    .replace(/\bCmd\+/g, "Ctrl+")
    .replace(/\bPorta-chaves\b/g, "Credential Manager")
    .replace(/\bKeychain\b/g, "Credential Manager")
    .replace(/\bdeste Mac\b/g, "deste PC")
    .replace(/\bthis Mac\b/g, "this PC")
    .replace(/\bmacOS\b/g, "Windows");
}
