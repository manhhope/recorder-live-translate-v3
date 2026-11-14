import test from "node:test";
import assert from "node:assert/strict";
import { afterFinal, fromPartial, type PartialDto } from "./live.ts";

const p = (sourceText: string, translatedText: string | null = null, source: "mic" | "system" = "system"): PartialDto =>
  ({ source, offsetMs: 0, sourceLang: "en", sourceText, translatedText });

test("each partial replaces the previous one instead of appending", () => {
  let live = fromPartial(p("Hel"));
  assert.equal(live?.source, "Hel");
  live = fromPartial(p("Hello wor", "Ol"));
  assert.deepEqual([live?.source, live?.translation], ["Hello wor", "Ol"]);
  live = fromPartial(p("Hello world", "Ola mun"));
  assert.deepEqual([live?.source, live?.translation], ["Hello world", "Ola mun"]);
});

test("null or blank partial clears", () => {
  assert.equal(fromPartial(null), null);
  assert.equal(fromPartial(p("   ")), null);
});

test("the origin of the live row follows the backend label (mixed mode: mic or system)", () => {
  assert.equal(fromPartial(p("a", null, "mic"))?.origin, "mic");
  assert.equal(fromPartial(p("a", null, "system"))?.origin, "system");
});

test("the final line of the same sentence removes the live row (no duplicate)", () => {
  const live = fromPartial(p("Hello world.", "Ola mu"));
  assert.equal(afterFinal(live, "Hello world.", "Ola mundo."), null);
});

test("a final line for an earlier sentence keeps only the sentence still being said", () => {
  const live = fromPartial(p("One. Two", "Um. Dois"));
  const rest = afterFinal(live, "One.", "Um.");
  assert.deepEqual([rest?.source, rest?.translation], ["Two", "Dois"]);
});

test("a final line that is not at the start of the live text leaves it alone", () => {
  const live = fromPartial(p("Second", "Segunda"));
  assert.equal(afterFinal(live, "First sentence.", "Primeira frase."), live);
});

test("it never cuts in the middle of a word", () => {
  const live = fromPartial(p("Oneness is", "x"));
  assert.equal(afterFinal(live, "One", null), live);
});
