<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import LangSelect from "../lib/LangSelect.svelte";
  import SourceButton from "../lib/SourceButton.svelte";
  import { app, fmtClock, type Level } from "../lib/app.svelte.ts";
  import { AUTO } from "../lib/langs.ts";
  import * as api from "../lib/api.ts";
  import type { LineDto, SessionStateName, StatusCode } from "../lib/api.ts";

  let state = $state<SessionStateName>("idle");
  let lines = $state<LineDto[]>([]);
  let level = $state(0);
  let status = $state<StatusCode | null>(null);
  let errCode = $state<string | null>(null);
  let hasKey = $state(true);
  let consent = $state(true);
  let askConsent = $state(false);
  let askKey = $state(false);
  let keyInput = $state("");
  let busy = $state(false);
  let loaded = $state(false);
  let lastStop = $state<{ lines: number; drained: boolean } | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;

  const active = $derived(state !== "idle");
  const mode = $derived<api.RecordMode | null>(app.sysOn && app.micOn ? "mixed" : app.sysOn ? "system" : app.micOn ? "mic" : null);
  const lvl = $derived<Level>(!active || state === "paused" ? "idle" : level > 0.9 ? "clip" : level > 0.03 ? "talk" : "quiet");
  const sysLevel = $derived<Level>(app.sysOn ? (status === "permission" && active ? "none" : lvl) : "idle");
  const micLevel = $derived<Level>(app.micOn ? lvl : "idle");
  const views = [["both", "rec.view.both"], ["dst", "rec.view.dst"], ["src", "rec.view.src"]] as const;
  const STATUS_KEY: Record<StatusCode, string> = {
    permission: "status.perm", auth: "status.badkey", quota: "status.quota", network: "status.offline", timeout: "status.timeout",
  };
  const statusKey = $derived(
    status ? STATUS_KEY[status] : state === "paused" ? "status.paused" : state === "stopping" ? "status.stopping"
      : state === "recording" ? (level > 0.03 ? "status.listening" : "status.silent") : "status.ready",
  );
  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);

  function setState(s: SessionStateName) {
    state = s;
    app.recording = s !== "idle";
    clearInterval(timer);
    if (s === "recording") timer = setInterval(() => app.seconds++, 1000);
  }

  function onEvent(e: api.UiEvent) {
    if (e.kind === "state") {
      if (e.state === "recording" && state === "idle") { app.seconds = 0; level = 0; }
      setState(e.state);
    } else if (e.kind === "line") {
      const i = lines.findIndex(l => l.captionLineId === e.line.captionLineId);
      if (i >= 0) lines[i] = e.line; else lines.push(e.line);
    } else if (e.kind === "level") level = e.level;
    else if (e.kind === "status") status = e.code;
    else if (e.kind === "stopped") lastStop = { lines: e.lines, drained: e.drained };
  }

  onMount(() => {
    (async () => {
      try {
        const [p, k, c, st] = await Promise.all([api.prefsGet(), api.hasApiKey(), api.consentGet(), api.sessionState()]);
        app.dst = p.targetLang;
        app.src = p.listenLangs.length === 1 ? p.listenLangs[0] : AUTO;
        app.sysOn = p.mode !== "mic";
        app.micOn = p.mode !== "system";
        hasKey = k; consent = c;
        if (st[0] !== "idle") setState(st[0] as SessionStateName);
      } catch (e) { errCode = api.errCode(e); }
      loaded = true;
    })();
    return () => clearInterval(timer);
  });

  async function begin() {
    if (!mode) return;
    busy = true; errCode = null; status = null; lastStop = null;
    try {
      const listenLangs = app.src === AUTO ? [] : [app.src];
      await api.prefsSet({ targetLang: app.dst, listenLangs, mode, uiLanguage: document.documentElement.lang === "en" ? "en" : "pt-PT" });
      lines = [];
      await api.sessionStart({ targetLang: app.dst, listenLangs, mode }, api.newChannel(onEvent));
    } catch (e) {
      const c = api.errCode(e);
      if (c === "key_missing") { hasKey = false; askKey = true; }
      else if (c === "consent_required") { consent = false; askConsent = true; }
      else errCode = c;
    } finally { busy = false; }
  }

  async function start() {
    if (!hasKey) { askKey = true; return; }
    if (!consent) { askConsent = true; return; }
    await begin();
  }

  async function acceptConsent() {
    try { await api.consentAccept(); consent = true; askConsent = false; await start(); }
    catch (e) { errCode = api.errCode(e); }
  }

  async function saveKey() {
    const v = keyInput.trim();
    if (!v) return;
    try { await api.saveApiKey(v); keyInput = ""; hasKey = true; askKey = false; await start(); }
    catch (e) { errCode = api.errCode(e); }
  }

  async function act(fn: () => Promise<unknown>) {
    errCode = null;
    try { await fn(); } catch (e) { errCode = api.errCode(e); }
  }
  const stop = () => act(api.sessionStop);
  const togglePause = () => act(state === "paused" ? api.sessionResume : api.sessionPause);

  /** US-45: ghi dang chay thi dung roi xoa man hinh; dang rong thi chi xoa man hinh. */
  async function newSession() {
    await act(async () => { await api.sessionNew(); lines = []; lastStop = null; app.seconds = 0; });
  }
  function toggle() { if (active) void stop(); else void start(); }
  const stoppedLbl = (n: number) => $_("rec.stopped", { values: { n } });
  const text = (l: LineDto) => l.translatedText?.trim() ?? "";
</script>

<div class="tool">
  <div class="tl">
    <button type="button" class="btn" disabled={busy} onclick={newSession}><Icon name="plus" />{$_("tool.new")}<kbd>{"⌘N"}</kbd></button>
    {#if active}
      <button type="button" class="btn stop" disabled={state === "stopping"} onclick={toggle}><Icon name="stop" />{$_("tool.stop")}<kbd>{"⌘."}</kbd></button>
      <button type="button" class="btn" disabled={state === "stopping"} onclick={togglePause}>
        <Icon name={state === "paused" ? "play" : "pause"} />{$_(state === "paused" ? "tool.resume" : "tool.pause")}
      </button>
      <span class="tm">{fmtClock(app.seconds)}</span>
    {:else}
      <button type="button" class="btn primary" disabled={busy || !mode || !loaded} onclick={toggle}><Icon name="play" />{$_("tool.start")}<kbd>{"⌘."}</kbd></button>
    {/if}
  </div>
  <div class="tr2">
    <span class="bd {state === 'recording' ? 'rec' : ''}"><Icon name="dot" />{$_(state === "recording" ? "tool.badge.rec" : statusKey)}</span>
  </div>
</div>

<div class="cfg {active ? 'lock' : ''}" role="group" aria-label={$_("cfg.group.aria")}>
  <div class="cg lg2" role="group" aria-label={$_("cfg.lang.group")}>
    <span class="seg" role="radiogroup" aria-label={$_("cfg.mode.group")}>
      <button type="button" class="on" role="radio" aria-checked="true" aria-label={$_("cfg.mode.one.aria")} data-tip={$_("cfg.mode.one.tip")}>{$_("cfg.mode.one")}</button>
      <button type="button" class="off" role="radio" aria-checked="false" aria-disabled="true" aria-label={$_("cfg.mode.two.aria")} data-tip={$_("cfg.mode.two.tip")}>{$_("cfg.mode.two")}<Icon name="lock" /></button>
    </span>
    <span class="lsub">
      <LangSelect bind:value={app.src} withAuto cls="sw1" locked={active} ariaKey="cfg.src.aria" tipKey="cfg.src.tip" />
      <span class="ar2" role="img" aria-label={$_("cfg.arrow.aria")} data-tip={$_("cfg.arrow.tip")}><Icon name="arrow" /></span>
      <LangSelect bind:value={app.dst} short cls="sw2" locked={active} ariaKey="cfg.dst.aria" tipKey="cfg.dst.tip" />
    </span>
  </div>
  <span class="vsep" role="separator" aria-orientation="vertical"></span>
  <div class="cg sr" role="group" aria-label={$_("cfg.audio.group")}>
    <SourceButton kind="sys" bind:on={app.sysOn} level={sysLevel} />
    <SourceButton kind="mic" bind:on={app.micOn} level={micLevel} />
  </div>
</div>

{#if errCode}
  <div class="card" role="alert" style="margin:8px 18px"><span class="bd rec"><Icon name="warn" />{$_(errKey(errCode))}</span></div>
{/if}

{#if askConsent}
  <div class="card" role="dialog" aria-modal="true" aria-labelledby="consent-t" style="margin:8px 18px">
    <h4 id="consent-t">{$_("rec.consent.title")}</h4>
    <p>{$_("rec.consent.body")}</p>
    <div class="tl2">
      <button type="button" class="btn primary" onclick={acceptConsent}>{$_("rec.consent.accept")}</button>
      <button type="button" class="btn" onclick={() => (askConsent = false)}>{$_("common.cancel")}</button>
    </div>
  </div>
{/if}

{#if askKey}
  <div class="card" role="dialog" aria-modal="true" aria-labelledby="key-t" style="margin:8px 18px">
    <h4 id="key-t">{$_("rec.key.title")}</h4>
    <p>{$_("rec.key.body")}</p>
    <div class="tl2">
      <label class="fld grow"><Icon name="key" /><input type="password" autocomplete="off" aria-label={$_("set.soniox.key")} placeholder={$_("rec.key.placeholder")} bind:value={keyInput} /></label>
      <button type="button" class="btn primary" disabled={!keyInput.trim()} onclick={saveKey}>{$_("rec.key.save")}</button>
      <button type="button" class="btn" onclick={() => (askKey = false)}>{$_("common.cancel")}</button>
    </div>
  </div>
{/if}

{#if active || lines.length}
  <div class="rowx">
    <span class="seg" role="group" aria-label={$_("rec.view.group")}>
      {#each views as [v, key] (v)}
        <button type="button" class={app.view === v ? "on" : ""} aria-pressed={app.view === v} onclick={() => (app.view = v)}>{$_(key)}</button>
      {/each}
    </span>
    {#if lastStop}<span class="hint">{stoppedLbl(lastStop.lines)}{#if !lastStop.drained} {$_("rec.stopped.partial")}{/if}</span>{/if}
  </div>
  <div class="tr" aria-live="polite">
    {#each lines as l (l.captionLineId)}
      <div class="ln {text(l) ? '' : 'prov'}">
        <span class="ts">{fmtClock(Math.floor(l.offsetMs / 1000))}</span>
        <div class="tx">
          <div class="meta"><b>{$_(mode === "mic" ? "src.name.mic" : "src.name.sys")}</b><span class="lg">{l.sourceLang}</span>{#if !text(l)}<span class="pv">{$_("ln.provisional")}</span>{/if}</div>
          {#if app.view !== "dst"}<div class="src">{l.sourceText}</div>{/if}
          {#if app.view !== "src" && text(l)}<div class="dst">{text(l)}</div>{/if}
        </div>
      </div>
    {/each}
    {#if active}
      <div class="ln live"><span class="ts">{fmtClock(app.seconds)}</span>
        <div class="tx"><div class="meta"><b>{$_(mode === "mic" ? "src.name.mic" : "src.name.sys")}</b><span class="pv">{$_("ln.listening")}</span></div><div class="dst">...</div></div>
      </div>
    {/if}
  </div>
{:else}
  <div class="empty"><Icon name="mic" /><b>{$_("rec.empty.title")}</b><span>{$_(!mode ? "rec.empty.nosrc" : "rec.empty.hint")}</span></div>
{/if}

<div class="foot">
  <span class="sb {state === 'recording' && !status ? 'ok' : ''}"><Icon name={active ? "mic" : "dot"} /><b>{$_(statusKey)}</b></span>
  {#if !hasKey && loaded}<span class="fs">{$_("rec.key.missing")}</span>{/if}
</div>
