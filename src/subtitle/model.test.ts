import test from "node:test";
import assert from "node:assert/strict";
import { backgroundCss, colorError, MODES, rowsFor } from "./model.ts";

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
