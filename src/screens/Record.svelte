<script lang="ts">
  import { _, formatNumber } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import LangSelect from "../lib/LangSelect.svelte";
  import SourceButton from "../lib/SourceButton.svelte";
  import { app, fmtClock, type Level } from "../lib/app.svelte.ts";
  import { LINES } from "../lib/mock.ts";

  const sysLevel: Level = $derived(app.recording ? "talk" : "idle");
  const micLevel: Level = $derived(app.recording ? "quiet" : "idle");
  let timer: ReturnType<typeof setInterval> | undefined;

  function toggle() {
    app.recording = !app.recording;
    clearInterval(timer);
    if (app.recording) { app.seconds = 768; timer = setInterval(() => app.seconds++, 1000); }
  }
  $effect(() => () => clearInterval(timer));
  const views = [["both", "rec.view.both"], ["dst", "rec.view.dst"], ["src", "rec.view.src"]] as const;
  const statusKey = $derived(app.recording ? "listening" : "ready");
</script>

<div class="tool">
  <div class="tl">
    <button type="button" class="btn"><Icon name="plus" />{$_("tool.new")}<kbd>{"⌘N"}</kbd></button>
    {#if app.recording}
      <button type="button" class="btn stop" onclick={toggle}><Icon name="stop" />{$_("tool.stop")}<kbd>{"⌘."}</kbd></button>
      <span class="tm">{fmtClock(app.seconds)}</span>
    {:else}
      <button type="button" class="btn primary" onclick={toggle}><Icon name="play" />{$_("tool.start")}<kbd>{"⌘."}</kbd></button>
    {/if}
  </div>
  <div class="tr2">
    <span class="bd {app.recording ? 'rec' : ''}"><Icon name="dot" />{$_(app.recording ? "tool.badge.rec" : "status.ready")}</span>
    <button type="button" class="btn"><Icon name="sparkle" />{$_("nav.askai")}<kbd>{"⌘⇧A"}</kbd></button>
  </div>
</div>

<div class="cfg {app.recording ? 'lock' : ''}" role="group" aria-label={$_("cfg.group.aria")}>
  <div class="cg lg2" role="group" aria-label={$_("cfg.lang.group")}>
    <span class="seg" role="radiogroup" aria-label={$_("cfg.mode.group")}>
      <button type="button" class="on" role="radio" aria-checked="true" aria-label={$_("cfg.mode.one.aria")} data-tip={$_("cfg.mode.one.tip")}>{$_("cfg.mode.one")}</button>
      <button type="button" class="off" role="radio" aria-checked="false" aria-disabled="true" aria-label={$_("cfg.mode.two.aria")} data-tip={$_("cfg.mode.two.tip")}>{$_("cfg.mode.two")}<Icon name="lock" /></button>
    </span>
    <span class="lsub">
      <LangSelect bind:value={app.src} withAuto cls="sw1" locked={app.recording} ariaKey="cfg.src.aria" tipKey="cfg.src.tip" />
      <span class="ar2" role="img" aria-label={$_("cfg.arrow.aria")} data-tip={$_("cfg.arrow.tip")}><Icon name="arrow" /></span>
      <LangSelect bind:value={app.dst} short cls="sw2" locked={app.recording} ariaKey="cfg.dst.aria" tipKey="cfg.dst.tip" />
    </span>
  </div>
  <span class="vsep" role="separator" aria-orientation="vertical"></span>
  <div class="cg sr" role="group" aria-label={$_("cfg.audio.group")}>
    <SourceButton kind="sys" bind:on={app.sysOn} level={sysLevel} />
    <SourceButton kind="mic" bind:on={app.micOn} level={micLevel} />
  </div>
</div>

{#if app.recording}
  <div class="rowx">
    <span class="seg" role="group" aria-label={$_("rec.view.group")}>
      {#each views as [v, key] (v)}
        <button type="button" class={app.view === v ? "on" : ""} aria-pressed={app.view === v} onclick={() => (app.view = v)}>{$_(key)}</button>
      {/each}
    </span>
  </div>
  <div class="tr">
    {#each LINES as l (l.ts)}
      <div class="ln {l.provisional ? 'prov' : ''}">
        <span class="ts">{l.ts}</span>
        <div class="tx">
          <div class="meta"><b>{$_("src.name.sys")}</b><span class="lg">{l.lang}</span>{#if l.provisional}<span class="pv">{$_("ln.provisional")}</span>{/if}</div>
          {#if app.view !== "dst"}<div class="src">{$_(l.src)}</div>{/if}
          {#if app.view !== "src"}<div class="dst">{$_(l.dst)}</div>{/if}
        </div>
      </div>
    {/each}
    <div class="ln live"><span class="ts">{fmtClock(app.seconds)}</span>
      <div class="tx"><div class="meta"><b>{$_("src.name.sys")}</b><span class="pv">{$_("ln.listening")}</span></div><div class="dst">...</div></div>
    </div>
  </div>
{:else}
  <div class="empty"><Icon name="mic" /><b>{$_("rec.empty.title")}</b><span>{$_("rec.empty.hint")}</span></div>
{/if}

<div class="foot">
  <span class="sb {app.recording ? 'ok' : ''}"><Icon name={app.recording ? "mic" : "dot"} /><b>{$_("status." + statusKey)}</b></span>
  <span class="fs">{$_("foot.quota")}</span>
</div>
