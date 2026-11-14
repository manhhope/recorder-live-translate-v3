<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import type { DriveConnState } from "../lib/api.ts";

  let state = $state<DriveConnState | null>(null);
  let clientId = $state("");
  let clientSecret = $state("");
  let busy = $state(false);
  let connecting = $state(false);
  let err = $state<string | null>(null);
  let note = $state<"revoked" | "local" | null>(null);

  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
  const stateKey = $derived(
    state === "connected" ? "set.drive.state.connected"
      : state === "disconnected" ? "set.drive.state.disconnected"
      : "set.drive.state.notconfigured");

  async function refresh() {
    try { state = (await api.driveConnectionStatus()).state; } catch (e) { err = api.errCode(e); }
  }
  onMount(refresh);

  async function saveConfig() {
    const id = clientId.trim();
    if (!id) return;
    busy = true; err = null; note = null;
    try {
      state = (await api.driveSaveClientConfig(id, clientSecret.trim() || null)).state;
      clientId = ""; clientSecret = "";
    } catch (e) { err = api.errCode(e); }
    busy = false;
  }
  async function connect() {
    busy = true; connecting = true; err = null; note = null;
    try { state = (await api.driveConnect()).state; } catch (e) { err = api.errCode(e); }
    connecting = false; busy = false;
  }
  async function disconnect() {
    busy = true; err = null; note = null;
    try {
      const r = await api.driveDisconnect();
      note = r.revoked ? "revoked" : "local";
      await refresh();
    } catch (e) { err = api.errCode(e); }
    busy = false;
  }
</script>

<h3>{$_("set.tab.drive")}</h3>

<div class="card">
  <h4>{$_("set.drive.title")}</h4>
  <div class="hint" style="margin:0 0 8px">{$_("set.drive.desc")}</div>
  <div class="rw">
    <div><b>{$_("set.drive.state")}</b>{#if state === "connected"}<small>{$_("set.drive.token.note")}</small>{/if}</div>
    <div class="rr">
      {#if state}
        <span class="bd {state === 'connected' ? 'ok' : ''}"><Icon name={state === "connected" ? "check" : "info"} />{$_(stateKey)}</span>
      {/if}
      {#if state === "connected"}
        <button type="button" class="btn sm ghost" disabled={busy} onclick={disconnect}>{$_("set.drive.disconnect")}</button>
      {:else if state === "disconnected"}
        <button type="button" class="btn sm primary" disabled={busy} onclick={connect}><Icon name="link" />{$_("set.drive.connect")}</button>
      {/if}
    </div>
  </div>

  {#if state === "not_configured"}
    <div class="hint">{$_("set.drive.notconfigured.hint")}</div>
  {/if}
  {#if state !== "connected"}
    <div class="rw">
      <div><b>{$_("set.drive.clientid")}</b></div>
      <div class="rr">
        <label class="fld"><Icon name="key" /><input type="text" autocomplete="off" spellcheck="false" aria-label={$_("set.drive.clientid")} placeholder={$_("set.drive.clientid.ph")} bind:value={clientId} /></label>
      </div>
    </div>
    <div class="rw">
      <div><b>{$_("set.drive.clientsecret")}</b><small>{$_("set.drive.clientsecret.hint")}</small></div>
      <div class="rr">
        <label class="fld"><Icon name="lock" /><input type="password" autocomplete="off" aria-label={$_("set.drive.clientsecret")} bind:value={clientSecret} /></label>
        <button type="button" class="btn sm primary" disabled={busy || !clientId.trim()} onclick={saveConfig}>{$_("set.drive.saveconfig")}</button>
      </div>
    </div>
  {/if}

  {#if connecting}
    <div class="hint"><Icon name="refresh" /> {$_("set.drive.connecting")}</div>
  {/if}
  {#if note}
    <div class="hint"><Icon name={note === "revoked" ? "check" : "warn"} /> {$_(note === "revoked" ? "set.drive.disconnected.ok" : "set.drive.disconnected.local")}</div>
  {/if}
  {#if err}
    <div class="hint"><span class="bd rec"><Icon name="warn" />{$_(errKey(err))}</span></div>
  {/if}
</div>
