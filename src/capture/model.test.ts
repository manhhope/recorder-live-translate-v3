import test from "node:test";
import assert from "node:assert/strict";
import { dragBox, errorKey, toGlobal } from "./model.ts";

test("global point adds the window origin, including negative origins of a left monitor", () => {
  assert.deepEqual(toGlobal({ x: 1512, y: 0 }, { x: 10, y: 20 }), { x: 1522, y: 20 });
  assert.deepEqual(toGlobal({ x: -1920, y: 100 }, { x: 5, y: 5 }), { x: -1915, y: 105 });
});

test("drag box is positive whatever the drag direction", () => {
  assert.deepEqual(dragBox({ x: 100, y: 80 }, { x: 40, y: 10 }), { x: 40, y: 10, w: 60, h: 70 });
  assert.deepEqual(dragBox({ x: 5, y: 5 }, { x: 5, y: 5 }), { x: 5, y: 5, w: 0, h: 0 });
});

test("known backend codes map to their key, anything else is unknown", () => {
  for (const c of ["no_permission", "no_monitor", "empty", "too_small", "all_black", "limit_reached"]) {
    assert.equal(errorKey(c), `cap.err.${c.replace(/_/g, "")}`);
  }
  assert.equal(errorKey("capture_failed:boom"), "cap.err.unknown");
  assert.equal(errorKey("save_failed:Db(x)"), "cap.err.unknown");
});
