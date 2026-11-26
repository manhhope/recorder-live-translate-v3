<script lang="ts">
  // Tab "Tro ly AI" trong Cai dat (phuong an A, US-47). Khoa API chi NHAP VAO: o nhap la mat khau, bi xoa ngay sau khi
  // luu, va giao dien chi biet "da co khoa / chua co" (`has_key`). Khong co duong nao doc khoa tu Keychain ve day.
  import { onMount, onDestroy } from "svelte";
  import { _, locale } from "svelte-i18n";
  import Icon from "../lib/Icon.svelte";
  import LoadError from "../lib/LoadError.svelte";
  import { withTimeout } from "../lib/ipc.ts";
  import * as api from "./api.ts";
  import type { AiSettings, ChipKind, Provider } from "./api.ts";

  const CHIPS: { kind: ChipKind; label: string; def: string }[] = [
    { kind: "summary", label: "ask.chip.summary", def: "set.ai.p1" },
    { kind: "explain", label: "ask.chip.explain", def: "set.ai.p2" },
    { kind: "action_items", label: "ask.chip.todo", def: "set.ai.p3" },
  ];
  const MAX_PROMPT = 2000;
  const KEYED: Provider[] = ["gemini", "openai"];

  let s = $state<AiSettings | null>(null);
  let defaultProvider = $state<Provider | null>(null);
  let models = $state<Record<string, string>>({});
  let ollamaUrl = $state("");
  let compatUrl = $state("");
  let cliPaths = $state<Record<string, string>>({ claude_cli: "", codex_cli: "", antigravity_cli: "" });
  let compatModels = $state<string[]>([]);
  let compatList = $state<"idle" | "running" | "ok" | "manual">("idle");
  let chars = $state(60000);
  let turns = $state(10);
  let prompts = $state<Record<string, string>>({});
  let keyInput = $state<Record<string, string>>({ gemini: "", openai: "", compat: "" });
  let show = $state<Record<string, boolean>>({});
  let test = $state<Record<string, "idle" | "running" | "ok" | string>>({});
  let saved = $state(false);
  let error = $state<string | null>(null);
  let loading = $state(true);
  let loadErr = $state<string | null>(null);

  // Nha cung cap hien/chon duoc do backend quyet; Codex TAT mac dinh nen khong nam trong cac danh sach nay (RV 🔴-2).
  const provs = $derived(api.enabledProviders(api.PROVIDERS, s));
  const cliProvs = $derived(api.enabledProviders(api.CLI_PROVIDERS, s));
  const hasKey = (p: Provider) => s?.providers.find(x => x.provider === p)?.has_key ?? false;
  // Endpoint tuong thich khong co model mac dinh gia (AC-57.5): ten model do nguoi dung nhap hoac chon.
  const defModel = (p: Provider) => ({ gemini: "gemini-2.5-flash", openai: "gpt-4.1-mini", ollama: "qwen2.5:1.5b", compat: "", claude_cli: "", codex_cli: "", antigravity_cli: "" })[p];
  // Duong dan CLI nhu phia Rust (`rlt_ai::cli::validate_path_override`): rong la tu do; con lai la duong dan tuyet doi, khong xuong dong.
  const pathOk = (raw: string) => { const t = raw.trim(); return !t || (t.length <= 500 && !/[\u0000-\u001f\u007f]/.test(t) && (t.startsWith("/") || /^[A-Za-z]:[\\/]/.test(t))); };
  const cliPathOk = $derived(cliProvs.every(p => pathOk(cliPaths[p] ?? "")));
  const compatUrlOk = $derived(api.validCompatUrl(compatUrl));
  const compatModelOk = $derived(!!models.compat?.trim());
  const compatHint = $derived(
    compatList === "ok" ? $_("set.compat.models.count", { values: { count: compatModels.length } })
    : compatList === "manual" ? $_("set.compat.models.manual")
    : !compatModelOk ? $_("set.compat.model.required")
    : $_("set.compat.model.hint"));
  const defPrompt = (k: ChipKind) => $_(CHIPS.find(c => c.kind === k)!.def);

  function load(v: AiSettings) {
    s = v; defaultProvider = v.default_provider; chars = v.context_max_chars; turns = v.context_max_turns;
    for (const p of v.providers) models[p.provider] = p.model_name;
    ollamaUrl = v.providers.find(p => p.provider === "ollama")?.base_url ?? "";
    compatUrl = v.providers.find(p => p.provider === "compat")?.base_url ?? api.COMPAT_DEFAULT_URL;
    for (const p of api.CLI_PROVIDERS) cliPaths[p] = v.providers.find(x => x.provider === p)?.base_url ?? "";
    for (const c of v.chip_prompts) prompts[c.chip] = c.prompt ?? defPrompt(c.chip);
  }
  async function refresh() { load(await withTimeout(api.getSettings())); }
  async function firstLoad() {
    loading = true; loadErr = null;
    try { await refresh(); } catch (e) { loadErr = e === "ipc_timeout" ? "set.load.timeout" : "set.load.err"; }
    loading = false;
  }

  async function saveKey(p: Provider) {
    const v = keyInput[p].trim();
    if (!v) return;
    error = null;
    try { await api.setApiKey(p, v); keyInput[p] = ""; show[p] = false; await refresh(); }
    catch (e) { error = String(e); }
  }
  async function removeKey(p: Provider) {
    try { await api.clearApiKey(p); await refresh(); } catch (e) { error = String(e); }
  }
  async function runTest(p: Provider) {
    test[p] = "running";
    try { await persist(); await api.testConnection(p); test[p] = "ok"; }
    catch (e) { test[p] = String(e); }
  }
  // Dang nhap CLI trong app. Backend giu link/ma; day chi hien, mo, chep va nhap ma dan. Khong ghi nhat ky, khong luu.
  let login = $state<api.LoginSnapshot | null>(null);
  let loginFor = $state<Provider | null>(null);
  let loginErr = $state<string | null>(null);
  let pasted = $state("");
  let copied = $state(false);
  let timer: ReturnType<typeof setInterval> | null = null;
  const busy = $derived(api.loginActive(login));
  function stopPoll() { if (timer) { clearInterval(timer); timer = null; } }
  async function poll() {
    try {
      login = await api.loginPoll();
      if (!api.loginActive(login)) {
        stopPoll();
        if (login?.phase === "done") { test[loginFor ?? "claude_cli"] = "idle"; if (loginFor) void recheck(loginFor); }
      }
    } catch { stopPoll(); }
  }
  async function startLogin(p: Provider) {
    loginErr = null; pasted = ""; copied = false; loginFor = p; login = null;
    try { await persist(); await api.loginStart(p); } catch (e) { loginErr = String(e); return; }
    await poll(); stopPoll(); timer = setInterval(poll, 1000);
  }
  async function sendCode() {
    const v = pasted.trim(); if (!v) return;
    try { await api.loginCode(v); pasted = ""; loginErr = null; await poll(); } catch (e) { loginErr = String(e); }
  }
  async function cancelLogin() { try { await api.loginCancel(); } catch { /* da dung */ } await poll(); }
  async function openLogin() { try { await api.loginOpen(); } catch (e) { loginErr = String(e); } }
  async function copyLink() {
    if (!login?.url) return;
    try { await navigator.clipboard.writeText(login.url); copied = true; setTimeout(() => (copied = false), 2000); } catch { loginErr = "login_copy_failed"; }
  }
  async function copyDeviceCode() {
    if (!login?.device_code) return;
    try { await navigator.clipboard.writeText(login.device_code); codeCopied = true; setTimeout(() => (codeCopied = false), 2000); } catch { loginErr = "login_copy_failed"; }
  }
  let codeCopied = $state(false);
  const remainingText = (n: number) => $_("set.login.remaining", { values: { time: `${Math.floor(n / 60)}:${String(n % 60).padStart(2, "0")}` } });
  // Tu do CLI khi mo tab: duong dan, phien ban, trang thai dang nhap (theo ma thoat cua CLI, KHONG doc file token).
  let cli = $state<Partial<Record<Provider, api.CliInfo | "checking">>>({});
  async function recheck(p: Provider) {
    cli[p] = "checking";
    try { cli[p] = await api.cliStatus(p); } catch { cli[p] = { path: null, version: null, status: "unknown" }; }
    if (info(p)?.status === "not_installed") { try { instInfo[p] = await api.installInfo(p); } catch { /* khong co lenh cai de hien */ } }
  }
  // Cai CLI trong app. Lenh/trang cai la hang so o backend; day chi hoi dong y, hien tien trinh va goi lai kiem tra.
  let instInfo = $state<Partial<Record<Provider, api.InstallInfo>>>({});
  let confirmFor = $state<Provider | null>(null);
  let inst = $state<api.InstallSnapshot | null>(null);
  let instFor = $state<Provider | null>(null);
  let instErr = $state<string | null>(null);
  let instNote = $state<{ p: Provider; key: string; command?: string; url?: string } | null>(null);
  let instTimer: ReturnType<typeof setInterval> | null = null;
  const instBusy = $derived(api.installActive(inst));
  function stopInstPoll() { if (instTimer) { clearInterval(instTimer); instTimer = null; } }
  async function pollInstall() {
    try {
      inst = await api.installPoll();
      if (!api.installActive(inst)) {
        stopInstPoll();
        if (inst?.phase === "done" && instFor) {
          const p = instFor; await recheck(p);
          if (info(p)?.status === "not_installed") instNote = { p, key: "set.install.notfound" };
        }
      }
    } catch { stopInstPoll(); }
  }
  async function runInstall(p: Provider) {
    confirmFor = null; instErr = null; instNote = null; instFor = p; inst = null;
    try { await api.installStart(p); } catch (e) { instErr = String(e); return; }
    await pollInstall(); stopInstPoll(); if (api.installActive(inst)) instTimer = setInterval(pollInstall, 1000);
  }
  const noteText = (n: { key: string; url?: string }) => $_(n.key, { values: { url: n.url ?? "" } });
  async function cancelInstall() { try { await api.installCancel(); } catch { /* da dung */ } await pollInstall(); }
  async function copyAndOpen(p: Provider) {
    const cmd = instInfo[p]?.command; instNote = null;
    if (!cmd) return;
    let ok = true;
    try { await navigator.clipboard.writeText(cmd); } catch { ok = false; }
    try { await api.installOpenPage(p); instNote = { p, key: ok ? "set.install.copied" : "set.install.copyfail", command: cmd }; }
    catch { instNote = { p, key: "set.install.openfail", command: cmd, url: instInfo[p]?.page }; }
  }
  async function recheckSaved(p: Provider) { try { await persist(); } catch { /* van kiem theo cai dat cu */ } await recheck(p); }
  const info = (p: Provider): api.CliInfo | null => { const v = cli[p]; return v && v !== "checking" ? v : null; };
  const foundText = (p: Provider) => $_("set.cli.found", { values: { path: info(p)?.path ?? "", version: info(p)?.version ?? "" } });
  const stKey = (p: Provider) => { const v = cli[p]; return api.cliStatusKey(!v || v === "checking" ? "checking" : v.status); };
  // M1: roi tab giua chung dang nhap thi huy, de khong con tien trinh CLI chay ngam.
  onDestroy(() => { stopPoll(); stopInstPoll(); if (instBusy) void api.installCancel().catch(() => {}); if (busy) void api.loginCancel().catch(() => {}); });
  async function persist() {
    const input: api.AiSettingsInput = {
      default_provider: defaultProvider,
      context_max_chars: Number(chars), context_max_turns: Number(turns),
      providers: provs.map(p => ({
        provider: p,
        model_name: models[p]?.trim() && models[p].trim() !== defModel(p) ? models[p].trim() : null,
        base_url: p === "ollama" ? (ollamaUrl.trim() || null) : p === "compat" ? (compatUrl.trim() || null) : api.isCli(p) ? (cliPaths[p]?.trim() || null) : null,
      })),
      chip_prompts: CHIPS.map(c => ({ chip: c.kind, prompt: prompts[c.kind]?.trim() && prompts[c.kind].trim() !== defPrompt(c.kind) ? prompts[c.kind].trim() : null })),
    };
    await withTimeout(api.saveSettings(input));
  }
  async function save() {
    error = null; saved = false;
    try { await persist(); await refresh(); saved = true; } catch (e) { error = String(e); }
  }
  function resetTab() {
    defaultProvider = null; chars = 60000; turns = 10;
    for (const p of provs) models[p] = defModel(p);
    ollamaUrl = "http://localhost:11434";
    compatUrl = api.COMPAT_DEFAULT_URL; compatModels = []; compatList = "idle";
    for (const p of cliProvs) cliPaths[p] = "";
    for (const c of CHIPS) prompts[c.kind] = defPrompt(c.kind);
  }
  const testText = (p: Provider) => {
    const t = test[p];
    if (!t || t === "idle") return "";
    if (t === "running") return $_("ai.test.running");
    if (t === "ok") return $_("ai.test.ok");
    return $_(api.errKey(t, p));
  };
  // Lay danh sach model tu `GET {base}/models` (AC-57.4). Loi thi KHONG bao do: chi chuyen sang nhap tay.
  async function loadModels() {
    compatList = "running"; compatModels = [];
    try { await persist(); compatModels = await api.listModels("compat"); compatList = compatModels.length ? "ok" : "manual"; }
    catch { compatList = "manual"; }
  }
  // Doi ngon ngu thi prompt mac dinh (chua sua) doi theo.
  $effect(() => { void $locale; if (s) for (const c of s.chip_prompts) if (c.prompt === null) prompts[c.chip] = defPrompt(c.chip); });

  onMount(() => { void firstLoad().then(() => { for (const p of cliProvs) void recheck(p); }); });
</script>

{#if s}
<div class="card">
  <h4>{$_("set.ai.default")}</h4>
  <div class="rw">
    <div><b>{$_("set.ai.default.hint")}</b><small>{$_("set.ai.default.note")}</small></div>
    <div class="rr">
      <select class="sel" aria-label={$_("ask.model.pick")} bind:value={defaultProvider}>
        <option value={null}>{$_("ask.model.none")}</option>
        {#each provs as p (p)}<option value={p}>{$_(api.providerLabelKey(p))}</option>{/each}
      </select>
    </div>
  </div>

  <div class="two" style="margin-top:6px">
    {#each KEYED as p (p)}
      <div class="card">
        <h4>{$_(api.providerLabelKey(p))}</h4>
        <div class="rw"><b>{$_("set.apikey")}</b>
          <div class="rr">
            <label class="fld {hasKey(p) ? '' : 'err'}"><Icon name="key" />
              <input type={show[p] ? "text" : "password"} autocomplete="off" spellcheck="false" aria-label={$_("set.apikey")}
                placeholder={hasKey(p) ? $_("ai.key.saved") : $_("set.key.none")} bind:value={keyInput[p]} />
              <button type="button" class="btn icon sm ghost" aria-label={$_("set.key.show")} onclick={() => (show[p] = !show[p])}><Icon name="eye" /></button>
            </label>
            {#if hasKey(p)}<span class="bd ok">{$_("ai.key.saved")}</span>{/if}
          </div>
        </div>
        <div class="rr" style="padding:6px 0">
          <button type="button" class="btn sm primary" disabled={!keyInput[p].trim()} onclick={() => saveKey(p)}>{$_("ai.key.save")}</button>
          {#if hasKey(p)}<button type="button" class="btn sm danger" onclick={() => removeKey(p)}>{$_("ai.key.remove")}</button>{/if}
        </div>
        <div class="rw"><b>{$_("set.model.name")}</b>
          <div class="rr"><label class="fld"><input aria-label={$_("set.model.name")} bind:value={models[p]} /></label>
            <button type="button" class="btn sm ghost" onclick={() => (models[p] = defModel(p))}><Icon name="refresh" />{$_("common.default")}</button></div>
        </div>
        <div class="rr" style="padding-top:6px">
          <button type="button" class="btn sm" disabled={!hasKey(p) || test[p] === "running"} onclick={() => runTest(p)}>{$_("set.test")}</button>
          <span class="hint" role="status">{testText(p)}</span>
        </div>
      </div>
    {/each}
  </div>

  <div class="card" style="margin-top:14px" data-testid="compat-card">
    <h4>{$_("set.compat.title")}</h4>
    <div class="rw"><b>{$_("set.compat.url")}</b>
      <div class="rr"><label class="fld {compatUrlOk ? '' : 'err'}"><input aria-label={$_("set.compat.url")} aria-invalid={!compatUrlOk} autocomplete="off" spellcheck="false" bind:value={compatUrl} /></label>
        <button type="button" class="btn sm ghost" onclick={() => (compatUrl = api.COMPAT_DEFAULT_URL)}><Icon name="refresh" />{$_("common.default")}</button></div></div>
    <p class="hint" role={compatUrlOk ? undefined : "alert"}>{compatUrlOk ? $_("set.compat.url.hint") : $_("set.compat.url.bad")}</p>
    <div class="rw"><b>{$_("set.model.name")}</b>
      <div class="rr"><label class="fld"><input list="compat-models" aria-label={$_("set.model.name")} autocomplete="off" spellcheck="false" bind:value={models.compat} /></label>
        <datalist id="compat-models">{#each compatModels as m (m)}<option value={m}></option>{/each}</datalist>
        <button type="button" class="btn sm" disabled={!compatUrlOk || compatList === "running"} onclick={loadModels}>{$_("set.compat.models.load")}</button></div></div>
    <p class="hint" role="status">{compatHint}</p>
    <div class="rw"><b>{$_("set.compat.key")}</b>
      <div class="rr">
        <label class="fld"><Icon name="key" />
          <input type={show.compat ? "text" : "password"} autocomplete="off" spellcheck="false" aria-label={$_("set.compat.key")}
            placeholder={hasKey("compat") ? $_("ai.key.saved") : $_("set.compat.key.none")} bind:value={keyInput.compat} />
          <button type="button" class="btn icon sm ghost" aria-label={$_("set.key.show")} onclick={() => (show.compat = !show.compat)}><Icon name="eye" /></button>
        </label>
        {#if hasKey("compat")}<span class="bd ok">{$_("ai.key.saved")}</span>{/if}
      </div>
    </div>
    <div class="rr" style="padding:6px 0">
      <button type="button" class="btn sm primary" disabled={!keyInput.compat.trim()} onclick={() => saveKey("compat")}>{$_("ai.key.save")}</button>
      {#if hasKey("compat")}<button type="button" class="btn sm danger" onclick={() => removeKey("compat")}>{$_("ai.key.remove")}</button>{/if}
    </div>
    <div class="rr" style="padding-top:6px">
      <button type="button" class="btn sm" disabled={!compatUrlOk || !compatModelOk || test.compat === "running"} onclick={() => runTest("compat")}>{$_("set.test")}</button>
      <span class="hint" role="status">{testText("compat")}</span>
    </div>
    <p class="hint">{$_("set.compat.need")}</p>
    <p class="hint">{$_("set.compat.privacy")}</p>
  </div>

  {#if cliProvs.length}<p class="hint risk" data-testid="cli-risk" style="margin-top:14px"><Icon name="warn" /> {$_("set.cli.risk")}</p>{/if}
  {#each cliProvs as p (p)}
  <div class="card" style="margin-top:8px" data-testid="cli-card-{p}">
    <h4>{$_(api.cliKey("set.cli.title", p))}</h4>
    <div class="rw" data-testid="cli-status-{p}">
      <div>
        <span class="bd {info(p)?.status === 'ready' ? 'ok' : ''}" role="status" data-testid="cli-st-{p}">{$_(stKey(p))}</span>
        {#if info(p)?.path}
          <small data-testid="cli-found-{p}">{foundText(p)}</small>
        {/if}
      </div>
      <div class="rr">
        {#if api.signInPrimary(p, info(p))}
          <button type="button" class="btn sm primary" disabled={busy} onclick={() => startLogin(p)}>{$_("set.login.btn")}</button>
        {/if}
        <button type="button" class="btn sm ghost" disabled={cli[p] === "checking" || (busy && loginFor === p)} onclick={() => recheckSaved(p)}><Icon name="refresh" />{$_("set.cli.recheck")}</button>
      </div>
    </div>
    {#if info(p)?.status === "not_installed"}
      <p class="hint" data-testid="cli-install-{p}">{$_(api.cliKey("set.cli.install", p))}</p>
      {#if instInfo[p]}
      <div class="rr" style="padding:6px 0" data-testid="install-btns-{p}">
        {#if api.canInstallHere(p)}
          <button type="button" class="btn sm primary" data-testid="install-run-{p}" disabled={!instInfo[p].can_run || instBusy || confirmFor === p} onclick={() => { confirmFor = p; instNote = null; }}>{$_("set.install.run")}</button>
        {/if}
        <button type="button" class="btn sm" data-testid="install-copy-{p}" onclick={() => copyAndOpen(p)}>{$_("set.install.copyopen")}</button>
      </div>
      {#if instInfo[p].npm_missing}<p class="hint errbox" role="alert" data-testid="install-nonpm-{p}">{$_("set.install.nonpm")}</p>{/if}
      {#if confirmFor === p}
      <div class="hint" role="alertdialog" aria-label={$_("set.install.confirm.title")} data-testid="install-confirm-{p}">
        <p><b>{$_("set.install.confirm.title")}</b></p>
        <p>{$_("set.install.confirm.body")}</p>
        <code class="mono" style="word-break:break-all" data-testid="install-confirm-cmd-{p}">{instInfo[p].command}</code>
        <div class="rr" style="padding-top:6px">
          <button type="button" class="btn sm primary" data-testid="install-confirm-ok-{p}" onclick={() => runInstall(p)}>{$_("set.install.confirm.ok")}</button>
          <button type="button" class="btn sm" data-testid="install-confirm-no-{p}" onclick={() => (confirmFor = null)}>{$_("set.install.confirm.no")}</button>
        </div>
      </div>
      {/if}
      {/if}
      {#if instFor === p && (inst || instErr)}
      <div class="hint {instErr || inst?.phase === 'failed' ? 'errbox' : ''}" role="status" data-testid="install-state-{p}">
        {#if instErr}<span role="alert">{$_(api.installErrKey(instErr))}</span>
        {:else if inst?.phase === "running"}{$_("set.install.running")}
          <div class="rr" style="padding-top:6px"><button type="button" class="btn sm" data-testid="install-cancel-{p}" onclick={cancelInstall}>{$_("set.install.cancel")}</button></div>
        {:else if inst?.phase === "done"}{$_("set.install.done")}
        {:else if inst?.phase === "cancelled"}{$_("set.install.cancelled")}
        {:else if inst?.phase === "failed"}<span role="alert">{$_(api.installErrKey(inst.error))}</span>{/if}
        {#if inst?.tail.length}<pre class="mono" style="white-space:pre-wrap;margin:4px 0 0" data-testid="install-tail-{p}">{inst.tail.join("\n")}</pre>{/if}
      </div>
      {/if}
      {#if instNote?.p === p}
      <p class="hint" role="status" data-testid="install-note-{p}">{noteText(instNote)}{#if instNote.command && instNote.key !== "set.install.openfail"}<br /><code class="mono" style="word-break:break-all" data-testid="install-note-cmd-{p}">{instNote.command}</code>{/if}</p>
      {/if}
    {:else if info(p)?.status === "not_logged_in" || info(p)?.status === "unknown"}
      <p class="hint" data-testid="cli-help-{p}">{$_(api.cliKey("set.cli.notloggedin.help", p))}</p>
    {/if}
    {#if api.canLogin(p)}
    {#if busy && loginFor === p}
    <div class="rr" style="padding-top:6px" data-testid="login-{p}">
      <button type="button" class="btn sm" onclick={cancelLogin}>{$_("set.login.cancel")}</button>
    </div>
    {:else if !api.signInPrimary(p, info(p)) && info(p)?.status === "ready"}
    <div class="rr" style="padding-top:6px" data-testid="login-{p}">
      <button type="button" class="btn sm ghost" disabled={busy} onclick={() => startLogin(p)}>{$_("set.login.again")}</button>
    </div>
    {/if}
    {#if loginFor === p && (login || loginErr)}
      <div class="hint" role="status" data-testid="login-state-{p}">
        {#if loginErr}<span role="alert">{$_(api.loginErrKey(loginErr, p))}</span>
        {:else if login?.phase === "starting"}{$_("set.login.starting")}
        {:else if login?.phase === "done"}{$_("set.login.done")}
        {:else if login?.phase === "cancelled"}{$_("set.login.cancelled")}
        {:else if login?.phase === "failed"}<span role="alert">{$_(api.loginErrKey(login.error, p))}</span>
        {:else if login?.phase === "verifying"}{$_("set.login.verifying")}
        {:else if login}
          {#if login.device_code}
            <p>{$_("set.login.devicecode")}</p>
            <div class="rr" style="padding:4px 0">
              <b class="mono" style="font-size:1.8em;letter-spacing:0.12em" data-testid="login-device-code">{login.device_code}</b>
              <button type="button" class="btn sm" onclick={copyDeviceCode}>{codeCopied ? $_("set.login.copied") : $_("set.login.copycode")}</button>
            </div>
          {/if}
          {#if login.url}
            <p>{$_("set.login.link")}</p>
            <code class="mono" style="word-break:break-all" data-testid="login-url">{login.url}</code>
            <div class="rr" style="padding-top:6px">
              <button type="button" class="btn sm primary" onclick={openLogin}>{login.device_code ? $_("set.login.opendevice") : $_("set.login.open")}</button>
              <button type="button" class="btn sm" onclick={copyLink}>{copied ? $_("set.login.copied") : $_("set.login.copy")}</button>
            </div>
          {:else}{$_("set.login.starting")}{/if}
          {#if login.needs_code}
            <div class="rw"><b>{$_("set.login.code")}</b>
              <div class="rr"><label class="fld"><input aria-label={$_("set.login.code")} type="password" autocomplete="off" spellcheck="false" bind:value={pasted} /></label>
                <button type="button" class="btn sm primary" disabled={!pasted.trim() || login.phase === "code_sent"} onclick={sendCode}>{$_("set.login.verify")}</button></div></div>
          {/if}
          <p>{remainingText(login.remaining_secs)}</p>
        {/if}
      </div>
    {/if}
    {:else if p === "antigravity_cli"}
    <p class="hint" data-testid="agy-signin-note">{$_("set.cli.signin.agy")}</p>
    {/if}
    <details class="adv" data-testid="cli-adv-{p}">
      <summary>{$_("set.cli.adv")}</summary>
      <div class="rw"><b>{$_(api.cliKey("set.cli.path", p))}</b>
        <div class="rr"><label class="fld {pathOk(cliPaths[p] ?? '') ? '' : 'err'}"><input aria-label={$_(api.cliKey("set.cli.path", p))} aria-invalid={!pathOk(cliPaths[p] ?? "")} autocomplete="off" spellcheck="false" bind:value={cliPaths[p]} /></label></div></div>
      <p class="hint">{$_(api.cliKey("set.cli.path.hint", p))}</p>
      <div class="rr" style="padding-top:6px">
        <button type="button" class="btn sm" disabled={!pathOk(cliPaths[p] ?? "") || test[p] === "running"} onclick={() => runTest(p)}>{$_("set.test")}</button>
        <span class="hint" role="status">{testText(p)}</span>
      </div>
    </details>
    {#if !pathOk(cliPaths[p] ?? "")}<p class="hint" role="alert">{$_("set.cli.path.bad")}</p>{/if}
    <p class="hint">{$_(api.cliKey("set.cli.need", p))}</p>
  </div>
  {/each}

  <div class="card" style="margin-top:14px">
    <h4>{$_("set.ollama.local")}</h4>
    <div class="rw"><b>{"URL"}</b><div class="rr"><label class="fld"><input aria-label={"URL"} bind:value={ollamaUrl} /></label></div></div>
    <div class="rw"><b>{$_("set.model.name")}</b>
      <div class="rr"><label class="fld"><input aria-label={$_("set.model.name")} bind:value={models.ollama} /></label>
        <button type="button" class="btn sm ghost" onclick={() => (models.ollama = defModel("ollama"))}><Icon name="refresh" />{$_("common.default")}</button></div></div>
    <div class="rr" style="padding-top:6px">
      <button type="button" class="btn sm" disabled={test.ollama === "running"} onclick={() => runTest("ollama")}>{$_("set.test")}</button>
      <span class="hint" role="status">{testText("ollama")}</span>
    </div>
    <p class="hint">{$_("set.ollama.need")}</p>
  </div>
</div>

<div class="card">
  <h4>{$_("set.ai.prompts")}</h4>
  {#each CHIPS as c (c.kind)}
    <div class="lab"><span>{$_(c.label)}</span><button type="button" class="btn sm ghost" onclick={() => (prompts[c.kind] = defPrompt(c.kind))}>{$_("common.restore")}</button></div>
    <textarea class="ta" rows="2" maxlength={MAX_PROMPT} aria-label={$_(c.label)} bind:value={prompts[c.kind]}></textarea>
    <div class="cnt">{(prompts[c.kind] ?? "").length} / {MAX_PROMPT}</div>
  {/each}
</div>

<div class="card">
  <h4>{$_("set.ai.ctx")}</h4>
  <div class="rw"><b>{$_("set.ai.chars")}</b><div class="rr"><label class="fld"><input type="number" min="2000" max="200000" step="1000" aria-label={$_("set.ai.chars")} bind:value={chars} /></label><span class="hint">{$_("set.ai.chars.range")}</span></div></div>
  <div class="rw"><b>{$_("set.ai.turns")}</b><div class="rr"><label class="fld"><input type="number" min="0" max="20" aria-label={$_("set.ai.turns")} bind:value={turns} /></label><span class="hint">{$_("set.ai.turns.range")}</span></div></div>
</div>

<div class="tl2">
  <button type="button" class="btn primary" disabled={!compatUrlOk || !cliPathOk} onclick={save}>{$_("ai.settings.save")}</button>
  <button type="button" class="btn" onclick={resetTab}><Icon name="refresh" />{$_("set.reset.tab")}</button>
  <span class="hint">{$_("set.reset.nokey")}</span>
  {#if saved}<span class="bd ok" role="status">{$_("ai.settings.saved")}</span>{/if}
  {#if error}<span class="bd rec" role="alert">{$_(error === "ipc_timeout" ? "set.load.timeout" : api.errKey(error))}</span>{/if}
</div>
{:else if loadErr}
  <LoadError message={$_(loadErr)} busy={loading} onretry={firstLoad} />
{:else}
  <p class="hint" role="status">{$_("common.loading")}</p>
{/if}
