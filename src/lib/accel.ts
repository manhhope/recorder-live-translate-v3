// Phim tat: doi giua su kien ban phim, chuoi cua Tauri ("CommandOrControl+Shift+T") va ky hieu hien thi ("⌘⇧T").
export const MODS = ["cmd", "ctrl", "alt", "shift"] as const;

const KEY_NAME: Record<string, string> = {
  Space: "Space", Enter: "Enter", Tab: "Tab", Escape: "Escape", Backspace: "Backspace", Delete: "Delete",
  ArrowUp: "Up", ArrowDown: "Down", ArrowLeft: "Left", ArrowRight: "Right", Home: "Home", End: "End", PageUp: "PageUp", PageDown: "PageDown",
  Minus: "-", Equal: "=", Comma: ",", Period: ".", Slash: "/", Semicolon: ";", Quote: "'", BracketLeft: "[", BracketRight: "]", Backslash: "\\", Backquote: "`",
};

/** Tu mot KeyboardEvent -> chuoi phim tat; null khi chi bam phim bo tro hoac phim khong ho tro. */
export function acceleratorFromEvent(e: Pick<KeyboardEvent, "code" | "metaKey" | "ctrlKey" | "altKey" | "shiftKey">): string | null {
  let key: string | null = null;
  if (/^Key[A-Z]$/.test(e.code)) key = e.code.slice(3);
  else if (/^Digit[0-9]$/.test(e.code)) key = e.code.slice(5);
  else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(e.code)) key = e.code;
  else key = KEY_NAME[e.code] ?? null;
  if (!key) return null;
  const parts: string[] = [];
  if (e.metaKey) parts.push("Command");
  if (e.ctrlKey) parts.push("Control");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  return [...parts, key].join("+");
}

/** Co it nhat mot phim ⌘, ⌃ hoac ⌥ (Shift mot minh khong du de lam phim tat toan cuc). */
export function hasMainModifier(accel: string): boolean {
  return accel.split("+").some(p => ["command", "commandorcontrol", "cmdorctrl", "cmd", "control", "ctrl", "alt", "option"].includes(p.toLowerCase()));
}

/** Chuan hoa de so trung: cung bo phim (khong phan biet thu tu, ten goi khac nhau) la cung mot chuoi. */
export function canonicalAccel(accel: string): string {
  const mods = new Set<string>(); let key = "";
  for (const raw of accel.split("+")) {
    const p = raw.trim().toLowerCase();
    if (["cmd", "command", "super", "meta", "commandorcontrol", "cmdorctrl", "commandorctrl"].includes(p)) mods.add("cmd");
    else if (["ctrl", "control"].includes(p)) mods.add("ctrl");
    else if (["alt", "option", "opt"].includes(p)) mods.add("alt");
    else if (p === "shift") mods.add("shift");
    else key = p;
  }
  return [...MODS.filter(m => mods.has(m)), key].join("+");
}

/** Ky hieu cho nguoi dung: ⌃⌥⇧⌘ + phim, theo thu tu quy uoc cua macOS. */
export function displayAccel(accel: string): string {
  const c = canonicalAccel(accel).split("+");
  const key = c[c.length - 1];
  const sym: Record<string, string> = { ctrl: "⌃", alt: "⌥", shift: "⇧", cmd: "⌘" };
  const out = (["ctrl", "alt", "shift", "cmd"] as const).filter(m => c.includes(m)).map(m => sym[m]).join("");
  const k: Record<string, string> = { up: "↑", down: "↓", left: "←", right: "→", space: "Space", enter: "↩", escape: "⎋", tab: "⇥", backspace: "⌫", delete: "⌦" };
  return out + (k[key] ?? key.toUpperCase());
}
