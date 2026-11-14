<script lang="ts">
  // Tab "Gioi thieu va quyen": phien ban, trang thai tung quyen (micro, ghi man hinh, Accessibility), muc Cap nhat "sap co".
  // Chua co lenh mo Cai dat He thong o backend nen nut do bi khoa kem chu thich.
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";

  let version = $state("");
  let mic = $state<string | null>(null);
  let screen = $state<string | null>(null);
  let ax = $state<boolean | null>(null);
  let err = $state(false);
  let busy = $state(false);

  const lockNote = $derived($_("set.locked.backend", { values: { cmd: "open_system_settings" } }));
  const stateKey = (s: string) => "set.perm.s." + s.toLowerCase();
  const okOf = (s: string | null) => s === "authorized" || s === "granted";

  async function load() {
    err = false;
    try { const p = await api.permissionsStatus(); mic = p.mic; screen = p.screen; } catch { err = true; }
    try { ax = await api.axStatus(); } catch { err = true; }
  }
  onMount(async () => {
    try { version = await api.coreVersion(); } catch { version = ""; }
    await load();
  });
  async function requestMic() {
    busy = true;
    try { mic = await api.micPermissionRequest(); } catch { err = true; }
    busy = false;
  }
</script>

<h3>{$_("set.tab.permissions")}</h3>

<div class="card">
  <h4>{$_("set.about.app")}</h4>
  <div class="rw"><b>{$_("set.about.version")}</b><div class="rr"><code>{version}</code></div></div>
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
        <button type="button" class="btn sm" disabled title={lockNote}><Icon name="lock" />{$_("set.perm.opensys")}</button>
      {/if}
    </div>
  </div>
  <div class="rw">
    <b>{$_("set.perm.screen")}</b>
    <div class="rr">
      {#if screen}<span class="bd {okOf(screen) ? 'ok' : 'warn'}"><Icon name={okOf(screen) ? "check" : "warn"} />{$_(stateKey(screen))}</span>{/if}
      {#if screen === "denied"}
        <button type="button" class="btn sm" disabled title={lockNote}><Icon name="lock" />{$_("set.perm.opensys")}</button>
      {/if}
    </div>
  </div>
  <div class="rw">
    <b>{$_("set.perm.ax")}</b>
    <div class="rr">
      {#if ax !== null}<span class="bd {ax ? 'ok' : 'warn'}"><Icon name={ax ? "check" : "warn"} />{$_(ax ? "set.perm.s.granted" : "set.perm.s.missing")}</span>{/if}
      {#if ax === false}
        <button type="button" class="btn sm" disabled title={lockNote}><Icon name="lock" />{$_("set.perm.opensys")}</button>
      {/if}
    </div>
  </div>
  <div class="tl2" style="margin-top:8px">
    <button type="button" class="btn sm" onclick={load}><Icon name="refresh" />{$_("set.perm.recheck")}</button>
    <span class="hint" style="margin:0"><Icon name="lock" /> {lockNote}</span>
    {#if err}<span class="bd rec" role="alert"><Icon name="warn" />{$_("set.perm.err")}</span>{/if}
  </div>
</div>

<div class="card">
  <h4>{$_("set.upd.title")}</h4>
  <div class="hint" style="margin:0"><Icon name="info" /> {$_("set.upd.soon")}</div>
</div>
