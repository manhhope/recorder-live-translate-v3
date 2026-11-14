// Trang thai giao dien dung chung.
import { DEFAULT_DST, DEFAULT_SRC } from "./langs.ts";

export type Screen = "rec" | "ses" | "ask" | "set";
export type SettingsTabId = "general" | "translate" | "shortcuts" | "glossary" | "subtitle" | "usage" | "drive" | "about";
export type Level = "idle" | "quiet" | "talk" | "clip" | "none";

export const app = $state({
  screen: "rec" as Screen,
  recording: false,
  seconds: 0,
  src: DEFAULT_SRC,
  dst: DEFAULT_DST,
  sysOn: true,
  micOn: true,
  view: "both" as "both" | "dst" | "src",
  theme: "system" as "system" | "light" | "dark",
  /** Phien ghi gan nhat (dang ghi hoac vua ghi xong); man Hoi AI dung no khi khong chon phien khac. */
  lastSession: null as string | null,
  /** Phien man Hoi AI dang hoi; null = tu chon phien moi nhat. */
  askSession: null as string | null,
  settingsTab: "general" as SettingsTabId,
});

/** Mo man Hoi AI (nut Record, thanh ben, phim tat/deeplink ask_ai). */
export function openAsk(sessionId: string | null = null) {
  app.askSession = sessionId ?? app.lastSession;
  app.screen = "ask";
}

/** Nhay toi Cai dat, muc "Dich va AI" (noi nhap khoa va chon mo hinh). */
export function openAiSettings() {
  app.settingsTab = "translate";
  app.screen = "set";
}

export function applyTheme(t: "system" | "light" | "dark") {
  app.theme = t;
  const el = document.documentElement;
  if (t === "system") el.removeAttribute("data-theme"); else el.setAttribute("data-theme", t);
}

export function fmtClock(total: number): string {
  const h = Math.floor(total / 3600), m = Math.floor((total % 3600) / 60), s = total % 60;
  return [h, m, s].map(n => String(n).padStart(2, "0")).join(":");
}
