<script lang="ts">
  import { onMount } from "svelte";
  import { _, formatCurrency, formatNumber, locale, currentLocale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import type { UsageSourceOut } from "../lib/api.ts";

  const loc = $derived(($locale as "en" | "pt-PT") ?? currentLocale());
  const weekKeys = ["set.week.0", "set.week.1", "set.week.2", "set.week.3"];
  const dayKeys = ["day.mon", "day.tue", "day.wed", "day.thu", "day.fri", "day.sat", "day.sun"];
  let week = $state(0);
  let data = $state<UsageSourceOut | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let hasKey = $state(false);
  let keyInput = $state("");
  let keyMsg = $state<{ ok: boolean; code: string } | null>(null);
  let busy = $state(false);
  let confirmClear = $state(false);
  let cleared = $state(false);

  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
  const max = $derived(Math.max(1, ...(data?.days.map(d => d.audioMs) ?? [0])));
  const isLocal = $derived(data?.source === "local");
  const emptyWhyKey = $derived((data?.failedStartCount ?? 0) > 0 ? "set.usage.none.failed" : "set.usage.none.noaudio");
  const emptyWhy = $derived($_(emptyWhyKey, { values: { n: data?.sessionsCount ?? 0, f: data?.failedStartCount ?? 0 } }));
  const empty = $derived(!data || (isLocal ? data.totalAudioMs === 0 : data.totalRequests === 0 && data.totalCostUsdMicros === 0));
  const reasonKey = (r: string) => "set.usage.reason." + r.replace(/_/g, "");
  const fallbackText = (r: string) => $_("set.usage.fallback", { values: { reason: $_(reasonKey(r)) } });
  const requestsText = (n: number) => $_("set.usage.requests", { values: { n: formatNumber(n, loc) } });
  const audioLabel = $derived.by(() => {
    const mins = Math.round((data?.totalAudioMs ?? 0) / 60000);
    return $_("set.usage.audio", { values: { h: formatNumber(Math.floor(mins / 60), loc), m: mins % 60 } });
  });
  const money = (micros: number) => formatCurrency(api.usd(micros), loc);

  async function loadWeek() {
    loading = true; error = null;
    try { data = await api.usageWeekSource(week); } catch (e) { error = api.errCode(e); }
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
      const csv = await api.usageExportWeekCsv(week);
      const url = URL.createObjectURL(new Blob([csv], { type: "text/csv" }));
      const a = document.createElement("a");
      a.href = url; a.download = "usage-logs.csv"; a.click();
      URL.revokeObjectURL(url);
    } catch (e) { error = api.errCode(e); }
  }
  async function clearAll() {
    try { await api.usageClear(); confirmClear = false; cleared = true; await loadWeek(); } catch (e) { error = api.errCode(e); }
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
      {#each weekKeys as k, i (k)}<button type="button" class={week === i ? "on" : ""} aria-pressed={week === i} onclick={() => { week = i; cleared = false; void loadWeek(); }}>{$_(k)}</button>{/each}
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
  {:else if data}
    <div class="tl2" style="gap:8px;flex-wrap:wrap">
      <span class="bd {isLocal ? 'rec' : 'acc'}" data-testid="usage-source"><Icon name={isLocal ? "warn" : "check"} />{$_(isLocal ? "set.usage.src.local" : "set.usage.src.soniox")}</span>
      <span class="hint" style="margin:0" data-testid="usage-basis">{$_(data.basis === "utc" ? "set.usage.basis.utc" : "set.usage.basis.local")}</span>
    </div>
    {#if isLocal && data.reason}
      <div class="hint" role="status" data-testid="usage-fallback">{fallbackText(data.reason)}</div>
    {/if}
    {#if !empty}
      <div class="hint" style="margin:0">{data.weekStart} → {data.weekEnd}</div>
      <div><b>{audioLabel}</b> <span class="hint">{$_("set.usage.hours")}</span></div>
      <div><b data-testid="usage-total">{data.estimated ? "≈ " : ""}{money(data.totalCostUsdMicros)}</b> <span class="hint">{$_("set.usage.total")}{data.estimated ? " " + $_("common.estimate") : ""}</span></div>
      <h4 style="margin-top:10px">{$_(data.basis === "utc" ? "set.usage.byday" : "set.usage.byday.local")}</h4>
      <div class="wkw">
        <div class="wk">
          {#each data.days as d, i (d.date)}
            <div title="{d.date}: {money(d.costUsdMicros)}"><div class="st" style="height:{(d.audioMs / max) * 85}%"></div><em>{$_(dayKeys[i])}</em></div>
          {/each}
        </div>
      </div>
      {#if !isLocal}
        <h4 style="margin-top:10px">{$_("set.usage.byscope")}</h4>
        <div class="rw"><div>{$_("set.usage.scope.api")}</div><div class="rr"><b>{money(data.apiCostUsdMicros)}</b></div></div>
        <div class="rw"><div>{$_("set.usage.scope.playground")}</div><div class="rr"><b>{money(data.playgroundCostUsdMicros)}</b></div></div>
        {#if data.models.length > 0}
          <h4 style="margin-top:10px">{$_("set.usage.bymodel")}</h4>
          {#each data.models as m (m.model)}
            <div class="rw"><div>{m.model}<small>{requestsText(m.requests)}</small></div><div class="rr"><b>{money(m.costUsdMicros)}</b></div></div>
          {/each}
        {/if}
        <div class="hint">{$_("set.usage.note.soniox")}</div>
        {#if data.cached}<div class="hint">{$_("set.usage.cached")}</div>{/if}
      {:else}
        <div class="hint">{$_("set.usage.note")}</div>
      {/if}
    {:else}
      <div class="empty" style="padding:30px"><Icon name="list" /><b>{$_("set.usage.none")}</b>
        {#if isLocal}
          <span>{$_("set.usage.none.localreason")}</span>
          {#if data.sessionsCount > 0}<span role="status">{emptyWhy}</span>{/if}
        {:else}
          <span>{$_("set.usage.none.soniox")}</span>
        {/if}
        <span>{$_("set.usage.none.hint")}</span>
      </div>
    {/if}
  {/if}
  <div class="tl2" style="margin-top:10px">
    {#if confirmClear}
      <button type="button" class="btn sm stop" onclick={clearAll}>{$_("set.usage.clear.confirm")}</button>
      <button type="button" class="btn sm" onclick={() => (confirmClear = false)}>{$_("common.cancel")}</button>
    {:else}
      <button type="button" class="btn sm ghost" onclick={() => (confirmClear = true)}><Icon name="trash" />{$_("set.usage.clear")}</button>
    {/if}
  </div>
  <div class="hint" data-testid="usage-clear-note">{$_("set.usage.clear.note")}</div>
  {#if cleared}<div class="hint" role="status">{$_("set.usage.clear.done")}</div>{/if}
</div>
