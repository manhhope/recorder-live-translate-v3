import test from "node:test";
import assert from "node:assert/strict";
import { charCount, errorKey, inputState, langLabelKey, MAX_CHARS, parseExcluded } from "./model.ts";

test("limit is counted in characters, not bytes or UTF-16 units", () => {
  assert.equal(inputState("a".repeat(MAX_CHARS)), "ok");
  assert.equal(inputState("a".repeat(MAX_CHARS + 1)), "toolong");
  assert.equal(charCount("😀".repeat(5000)), 5000);
  assert.equal(inputState("😀".repeat(5000)), "ok");
  assert.equal(inputState("ã".repeat(5001)), "toolong");
});

test("blank input is empty", () => {
  assert.equal(inputState(""), "empty");
  assert.equal(inputState("  \n\t"), "empty");
});

test("error codes map to quick.err keys, unknown goes to provider", () => {
  assert.equal(errorKey("notverified"), "quick.err.notverified");
  assert.equal(errorKey("quota"), "quick.err.quota");
  assert.equal(errorKey("???"), "quick.err.provider");
});

test("language label key and excluded-app parsing", () => {
  assert.equal(langLabelKey("pt-PT"), "lang.pt");
  assert.equal(langLabelKey("en"), "lang.en");
  assert.deepEqual(parseExcluded("com.a\n\n com.b \r\ncom.a"), ["com.a", "com.b"]);
});
