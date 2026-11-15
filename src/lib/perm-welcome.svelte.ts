// Trang thai modal Welcome: cap quyen. Dung chung co che xep hang runWhenIdle cua What's New (khong chong hop thoai).
// Thu tu: Welcome cap quyen truoc, What's New sau (What's New chi duoc goi khi Welcome khong hien hoac sau khi no dong).
import * as api from "./api.ts";
import { boolState, loadOff, micState, missingRequired, PERM_ORDER, remindOnStart, saveOff, screenState, shouldAutoShow, takeReminders, type PermKey, type PermStates } from "./perm-welcome.ts";
import { runWhenIdle } from "./whats-new.ts";
import { evaluateWhatsNew } from "./whats-new.svelte.ts";

export const permWelcome = $state({
  open: false,
  states: { mic: "notasked", screen: "notasked", ax: "notasked" } as PermStates,
  focus: null as PermKey | null, // quyen thieu khi bam Bat dau (lam noi bat dong do)
  off: false, // o "Dung hien lai"
  loaded: false,
  err: false,
  busy: false,
  errCode: null as string | null,
});

const reminded = new Set<PermKey>(); // quyen da nhac trong lan mo app nay
const settingsOpened: Record<PermKey, boolean> = { mic: false, screen: false, ax: false };
let chainWhatsNew = false; // dong Welcome xong thi moi cho What's New (chi khi Welcome mo luc khoi dong)

/** Doc lai ca ba quyen. Ghi man hinh: doc khong ho (CGPreflight); chi khi nguoi dung da mo Cai dat He thong tu man nay ma van thieu thi do SCShareableContent (khong bat hinh, khong hop thoai) de nhan ra "can khoi dong lai". */
export async function refreshPerms(): Promise<PermStates> {
  let err = false;
  const next: PermStates = { ...permWelcome.states };
  try {
    const p = await api.permissionsStatus();
    next.mic = micState(p.mic);
    let screen = p.screen;
    if (screen !== "granted" && settingsOpened.screen) {
      try { screen = await api.screenPermissionCheck(true); } catch { /* giu ket qua doc nhanh */ }
    }
    next.screen = screenState(screen, settingsOpened.screen);
  } catch { err = true; }
  try { next.ax = boolState(await api.axStatus(), settingsOpened.ax); } catch { err = true; }
  permWelcome.states = next; permWelcome.err = err; permWelcome.loaded = true;
  return next;
}

/** Goi mot lan luc khoi dong. Tra ve true neu Welcome da mo (khi do What's New se duoc goi sau khi no dong). */
export async function evaluatePermWelcome(): Promise<boolean> {
  permWelcome.off = loadOff(localStorage);
  if (permWelcome.off) return false;
  const st = await refreshPerms();
  if (!shouldAutoShow(st, permWelcome.off)) return false;
  const otherDialog = () => !!document.querySelector('[role="dialog"][aria-modal="true"]');
  return new Promise<boolean>(resolve => {
    runWhenIdle(() => {
      if (otherDialog()) return resolve(false); // het luot cho van con hop thoai khac: de lan mo sau, khong chong
      takeReminders(reminded, missingRequired(permWelcome.states));
      chainWhatsNew = true; permWelcome.focus = null; permWelcome.open = true; resolve(true);
    }, otherDialog, (f, ms) => setTimeout(f, ms));
  });
}

/** Mo thu cong tu Cai dat > Quyen hoac nut "Xem lai quyen" (khong tinh la nhac). */
export function openPermWelcome(focus: PermKey | null = null): void {
  permWelcome.focus = focus; permWelcome.errCode = null; chainWhatsNew = false;
  permWelcome.open = true; void refreshPerms();
}

/** Bam Bat dau ma loi la thieu quyen: mo hop thoai nhac dung quyen do, toi da mot lan moi quyen moi lan mo app. */
export async function remindMissingOnStart(code: string): Promise<boolean> {
  if (permWelcome.open) return false;
  const k = remindOnStart(code, reminded, loadOff(localStorage));
  if (!k) return false;
  chainWhatsNew = false; permWelcome.focus = k; permWelcome.errCode = null; permWelcome.open = true;
  void refreshPerms();
  return true;
}

/** Continue, Skip for now, X, Esc: dong; o "Dung hien lai" duoc luu. */
export function closePermWelcome(): void {
  saveOff(localStorage, permWelcome.off);
  permWelcome.open = false; permWelcome.focus = null;
  if (chainWhatsNew) { chainWhatsNew = false; requestAnimationFrame(() => evaluateWhatsNew()); }
}

export async function allowMic(): Promise<void> {
  permWelcome.busy = true; permWelcome.errCode = null;
  try { permWelcome.states = { ...permWelcome.states, mic: micState(await api.micPermissionRequest()) }; }
  catch (e) { permWelcome.errCode = api.errCode(e); }
  permWelcome.busy = false;
}

const PANE: Record<PermKey, api.SettingsPane> = { mic: "microphone", screen: "screen_recording", ax: "accessibility" };
export async function openPane(k: PermKey): Promise<void> {
  permWelcome.errCode = null;
  try { await api.openSystemSettings(PANE[k]); settingsOpened[k] = true; } catch (e) { permWelcome.errCode = api.errCode(e); }
}

export async function restartNow(): Promise<void> {
  permWelcome.errCode = null;
  try { await api.restartApp(); } catch (e) { permWelcome.errCode = api.errCode(e); }
}

export const PERM_KEYS = PERM_ORDER;
