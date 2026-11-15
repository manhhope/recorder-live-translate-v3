import { test } from "node:test";
import assert from "node:assert/strict";
import { withTimeout } from "./ipc.ts";

test("withTimeout tra gia tri khi lenh xong kip", async () => {
  assert.equal(await withTimeout(Promise.resolve(7), 50), 7);
});
test("withTimeout giu nguyen ma loi cua lenh", async () => {
  await assert.rejects(withTimeout(Promise.reject("key_missing"), 50), (e) => e === "key_missing");
});
test("withTimeout bao ipc_timeout khi lenh khong bao gio tra ve", async () => {
  await assert.rejects(withTimeout(new Promise(() => {}), 20), (e) => e === "ipc_timeout");
});
