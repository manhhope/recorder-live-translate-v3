<script lang="ts">
  import { onMount } from "svelte";
  import { _, formatCurrency, formatNumber, locale, currentLocale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import type { WeekOut } from "../lib/api.ts";

  const loc = $derived(($locale as "en" | "pt-PT") ?? currentLocale());
  const weekKeys = ["set.week.0", "set.week.1", "set.week.2", "set.week.3"];
  const dayKeys = ["day.mon", "day.tue", "day.wed", "day.thu", "day.fri", "day.sat", "day.sun"];
  let week = $state(0);
  let data = $state<WeekOut | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let hasKey = $state(false);
  let keyInput = $state("");
  let keyMsg = $state<{ ok: boolean; code: string } | null>(null);
  let busy = $state(false);
  let confirmClear = $state(false);

  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
  const max = $derived(Math.max(1, ...(data?.days.map(d => d.audioMs) ?? [0])));
  const empty = $derived(!data || data.totalAudioMs === 0);
  const audioLabel = $derived.by(() => {
    const mins = Math.round((data?.totalAudioMs ?? 0) / 60000);
    return $_("set.usage.audio", { values: { h: formatNumber(Math.floor(mins / 60), loc), m: mins % 60 } });
  });

  async function loadWeek() {
    loading = true; error = null;
    try { data = await api.usageWeek(api.mondayOf(week)); } catch (e) { error = api.errCode(e); }
    loading = false;
  }
  onMount(async () => {
    try { hasKey = await api.hasApiKey(); } catch { hasKey = false; }
    await loadWeek();
  });

  async function saveKey() {
    const v = keyInput.trim();
    if (!v) return;
    busy = true; keyMsg = null;
    try { await api.saveApiKey(v); keyInput = ""; hasKey = true; } catch (e) { keyMsg = { ok: false, code: api.errCode(e) }; }
    busy = false;
  }
  async function test() {
    busy = true; keyMsg = null;
    try { await api.testConnection(); keyMsg = { ok: true, code: "ok" }; } catch (e) { keyMsg = { ok: false, code: api.errCode(e) }; }
    busy = false;
  }
  async function removeKey() {
    busy = true; keyMsg = null;
    try { await api.clearApiKey(); hasKey = false; } catch (e) { keyMsg = { ok: false, code: api.errCode(e) }; }
    busy = false;
  }
  async function exportCsv() {
    try {
      const csv = await api.usageExportCsv();
      const url = URL.createObjectURL(new Blob([csv], { type: "text/csv" }));
      const a = document.createElement("a");
      a.href = url; a.download = "usage-logs.csv"; a.click();
      URL.revokeObjectURL(url);
    } catch (e) { error = api.errCode(e); }
  }
  async function clearAll() {
    try { await api.usageClear(); confirmClear = false; await loadWeek(); } catch (e) { error = api.errCode(e); }
  }
</script>

<h3>{$_("set.tab.sonioxusage")}</h3>

<div class="card">
  <h4>{$_("set.soniox.title")}</h4>
  <div class="rw">
    <div><b>{$_("set.soniox.key")}</b><small>{$_("set.soniox.store")}</small></div>
    <div class="rr">
      {#if hasKey}
        <span class="bd acc"><Icon name="key" />{$_("set.soniox.saved")}</span>
        <button type="button" class="btn sm" disabled={busy} onclick={test}>{$_("set.test")}</button>
        <button type="button" class="btn sm ghost" disabled={busy} onclick={removeKey}>{$_("set.soniox.remove")}</button>
      {:else}
        <label class="fld err"><Icon name="key" /><input type="password" autocomplete="off" aria-label={$_("set.soniox.key")} placeholder={$_("set.key.none")} bind:value={keyInput} /></label>
        <button type="button" class="btn sm primary" disabled={busy || !keyInput.trim()} onclick={saveKey}>{$_("rec.key.save")}</button>
      {/if}
      {#if keyMsg}
        <span class="bd {keyMsg.ok ? 'ok' : 'rec'}"><Icon name={keyMsg.ok ? "check" : "warn"} />{keyMsg.ok ? $_("set.soniox.ok") : $_(errKey(keyMsg.code))}</span>
      {/if}
    </div>
  </div>
</div>

<div class="card">
  <h4>{$_("set.tab.usagelogs")}</h4>
  <div class="tl2" style="justify-content:space-between;margin-bottom:8px">
    <span class="seg" role="group" aria-label={$_("set.week.pick")}>
      {#each weekKeys as k, i (k)}<button type="button" class={week === i ? "on" : ""} aria-pressed={week === i} onclick={() => { week = i; void loadWeek(); }}>{$_(k)}</button>{/each}
    </span>
    <span class="rr">
      <button type="button" class="btn sm" onclick={loadWeek}><Icon name="refresh" />{$_("common.refresh")}</button>
      <button type="button" class="btn sm" onclick={exportCsv}><Icon name="download" />{$_("set.usage.export")}</button>
    </span>
  </div>
  {#if error}
    <div class="empty" style="padding:30px"><Icon name="warn" /><b>{$_(errKey(error))}</b>
      <button type="button" class="btn sm" onclick={loadWeek}><Icon name="refresh" />{$_("common.retry")}</button></div>
  {:else if loading}
    <div class="empty" style="padding:30px"><Icon name="refresh" /><span>{$_("common.loading")}</span></div>
  {:else if !empty && data}
    <div class="hint" style="margin:0">{$_("set.usage.week")}</div>
    <div><b>{audioLabel}</b> {$_("set.usage.cost")} <b>{"≈"} {formatCurrency(api.usd(data.totalEstCostUsdMicros), loc)}</b> <span class="hint">{$_("common.estimate")}</span></div>
    <div class="wkw">
      <div class="wk">
        {#each data.days as d, i (d.localDate)}
          <div><div class="st" style="height:{(d.audioMs / max) * 85}%"></div><em>{$_(dayKeys[i])}</em></div>
        {/each}
      </div>
    </div>
    <div class="hint">{$_("set.usage.note")}</div>
  {:else}
    <div class="empty" style="padding:30px"><Icon name="list" /><b>{$_("set.usage.none")}</b><span>{$_("set.usage.none.hint")}</span></div>
  {/if}
  <div class="tl2" style="margin-top:10px">
    {#if confirmClear}
      <button type="button" class="btn sm stop" onclick={clearAll}>{$_("set.usage.clear.confirm")}</button>
      <button type="button" class="btn sm" onclick={() => (confirmClear = false)}>{$_("common.cancel")}</button>
    {:else}
      <button type="button" class="btn sm ghost" onclick={() => (confirmClear = true)}><Icon name="trash" />{$_("set.usage.clear")}</button>
    {/if}
  </div>
</div>
