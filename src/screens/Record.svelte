<script lang="ts">
  import { platformize } from "../lib/platform.ts";
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import LangSelect from "../lib/LangSelect.svelte";
  import SourceButton from "../lib/SourceButton.svelte";
  import TermContext from "../lib/TermContext.svelte";
  import TranscriptPrefs from "../lib/TranscriptPrefs.svelte";
  import { cssVars, loadPrefs, savePrefs } from "../lib/transcript-prefs.ts";
  import { app, fmtClock, openAsk, type Level } from "../lib/app.svelte.ts";
  import { AUTO } from "../lib/langs.ts";
  import * as api from "../lib/api.ts";
  import { openPermWelcome, remindMissingOnStart } from "../lib/perm-welcome.svelte.ts";
  import type { LineDto, SessionStateName, StatusCode } from "../lib/api.ts";
  import { Follow } from "../lib/follow.svelte.ts";
  import { afterFinal, fromPartial, type LiveText } from "../lib/live.ts";

  let state = $state<SessionStateName>("idle");
  let lines = $state<LineDto[]>([]);
  let level = $state(0);
  let status = $state<StatusCode | null>(null);
  let errCode = $state<string | null>(null);
  let errTech = $state<string | null>(null);
  let statusDetail = $state<string | null>(null);
  let hasKey = $state(true);
  let consent = $state(true);
  let askConsent = $state(false);
  let askKey = $state(false);
  let keyInput = $state("");
  let busy = $state(false);
  let loaded = $state(false);
  let sessionId = $state<string | null>(null);
  let lastStop = $state<{ lines: number; drained: boolean } | null>(null);
  let notices = $state<string[]>([]);
  let timer: ReturnType<typeof setInterval> | undefined;
  const follow = new Follow();

  // Tuy chinh hien thi transcript: nho lan sau, ap ngay. `view` dung chung voi bo chuyen Both/Dich/Nguon (app.view).
  let tp = $state(loadPrefs());
  app.view = tp.view;
  const mq = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;
  let sysDark = $state(mq?.matches ?? false);
  $effect(() => { const f = () => (sysDark = mq!.matches); mq?.addEventListener("change", f); return () => mq?.removeEventListener("change", f); });
  const dark = $derived(app.theme === "dark" || (app.theme === "system" && sysDark));
  const tpStyle = $derived(cssVars(tp, dark));
  $effect(() => savePrefs({ ...tp, view: app.view }));

  const active = $derived(state !== "idle");
  const mode = $derived<api.RecordMode | null>(app.sysOn && app.micOn ? "mixed" : app.sysOn ? "system" : app.micOn ? "mic" : null);
  const lvl = $derived<Level>(!active || state === "paused" ? "idle" : level > 0.9 ? "clip" : level > 0.03 ? "talk" : "quiet");
  const sysLevel = $derived<Level>(app.sysOn ? (status === "permission" && active ? "none" : lvl) : "idle");
  const micLevel = $derived<Level>(app.micOn ? lvl : "idle");
  const views = [["both", "rec.view.both"], ["dst", "rec.view.dst"], ["src", "rec.view.src"]] as const;
  const STATUS_KEY: Record<StatusCode, string> = {
    permission: "status.perm", audio_device: "status.audiodevice", auth: "status.badkey", rejected: "status.rejected", quota: "status.quota", network: "status.offline", timeout: "status.timeout", storage: "status.storage", internal: "status.internal",
  };
  const statusKey = $derived(
    status ? STATUS_KEY[status] : state === "paused" ? "status.paused" : state === "stopping" ? "status.stopping"
      : state === "recording" ? (level > 0.03 ? "status.listening" : "status.silent") : "status.ready",
  );
  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);

  function setState(s: SessionStateName) {
    state = s;
    app.recording = s !== "idle";
    void api.shellSetRecording(s === "recording").catch(() => {});
    clearInterval(timer);
    if (s === "recording") timer = setInterval(() => app.seconds++, 1000);
  }

  // Chu tam (token chua chot cua Soniox): moi ban THAY ban truoc, dong chot den thi bo phan da chot.
  let live = $state<LiveText | null>(null);

  function onEvent(e: api.UiEvent) {
    if (e.kind === "partial") live = fromPartial(e.partial);
    else if (e.kind === "state") {
      if (e.state === "idle") live = null;
      if (e.state === "recording" && state === "idle") { app.seconds = 0; level = 0; }
      setState(e.state);
      void api.sessionState().then(st => (sessionId = st[1])).catch(() => {});
    } else if (e.kind === "line") {
      const i = lines.findIndex(l => l.captionLineId === e.line.captionLineId);
      if (i >= 0) lines[i] = e.line; else lines.push(e.line);
      live = afterFinal(live, e.line.sourceText, e.line.translatedText);
    } else if (e.kind === "level") level = e.level;
    else if (e.kind === "status") { status = e.code; statusDetail = e.code === "rejected" || e.code === "network" ? (e.detail ?? null) : null; }
    else if (e.kind === "notice") { if (!notices.includes(e.code)) notices.push(e.code); }
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
        sessionId = st[1];
        if (st[0] !== "idle") setState(st[0] as SessionStateName);
      } catch (e) { errCode = api.errCode(e); }
      loaded = true;
    })();
    return () => clearInterval(timer);
  });

  async function begin() {
    if (!mode) return;
    busy = true; errCode = null; errTech = null; status = null; statusDetail = null; lastStop = null; notices = [];
    try {
      const listenLangs = app.src === AUTO ? [] : [app.src];
      await api.prefsSet({ targetLang: app.dst, listenLangs, mode, uiLanguage: document.documentElement.lang === "en" ? "en" : "pt-PT" });
      lines = [];
      await api.sessionStart({ targetLang: app.dst, listenLangs, mode }, api.newChannel(onEvent));
    } catch (e) {
      const c = api.errCode(e);
      if (c === "key_missing") { hasKey = false; askKey = true; }
      else if (c === "consent_required") { consent = false; askConsent = true; }
      else { errCode = c; errTech = api.errDetail(e); void remindMissingOnStart(c); } // thieu quyen: nhac dung quyen thieu (toi da mot lan moi lan mo app), thong bao loi van o lai
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
  /** Chup vung: chi khi dang co phien; the capture overlay doc phien va moc thoi gian tu localStorage. */
  async function capture() {
    if (!sessionId) return;
    localStorage.setItem("rlt.capture.session", sessionId);
    localStorage.setItem("rlt.capture.offset_ms", String(app.seconds * 1000));
    await act(api.captureBegin);
  }
  // Hoi AI: dang ghi thi hoi ve phien dang chay, ghi xong thi ve phien vua ghi (hoac phien moi nhat neu chua co).
  $effect(() => { if (sessionId) app.lastSession = sessionId; });
  const askAi = () => openAsk(sessionId);
  const openSubtitle = () => act(api.subtitleOpen);
  function toggle() { if (active) void stop(); else void start(); }
  const stoppedLbl = (n: number) => $_("rec.stopped", { values: { n } });
  const text = (l: LineDto) => l.translatedText?.trim() ?? "";
</script>

<div class="tool">
  <div class="tl">
    <button type="button" class="btn" disabled={busy} onclick={newSession}><Icon name="plus" />{$_("tool.new")}<kbd>{platformize("⌘N")}</kbd></button>
    {#if active}
      <button type="button" class="btn stop" disabled={state === "stopping"} onclick={toggle}><Icon name="stop" />{$_("tool.stop")}<kbd>{platformize("⌘.")}</kbd></button>
      <button type="button" class="btn" disabled={state === "stopping"} onclick={togglePause}>
        <Icon name={state === "paused" ? "play" : "pause"} />{$_(state === "paused" ? "tool.resume" : "tool.pause")}
      </button>
      <span class="tm">{fmtClock(app.seconds)}</span>
    {:else}
      <button type="button" class="btn primary" disabled={busy || !mode || !loaded} onclick={toggle}><Icon name="play" />{$_("tool.start")}<kbd>{platformize("⌘.")}</kbd></button>
    {/if}
  </div>
  <div class="tr2">
    <button type="button" class="btn" onclick={askAi}><Icon name="sparkle" />{$_("nav.askai")}</button>
    <button type="button" class="btn" onclick={openSubtitle}><Icon name="caption" />{$_("rec.subtitle.open")}</button>
    <button type="button" class="btn" disabled={!active || !sessionId} title={active ? $_("rec.capture") : $_("rec.capture.needsession")} onclick={capture}><Icon name="crop" />{$_("rec.capture")}</button>
    <span class="bd {state === 'recording' ? 'rec' : ''}"><Icon name="dot" />{$_(state === "recording" ? "tool.badge.rec" : statusKey)}</span>
  </div>
</div>

{#each notices as n (n)}
  <div class="card" role="status" style="margin:8px 18px"><span class="bd warn"><Icon name="warn" />{$_("rec.notice." + n.replace(/_/g, ""))}</span></div>
{/each}

<div class="cfg {active ? 'lock' : ''}" role="group" aria-label={$_("cfg.group.aria")}>
  <div class="cg lg2" role="group" aria-label={$_("cfg.lang.group")}>
    <span class="seg" role="radiogroup" aria-label={$_("cfg.mode.group")}>
      <button type="button" class="on" role="radio" aria-checked="true" aria-label={$_("cfg.mode.one.aria")} data-tip={$_("cfg.mode.one.tip")}>{$_("cfg.mode.one")}</button>
      <button type="button" class="off" role="radio" aria-checked="false" aria-disabled="true" aria-label={$_("cfg.mode.two.aria")} data-tip={$_("cfg.mode.two.tip")}>{$_("cfg.mode.two")}<Icon name="lock" /></button>
    </span>
    <span class="lsub">
      <LangSelect bind:value={app.src} withAuto short cls="sw1" locked={active} ariaKey="cfg.src.aria" tipKey="cfg.src.tip" />
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

{#if status === "rejected"}
  <div class="card" role="alert" data-testid="rejected-banner" style="margin:8px 18px"><span class="bd rec"><Icon name="warn" />{$_("err.rejected")}</span>
    {#if statusDetail}<div data-testid="rejected-detail" style="margin-top:6px;font-size:12px;opacity:.85">{statusDetail}</div>{/if}</div>
{/if}

{#if status === "network" && statusDetail}
  <div class="card" role="status" data-testid="network-banner" style="margin:8px 18px"><span class="bd rec"><Icon name="warn" />{$_("err.network")}</span>
    <div data-testid="network-detail" style="margin-top:6px;font-size:12px;opacity:.85;word-break:break-word">{statusDetail}</div></div>
{/if}

{#if errCode}
  <div class="card" role="alert" style="margin:8px 18px"><span class="bd rec"><Icon name="warn" />{$_(errKey(errCode))}</span>
    {#if errTech}<div data-testid="err-tech" style="margin-top:6px;font-size:12px;opacity:.85;word-break:break-word">{$_("err.tech")}: {errTech}</div>{/if}
    {#if errCode.startsWith("permission")}<button type="button" class="btn sm" data-testid="perm-review" style="margin-left:8px" onclick={() => openPermWelcome(errCode === "permission_mic" ? "mic" : "screen")}><Icon name="shield" />{$_("pw.review")}</button>{/if}</div>
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
  <div class="trw">
  <div class="trs" use:follow.action style={tpStyle}>
  <TermContext dstLang={app.dst}>
  <div class="tr" aria-live="polite">
    {#each lines as l (l.captionLineId)}
      <div class="ln {text(l) ? '' : 'prov'}" data-lang={l.sourceLang}>
        <span class="ts">{fmtClock(Math.floor(l.offsetMs / 1000))}</span>
        <div class="tx">
          <div class="meta"><b>{$_(mode === "mic" ? "src.name.mic" : "src.name.sys")}</b><span class="lg">{l.sourceLang}</span>{#if !text(l)}<span class="pv">{$_("ln.provisional")}</span>{/if}</div>
          {#if app.view !== "dst"}<div class="src">{l.sourceText}</div>{/if}
          {#if app.view !== "src" && text(l)}<div class="dst">{text(l)}</div>{/if}
        </div>
      </div>
    {/each}
    {#if live}
      <div class="ln live" data-lang={live.lang}><span class="ts">{fmtClock(Math.floor(live.offsetMs / 1000))}</span>
        <div class="tx">
          <div class="meta"><b>{$_((mode === "mixed" ? live.origin === "mic" : mode === "mic") ? "src.name.mic" : "src.name.sys")}</b><span class="lg">{live.lang}</span><span class="pv">{$_("ln.listening")}</span></div>
          {#if app.view !== "dst"}<div class="src">{live.source}</div>{/if}
          {#if app.view !== "src" && live.translation}<div class="dst">{live.translation}...</div>{/if}
        </div>
      </div>
    {:else if active}
      <div class="ln live"><span class="ts">{fmtClock(app.seconds)}</span>
        <div class="tx"><div class="meta"><b>{$_(mode === "mic" ? "src.name.mic" : "src.name.sys")}</b><span class="pv">{$_("ln.listening")}</span></div><div class="dst">...</div></div>
      </div>
    {/if}
  </div>
  </TermContext>
  </div>
  {#if follow.away}<button type="button" class="jump" onclick={follow.jump}>{$_("rec.jump.latest")}</button>{/if}
  </div>
{:else}
  <div class="empty"><Icon name="mic" /><b>{$_("rec.empty.title")}</b><span>{$_(!mode ? "rec.empty.nosrc" : "rec.empty.hint")}</span></div>
{/if}

<div class="foot">
  <span class="sb {state === 'recording' && !status ? 'ok' : ''}"><Icon name={active ? "mic" : "dot"} /><b>{$_(statusKey)}</b></span>
  {#if !hasKey && loaded}<span class="fs">{$_("rec.key.missing")}</span>{/if}
  <TranscriptPrefs bind:prefs={tp} bind:view={app.view} {dark} />
</div>
