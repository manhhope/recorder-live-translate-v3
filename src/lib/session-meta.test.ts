import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { imagesLabel, sessionSubline, type Translate } from "./session-meta.ts";

const load = (f: string): Record<string, string> => JSON.parse(fs.readFileSync(path.join(import.meta.dirname, "..", "locales", f), "utf8"));
const tr = (dict: Record<string, string>): Translate => (k, v) => dict[k].replace("{n}", String(v?.n));
const en = tr(load("en.json"));
const pt = tr(load("pt-PT.json"));
const BASE = "11/15 · 00:04:40 · vi · 35 lines";

test("0 anh: khong them gi vao dong phu", () => {
  assert.equal(imagesLabel(0, en), "");
  assert.equal(sessionSubline(BASE, 0, en), BASE);
  assert.equal(sessionSubline(BASE, -1, en), BASE);
});

test("1 anh: so it", () => {
  assert.equal(sessionSubline(BASE, 1, en), `${BASE} · 1 image`);
  assert.equal(imagesLabel(1, pt), "1 imagem");
});

test("nhieu anh: so nhieu", () => {
  assert.equal(sessionSubline(BASE, 3, en), `${BASE} · 3 images`);
  assert.equal(imagesLabel(12, pt), "12 imagens");
});

test("linesLabel: 1 dong dung khoa so it, 0 va nhieu dong dung khoa so nhieu (en va pt-PT)", async () => {
  const { linesLabel } = await import("./session-meta.ts");
  assert.equal(linesLabel(1, en), "1 line");
  assert.equal(linesLabel(0, en), "0 lines");
  assert.equal(linesLabel(3, en), "3 lines");
  assert.equal(linesLabel(1, pt), "1 linha");
  assert.equal(linesLabel(2, pt), "2 linhas");
});
