<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import type { DriveConnOut, DriveSyncAllOut } from "../lib/api.ts";

  let conn = $state<DriveConnOut | null>(null);
  let clientId = $state("");
  let clientSecret = $state("");
  let busy = $state(false);
  /** Dang cho nguoi dung xac nhan o trinh duyet (lenh drive_connect chua tra ve). */
  let waiting = $state(false);
  let syncing = $state(false);
  let advanced = $state(false);
  let err = $state<string | null>(null);
  let cancelled = $state(false);
  let note = $state<"revoked" | "local" | null>(null);
  let syncRes = $state<DriveSyncAllOut | null>(null);

  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
  const state = $derived(conn?.state ?? null);
  const accountLabel = $derived(
    conn?.email ? $_("set.drive.account", { values: { email: conn.email } }) : $_("set.drive.account.unknown"),
  );
  const stateKey = $derived(
    state === "connected" ? "set.drive.state.connected"
      : state === "disconnected" ? "set.drive.state.disconnected"
      : "set.drive.state.notconfigured");
  const syncMsg = $derived.by(() => {
    const r = syncRes;
    if (!r) return null;
    if (!r.oauthConnected) return $_("set.drive.sync.notconnected");
    if (r.total === 0) return $_("set.drive.sync.nothing");
    if (r.failed === 0 && r.waiting === 0) return $_("set.drive.sync.done", { values: { ok: r.synced } });
    return $_("set.drive.sync.partial", { values: { ok: r.synced, failed: r.failed, waiting: r.waiting } });
  });

  function clearMsgs() { err = null; cancelled = false; note = null; syncRes = null; }

  async function refresh() {
    try { conn = await api.driveConnectionStatus(); } catch (e) { err = api.errCode(e); }
  }
  onMount(refresh);

  async function saveConfig() {
    const id = clientId.trim();
    if (!id) return;
    busy = true; clearMsgs();
    try {
      conn = await api.driveSaveClientConfig(id, clientSecret.trim() || null);
      clientId = ""; clientSecret = "";
    } catch (e) { err = api.errCode(e); }
    busy = false;
  }
  async function connect() {
    busy = true; waiting = true; clearMsgs();
    try { conn = await api.driveConnect(); }
    catch (e) {
      const c = api.errCode(e);
      if (c === "drive_auth_cancelled") cancelled = true; else err = c;
    }
    waiting = false; busy = false;
    await refresh();
  }
  /** Huy luc dang cho: lenh drive_connect tu ket thuc voi `drive_auth_cancelled`. */
  async function cancel() {
    try { await api.driveConnectCancel(); } catch (e) { err = api.errCode(e); }
  }
  async function disconnect() {
    busy = true; clearMsgs();
    try {
      const r = await api.driveDisconnect();
      note = r.revoked ? "revoked" : "local";
      await refresh();
    } catch (e) { err = api.errCode(e); }
    busy = false;
  }
  async function syncNow() {
    syncing = true; clearMsgs();
    try { syncRes = await api.driveSyncNow(); } catch (e) { err = api.errCode(e); }
    syncing = false;
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
    </div>
  </div>

  {#if state === "connected"}
    <div class="rw" data-testid="drive-account">
      <div><b>{accountLabel}</b></div>
      <div class="rr">
        <button type="button" class="btn sm" disabled={busy || syncing} onclick={syncNow}><Icon name="refresh" />{$_(syncing ? "set.drive.syncing" : "set.drive.syncnow")}</button>
        <button type="button" class="btn sm ghost" disabled={busy || syncing} onclick={disconnect}>{$_("set.drive.disconnect")}</button>
      </div>
    </div>
  {:else if state}
    <div class="rw">
      <div>{#if !waiting}<small>{$_("set.drive.connect.hint")}</small>{/if}</div>
      <div class="rr">
        {#if waiting}
          <button type="button" class="btn sm" onclick={cancel}>{$_("set.drive.cancel")}</button>
        {:else}
          <button type="button" class="btn sm primary" disabled={busy || state !== "disconnected"} onclick={connect}><Icon name="link" />{$_("set.drive.connect")}</button>
        {/if}
      </div>
    </div>
    {#if state === "not_configured"}
      <div class="hint">{$_("set.drive.notconfigured.hint")}
        <button type="button" class="btn sm ghost" onclick={() => (advanced = true)}>{$_("set.drive.openadvanced")}</button>
      </div>
    {/if}
  {/if}

  {#if waiting}
    <div class="hint" role="status"><Icon name="refresh" /> {$_("set.drive.connecting")} {$_("set.drive.connect.hint")}</div>
  {/if}
  {#if cancelled}
    <div class="hint" role="status"><Icon name="info" /> {$_("set.drive.cancelled")}</div>
  {/if}
  {#if note}
    <div class="hint"><Icon name={note === "revoked" ? "check" : "warn"} /> {$_(note === "revoked" ? "set.drive.disconnected.ok" : "set.drive.disconnected.local")}</div>
  {/if}
  {#if syncMsg}
    <div class="hint" role="status"><Icon name="check" /> {syncMsg}</div>
  {/if}
  {#if err}
    <div class="hint"><span class="bd rec"><Icon name="warn" />{$_(errKey(err))}</span></div>
  {/if}

  {#if state !== "connected"}
    <div class="rw">
      <div><b>{$_("set.drive.advanced")}</b><small>{$_("set.drive.advanced.desc")}</small></div>
      <div class="rr">
        <button type="button" class="btn sm ghost" aria-expanded={advanced} aria-controls="drive-adv" onclick={() => (advanced = !advanced)}><Icon name="chev" />{$_("set.drive.advanced")}</button>
      </div>
    </div>
    {#if advanced}
      <div id="drive-adv">
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
      </div>
    {/if}
  {/if}
</div>
