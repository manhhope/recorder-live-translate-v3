// Trang thai cap nhat tu dong (US-33), song o module toan cuc nhu rec.svelte.ts: kiem/tai ngam khong phu thuoc man dang mo.
// Quy tac: cong tat tat thi KHONG tu gui yeu cau nao (AC-33.7); dang ghi thi tai duoc nhung khong cai (AC-33.1).
import * as api from "./api.ts";
import { app } from "./app.svelte.ts";
import { AUTO_UPDATE_KEY, installGate, isNewer, readAutoUpdate } from "./update.ts";
import { APP_VERSION } from "./whats-new.svelte.ts";

export type UpdatePhase = "idle" | "checking" | "uptodate" | "available" | "downloading" | "ready" | "installing" | "error";

export const upd = $state({
  phase: "idle" as UpdatePhase,
  info: null as api.UpdateInfo | null,
  downloaded: 0,
  total: null as number | null,
  /** Ma loi ngan (vd `update_signature`); giao dien doi sang khoa `err.*`. */
  error: null as string | null,
  /** Buoc dang loi, de nut "Thu lai" lam dung buoc. */
  failedStep: null as "check" | "download" | "install" | null,
  lastCheck: null as number | null,
  auto: true,
  /** Nguoi dung bam "De sau" tren thanh nhac. */
  dismissed: false,
});

/** Nguon su that la cot DB `settings.auto_update_enabled`; localStorage chi la ban sao dung khi DB khong doc duoc (vd trinh duyet thuan). */
export const initUpdate = async (): Promise<void> => {
  upd.auto = readAutoUpdate(globalThis.localStorage);
  try {
    upd.auto = await api.updateAutoGet();
    try { globalThis.localStorage?.setItem(AUTO_UPDATE_KEY, upd.auto ? "1" : "0"); } catch { /* bo qua */ }
  } catch { /* giu ban sao localStorage */ }
};
export const installBlocked = () => installGate(app.recording) !== "ok";

export function setAutoUpdate(on: boolean): void {
  upd.auto = on;
  void api.updateAutoSet(on).catch(() => { /* ban sao localStorage van giu */ });
  try { globalThis.localStorage?.setItem(AUTO_UPDATE_KEY, on ? "1" : "0"); } catch { /* bo qua */ }
}

const code = (e: unknown) => api.errCode(e);

/** `silent`: kiem nen (tu dong) khong doi giao dien neu khong co gi moi; loi mang nen khong lam phien nguoi dung. */
export async function checkNow(silent = false): Promise<void> {
  if (upd.phase === "checking" || upd.phase === "downloading" || upd.phase === "installing") return;
  if (upd.phase === "ready") return;
  upd.phase = "checking"; upd.error = null; upd.failedStep = null;
  try {
    const info = await api.updateCheck();
    upd.lastCheck = Date.now();
    // Phong thu them: Rust da chi bao ban moi hon, o day kiem lai de khong bao gio hien/cai ban thap hon hoac bang (AC-33.10).
    if (!info || !isNewer(APP_VERSION, info.version)) { upd.info = null; upd.phase = "uptodate"; return; }
    upd.info = info; upd.phase = "available"; upd.dismissed = false;
    if (silent && upd.auto) await download();
  } catch (e) {
    upd.lastCheck = Date.now();
    if (silent && code(e) !== "update_signature") { upd.phase = "idle"; return; }
    upd.phase = "error"; upd.error = code(e); upd.failedStep = "check";
  }
}

export async function download(): Promise<void> {
  if (upd.phase !== "available" && !(upd.phase === "error" && upd.failedStep === "download")) return;
  upd.phase = "downloading"; upd.downloaded = 0; upd.total = null; upd.error = null; upd.failedStep = null;
  try {
    await api.updateDownload(ev => {
      if (ev.kind === "progress") { upd.downloaded = ev.downloaded; upd.total = ev.total; }
    });
    upd.phase = "ready";
  } catch (e) {
    upd.phase = "error"; upd.error = code(e); upd.failedStep = "download";
  }
}

export async function install(): Promise<void> {
  if (upd.phase !== "ready" && !(upd.phase === "error" && upd.failedStep === "install")) return;
  if (installBlocked()) return; // hoan toi khi phien Dung
  upd.phase = "installing"; upd.error = null; upd.failedStep = null;
  try {
    await api.updateInstall(); // thanh cong thi app khoi dong lai, khong tro ve
  } catch (e) {
    upd.phase = "error"; upd.error = code(e); upd.failedStep = "install";
  }
}

let loop: ReturnType<typeof setInterval> | undefined;
export const AUTO_FIRST_MS = 10_000;
export const AUTO_EVERY_MS = 6 * 3600_000;
/** Goi mot lan khi app mo: kiem nen sau vai giay roi moi 6 gio, chi khi cong tat bat. */
export function startAutoUpdate(): () => void {
  initUpdate();
  const tick = () => { if (upd.auto) void checkNow(true); };
  const first = setTimeout(tick, AUTO_FIRST_MS);
  loop = setInterval(tick, AUTO_EVERY_MS);
  return () => { clearTimeout(first); clearInterval(loop); };
}
