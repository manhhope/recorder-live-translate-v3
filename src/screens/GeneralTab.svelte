<script lang="ts">
  // Tab "Chung": ngon ngu giao dien (doi ngay, bao ca khay he thong), tu khoi dong, giu may thuc.
  // Thong bao ghi am chua co lenh o backend nen khoa kem chu thich.
  import { onMount } from "svelte";
  import { _, locale, setLocale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import { withTimeout } from "../lib/ipc.ts";

  let autostart = $state<boolean | null>(null);
  let keepAwake = $state(false);
  let err = $state<string | null>(null);
  let autoLoadErr = $state(false);
  let autoBusy = $state(false);
  const isPt = $derived($locale === "pt-PT");

  const readKeep = () => { try { return globalThis.localStorage?.getItem(api.KEEP_AWAKE_KEY) === "1"; } catch { return false; } };

  const errKey = (e: unknown) => (e === "ipc_timeout" ? "set.load.timeout" : "set.gen.err");
  async function loadAutostart() {
    autoLoadErr = false; err = null; autoBusy = true;
    try { autostart = await withTimeout(api.autostartGet()); } catch (e) { autoLoadErr = true; err = errKey(e); }
    autoBusy = false;
  }
  onMount(() => { keepAwake = readKeep(); void loadAutostart(); });

  const pick = (l: "en" | "pt-PT") => api.setUiLocale(setLocale, l);
  async function toggleAutostart(on: boolean, box: HTMLInputElement) {
    err = null; autoBusy = true;
    try { autostart = await withTimeout(api.autostartSet(on)); } catch (e) { err = errKey(e); autostart = !on; }
    box.checked = autostart ?? false; // that bai thi cong tat ve dung trang thai that
    autoBusy = false;
  }
  async function toggleKeep(on: boolean) {
    err = null; keepAwake = on;
    try {
      await api.shellSetKeepAwake(on);
      try { globalThis.localStorage?.setItem(api.KEEP_AWAKE_KEY, on ? "1" : "0"); } catch { /* bo qua */ }
    } catch { err = "set.gen.err"; keepAwake = !on; }
  }
</script>

<h3>{$_("set.tab.general")}</h3>

<div class="card">
  <div class="rw">
    <div><b>{$_("set.gen.lang")}</b><small>{$_("set.gen.lang.hint")}</small></div>
    <div class="rr">
      <span class="seg" role="radiogroup" aria-label={$_("set.gen.lang")}>
        <button type="button" role="radio" class={!isPt ? "on" : ""} aria-checked={!isPt} onclick={() => pick("en")}>{$_("set.gen.lang.en")}</button>
        <button type="button" role="radio" class={isPt ? "on" : ""} aria-checked={isPt} onclick={() => pick("pt-PT")}>{$_("set.gen.lang.pt")}</button>
      </span>
    </div>
  </div>
  <div class="rw">
    <div><b>{$_("set.gen.autostart")}</b><small>{$_("set.gen.autostart.hint")}</small></div>
    <div class="rr">
      <input type="checkbox" class="chk" role="switch" aria-label={$_("set.gen.autostart")} disabled={autostart === null || autoBusy}
        checked={autostart ?? false} onchange={(e) => toggleAutostart(e.currentTarget.checked, e.currentTarget)} />
    </div>
  </div>
  <div class="rw">
    <div><b>{$_("set.gen.keepawake")}</b><small>{$_("set.gen.keepawake.hint")}</small></div>
    <div class="rr">
      <input type="checkbox" class="chk" role="switch" aria-label={$_("set.gen.keepawake")} checked={keepAwake} onchange={(e) => toggleKeep(e.currentTarget.checked)} />
    </div>
  </div>
  <div class="rw lockrow">
    <div><b>{$_("set.gen.notify")}</b><small>{$_("set.gen.notify.hint")}</small>
      <small><Icon name="lock" /> {$_("set.gen.notify.na")}</small></div>
    <div class="rr">
      <input type="checkbox" class="chk" role="switch" aria-label={$_("set.gen.notify")} disabled title={$_("set.gen.notify.na")} />
    </div>
  </div>
  {#if err}<div class="hint" role="alert"><span class="bd rec"><Icon name="warn" />{$_(err)}</span>
    {#if autoLoadErr}<button type="button" class="btn sm" disabled={autoBusy} onclick={loadAutostart}><Icon name="refresh" />{$_("common.retry")}</button>{/if}</div>{/if}
</div>
