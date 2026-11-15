import { test } from "node:test";
import assert from "node:assert/strict";
import { applyRefresh, applyFailure, updatedTime, showsUpdatedAt, hasSoniox } from "./usage-swr.ts";
import type { UsageSourceOut } from "./api.ts";

const base = { basis: "utc", reason: null, weekStart: "2026-11-09", weekEnd: "2026-11-15", days: [], totalCostUsdMicros: 0, totalAudioMs: 0, apiCostUsdMicros: 0,
  playgroundCostUsdMicros: 0, totalRequests: 0, models: [], estimated: false, cached: false, fetchedAt: 0, sessionsCount: 0, failedStartCount: 0 } as const;
const soniox = (cost: number, at: number, cached = false): UsageSourceOut => ({ ...base, source: "soniox", totalCostUsdMicros: cost, fetchedAt: at, cached });
const local = (reason: string): UsageSourceOut => ({ ...base, source: "local", basis: "local", reason, estimated: true, totalCostUsdMicros: 7 });

test("tai xong bang Soniox: thay so moi, xoa canh bao, gio doi", () => {
  const v = applyRefresh(soniox(100, 1000, true), soniox(250, 2000));
  assert.equal(v.data?.totalCostUsdMicros, 250);
  assert.equal(v.data?.fetchedAt, 2000);
  assert.equal(v.warning, null);
});

for (const reason of ["network", "unauthorized", "forbidden", "rate_limited", "server", "bad_response", "no_key"]) {
  test(`loi '${reason}' khi dang co so Soniox cu: giu so cu va gio cu, co canh bao`, () => {
    const old = soniox(100, 1000, true);
    const v = applyRefresh(old, local(reason));
    assert.equal(v.data, old);
    assert.deepEqual(v.warning, { kind: "reason", code: reason });
  });
}

test("chua co so Soniox (lan dau): loi thi hien so cuc bo kem canh bao, khong de trong", () => {
  const first = local("no_cache");
  const res = local("network");
  const v = applyRefresh(first, res);
  assert.equal(v.data, res);
  assert.equal(v.warning?.code, "network");
});

test("lenh nem loi: giu nguyen so dang co, canh bao la ma loi", () => {
  const old = soniox(100, 1000, true);
  assert.deepEqual(applyFailure(old, "store_error"), { data: old, warning: { kind: "err", code: "store_error" } });
  assert.deepEqual(applyFailure(null, "store_error"), { data: null, warning: { kind: "err", code: "store_error" } });
});

test("chi ban Soniox co gio 'Cap nhat luc'; so cuc bo thi khong", () => {
  assert.equal(hasSoniox(soniox(1, 5)), true);
  assert.equal(showsUpdatedAt(soniox(1, 5)), true);
  assert.equal(showsUpdatedAt(soniox(1, 0)), false);
  assert.equal(showsUpdatedAt(local("no_cache")), false);
  assert.equal(showsUpdatedAt(null), false);
});

test("updatedTime: HH:mm 24 gio", () => {
  const t = updatedTime(Date.UTC(2026, 10, 15, 9, 5) / 1000, "en");
  assert.match(t, /^\d{2}:\d{2}$/);
});
