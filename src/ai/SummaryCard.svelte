<script lang="ts">
  // Tom tat cuoc ghi (US-63, US-64), nam trong cua so Hoi AI. Moi chu cua mo hinh hien bang van ban thuan ({...} cua
  // Svelte, tuc textContent), KHONG dung {@html}. Nha cung cap va hop thoai dong y do AskAiPopup quan ly.
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { formatNumber } from "../i18n/format.ts";
  import * as api from "./api.ts";
  import * as sm from "./summary.ts";
  import type { Provider } from "./api.ts";
  import type { SummaryOutcome, SummaryParts, SummaryPlan, SummaryProgress, TargetLang } from "./summary.ts";

  let { sessionId, provider, lang, canRun, onBack }: {
    sessionId: string; provider: Provider | null; lang: TargetLang; canRun: boolean; onBack: () => void;
  } = $props();

  let translate = $state(false);
  let plan = $state<SummaryPlan | null>(null);
  let planErr = $state<string | null>(null);
  let busy = $state(false);
  let progress = $state<SummaryProgress | null>(null);
  let outcome = $state<SummaryOutcome | null>(null);
  let errCode = $state<string | null>(null);

  const target = $derived(sm.translateTarget(lang));
  const notices = $derived(plan ? sm.cutNotices(plan) : []);
  const tk = $derived(target === "pt-PT" ? "pt" : "en");
  const num = (n: number) => formatNumber(n, lang);
  const estimateText = $derived(plan ? $_("sum.estimate", { values: { chars: num(plan.sent_chars), calls: plan.calls, chunks: plan.chunks } }) : "");
  const noticeTexts = $derived(notices.map(n => ({ key: n.key, text: $_(n.key, { values: { total: num(n.values.total ?? 0), cap: num(n.values.cap ?? 0), n: n.values.n ?? 0 } }) })));
  const progressText = $derived(progress ? $_(sm.progressKey(progress.stage), { values: { i: progress.index, n: progress.total } }) : "");
  const partialText = $derived(outcome?.failed_at ? $_("sum.partial", { values: { i: outcome.failed_at } }) : "");
  const translateLabel = $derived($_(`sum.translate.${tk}`));
  const translatedTitle = $derived($_(`sum.translated.title.${tk}`));

  async function loadPlan() {
    planErr = null;
    try { plan = await sm.summaryPlan(sessionId, translate); }
    catch (e) { plan = null; planErr = String(e); }
  }
  async function run() {
    if (busy || !canRun) return;
    busy = true; errCode = null; outcome = null; progress = null;
    try {
      outcome = await sm.summarize(sessionId, provider, lang, translate ? target : null, p => (progress = p));
      if (outcome.status !== "completed") errCode = outcome.error_code ?? outcome.status;
    } catch (e) { errCode = String(e); }
    finally { busy = false; progress = null; }
  }
  const cancel = () => { void api.cancelAsk(); };

  onMount(() => { void loadPlan(); });
</script>

<section class="sum" aria-label={$_("sum.title")} data-testid="sum-card">
  <header class="hd">
    <b>{$_("sum.title")}</b>
    <button type="button" class="btn sm ghost" onclick={onBack} disabled={busy}>{$_("sum.back")}</button>
  </header>

  {#if planErr}
    <div class="err" role="alert">{planErr === "invalid_input" ? $_("sum.empty") : $_("sum.planerr")}</div>
  {:else if plan}
    <p class="hint" data-testid="sum-estimate">{estimateText}</p>
    {#each noticeTexts as n (n.key)}
      <p class="warn" role="status" data-testid={n.key}>{n.text}</p>
    {/each}
    <label class="opt"><input type="checkbox" bind:checked={translate} disabled={busy} onchange={loadPlan} />{translateLabel}</label>
  {/if}

  <div class="acts">
    {#if busy}
      <button type="button" class="btn sm danger" onclick={cancel}>{$_("common.cancel")}</button>
      {#if progress}<span class="hint" role="status">{progressText}</span>{/if}
    {:else}
      <button type="button" class="btn sm primary" disabled={!canRun || !plan} onclick={run}>{$_("sum.run")}</button>
    {/if}
  </div>

  {#if errCode}
    <div class="err" role="alert">
      {outcome?.status === "cancelled" ? $_("sum.cancelled") : $_(api.errKey(errCode, provider))}
      {#if outcome && outcome.status !== "cancelled" && outcome.failed_at}{partialText}{/if}
    </div>
  {/if}

  {#snippet block(p: SummaryParts, title: string, tag: string)}
    <div class="res" data-testid={tag}>
      {#if title}<h4>{title}</h4>{/if}
      <h5>{$_("sum.h.summary")}</h5>
      <p class="txt" data-testid="sum-summary">{p.summary}</p>
      <h5>{$_("sum.h.points")}</h5>
      {#if p.key_points.length}<ul data-testid="sum-points">{#each p.key_points as k, i (i)}<li>{k}</li>{/each}</ul>{:else}<p class="hint">{$_("sum.none")}</p>{/if}
      <h5>{$_("sum.h.actions")}</h5>
      {#if p.action_items.length}<ul data-testid="sum-actions">{#each p.action_items as k, i (i)}<li>{k}</li>{/each}</ul>{:else}<p class="hint">{$_("sum.none")}</p>{/if}
    </div>
  {/snippet}

  {#if outcome?.parts}{@render block(outcome.parts, "", "sum-result")}{/if}
  {#if outcome?.translated}
    <p class="hint" data-testid="sum-ai-label">{$_("sum.translated.label")}</p>
    {@render block(outcome.translated, translatedTitle, "sum-translated")}
  {/if}
</section>

<style>
  .sum{display:flex;flex-direction:column;gap:8px;flex:1;overflow:auto;min-height:0}
  .hd{display:flex;justify-content:space-between;align-items:center;gap:8px}
  .acts{display:flex;gap:8px;align-items:center}
  .opt{display:flex;gap:6px;align-items:center;font-size:var(--rlt-fs)}
  .warn{margin:0;padding:6px 8px;border:1px solid var(--rlt-warn);border-radius:var(--rlt-rb);font-size:12px}
  .err{color:var(--rlt-rec);font-size:12px}
  .res{border:1px solid var(--rlt-line);border-radius:var(--rlt-rb);padding:8px 10px}
  .res h4{margin:0 0 4px}
  .res h5{margin:8px 0 2px;color:var(--rlt-text2)}
  .txt{white-space:pre-wrap;overflow-wrap:anywhere;margin:0}
  ul{margin:0;padding-left:18px;overflow-wrap:anywhere}
</style>
