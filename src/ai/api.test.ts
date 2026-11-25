import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { CLI_PROVIDERS, PROVIDERS, cliKey, errKey, isCli, providerLabelKey } from "./api.ts";

const load = (l: string) => JSON.parse(readFileSync(new URL(`../locales/${l}.json`, import.meta.url), "utf8")) as Record<string, string>;

test("Codex va Claude deu la nha cung cap CLI va deu hien trong danh sach", () => {
  assert.deepEqual(CLI_PROVIDERS, ["claude_cli", "codex_cli"]);
  for (const p of CLI_PROVIDERS) { assert.ok(PROVIDERS.includes(p)); assert.ok(isCli(p)); }
  assert.ok(!isCli("openai") && !isCli(null));
  assert.equal(providerLabelKey("codex_cli"), "ai.provider.codexcli");
});

test("loi CLI cua Codex dung chuoi rieng, cac nha cung cap khac giu nguyen", () => {
  assert.equal(errKey("cli_not_installed", "codex_cli"), "ai.err.cli.notinstalled.codex");
  assert.equal(errKey("cli_not_installed", "claude_cli"), "ai.err.cli.notinstalled");
  assert.equal(errKey("provider_rate_limited", "codex_cli"), "ai.err.rate");
  assert.equal(errKey("khong_biet", "codex_cli"), "ai.err.generic");
  assert.equal(cliKey("set.cli.title", "codex_cli"), "set.cli.title.codex");
  assert.equal(cliKey("set.cli.title", "claude_cli"), "set.cli.title");
});

test("moi chuoi CLI cua Codex co du trong en va pt-PT", () => {
  const codes = ["cli_not_installed", "cli_not_logged_in", "cli_login_expired", "cli_quota_exhausted", "cli_timeout", "cli_unknown"];
  const keys = [
    ...codes.map(c => errKey(c, "codex_cli")),
    ...["set.cli.title", "set.cli.path", "set.cli.path.hint", "set.cli.need", "ask.cli.risk", "ai.consent.cli.title", "ai.consent.cli.body"].map(k => cliKey(k, "codex_cli")),
    providerLabelKey("codex_cli"),
  ];
  for (const l of ["en", "pt-PT"]) { const m = load(l); for (const k of keys) assert.ok(m[k], `${l}: ${k}`); }
});
