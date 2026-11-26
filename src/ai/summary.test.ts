import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { cutNotices, hasResult, progressKey, translateTarget, type SummaryPlan } from "./summary.ts";

const load = (l: string) => JSON.parse(readFileSync(new URL(`../locales/${l}.json`, import.meta.url), "utf8")) as Record<string, string>;
const plan = (o: Partial<SummaryPlan> = {}): SummaryPlan => ({ total_chars: 1000, sent_chars: 1000, input_cut: false, dropped_head: false, cap_chars: 200_000, chunks: 1, calls: 1, translate: false, ...o });

test("dich sang ngon ngu con lai cua cap en va pt-PT", () => {
  assert.equal(translateTarget("en"), "pt-PT");
  assert.equal(translateTarget("pt-PT"), "en");
});

test("khong cat thi khong co thong bao; cat o tran 200.000 thi bao ro", () => {
  assert.deepEqual(cutNotices(plan()), []);
  const n = cutNotices(plan({ total_chars: 250_000, sent_chars: 200_000, input_cut: true }));
  assert.deepEqual(n, [{ key: "sum.cut.input", values: { total: 250_000, cap: 200_000 } }]);
  assert.equal(cutNotices(plan({ dropped_head: true, chunks: 20 }))[0].key, "sum.cut.chunks");
});

test("co ket qua chi khi co phan tom tat", () => {
  assert.equal(hasResult(null), false);
  assert.equal(hasResult({ status: "cancelled", error_code: "provider_cancelled", failed_at: 2, parts: null, translated: null, plan: plan() }), false);
  assert.equal(hasResult({ status: "failed", error_code: "x", failed_at: null, parts: { summary: "a", key_points: [], action_items: [] }, translated: null, plan: plan() }), true);
});

test("moi chuoi cua tom tat co trong en va pt-PT", () => {
  const keys = [
    ...(["map", "reduce", "translate"] as const).map(progressKey),
    "ses.summarize", "sum.title", "sum.estimate", "sum.cut.input", "sum.cut.chunks", "sum.translate.pt", "sum.translate.en", "sum.run",
    "sum.back", "sum.h.summary", "sum.h.points", "sum.h.actions", "sum.none", "sum.cancelled", "sum.partial", "sum.translated.label",
    "sum.translated.title.pt", "sum.translated.title.en", "sum.empty", "sum.planerr",
  ];
  for (const l of ["en", "pt-PT"]) { const m = load(l); for (const k of keys) assert.ok(m[k], `${l}: ${k}`); }
});
