// Trang thai giao dien dung chung (M2: du lieu gia, chua noi Tauri command).
import { DEFAULT_DST, DEFAULT_SRC } from "./mock.ts";

export type Screen = "rec" | "ses" | "set";
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
});

export function applyTheme(t: "system" | "light" | "dark") {
  app.theme = t;
  const el = document.documentElement;
  if (t === "system") el.removeAttribute("data-theme"); else el.setAttribute("data-theme", t);
}

export function fmtClock(total: number): string {
  const h = Math.floor(total / 3600), m = Math.floor((total % 3600) / 60), s = total % 60;
  return [h, m, s].map(n => String(n).padStart(2, "0")).join(":");
}
