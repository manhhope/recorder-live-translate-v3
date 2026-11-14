<script lang="ts">
  // Cua so Quick Translate (US-31): go chu, chon ngon ngu (nho lua chon), dich, sao chep, lich su.
  // Loi dich KHONG xoa chu da go (AC-31.6). Esc dong cua so va huy yeu cau dang chay (AC-31.9).
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { _ } from "svelte-i18n";
  import * as api from "./api.ts";
  import type { HistoryItem, QuickPrefs } from "./api.ts";
  import { charCount, errorKey, inputState, langLabelKey, MAX_CHARS, parseExcluded, SOURCE_LANGS, TARGET_LANGS } from "./model.ts";

  let text = $state("");
  let out = $state("");
  let errKey = $state<string | null>(null);
  let busy = $state(false);
  let copied = $state(false);
  let items = $state<HistoryItem[]>([]);
  let showHistory = $state(false);
  let showOptions = $state(false);
  let prefs = $state<QuickPrefs>({ toolbar_enabled: false, excluded_apps: [], source_lang: null, target_lang: "pt-PT" });
  let excludedText = $state("");
  let axTrusted = $state(false);
  let needsAx = $state(false);
  let input: HTMLTextAreaElement | undefined = $state();

  const st = $derived(inputState(text));
  const n = $derived(charCount(text));
  const counter = $derived($_("quick.counter", { values: { n, max: MAX_CHARS } }));

  async function refresh() {
    items = await api.history(50).catch(() => []);
    axTrusted = await api.axStatus().catch(() => false);
  }

  onMount(() => {
    api.getPrefs().then((p) => { prefs = p; excludedText = p.excluded_apps.join("\n"); }).catch(() => {});
    refresh();
    // Moi lan cua so duoc hien: dua con tro vao o nhap (pre-warm: cua so da co san, chi hien).
    const un = listen("quick://open", async () => { await tick(); input?.focus(); refresh(); });
    return () => { un.then((f) => f()); };
  });

  async function savePrefs() {
    const r = await api.setPrefs({ ...prefs, excluded_apps: parseExcluded(excludedText) });
    prefs = r.prefs;
    needsAx = r.needs_accessibility;
  }

  async function run() {
    if (st !== "ok" || busy) return;
    busy = true; errKey = null; copied = false;
    await savePrefs();
    try {
      const r = await api.translate(text, prefs.source_lang, prefs.target_lang);
      out = r.text;
      refresh();
    } catch (e) {
      // Giu nguyen `text`; chi hien thong bao loi.
      const code = typeof e === "object" && e && "code" in e ? String((e as { code: string }).code) : "provider";
      errKey = errorKey(code);
    } finally { busy = false; }
  }

  async function copyOut() {
    await navigator.clipboard.writeText(out);
    copied = true;
  }

  async function close() {
    await api.hide();
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); close(); }
    else if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) { e.preventDefault(); run(); }
  }

  async function wipe() {
    await api.clearHistory();
    items = [];
  }

  function reuse(it: HistoryItem) {
    text = it.source_text; out = it.translated_text; showHistory = false;
  }
</script>

<svelte:window onkeydown={onKey} />

<main>
  <header>
    <h1>{$_("quick.title")}</h1>
    <button type="button" class="ghost" aria-label={$_("common.close")} onclick={close}>{$_("sub.close.short")}</button>
  </header>

  <div class="row">
    <label>{$_("quick.source")}
      <select value={prefs.source_lang ?? "auto"} onchange={(e) => { const v = (e.currentTarget as HTMLSelectElement).value; prefs.source_lang = v === "auto" ? null : v; }}>
        {#each SOURCE_LANGS as l}<option value={l}>{$_(langLabelKey(l))}</option>{/each}
      </select>
    </label>
    <label>{$_("quick.target")}
      <select bind:value={prefs.target_lang}>
        {#each TARGET_LANGS as l}<option value={l}>{$_(langLabelKey(l))}</option>{/each}
      </select>
    </label>
  </div>

  <textarea bind:this={input} bind:value={text} placeholder={$_("quick.input.ph")} aria-label={$_("quick.input.ph")} rows="5"></textarea>
  <div class="meta" class:bad={st === "toolong"}><span>{counter}</span>{#if st === "toolong"} <span>{$_("quick.err.toolong")}</span>{/if}</div>

  <div class="row">
    <button type="button" class="primary" disabled={st !== "ok" || busy} onclick={run}>{busy ? $_("quick.working") : $_("common.translate")}</button>
    <span class="hint">{$_("quick.esc.hint")}</span>
  </div>

  {#if errKey}<p class="err" role="alert">{$_(errKey)}</p>{/if}

  {#if out}
    <section aria-live="polite">
      <p class="out">{out}</p>
      <button type="button" onclick={copyOut}>{copied ? $_("quick.copied") : $_("common.copy")}</button>
    </section>
  {/if}

  <footer>
    <button type="button" class="ghost" onclick={() => (showHistory = !showHistory)}>{$_("quick.history")}</button>
    <button type="button" class="ghost" onclick={() => (showOptions = !showOptions)}>{$_("quick.options")}</button>
  </footer>

  {#if showHistory}
    <section class="panel">
      {#if items.length === 0}
        <p class="hint">{$_("quick.history.empty")}</p>
      {:else}
        <ul>
          {#each items as it (it.id)}
            <li><button type="button" class="ghost" onclick={() => reuse(it)}>{it.source_text.slice(0, 60)}</button></li>
          {/each}
        </ul>
        <button type="button" onclick={wipe}>{$_("quick.history.clear")}</button>
      {/if}
    </section>
  {/if}

  {#if showOptions}
    <section class="panel">
      <label class="check"><input type="checkbox" checked={prefs.toolbar_enabled} onchange={async (e) => { prefs.toolbar_enabled = (e.currentTarget as HTMLInputElement).checked; await savePrefs(); }} /> {$_("quick.toolbar.setting")}</label>
      {#if needsAx || (!axTrusted && prefs.toolbar_enabled)}<p class="err" role="alert">{$_("quick.toolbar.needs.ax")}</p>{/if}
      <label>{$_("quick.toolbar.excluded")}
        <textarea rows="3" bind:value={excludedText} onblur={savePrefs}></textarea>
      </label>
    </section>
  {/if}
</main>

<style>
  main { display: flex; flex-direction: column; gap: 8px; padding: 12px; box-sizing: border-box; height: 100vh; overflow: auto; }
  header { display: flex; justify-content: space-between; align-items: center; }
  h1 { font-size: 14px; margin: 0; }
  .row { display: flex; gap: 8px; align-items: center; }
  textarea { width: 100%; box-sizing: border-box; resize: vertical; font: inherit; }
  .meta, .hint { font-size: 12px; opacity: 0.7; }
  .bad, .err { color: var(--danger, #c0392b); opacity: 1; }
  .out { white-space: pre-wrap; margin: 0 0 6px; }
  footer { display: flex; gap: 8px; margin-top: auto; }
  .panel { border-top: 1px solid var(--border, #8884); padding-top: 8px; }
  ul { list-style: none; padding: 0; margin: 0 0 8px; }
  .ghost { background: none; border: none; cursor: pointer; font: inherit; text-align: left; }
  .check { display: flex; gap: 6px; align-items: center; }
</style>
