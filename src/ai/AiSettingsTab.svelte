<script lang="ts">
  // Tab "Tro ly AI" trong Cai dat (phuong an A, US-47). Khoa API chi NHAP VAO: o nhap la mat khau, bi xoa ngay sau khi
  // luu, va giao dien chi biet "da co khoa / chua co" (`has_key`). Khong co duong nao doc khoa tu Keychain ve day.
  import { onMount } from "svelte";
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

  const hasKey = (p: Provider) => s?.providers.find(x => x.provider === p)?.has_key ?? false;
  // Endpoint tuong thich khong co model mac dinh gia (AC-57.5): ten model do nguoi dung nhap hoac chon.
  const defModel = (p: Provider) => ({ gemini: "gemini-2.5-flash", openai: "gpt-4.1-mini", ollama: "qwen2.5:1.5b", compat: "" })[p];
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
  async function persist() {
    const input: api.AiSettingsInput = {
      default_provider: defaultProvider,
      context_max_chars: Number(chars), context_max_turns: Number(turns),
      providers: api.PROVIDERS.map(p => ({
        provider: p,
        model_name: models[p]?.trim() && models[p].trim() !== defModel(p) ? models[p].trim() : null,
        base_url: p === "ollama" ? (ollamaUrl.trim() || null) : p === "compat" ? (compatUrl.trim() || null) : null,
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
    for (const p of api.PROVIDERS) models[p] = defModel(p);
    ollamaUrl = "http://localhost:11434";
    compatUrl = api.COMPAT_DEFAULT_URL; compatModels = []; compatList = "idle";
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

  onMount(() => { void firstLoad(); });
</script>

{#if s}
<div class="card">
  <h4>{$_("set.ai.default")}</h4>
  <div class="rw">
    <div><b>{$_("set.ai.default.hint")}</b><small>{$_("set.ai.default.note")}</small></div>
    <div class="rr">
      <select class="sel" aria-label={$_("ask.model.pick")} bind:value={defaultProvider}>
        <option value={null}>{$_("ask.model.none")}</option>
        {#each api.PROVIDERS as p (p)}<option value={p}>{$_(api.providerLabelKey(p))}</option>{/each}
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
  <button type="button" class="btn primary" disabled={!compatUrlOk} onclick={save}>{$_("ai.settings.save")}</button>
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
