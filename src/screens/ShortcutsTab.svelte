<script lang="ts">
  // Tab "Phim tat": xem, doi, dat lai, bao trung, bat/tat tung phim toan cuc (AC-50.x). Loi do Rust tra da dich san.
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import type { ShortcutAction, ShortcutStatus } from "../lib/api.ts";
  import { acceleratorFromEvent, canonicalAccel, displayAccel, hasMainModifier } from "../lib/accel.ts";

  let list = $state<ShortcutStatus[]>([]);
  let capturing = $state<ShortcutAction | null>(null);
  let msg = $state<{ action: ShortcutAction; text: string } | null>(null);
  let loadErr = $state(false);

  onMount(async () => { try { list = await api.shortcutsStatus(); } catch { loadErr = true; } });

  async function apply(action: ShortcutAction, accel: string) {
    msg = null;
    const clash = list.find(s => s.action !== action && canonicalAccel(s.accelerator) === canonicalAccel(accel));
    if (clash) { msg = { action, text: $_("set.sc.dup") }; return; }
    if (!hasMainModifier(accel)) { msg = { action, text: $_("set.sc.needmod") }; return; }
    try { list = await api.shortcutRebind(action, accel); }
    catch (e) { msg = { action, text: api.errCode(e) }; }
  }
  async function setEnabled(action: ShortcutAction, on: boolean) {
    msg = null;
    try { list = await api.shortcutSetEnabled(action, on); }
    catch (e) { msg = { action, text: api.errCode(e) }; list = await api.shortcutsStatus().catch(() => list); }
  }
  function onKey(e: KeyboardEvent) {
    if (!capturing) return;
    e.preventDefault(); e.stopPropagation();
    if (e.key === "Escape") { capturing = null; return; }
    const accel = acceleratorFromEvent(e);
    if (!accel) return;
    const a = capturing;
    capturing = null;
    void apply(a, accel);
  }
</script>

<svelte:window onkeydowncapture={onKey} />

<h3>{$_("set.tab.shortcuts")}</h3>

<div class="card">
  <h4>{$_("set.sc.title")}</h4>
  <div class="hint" style="margin:0 0 8px">{$_("set.sc.hint")}</div>
  {#if loadErr}<div class="hint"><span class="bd rec"><Icon name="warn" />{$_("err.generic")}</span></div>{/if}
  {#each list as s (s.action)}
    <div class="rw">
      <div>
        <b>{$_(api.shortcutKeyOf(s.action))}</b>
        <small>{$_(s.registered ? "set.sc.registered" : "set.sc.notregistered")}{#if s.error_key && $_(s.error_key) !== s.error_key} {"·"} {$_(s.error_key)}{/if}</small>
        {#if msg?.action === s.action}<small role="alert"><span class="bd rec"><Icon name="warn" />{msg.text}</span></small>{/if}
      </div>
      <div class="rr">
        <kbd class="acc-kbd" aria-label={s.accelerator}>{capturing === s.action ? $_("set.sc.press") : displayAccel(s.accelerator)}</kbd>
        {#if capturing === s.action}
          <button type="button" class="btn sm" onclick={() => (capturing = null)}>{$_("common.cancel")}</button>
        {:else}
          <button type="button" class="btn sm" onclick={() => (capturing = s.action)}>{$_("set.sc.change")}</button>
        {/if}
        <button type="button" class="btn sm ghost" disabled={canonicalAccel(s.accelerator) === canonicalAccel(api.SHORTCUT_DEFAULTS[s.action])}
          onclick={() => apply(s.action, api.SHORTCUT_DEFAULTS[s.action])}><Icon name="refresh" />{$_("set.sc.reset")}</button>
        <input type="checkbox" class="chk" role="switch" aria-label={$_("set.sc.enabled")} title={$_("set.sc.enabled")} checked={s.enabled}
          onchange={(e) => setEnabled(s.action, e.currentTarget.checked)} />
      </div>
    </div>
  {/each}
</div>
