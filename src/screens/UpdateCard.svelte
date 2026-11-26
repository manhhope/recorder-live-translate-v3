<script lang="ts">
  // The Cap nhat (US-33): cong tat tu kiem tra, nut Check, phien ban hien tai/moi, ghi chu, tien do tai, "Cai va khoi dong lai".
  // Trang thai nam o lib/update.svelte.ts (kiem/tai chay nen khi khong mo man nay).
  import { _, formatTime, currentLocale, locale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import { upd, checkNow, download, install, setAutoUpdate, installBlocked } from "../lib/update.svelte.ts";
  import { APP_VERSION } from "../lib/whats-new.svelte.ts";
  import { isWindows } from "../lib/platform.ts";

  const busy = $derived(upd.phase === "checking" || upd.phase === "downloading" || upd.phase === "installing");
  const errKey = $derived(upd.error ? api.errKeyOf(upd.error, k => $_(k) !== k) : null);
  const blocked = $derived(installBlocked());
  const pct = $derived(upd.total ? Math.min(100, Math.round((upd.downloaded / upd.total) * 100)) : null);
  const mb = (n: number) => (n / 1048576).toFixed(1) + " MB";
  const localTz = Intl.DateTimeFormat().resolvedOptions().timeZone;
  const last = $derived(upd.lastCheck ? formatTime(upd.lastCheck, ($locale as "en" | "pt-PT") ?? currentLocale(), localTz) : "");
  const tDownloading = $derived(pct !== null ? $_("set.upd.downloading", { values: { pct: pct + "%" } }) : $_("set.upd.downloadingnototal", { values: { size: mb(upd.downloaded) } }));
  const tReady = $derived($_("set.upd.ready", { values: { version: upd.info?.version ?? "" } }));
  const tLast = $derived($_("set.upd.lastcheck", { values: { time: last } }));
  const retry = () => (upd.failedStep === "download" ? download() : upd.failedStep === "install" ? install() : checkNow());
</script>

<div class="card" data-testid="update-card">
  <h4>{$_("set.upd.title")}</h4>
  <div class="rw">
    <div><b>{$_("set.upd.auto")}</b><small>{$_("set.upd.autohint")}</small></div>
    <div class="rr">
      <input type="checkbox" class="chk" role="switch" data-testid="update-auto" aria-label={$_("set.upd.auto")} checked={upd.auto} onchange={(e) => setAutoUpdate(e.currentTarget.checked)} />
    </div>
  </div>
  <div class="rw">
    <b>{$_("set.upd.current")}</b>
    <div class="rr"><code data-testid="update-current">{APP_VERSION}</code></div>
  </div>
  {#if upd.info && upd.phase !== "uptodate"}
    <div class="rw">
      <b>{$_("set.upd.new")}</b>
      <div class="rr"><code data-testid="update-new">{upd.info.version}</code></div>
    </div>
    <div class="hint" style="margin:0 0 8px" data-testid="update-notes"><b>{$_("set.upd.notes")}</b><br />{upd.info.notes?.trim() || $_("set.upd.nonotes")}</div>
  {/if}

  {#if upd.phase === "downloading"}
    <div class="hint" style="margin:0" data-testid="update-progress" role="status">
      {tDownloading}
    </div>
    {#if pct !== null}<progress max="100" value={pct} aria-label={tDownloading} style="width:100%"></progress>{/if}
  {:else if upd.phase === "ready"}
    <div class="hint" style="margin:0" role="status" data-testid="update-ready"><Icon name="check" /> {tReady}</div>
    {#if blocked}<div class="hint" style="margin:4px 0 0" data-testid="update-deferred"><Icon name="info" /> {$_("set.upd.deferred")}</div>{/if}
  {:else if upd.phase === "installing"}
    <div class="hint" style="margin:0" role="status">{$_("set.upd.installing")}</div>
  {:else if upd.phase === "uptodate"}
    <div class="hint" style="margin:0" role="status" data-testid="update-uptodate"><Icon name="check" /> {$_("set.upd.uptodate")}</div>
  {/if}

  <div class="tl2" style="margin-top:8px">
    {#if upd.phase === "available"}
      <button type="button" class="btn sm primary" data-testid="update-download" onclick={download}><Icon name="refresh" />{$_("set.upd.download")}</button>
    {:else if upd.phase === "ready" || (upd.phase === "error" && upd.failedStep === "install")}
      <button type="button" class="btn sm primary" data-testid="update-install" disabled={blocked} onclick={install}><Icon name="refresh" />{$_("set.upd.install")}</button>
    {/if}
    {#if upd.phase === "error"}
      <button type="button" class="btn sm" data-testid="update-retry" onclick={retry}>{$_("set.upd.retry")}</button>
    {/if}
    <button type="button" class="btn sm" data-testid="update-check" disabled={busy || upd.phase === "ready"} onclick={() => checkNow()}>
      <Icon name="refresh" />{upd.phase === "checking" ? $_("set.upd.checking") : $_("set.upd.check")}
    </button>
    {#if last && upd.phase !== "error"}<span class="hint" style="margin:0">{tLast}</span>{/if}
    {#if errKey}<span class="bd rec" role="alert" data-testid="update-error"><Icon name="warn" />{$_(errKey)}</span>{/if}
  </div>
  {#if !isWindows()}<div class="hint" style="margin:8px 0 0" data-testid="update-maccaveat"><Icon name="info" /> {$_("set.upd.maccaveat")}</div>{/if}
</div>
