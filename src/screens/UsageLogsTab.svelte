<script lang="ts">
  import { onMount } from "svelte";
  import { _, formatCurrency, formatNumber, locale, currentLocale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import type { UsageSourceOut } from "../lib/api.ts";
  import { applyFailure, applyRefresh, hasSoniox, shortDate, showsUpdatedAt, updatedTime, weekdayIndex, type UsageWarning } from "../lib/usage-swr.ts";

  const loc = $derived(($locale as "en" | "pt-PT") ?? currentLocale());
  const weekKeys = ["set.week.0", "set.week.1", "set.week.2", "set.week.3"];
  const dayKeys = ["day.mon", "day.tue", "day.wed", "day.thu", "day.fri", "day.sat", "day.sun"];
  let week = $state(0);
  let data = $state<UsageSourceOut | null>(null);
  /** Dang tai lai o nen (so cu van hien, khong xoa). */
  let refreshing = $state(true);
  /** Canh bao gon khi tai lai that bai (so cu van giu neu co). */
  let warning = $state<UsageWarning | null>(null);
  let dataWeek = -1;
  let seq = 0;
  let hasKey = $state(false);
  let keyInput = $state("");
  let keyMsg = $state<{ ok: boolean; code: string } | null>(null);
  let busy = $state(false);
  let confirmClear = $state(false);
  let cleared = $state(false);

  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
  const max = $derived(Math.max(1, ...(data?.days.map(d => d.audioMs) ?? [0])));
  const isLocal = $derived(data?.source === "local");
  /** Chua co ban Soniox nao da luu cho tuan nay: dang hien uoc tinh cuc bo + skeleton trong luc cho mang. */
  const firstLoad = $derived(isLocal && data?.reason === "no_cache" && refreshing && !warning);
  const fatal = $derived(!data && warning?.kind === "err" ? warning.code : null);
  const updatedAt = $derived(showsUpdatedAt(data) && data ? $_("set.usage.updatedat", { values: { time: updatedTime(data.fetchedAt, loc) } }) : "");
  const warnText = $derived.by(() => {
    if (!warning) return "";
    if (warning.kind === "err") return $_(errKey(warning.code));
    const why = $_(reasonKey(warning.code));
    return $_(hasSoniox(data) ? "set.usage.stale.warn" : "set.usage.fallback", { values: { reason: why } });
  });
  const emptyWhyKey = $derived((data?.failedStartCount ?? 0) > 0 ? "set.usage.none.failed" : "set.usage.none.noaudio");
  const emptyWhy = $derived($_(emptyWhyKey, { values: { n: data?.sessionsCount ?? 0, f: data?.failedStartCount ?? 0 } }));
  const empty = $derived(!data || (isLocal ? data.totalAudioMs === 0 : data.totalRequests === 0 && data.totalCostUsdMicros === 0));
  const reasonKey = (r: string) => "set.usage.reason." + r.replace(/_/g, "");
  const requestsText = (n: number) => $_("set.usage.requests", { values: { n: formatNumber(n, loc) } });
  const audioLabel = $derived.by(() => {
    const mins = Math.round((data?.totalAudioMs ?? 0) / 60000);
    return $_("set.usage.audio", { values: { h: formatNumber(Math.floor(mins / 60), loc), m: mins % 60 } });
  });
  const rangeText = $derived(data ? $_("set.usage.range", { values: { from: shortDate(data.weekStart, loc), to: shortDate(data.weekEnd, loc) } }) : "");
  const money = (micros: number) => formatCurrency(api.usd(micros), loc);

  /** Stale-while-revalidate: hien ngay ban da luu (hoac uoc tinh cuc bo), roi hoi Soniox o nen va thay so moi. */
  async function loadWeek() {
    const my = ++seq;
    const w = week;
    warning = null; refreshing = true;
    if (!data || dataWeek !== w) {
      data = null;
      try { data = await api.usageWeekCached(w); dataWeek = w; } catch (e) {
        if (my !== seq) return;
        warning = { kind: "err", code: api.errCode(e) }; refreshing = false; return;
      }
      if (my !== seq) return;
    }
    let v;
    try { v = applyRefresh(data, await api.usageWeekSource(w)); } catch (e) { v = applyFailure(data, api.errCode(e)); }
    if (my !== seq) return;
    data = v.data; warning = v.warning; refreshing = false;
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
  let diagMsg = $state<{ ok: boolean; text: string } | null>(null);
  async function openDiagLogs() {
    diagMsg = null;
    try { diagMsg = { ok: true, text: await api.diagOpenLogs() }; } catch (e) { diagMsg = { ok: false, text: api.errCode(e) }; }
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
    } catch (e) { warning = { kind: "err", code: api.errCode(e) }; }
  }
  async function clearAll() {
    try { await api.usageClear(); confirmClear = false; cleared = true; data = null; await loadWeek(); } catch (e) { warning = { kind: "err", code: api.errCode(e) }; }
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

<div class="card" data-testid="diag-card">
  <h4>{$_("set.diag.title")}</h4>
  <div class="rw">
    <div><b>{$_("set.diag.label")}</b><small>{$_("set.diag.hint")}</small></div>
    <div class="rr">
      <button type="button" class="btn sm" data-testid="diag-open-logs" onclick={openDiagLogs}><Icon name="list" />{$_("set.diag.open")}</button>
      {#if diagMsg}<span class="bd {diagMsg.ok ? 'ok' : 'rec'}" data-testid="diag-msg"><Icon name={diagMsg.ok ? "check" : "warn"} />{diagMsg.ok ? diagMsg.text : $_(errKey(diagMsg.text))}</span>{/if}
    </div>
  </div>
</div>

<div class="card">
  <h4>{$_("set.tab.usagelogs")}</h4>
  <div class="ubar" data-testid="usage-toolbar">
    <span class="seg" role="group" aria-label={$_("set.week.pick")}>
      {#each weekKeys as k, i (k)}<button type="button" class={week === i ? "on" : ""} aria-pressed={week === i} onclick={() => { week = i; cleared = false; data = null; void loadWeek(); }}>{$_(k)}</button>{/each}
    </span>
    <span class="usp"></span>
    {#if data}
      <span class="bd {isLocal ? 'rec' : 'acc'}" data-testid="usage-source"><Icon name={isLocal ? "warn" : "check"} />{$_(isLocal ? "set.usage.src.local" : "set.usage.src.soniox")}</span>
      {#if updatedAt}<span class="hint" data-testid="usage-updated">{updatedAt}</span>{/if}
      {#if refreshing}<span class="bd spin" role="status" data-testid="usage-refreshing"><Icon name="refresh" />{$_("set.usage.refreshing")}</span>{/if}
    {/if}
    <button type="button" class="btn sm icon" disabled={refreshing} onclick={loadWeek} title={$_("common.refresh")} aria-label={$_("common.refresh")}><Icon name="refresh" /></button>
    <button type="button" class="btn sm icon" onclick={exportCsv} title={$_("set.usage.export")} aria-label={$_("set.usage.export")}><Icon name="download" /></button>
  </div>
  {#if fatal}
    <div class="empty" style="padding:30px"><Icon name="warn" /><b>{$_(errKey(fatal))}</b>
      <button type="button" class="btn sm" onclick={loadWeek}><Icon name="refresh" />{$_("common.retry")}</button></div>
  {:else if !data}
    <div class="empty" style="padding:30px"><Icon name="refresh" /><span>{$_("common.loading")}</span></div>
  {:else}
    {#if warning}
      <div class="tl2 warnbar" role="alert" data-testid="usage-warning"><Icon name="warn" /><span>{warnText}</span>
        <button type="button" class="btn sm" disabled={refreshing} onclick={loadWeek}><Icon name="refresh" />{$_("common.retry")}</button></div>
    {/if}
    {#if firstLoad}
      <div class="hint" role="status" data-testid="usage-nocache">{$_("set.usage.reason.nocache")}</div>
      <div class="skel" data-testid="usage-skeleton" aria-hidden="true"><i></i><i></i><i></i></div>
    {/if}
    {#if !empty}
      <div class="usum">
        <span><b>{audioLabel}</b><small>{$_("set.usage.sub.audio")}</small></span><span class="udot" aria-hidden="true">·</span>
        <span><b data-testid="usage-total">{data.estimated ? "≈ " : ""}{money(data.totalCostUsdMicros)}</b><small>{$_("set.usage.sub.cost")}{data.estimated ? " · " + $_("common.estimate") : ""}</small></span>
        <span class="usp"></span><span class="hint" data-testid="usage-range" title="{data.weekStart} → {data.weekEnd}">{rangeText}</span>
      </div>
      <h5 class="ush">{$_("set.usage.byday")}</h5>
      <div class="wkw">
        <div class="wk">
          {#each data.days as d (d.date)}
            <div title="{d.date}: {money(d.costUsdMicros)}"><div class="st" style="height:{(d.audioMs / max) * 85}%"></div><em>{$_(dayKeys[weekdayIndex(d.date)])}</em></div>
          {/each}
        </div>
      </div>
      {#if !isLocal}
        <h5 class="ush">{$_("set.usage.byscope")}</h5>
        <div class="rw cp"><div>{$_("set.usage.scope.api")}</div><div class="rr"><b>{money(data.apiCostUsdMicros)}</b></div></div>
        <div class="rw cp"><div>{$_("set.usage.scope.playground")}</div><div class="rr"><b>{money(data.playgroundCostUsdMicros)}</b></div></div>
        {#if data.models.length > 0}
          <h5 class="ush">{$_("set.usage.bymodel")}</h5>
          {#each data.models as m (m.model)}
            <div class="rw cp"><div>{m.model}<small>{requestsText(m.requests)}</small></div><div class="rr"><b>{money(m.costUsdMicros)}</b></div></div>
          {/each}
        {/if}
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
