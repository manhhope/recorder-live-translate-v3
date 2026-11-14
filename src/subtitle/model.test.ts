import test from "node:test";
import assert from "node:assert/strict";
import { backgroundCss, colorError, MAX_LINES, mergeSnapshot, MODES, rowsFor, rowsForLines, toolbarVisible, upsertLine } from "./model.ts";

const line = { source: "Hello", translation: "Ola" };

test("bilingual shows source then translation", () => {
  assert.deepEqual(rowsFor("bilingual", line).map(r => r.kind), ["source", "translation"]);
});

test("single modes show exactly one row", () => {
  assert.deepEqual(rowsFor("translation_only", line), [{ kind: "translation", text: "Ola" }]);
  assert.deepEqual(rowsFor("source_only", line), [{ kind: "source", text: "Hello" }]);
});

test("empty parts and missing line produce no rows", () => {
  assert.deepEqual(rowsFor("bilingual", null), []);
  assert.deepEqual(rowsFor("bilingual", { source: "Hi", translation: "  " }).map(r => r.kind), ["source"]);
});

test("color error codes map to i18n keys", () => {
  assert.deepEqual(colorError("invalid"), { key: "sub.color.err.invalid" });
  assert.deepEqual(colorError("low_contrast:2.3"), { key: "sub.color.err.contrast", values: { ratio: "2.3" } });
});

test("background alpha follows opacity and clamps to 0..80", () => {
  assert.equal(backgroundCss(0), "rgba(16, 20, 24, 1.00)");
  assert.equal(backgroundCss(80), "rgba(16, 20, 24, 0.20)");
  assert.equal(backgroundCss(150), "rgba(16, 20, 24, 0.20)");
  assert.equal(backgroundCss(-5), "rgba(16, 20, 24, 1.00)");
});

test("modes list matches the three backend values", () => {
  assert.deepEqual(MODES.map(m => m.mode), ["bilingual", "translation_only", "source_only"]);
});

const fl = (id: number, s: string, t: string | null = null) => ({ captionLineId: id, sourceText: s, translatedText: t });

test("upsertLine replaces the same id in place and appends new ids", () => {
  const a = [fl(1, "he"), fl(2, "x")];
  const b = upsertLine(a, fl(1, "hello", "ola"));
  assert.deepEqual(b.map(l => l.sourceText), ["hello", "x"]);
  assert.equal(a[0].sourceText, "he", "input untouched");
  assert.deepEqual(upsertLine(b, fl(3, "y")).map(l => l.captionLineId), [1, 2, 3]);
});

test("upsertLine keeps only the latest MAX_LINES", () => {
  let l: ReturnType<typeof fl>[] = [];
  for (let i = 0; i < MAX_LINES + 5; i++) l = upsertLine(l, fl(i, "t"));
  assert.equal(l.length, MAX_LINES);
  assert.equal(l[0].captionLineId, 5);
});

test("mergeSnapshot: snapshot replays old lines, live event wins for the same id, newer ids stay at the end", () => {
  const live = [fl(2, "two-final", "dois"), fl(4, "four")];
  const snap = [fl(1, "one"), fl(2, "two-interim"), fl(3, "three")];
  const m = mergeSnapshot(live, snap);
  assert.deepEqual(m.map(l => l.captionLineId), [1, 2, 3, 4]);
  assert.equal(m[1].sourceText, "two-final");
});

test("mergeSnapshot with empty sides", () => {
  assert.deepEqual(mergeSnapshot([], [fl(1, "a")]).length, 1);
  assert.deepEqual(mergeSnapshot([fl(1, "a")], []).length, 1);
  assert.deepEqual(mergeSnapshot([], []), []);
});

test("rowsForLines flattens every line by mode and drops empty parts", () => {
  const ls = [fl(1, "Hello", "Ola"), fl(2, "Bye", null)];
  assert.deepEqual(rowsForLines("bilingual", ls).map(r => r.text), ["Hello", "Ola", "Bye"]);
  assert.deepEqual(rowsForLines("translation_only", ls).map(r => r.text), ["Ola"]);
  assert.deepEqual(rowsForLines("source_only", []), []);
});

test("toolbarVisible is true when any of hover, rust hover, focus or open menu holds", () => {
  assert.equal(toolbarVisible(false, false, false, false), false);
  for (const i of [0, 1, 2, 3]) { const a = [false, false, false, false]; a[i] = true; assert.equal(toolbarVisible(a[0], a[1], a[2], a[3]), true); }
});
