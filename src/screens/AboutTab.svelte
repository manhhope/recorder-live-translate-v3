<script lang="ts">
  // Tab "Gioi thieu va quyen": phien ban, trang thai tung quyen (micro, ghi man hinh, Accessibility), muc Cap nhat "sap co".
  // Nut "Mo Cai dat He thong" goi lenh backend open_system_settings (whitelist pane). Accessibility khong co hop thoai xin quyen.
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import { openWhatsNew } from "../lib/whats-new.svelte.ts";

  let version = $state("");
  let mic = $state<string | null>(null);
  let screen = $state<string | null>(null);
  let ax = $state<boolean | null>(null);
  let err = $state(false);
  let busy = $state(false);
  let errKeyShown = $state<string | null>(null);
  let opened = $state(false);
  let openedScreen = $state(false); // da mo Cai dat > Ghi man hinh tu man nay (tin hieu cho "can khoi dong lai")

  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
  const stateKey = (s: string) => "set.perm.s." + s.toLowerCase();
  const needsRestart = $derived(screen === "needsrestart");
  const okOf = (s: string | null) => s === "authorized" || s === "granted";

  async function load() {
    err = false; errKeyShown = null;
    try { const p = await api.permissionsStatus(); mic = p.mic; screen = p.screen; } catch { err = true; }
    try { ax = await api.axStatus(); } catch { err = true; }
  }
  // Check again: doc nhanh, neu ghi man hinh van "denied" thi do that (SCShareableContent) de phan biet cho khoi dong lai.
  async function recheck() {
    await load();
    if (screen === "denied" || screen === "needsrestart") {
      try { screen = await api.screenPermissionCheck(openedScreen); } catch (e) { errKeyShown = errKey(api.errCode(e)); }
    }
  }
  async function restart() {
    errKeyShown = null;
    try { await api.restartApp(); } catch (e) { errKeyShown = errKey(api.errCode(e)); }
  }
  onMount(async () => {
    try { version = await api.coreVersion(); } catch { version = ""; }
    await load();
  });
  async function requestMic() {
    busy = true;
    errKeyShown = null;
    try { mic = await api.micPermissionRequest(); } catch (e) { errKeyShown = errKey(api.errCode(e)); }
    busy = false;
  }
  async function openSettings(pane: api.SettingsPane) {
    errKeyShown = null; opened = false;
    try { await api.openSystemSettings(pane); opened = true; if (pane === "screen_recording") openedScreen = true; } catch (e) { errKeyShown = errKey(api.errCode(e)); }
  }
</script>

<h3>{$_("set.tab.permissions")}</h3>

<div class="card">
  <h4>{$_("set.about.app")}</h4>
  <div class="rw"><b>{$_("set.about.version")}</b><div class="rr"><code>{version}</code></div></div>
  <div class="rw"><b>{$_("set.about.whatsnew")}</b><div class="rr"><button type="button" class="btn sm" data-testid="whatsnew-open" onclick={openWhatsNew}>{$_("wn.open")}</button></div></div>
</div>

<div class="card">
  <h4>{$_("set.perm.title")}</h4>
  <div class="hint" style="margin:0 0 8px">{$_("set.perm.desc")}</div>

  <div class="rw">
    <b>{$_("set.perm.mic")}</b>
    <div class="rr">
      {#if mic}<span class="bd {okOf(mic) ? 'ok' : 'warn'}"><Icon name={okOf(mic) ? "check" : "warn"} />{$_(stateKey(mic))}</span>{/if}
      {#if mic === "notDetermined"}
        <button type="button" class="btn sm primary" disabled={busy} onclick={requestMic}>{$_("set.perm.request")}</button>
      {:else if mic === "denied" || mic === "restricted"}
        <button type="button" class="btn sm" onclick={() => openSettings("microphone")}><Icon name="link" />{$_("set.perm.opensys")}</button>
      {/if}
    </div>
  </div>
  <div class="rw">
    <b>{$_("set.perm.screen")}</b>
    <div class="rr">
      {#if screen}<span class="bd {okOf(screen) ? 'ok' : 'warn'}"><Icon name={okOf(screen) ? "check" : "warn"} />{$_(stateKey(screen))}</span>{/if}
      {#if screen === "denied" || needsRestart}
        <button type="button" class="btn sm" onclick={() => openSettings("screen_recording")}><Icon name="link" />{$_("set.perm.opensys")}</button>
      {/if}
      {#if needsRestart}
        <button type="button" class="btn sm primary" onclick={restart}><Icon name="refresh" />{$_("set.perm.restart")}</button>
      {/if}
    </div>
  </div>
  <div class="rw">
    <b>{$_("set.perm.ax")}</b>
    <div class="rr">
      {#if ax !== null}<span class="bd {ax ? 'ok' : 'warn'}"><Icon name={ax ? "check" : "warn"} />{$_(ax ? "set.perm.s.granted" : "set.perm.s.missing")}</span>{/if}
      {#if ax === false}
        <button type="button" class="btn sm" onclick={() => openSettings("accessibility")}><Icon name="link" />{$_("set.perm.opensys")}</button>
      {/if}
    </div>
  </div>
  {#if needsRestart}<div class="hint" style="margin:4px 0 0"><Icon name="info" /> {$_("set.perm.screenhelp")}</div>{/if}
  {#if ax === false}<div class="hint" style="margin:4px 0 0"><Icon name="info" /> {$_("set.perm.axhelp")}</div>{/if}
  <div class="tl2" style="margin-top:8px">
    <button type="button" class="btn sm" onclick={recheck}><Icon name="refresh" />{$_("set.perm.recheck")}</button>
    {#if opened}<span class="hint" style="margin:0"><Icon name="info" /> {$_("set.perm.openedhint")}</span>{/if}
    {#if errKeyShown}<span class="bd rec" role="alert"><Icon name="warn" />{$_(errKeyShown)}</span>{/if}
    {#if err}<span class="bd rec" role="alert"><Icon name="warn" />{$_("set.perm.err")}</span>{/if}
  </div>
</div>

<div class="card">
  <h4>{$_("set.upd.title")}</h4>
  <div class="hint" style="margin:0"><Icon name="info" /> {$_("set.upd.soon")}</div>
</div>
