<script lang="ts">
  // Tab "Phu de": co chu, hai mau, che do, do trong, an khoi chia se. Moi lan doi luu ngay o backend (AC-14.2).
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import LoadError from "../lib/LoadError.svelte";
  import { withTimeout } from "../lib/ipc.ts";
  import * as api from "../lib/api.ts";
  import type { DisplayMode, SubtitleSettings } from "../lib/api.ts";

  let s = $state<SubtitleSettings | null>(null);
  let err = $state<string | null>(null);
  let colorErr = $state<{ which: string; text: string } | null>(null);
  let opened = $state(false);
  let loading = $state(true);

  const modes: [DisplayMode, string][] = [["bilingual", "sub.mode.bilingual"], ["translation_only", "sub.mode.translation"], ["source_only", "sub.mode.source"]];
  const colors = [["source", "sub.color.source"], ["translation", "sub.color.translation"]] as const;

  async function run(fn: () => Promise<SubtitleSettings>) {
    err = null;
    try { s = await withTimeout(fn()); } catch (e) { err = api.errCode(e); }
  }
  async function firstLoad() { loading = true; await run(api.subtitleGet); loading = false; }
  onMount(() => { void firstLoad(); });

  async function setColor(which: "source" | "translation", hex: string) {
    colorErr = null;
    try { s = await api.subtitleSetColor(which, hex.toUpperCase()); }
    catch (e) {
      const c = api.errCode(e);
      colorErr = { which, text: c.startsWith("low_contrast:") ? $_("set.sub.contrast", { values: { r: c.slice(13) } }) : $_("set.sub.invalidhex") };
      s = await api.subtitleGet().catch(() => s);
    }
  }
  async function toggleWindow() {
    err = null;
    try { if (opened) await api.subtitleClose(); else await api.subtitleOpen(); opened = !opened; } catch (e) { err = api.errCode(e); }
  }
  const sizeLbl = $derived(s ? $_("set.sub.size", { values: { n: s.font_size_pt } }) : "");
  const opacityLbl = $derived(s ? $_("set.sub.opacity", { values: { n: s.opacity_percent } }) : "");
  const colorOf = (w: "source" | "translation") => (w === "source" ? s?.source_color : s?.translation_color) ?? "#000000";
</script>

<h3>{$_("set.tab.subtitle")}</h3>

{#if s}
<div class="card">
  <div class="rw">
    <b>{$_(opened ? "set.sub.close" : "set.sub.open")}</b>
    <div class="rr"><button type="button" class="btn sm primary" onclick={toggleWindow}><Icon name="caption" />{$_(opened ? "sub.close" : "set.sub.open")}</button></div>
  </div>
  <div class="rw">
    <b>{$_("set.sub.mode")}</b>
    <div class="rr">
      <span class="seg" role="group" aria-label={$_("sub.mode.label")}>
        {#each modes as [m, k] (m)}<button type="button" class={s.display_mode === m ? "on" : ""} aria-pressed={s.display_mode === m} onclick={() => run(() => api.subtitleSetMode(m))}>{$_(k)}</button>{/each}
      </span>
    </div>
  </div>
  <div class="rw">
    <b>{sizeLbl}</b>
    <div class="rr">
      <button type="button" class="btn sm icon" aria-label={$_("sub.font.decrease")} title={$_("sub.font.decrease")} onclick={() => run(() => api.subtitleFontStep(-2))}><Icon name="minus" /></button>
      <button type="button" class="btn sm icon" aria-label={$_("sub.font.increase")} title={$_("sub.font.increase")} onclick={() => run(() => api.subtitleFontStep(2))}><Icon name="plus" /></button>
    </div>
  </div>
  <div class="rw">
    <b>{opacityLbl}</b>
    <div class="rr"><input type="range" min="0" max="80" step="5" aria-label={$_("sub.opacity")} value={s.opacity_percent} onchange={(e) => run(() => api.subtitleSetOpacity(Number(e.currentTarget.value)))} /></div>
  </div>
  {#each colors as [w, k] (w)}
    <div class="rw">
      <div><b>{$_(k)}</b>{#if colorErr?.which === w}<small role="alert"><span class="bd rec"><Icon name="warn" />{colorErr.text}</span></small>{/if}</div>
      <div class="rr">
        <code class="hint" style="margin:0">{colorOf(w)}</code>
        <input type="color" aria-label={$_(k)} value={colorOf(w).toLowerCase()} onchange={(e) => setColor(w, e.currentTarget.value)} />
      </div>
    </div>
  {/each}
  <div class="rw">
    <div><b>{$_("sub.hideshare")}</b><small>{$_("sub.hideshare.note")}</small></div>
    <div class="rr"><input type="checkbox" class="chk" role="switch" aria-label={$_("sub.hideshare")} checked={s.hidden_from_capture} onchange={(e) => run(() => api.subtitleSetHidden(e.currentTarget.checked))} /></div>
  </div>
  <div class="tl2" style="margin-top:8px">
    <button type="button" class="btn sm" onclick={() => run(api.subtitleResetColors)}><Icon name="refresh" />{$_("set.sub.resetall")}</button>
    <button type="button" class="btn sm" onclick={() => api.subtitleResetPosition().catch(e => (err = api.errCode(e)))}><Icon name="refresh" />{$_("sub.resetposition")}</button>
  </div>
</div>
{/if}
{#if !s && !loading && err}
  <LoadError message={$_(err === "ipc_timeout" ? "set.load.timeout" : "set.load.err")} onretry={firstLoad} />
{:else if !s}
  <p class="hint" role="status">{$_("common.loading")}</p>
{:else if err}<div class="hint" role="alert"><span class="bd rec"><Icon name="warn" />{$_(err === "ipc_timeout" ? "set.load.timeout" : "err.generic")}</span></div>{/if}
