import test from "node:test";
import assert from "node:assert/strict";
import {
  COLOR_IDS, DEFAULTS, PALETTE, SIZES, SURFACES, TRANSCRIPT_PREFS_KEY, contrast, cssVars, loadPrefs, parsePrefs, resolveColor,
  savePrefs, showsSource, stepSize, withShowSource,
} from "./transcript-prefs.ts";

const mem = () => { const m = new Map<string, string>(); return { getItem: (k: string) => m.get(k) ?? null, setItem: (k: string, v: string) => void m.set(k, v), m }; };

test("defaults: mid size, default colours, both lines", () => {
  assert.deepEqual(parsePrefs(null), DEFAULTS);
  assert.equal(SIZES[DEFAULTS.size], 15);
  assert.equal(cssVars(DEFAULTS, false), "--rlt-fdst:15px;--rlt-fsrc:13px");
});

test("save then load returns the same choice (persistence across launches)", () => {
  const s = mem();
  const p = { size: 5, srcColor: "green" as const, dstColor: "amber" as const, view: "dst" as const };
  savePrefs(p, s);
  assert.deepEqual(loadPrefs(s), p);
  assert.ok(s.m.has(TRANSCRIPT_PREFS_KEY));
});

test("corrupt or unknown stored values fall back field by field", () => {
  assert.deepEqual(parsePrefs("{not json"), DEFAULTS);
  assert.deepEqual(parsePrefs(JSON.stringify({ size: 99, srcColor: "pink", dstColor: "blue", view: "x" })), { ...DEFAULTS, dstColor: "blue" });
  assert.deepEqual(parsePrefs(JSON.stringify({ size: 1.5 })), DEFAULTS);
  assert.deepEqual(loadPrefs({ getItem: () => { throw new Error("denied"); }, setItem: () => {} }), DEFAULTS);
});

test("save does not throw when storage is unavailable", () => {
  assert.doesNotThrow(() => savePrefs(DEFAULTS, { getItem: () => null, setItem: () => { throw new Error("full"); } }));
});

test("size steps clamp at both ends and change the css variables applied live", () => {
  let p = DEFAULTS;
  for (let i = 0; i < 20; i++) p = stepSize(p, 1);
  assert.equal(p.size, SIZES.length - 1);
  assert.equal(cssVars(p, false), `--rlt-fdst:22px;--rlt-fsrc:20px`);
  for (let i = 0; i < 20; i++) p = stepSize(p, -1);
  assert.equal(p.size, 0);
  assert.match(cssVars(p, true), /^--rlt-fdst:12px;--rlt-fsrc:10px/);
});

test("colours: default sets no variable, others resolve per theme", () => {
  assert.equal(resolveColor("default", true), null);
  const p = { ...DEFAULTS, srcColor: "blue" as const, dstColor: "red" as const };
  assert.match(cssVars(p, false), /--rlt-usrc:#0A5BC4;--rlt-udst:#C5221A$/);
  assert.match(cssVars(p, true), /--rlt-usrc:#6AA6FF;--rlt-udst:#FF6B63$/);
});

test("show-original switch maps onto the shared view (Both/Translation/Source)", () => {
  assert.equal(withShowSource("both", false), "dst");
  assert.equal(withShowSource("dst", true), "both");
  assert.equal(withShowSource("src", true), "src");
  assert.equal(showsSource("src"), true);
  assert.equal(showsSource("dst"), false);
  assert.equal(showsSource(withShowSource("both", false)), false);
});

test("every palette colour reaches WCAG AA (4.5:1) on both surfaces of its theme", () => {
  for (const id of COLOR_IDS) {
    if (id === "default") continue;
    for (const theme of ["light", "dark"] as const) for (const bg of SURFACES[theme]) {
      const r = contrast(PALETTE[id][theme], bg);
      assert.ok(r >= 4.5, `${id} ${theme} on ${bg}: ${r.toFixed(2)}`);
    }
  }
});

test("contrast helper matches known values", () => {
  assert.ok(Math.abs(contrast("#000000", "#FFFFFF") - 21) < 0.01);
  assert.ok(Math.abs(contrast("#777777", "#777777") - 1) < 0.01);
});
