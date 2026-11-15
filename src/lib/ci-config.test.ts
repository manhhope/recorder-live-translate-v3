// Dam bao CI luon chay kiem tra tinh capability (check:caps) va cac lenh bat buoc.
import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const ci = readFileSync(new URL("../../.github/workflows/ci.yml", import.meta.url), "utf8");

for (const cmd of ["npm run check:caps", "npm run test:i18n", "npm run scan:i18n", "npm run build", "npx playwright test", "cargo test --workspace", "-D warnings"]) {
  test(`ci.yml chay: ${cmd}`, () => {
    assert.ok(ci.includes(cmd), `ci.yml thieu buoc: ${cmd}`);
  });
}
