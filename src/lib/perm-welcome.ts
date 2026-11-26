// Welcome: cap quyen. Logic thuan (khong phu thuoc giao dien) de test don vi.
// Ba quyen: micro (bat buoc), Ghi man hinh + am thanh he thong (bat buoc), Accessibility (tuy chon, chi cho thanh chon chu).
export type PermKey = "mic" | "screen" | "ax";
export const PERM_ORDER: PermKey[] = ["mic", "screen", "ax"];
export const REQUIRED: PermKey[] = ["mic", "screen"];
/** allowed | notasked (chua hoi, hoac he thong khong cho phan biet) | denied | needsrestart (Ghi man hinh: da bat nhung tien trinh chua nhan). */
export type PermState = "allowed" | "notasked" | "denied" | "needsrestart";
export type PermStates = Record<PermKey, PermState>;

/** mic: AVCaptureDevice (`authorized|notDetermined|denied|restricted`); screen: `granted|denied|needsrestart|unknown`. */
export function micState(raw: string | null | undefined): PermState {
  if (raw === "authorized" || raw === "granted") return "allowed";
  if (raw === "denied" || raw === "restricted") return "denied";
  return "notasked";
}
/**
 * Ghi man hinh va Accessibility chi co "co/khong" tu lenh doc khong ho (CGPreflight, AXIsProcessTrusted), nen khong phan biet duoc
 * "chua hoi" va "da tu choi": coi la `notasked` cho den khi nguoi dung da mo Cai dat He thong tu man nay, luc do la `denied`.
 */
export function boolState(granted: boolean | null | undefined, settingsOpened: boolean): PermState {
  if (granted) return "allowed";
  return settingsOpened ? "denied" : "notasked";
}
export function screenState(raw: string | null | undefined, settingsOpened: boolean): PermState {
  if (raw === "granted" || raw === "authorized") return "allowed";
  if (raw === "needsrestart") return "needsrestart";
  return boolState(false, settingsOpened);
}

/** Quyen ap dung tren nen tang: Windows chi co micro (khong co Ghi man hinh / Accessibility). */
export function keysFor(platform: string | null | undefined): PermKey[] {
  return platform === "windows" ? ["mic"] : PERM_ORDER;
}

export const isAllowed = (s: PermState) => s === "allowed";
/** Quyen bat buoc con thieu (Accessibility la tuy chon, khong tinh). */
export function missingRequired(st: PermStates): PermKey[] {
  return REQUIRED.filter(k => !isAllowed(st[k]));
}

export const OFF_KEY = "rlt.ui.permWelcomeOff";
type Store = Pick<Storage, "getItem" | "setItem" | "removeItem">;
export function loadOff(s: Store): boolean {
  try { return s.getItem(OFF_KEY) === "1"; } catch { return false; }
}
export function saveOff(s: Store, off: boolean): void {
  try { if (off) s.setItem(OFF_KEY, "1"); else s.removeItem(OFF_KEY); } catch { /* bo qua: lan sau hien lai */ }
}

/** Lan mo app: hien tu dong khi con quyen bat buoc chua cap va nguoi dung chua chon "Dung hien lai". */
export function shouldAutoShow(st: PermStates, off: boolean): boolean {
  return !off && missingRequired(st).length > 0;
}

/**
 * Mot quyen chi duoc nhac toi da mot lan moi lan mo app. Tra ve cac quyen CHUA nhac trong `missing` va ghi chung vao `reminded`.
 * Mo tay tu Cai dat khong goi ham nay (khong tinh la nhac).
 */
export function takeReminders(reminded: Set<PermKey>, missing: PermKey[]): PermKey[] {
  const fresh = missing.filter(k => !reminded.has(k));
  for (const k of fresh) reminded.add(k);
  return fresh;
}

/** Ma loi luc bam Bat dau -> quyen thieu (Rust: session/service.rs `permission_screen`, `permission_mic`). */
export function permOfStartError(code: string): PermKey | null {
  if (code === "permission_screen") return "screen";
  if (code === "permission_mic") return "mic";
  return null;
}

/**
 * Bam Bat dau ma thieu quyen: co mo hop thoai nhac khong? Chi khi quyen do chua nhac lan mo app nay va nguoi dung chua chon
 * "Dung hien lai". Neu khong, van co thong bao loi san co o man Ghi (khong mat thong tin).
 */
export function remindOnStart(code: string, reminded: Set<PermKey>, off: boolean): PermKey | null {
  const k = permOfStartError(code);
  if (!k || off) return null;
  return takeReminders(reminded, [k]).length ? k : null;
}

/**
 * Ngan Cai dat He thong mo thang tu the loi quyen o man Ghi. Windows chi co ngan Micro (am thanh he thong khong can quyen),
 * macOS: micro -> Micro, con lai (screen, permission chung) -> Ghi man hinh va am thanh he thong.
 */
export function paneForStartError(code: string, windows: boolean): "microphone" | "screen_recording" | null {
  if (!code.startsWith("permission")) return null;
  if (windows || code === "permission_mic") return "microphone";
  return "screen_recording";
}
