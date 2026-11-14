<script lang="ts">
  // Man "Hoi AI" ngay trong cua so chinh (ADR-10: khong can cua so rieng). Phim tat/deeplink ask_ai, thanh ben
  // va nut o man Ghi deu dan toi day. Chat that nam o AskAiPopup; man nay chi chon phien va xu ly truong hop trong.
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import AskAiPopup from "../ai/AskAiPopup.svelte";
  import { app, openAiSettings } from "../lib/app.svelte.ts";
  import * as api from "../lib/api.ts";
  import type { SessionRow } from "../lib/api.ts";

  let rows = $state<SessionRow[]>([]);
  let loading = $state(true);
  let failed = $state(false);

  onMount(async () => {
    try { rows = await api.sessionsList(); } catch { failed = true; }
    // Chua chon phien nao (vd phim tat khi chua ghi): lay phien moi nhat.
    if (!app.askSession) app.askSession = rows[0]?.sessionId ?? null;
    loading = false;
  });
</script>

<div class="tool">
  <div class="tl"><b>{$_("nav.askai")}</b></div>
  {#if rows.length}
    <div class="tr2">
      <label class="fld">
        <span class="hint">{$_("ask.screen.pick")}</span>
        <select aria-label={$_("ask.screen.pick")} bind:value={app.askSession}>
          {#each rows as r (r.sessionId)}<option value={r.sessionId}>{r.title}</option>{/each}
        </select>
      </label>
    </div>
  {/if}
</div>

{#if loading}
  <div class="empty"><Icon name="refresh" /><span>{$_("common.loading")}</span></div>
{:else if app.askSession}
  <div class="askpane">
    {#key app.askSession}
      <AskAiPopup sessionId={app.askSession} onOpenSettings={openAiSettings} onClose={() => (app.screen = "rec")} />
    {/key}
  </div>
{:else}
  <div class="empty" data-testid="ask-empty">
    <Icon name={failed ? "warn" : "sparkle"} />
    <b>{$_(failed ? "ask.screen.error" : "ask.screen.nosession.title")}</b>
    <span>{$_("ask.screen.nosession.hint")}</span>
    <button type="button" class="btn" onclick={() => (app.screen = "rec")}><Icon name="mic" />{$_("ask.screen.gorecord")}</button>
  </div>
{/if}

<style>
  .askpane{flex:1;min-height:0;display:flex;justify-content:center;padding:12px 18px}
  .askpane :global(.ask){width:min(720px,100%)}
</style>
