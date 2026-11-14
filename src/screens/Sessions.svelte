<script lang="ts">
  import { _, formatDate, currentLocale, locale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import { app } from "../lib/app.svelte.ts";
  import { SESSIONS, type SessionStatus } from "../lib/mock.ts";

  let q = $state("");
  const loc = $derived(($locale as "en" | "pt-PT") ?? currentLocale());
  const rows = $derived(SESSIONS.filter(s => !q || $_(s.title).toLowerCase().includes(q.toLowerCase())));
  const meta = (s: (typeof SESSIONS)[number]) =>
    [formatDate(s.date, loc, { day: "2-digit", month: "2-digit", timeZone: "UTC" }), s.dur, `${s.src} \u2192 ${s.dst}`].join(" \u00b7 ");
  const badge: Record<SessionStatus, { cls: string; icon: string; key: string }> = {
    saved: { cls: "ok", icon: "check", key: "ses.saved" },
    saving: { cls: "warn", icon: "refresh", key: "ses.saving" },
    error: { cls: "rec", icon: "warn", key: "ses.saveerr" },
  };
</script>

<div class="tool">
  <div class="tl"><label class="fld"><Icon name="search" /><input type="search" placeholder={$_("ses.search")} aria-label={$_("ses.search")} bind:value={q} /></label></div>
  <div class="tr2"><button type="button" class="btn primary" onclick={() => (app.screen = "rec")}><Icon name="plus" />{$_("tool.new")}<kbd>{"⌘N"}</kbd></button></div>
</div>
{#if rows.length}
  <div style="padding:6px 18px">
    {#each rows as s (s.id)}
      {@const b = badge[s.status]}
      <div class="sesrow">
        <div><b>{$_(s.title)}</b><small>{meta(s)}</small></div>
        <div class="rr"><span class="bd {b.cls}"><Icon name={b.icon} />{$_(b.key)}</span></div>
      </div>
    {/each}
  </div>
{:else}
  <div class="empty"><Icon name="clock" /><span>{$_("ses.empty")}</span></div>
{/if}
