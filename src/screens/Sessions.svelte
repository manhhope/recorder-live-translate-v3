<script lang="ts">
  import { onMount } from "svelte";
  import { _, formatDate, currentLocale, locale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import AskAiPopup from "../ai/AskAiPopup.svelte";
  import TermContext from "../lib/TermContext.svelte";
  import { DEFAULT_DST } from "../lib/langs.ts";
  import { app, fmtClock, openAiSettings } from "../lib/app.svelte.ts";
  import * as api from "../lib/api.ts";
  import type { SessionRow, SessionDetail, SearchOut } from "../lib/api.ts";

  let rows = $state<SessionRow[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);
  let q = $state("");
  let hits = $state<SearchOut | null>(null);
  let detail = $state<SessionDetail | null>(null);
  let caps = $state<api.CaptureRow[]>([]);
  let drive = $state<api.DriveOut | null>(null);
  let renaming = $state<string | null>(null);
  let confirmDel = $state<string | null>(null);
  let askOpen = $state(false);

  const loc = $derived(($locale as "en" | "pt-PT") ?? currentLocale());
  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
  const linesLbl = (n: number) => $_("ses.lines", { values: { n } });
  const driveLbl = (st: string) => { const k = api.driveKeyOf(st); return $_(k) === k ? $_("ses.drive.unknown") : $_(k); };
  const dur = (ms: number | null) => (ms == null ? "--:--:--" : fmtClock(Math.floor(ms / 1000)));
  const when = (iso: string) => formatDate(Date.parse(iso), loc, { day: "2-digit", month: "2-digit" });
  const meta = (s: SessionRow) => [when(s.startedAt), dur(s.durationMs), s.targetLang].join(" · ");
  const BADGE: Record<string, { cls: string; icon: string; key: string }> = {
    completed: { cls: "ok", icon: "check", key: "ses.saved" },
    recording: { cls: "warn", icon: "refresh", key: "ses.recording" },
    error: { cls: "rec", icon: "warn", key: "ses.saveerr" },
  };
  const badge = (s: SessionRow) => BADGE[s.status] ?? BADGE.completed;

  async function load() {
    loading = true; error = null;
    try { rows = await api.sessionsList(); } catch (e) { error = api.errCode(e); }
    loading = false;
  }
  onMount(load);

  let searchSeq = 0;
  $effect(() => {
    const term = q.trim();
    const my = ++searchSeq;
    if (!term) { hits = null; return; }
    api.recordsSearch(term).then(r => { if (my === searchSeq) hits = r; }).catch(e => { if (my === searchSeq) error = api.errCode(e); });
  });

  async function open(id: string) {
    error = null; notice = null; drive = null; askOpen = false;
    try { detail = await api.sessionGet(id); } catch (e) { error = api.errCode(e); }
    caps = [];
    try { caps = await api.sessionCaptures(id); } catch { /* chi tiet van hien du khi khong doc duoc anh */ }
  }
  async function guard(fn: () => Promise<void>) {
    error = null;
    try { await fn(); } catch (e) { error = api.errCode(e); }
  }
  const exportAs = (f: "md" | "txt" | "srt") => guard(async () => {
    if (!detail) return;
    const p = await api.sessionExport(detail.session.sessionId, f, loc);
    notice = p ? $_("ses.exported") : null;
  });
  const star = (lineId: number) => guard(async () => {
    if (!detail) return;
    await api.highlightToggle(detail.session.sessionId, lineId);
    detail = await api.sessionGet(detail.session.sessionId);
  });
  const sync = () => guard(async () => { if (detail) drive = await api.driveSync(detail.session.sessionId); });
  const rename = (id: string, title: string) => guard(async () => {
    await api.sessionRename(id, title.trim()); renaming = null;
    await load();
    if (detail?.session.sessionId === id) detail = await api.sessionGet(id);
  });
  const remove = (id: string) => guard(async () => {
    await api.sessionDelete(id); confirmDel = null;
    if (detail?.session.sessionId === id) detail = null;
    await load();
  });
  const importNet = () => guard(async () => {
    const r = await api.importDotnetPick();
    if (r) { notice = $_("ses.import.done", { values: { n: r.totalRows } }); await load(); }
  });
</script>

{#if detail}
  {@const d = detail}
  <div class="tool">
    <div class="tl">
      <button type="button" class="btn" onclick={() => (detail = null)}><Icon name="arrow" />{$_("ses.back")}</button>
      <b>{d.session.title}</b>
    </div>
    <div class="tr2">
      {#each ["md", "txt", "srt"] as const as f (f)}
        <button type="button" class="btn sm" onclick={() => exportAs(f)}><Icon name="download" />{f.toUpperCase()}</button>
      {/each}
      <button type="button" class="btn sm" aria-pressed={askOpen} onclick={() => (askOpen = !askOpen)}><Icon name="sparkle" />{$_("nav.askai")}</button>
      <button type="button" class="btn sm" onclick={sync}><Icon name="refresh" />{$_("ses.drive.sync")}</button>
    </div>
  </div>
  {#if error}<div class="card" role="alert" style="margin:8px 18px"><span class="bd rec"><Icon name="warn" />{$_(errKey(error))}</span></div>{/if}
  {#if notice}<div class="hint" style="padding:0 18px">{notice}</div>{/if}
  <div class="hint" style="padding:0 18px">
    {meta(d.session)} {"·"} {linesLbl(d.session.lines)} {"·"}
    {driveLbl(drive ? drive.status : d.session.syncStatus)}
    {#if !(drive?.oauthConnected)} {"·"} {$_("ses.drive.nooauth")}{/if}
  </div>
  {#if askOpen}
    <div class="askdock">
      {#key d.session.sessionId}
        <AskAiPopup sessionId={d.session.sessionId} onOpenSettings={openAiSettings} onClose={() => (askOpen = false)} />
      {/key}
    </div>
  {/if}
  {#if caps.length}
    <div class="caps" aria-label={$_("ses.captures")}>
      <b>{$_("ses.captures")}</b>
      {#each caps as c (c.screenCaptureId)}
        <figure class="cap">
          {#if c.thumbDataUrl}<img src={c.thumbDataUrl} alt={fmtClock(Math.floor(c.offsetMs / 1000))} />{/if}
          <figcaption class="ts">{fmtClock(Math.floor(c.offsetMs / 1000))}</figcaption>
        </figure>
      {/each}
    </div>
  {/if}
  {#if d.lines.length}
    <TermContext dstLang={d.session.targetLang || DEFAULT_DST}>
    <div class="tr">
      {#each d.lines as l (l.captionLineId)}
        <div class="ln" data-lang={l.sourceLang ?? ""}>
          <span class="ts">{fmtClock(Math.floor(l.offsetMs / 1000))}</span>
          <div class="tx">
            {#if l.lineType === "gap"}<div class="meta"><span class="pv">{$_("ses.gap")}</span></div>
            {:else}
              <div class="meta"><b>{$_(l.source === "mic" ? "src.name.mic" : "src.name.sys")}</b><span class="lg">{l.sourceLang ?? ""}</span>
                <button type="button" class="btn sm icon" aria-pressed={l.highlighted} aria-label={$_("ses.highlight")} title={$_("ses.highlight")} onclick={() => star(l.captionLineId)}><Icon name={l.highlighted ? "check" : "star"} /></button></div>
              <div class="src">{l.sourceText ?? ""}</div>
              {#if l.translatedText}<div class="dst">{l.translatedText}</div>{/if}
              {#if l.note}<div class="hint">{l.note}</div>{/if}
            {/if}
          </div>
        </div>
      {/each}
    </div>
    </TermContext>
  {:else}
    <div class="empty"><Icon name="list" /><span>{$_("ses.nolines")}</span></div>
  {/if}
{:else}
  <div class="tool">
    <div class="tl"><label class="fld"><Icon name="search" /><input type="search" placeholder={$_("ses.search")} aria-label={$_("ses.search")} bind:value={q} /></label></div>
    <div class="tr2">
      <button type="button" class="btn" onclick={importNet}><Icon name="download" />{$_("ses.import")}</button>
      <button type="button" class="btn primary" onclick={() => (app.screen = "rec")}><Icon name="plus" />{$_("tool.new")}<kbd>{"⌘N"}</kbd></button>
    </div>
  </div>
  {#if error}
    <div class="empty"><Icon name="warn" /><b>{$_(errKey(error))}</b><button type="button" class="btn sm" onclick={load}><Icon name="refresh" />{$_("common.retry")}</button></div>
  {:else if loading}
    <div class="empty"><Icon name="refresh" /><span>{$_("common.loading")}</span></div>
  {:else if hits}
    <div style="padding:6px 18px">
      {#each hits.lines as h (h.captionLineId)}
        <button type="button" class="sesrow" style="width:100%;text-align:left;background:none;border:0;border-bottom:1px solid var(--rlt-line)" onclick={() => open(h.sessionId)}>
          <div><b>{h.snippet}</b><small>{fmtClock(Math.floor(h.offsetMs / 1000))}</small></div>
        </button>
      {/each}
      {#each hits.terms as t (t.id)}<div class="sesrow"><div><b>{t.termSource}</b><small>{t.termTarget ?? ""}</small></div></div>{/each}
      {#if !hits.lines.length && !hits.terms.length && !hits.notes.length}
        <div class="empty"><Icon name="search" /><span>{$_("ses.noresults")}</span></div>
      {/if}
    </div>
  {:else if rows.length}
    {#if notice}<div class="hint" style="padding:0 18px">{notice}</div>{/if}
    <div style="padding:6px 18px">
      {#each rows as s (s.sessionId)}
        {@const b = badge(s)}
        <div class="sesrow">
          {#if renaming === s.sessionId}
            <label class="fld grow"><input value={s.title} aria-label={$_("ses.rename")}
              onkeydown={(e) => { if (e.key === "Enter") void rename(s.sessionId, e.currentTarget.value); if (e.key === "Escape") renaming = null; }} /></label>
          {:else}
            <button type="button" style="background:none;border:0;text-align:left;cursor:pointer;color:inherit;font:inherit" onclick={() => open(s.sessionId)}>
              <b>{s.title}</b><small>{meta(s)} {"·"} {linesLbl(s.lines)}</small>
            </button>
          {/if}
          <div class="rr">
            <span class="bd {b.cls}"><Icon name={b.icon} />{$_(b.key)}</span>
            <button type="button" class="btn sm" onclick={() => (renaming = s.sessionId)}>{$_("ses.rename")}</button>
            {#if confirmDel === s.sessionId}
              <button type="button" class="btn sm stop" onclick={() => remove(s.sessionId)}>{$_("ses.delete.confirm")}</button>
              <button type="button" class="btn sm" onclick={() => (confirmDel = null)}>{$_("common.cancel")}</button>
            {:else}
              <button type="button" class="btn sm icon" aria-label={$_("common.delete")} title={$_("common.delete")} onclick={() => (confirmDel = s.sessionId)}><Icon name="trash" /></button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty"><Icon name="clock" /><span>{$_("ses.empty")}</span></div>
  {/if}
{/if}

<style>
  .caps { display: flex; flex-wrap: wrap; gap: 10px; align-items: flex-start; padding: 8px 18px; }
  .caps > b { flex: 0 0 100%; font-size: 12px; }
  .cap { margin: 0; text-align: center; }
  .cap img { display: block; max-width: 160px; max-height: 100px; border-radius: 6px; border: 1px solid var(--rlt-line); }
  .askdock{display:flex;justify-content:center;height:480px;flex:none;padding:8px 18px}
  .askdock :global(.ask){width:min(720px,100%)}
</style>
