import { test } from "node:test";
import assert from "node:assert/strict";
import { keysFor, micState, screenState, boolState, missingRequired, shouldAutoShow, takeReminders, permOfStartError, remindOnStart, loadOff, saveOff, OFF_KEY, type PermStates, type PermKey } from "./perm-welcome.ts";

const S = (mic: PermStates["mic"], screen: PermStates["screen"], ax: PermStates["ax"] = "allowed"): PermStates => ({ mic, screen, ax });
const mem = () => { const m = new Map<string, string>(); return { getItem: (k: string) => m.get(k) ?? null, setItem: (k: string, v: string) => void m.set(k, v), removeItem: (k: string) => void m.delete(k) }; };

test("micState: authorized -> allowed; notDetermined va gia tri la -> notasked; denied/restricted -> denied", () => {
  assert.equal(micState("authorized"), "allowed");
  assert.equal(micState("notDetermined"), "notasked");
  assert.equal(micState("whatever"), "notasked");
  assert.equal(micState(null), "notasked");
  assert.equal(micState("denied"), "denied");
  assert.equal(micState("restricted"), "denied");
});

test("screenState/boolState: khong phan biet chua hoi va tu choi cho den khi da mo Cai dat", () => {
  assert.equal(screenState("granted", false), "allowed");
  assert.equal(screenState("denied", false), "notasked");
  assert.equal(screenState("denied", true), "denied");
  assert.equal(screenState("unknown", false), "notasked");
  assert.equal(screenState("needsrestart", false), "needsrestart");
  assert.equal(boolState(true, false), "allowed");
  assert.equal(boolState(false, false), "notasked");
  assert.equal(boolState(false, true), "denied");
  assert.equal(boolState(null, true), "denied");
});

test("missingRequired: Accessibility tuy chon khong tinh; needsrestart van la thieu", () => {
  assert.deepEqual(missingRequired(S("allowed", "allowed", "notasked")), []);
  assert.deepEqual(missingRequired(S("notasked", "allowed")), ["mic"]);
  assert.deepEqual(missingRequired(S("denied", "needsrestart")), ["mic", "screen"]);
});

test("shouldAutoShow: hien khi con quyen bat buoc thieu; du quyen thi khong; da chon Dung hien lai thi khong", () => {
  assert.equal(shouldAutoShow(S("notasked", "notasked"), false), true);
  assert.equal(shouldAutoShow(S("allowed", "denied"), false), true);
  assert.equal(shouldAutoShow(S("allowed", "allowed", "denied"), false), false);
  assert.equal(shouldAutoShow(S("notasked", "notasked"), true), false);
});

test("takeReminders: moi quyen toi da mot lan moi lan mo app", () => {
  const r = new Set<PermKey>();
  assert.deepEqual(takeReminders(r, ["mic", "screen"]), ["mic", "screen"]);
  assert.deepEqual(takeReminders(r, ["mic", "screen"]), []);
  assert.deepEqual(takeReminders(new Set<PermKey>(["mic"]), ["mic", "screen"]), ["screen"]);
});

test("permOfStartError: chi hai ma quyen cu the; ma khac la null", () => {
  assert.equal(permOfStartError("permission_screen"), "screen");
  assert.equal(permOfStartError("permission_mic"), "mic");
  for (const c of ["permission", "key_missing", "network", ""]) assert.equal(permOfStartError(c), null);
});

test("remindOnStart: nhac dung quyen thieu mot lan; da nhac luc mo app, da tat, hoac ma khac thi khong", () => {
  const r = new Set<PermKey>();
  assert.equal(remindOnStart("permission_screen", r, false), "screen");
  assert.equal(remindOnStart("permission_screen", r, false), null); // da nhac
  assert.equal(remindOnStart("permission_mic", r, false), "mic"); // quyen khac van duoc nhac
  assert.equal(remindOnStart("permission_mic", new Set<PermKey>(), true), null); // Dung hien lai
  assert.equal(remindOnStart("network", new Set<PermKey>(), false), null);
  assert.equal(remindOnStart("permission_screen", new Set<PermKey>(["screen"]), false), null); // da hien o man Welcome luc mo app
});

test("loadOff/saveOff: nho lua chon Dung hien lai, bo chon thi xoa; luu tru hong khong nem loi", () => {
  const s = mem();
  assert.equal(loadOff(s), false);
  saveOff(s, true);
  assert.equal(s.getItem(OFF_KEY), "1");
  assert.equal(loadOff(s), true);
  saveOff(s, false);
  assert.equal(loadOff(s), false);
  const bad = { getItem: () => { throw new Error("x"); }, setItem: () => { throw new Error("x"); }, removeItem: () => { throw new Error("x"); } };
  assert.equal(loadOff(bad), false);
  assert.doesNotThrow(() => saveOff(bad, true));
});

test("keysFor: Windows chi co micro; macOS va nen tang khac giu du ba quyen", () => {
  assert.deepEqual(keysFor("windows"), ["mic"]);
  assert.deepEqual(keysFor("macos"), ["mic", "screen", "ax"]);
  assert.deepEqual(keysFor(undefined), ["mic", "screen", "ax"]);
});
