import { test } from "node:test";
import assert from "node:assert/strict";
import { isWindows, platformize } from "./platform.ts";
import { acceleratorFromEvent, canonicalAccel, displayAccel } from "./accel.ts";

test("isWindows nhan UA", () => {
  assert.equal(isWindows("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Edg/120"), true);
  assert.equal(isWindows("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"), false);
});

test("platformize doi ky hieu Mac sang Windows, giu nguyen tren Mac", () => {
  assert.equal(platformize("Nhan ⌘⇧T", true), "Nhan Ctrl+Shift+T");
  assert.equal(platformize("Cmd+K", true), "Ctrl+K");
  assert.equal(platformize("Keychain cua macOS tren this Mac", true), "Credential Manager cua Windows tren this PC");
  assert.equal(platformize("Porta-chaves deste Mac", true), "Credential Manager deste PC");
  assert.equal(platformize("⌘⇧T Keychain macOS", false), "⌘⇧T Keychain macOS");
});

test("accel Windows: Ctrl thay Cmd, metaKey la Super", () => {
  const ev = { code: "KeyT", metaKey: false, ctrlKey: true, altKey: false, shiftKey: true };
  assert.equal(acceleratorFromEvent(ev, true), "Control+Shift+T");
  assert.equal(acceleratorFromEvent({ ...ev, ctrlKey: false, metaKey: true }, true), "Super+Shift+T");
  assert.equal(canonicalAccel("CommandOrControl+Shift+T", true), "ctrl+shift+t");
  assert.equal(canonicalAccel("CommandOrControl+Shift+T", false), "cmd+shift+t");
  assert.equal(displayAccel("CommandOrControl+Alt+Shift+T", true), "Ctrl+Alt+Shift+T");
  assert.equal(displayAccel("CommandOrControl+Shift+T", false), "⇧⌘T");
});
