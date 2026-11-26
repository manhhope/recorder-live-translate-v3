import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { CLI_PROVIDERS, PROVIDERS, canLogin, cliKey, cliStatusKey, enabledProviders, errKey, isCli, loginActive, loginErrKey, providerLabelKey, signInPrimary } from "./api.ts";

const load = (l: string) => JSON.parse(readFileSync(new URL(`../locales/${l}.json`, import.meta.url), "utf8")) as Record<string, string>;

test("Codex va Claude deu la nha cung cap CLI (Codex chi hien khi backend bat co)", () => {
  assert.deepEqual(CLI_PROVIDERS, ["claude_cli", "codex_cli", "antigravity_cli"]);
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

test("Codex BAT mac dinh (3.0.6): giao dien liet ke dung nha cung cap backend tra ve", () => {
  const all = { providers: PROVIDERS.map(provider => ({ provider })) };
  assert.deepEqual(enabledProviders(CLI_PROVIDERS, all), ["claude_cli", "codex_cli", "antigravity_cli"]);
  // Chua co du lieu backend: hien du (ban mac dinh bat Codex).
  assert.deepEqual(enabledProviders(CLI_PROVIDERS, null), ["claude_cli", "codex_cli", "antigravity_cli"]);
  // Build tat co (`--no-default-features`): backend khong tra Codex thi khong hien.
  const off = { providers: (["gemini", "openai", "ollama", "compat", "claude_cli", "antigravity_cli"] as const).map(provider => ({ provider })) };
  assert.deepEqual(enabledProviders(CLI_PROVIDERS, off), ["claude_cli", "antigravity_cli"]);
});

test("tu do CLI: Sign in la nut chinh khi da cai ma chua dang nhap; agy khong co nut", () => {
  const info = (status: "ready" | "not_installed" | "not_logged_in" | "unknown") => ({ path: "/x/claude", version: "1", status });
  assert.ok(signInPrimary("claude_cli", info("not_logged_in")));
  assert.ok(signInPrimary("codex_cli", info("unknown")));
  assert.ok(!signInPrimary("claude_cli", info("ready")));
  assert.ok(!signInPrimary("claude_cli", info("not_installed")));
  assert.ok(!signInPrimary("claude_cli", null));
  assert.ok(!signInPrimary("antigravity_cli", info("not_logged_in")));
  assert.equal(cliStatusKey("not_logged_in"), "set.cli.st.notloggedin");
});

test("moi chuoi moi cua man tu do va dang nhap co du trong en va pt-PT, khong co tieng Viet", () => {
  const keys = [
    ...["checking", "ready", "not_installed", "not_logged_in", "unknown"].map(k => cliStatusKey(k as "ready")),
    "set.cli.found", "set.cli.recheck", "set.cli.adv", "set.login.copycode", "set.login.opendevice", "set.login.verify",
    ...CLI_PROVIDERS.flatMap(p => ["set.cli.install", "set.cli.notloggedin.help"].map(k => cliKey(k, p))),
  ];
  const vi = /[\u0103\u0111\u01a1\u01b0\u1ea0-\u1ef9]/i;
  for (const l of ["en", "pt-PT"]) {
    const m = load(l);
    for (const k of keys) { assert.ok(m[k], `${l}: ${k}`); assert.ok(!vi.test(m[k]), `${l}: ${k} co chu tieng Viet`); }
  }
});

test("Antigravity (agy): chuoi rieng .agy, co du trong en va pt-PT, khong co nut dang nhap", () => {
  assert.equal(cliKey("set.cli.title", "antigravity_cli"), "set.cli.title.agy");
  assert.equal(errKey("cli_timeout", "antigravity_cli"), "ai.err.cli.timeout.agy");
  assert.equal(providerLabelKey("antigravity_cli"), "ai.provider.antigravitycli");
  assert.ok(isCli("antigravity_cli"));
  assert.ok(canLogin("claude_cli") && canLogin("codex_cli") && !canLogin("antigravity_cli") && !canLogin("openai"));
  const codes = ["cli_not_installed", "cli_not_logged_in", "cli_login_expired", "cli_quota_exhausted", "cli_timeout", "cli_unknown"];
  const keys = [
    ...codes.map(c => errKey(c, "antigravity_cli")),
    ...["set.cli.title", "set.cli.path", "set.cli.path.hint", "set.cli.need", "ask.cli.risk", "ai.consent.cli.title", "ai.consent.cli.body"].map(k => cliKey(k, "antigravity_cli")),
    providerLabelKey("antigravity_cli"), "set.cli.signin.agy",
  ];
  for (const l of ["en", "pt-PT"]) { const m = load(l); for (const k of keys) assert.ok(m[k], `${l}: ${k}`); }
});

test("dang nhap CLI: moi ma loi co chuoi trong en va pt-PT, pha dang chay duoc nhan dung", () => {
  const codes = ["cli_not_installed", "login_no_link", "login_expired", "login_timeout", "login_not_saved", "login_already_running",
    "login_invalid_code", "login_unsupported", "login_spawn_failed", "login_unknown", "khong_biet", null];
  for (const l of ["en", "pt-PT"]) { const m = load(l); for (const c of codes) assert.ok(m[loginErrKey(c, "claude_cli")], `${l}: ${c}`); }
  assert.equal(loginErrKey("cli_not_installed", "codex_cli"), "ai.err.cli.notinstalled.codex");
  assert.equal(loginErrKey("khong_biet"), "set.login.err.unknown");
  const snap = (phase: any) => ({ phase, url: null, device_code: null, error: null, needs_code: false, remaining_secs: 1 });
  for (const p of ["starting", "waiting_user", "code_sent", "verifying"]) assert.ok(loginActive(snap(p)));
  for (const p of ["done", "failed", "cancelled"]) assert.ok(!loginActive(snap(p)));
  assert.ok(!loginActive(null));
  for (const l of ["en", "pt-PT"]) { const m = load(l); for (const k of Object.keys(m).filter(k => k.startsWith("set.login."))) assert.ok(m[k]); }
});
