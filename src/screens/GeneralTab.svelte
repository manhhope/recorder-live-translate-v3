<script lang="ts">
  // Tab "Chung": ngon ngu giao dien (doi ngay, bao ca khay he thong), tu khoi dong, giu may thuc.
  // Thong bao ghi am chua co lenh o backend nen khoa kem chu thich.
  import { onMount } from "svelte";
  import { _, locale, setLocale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";

  const lockNote = $derived($_("set.locked.backend", { values: { cmd: "recording_notify_set" } }));

  let autostart = $state<boolean | null>(null);
  let keepAwake = $state(false);
  let err = $state(false);
  const isPt = $derived($locale === "pt-PT");

  const readKeep = () => { try { return globalThis.localStorage?.getItem(api.KEEP_AWAKE_KEY) === "1"; } catch { return false; } };

  onMount(async () => {
    keepAwake = readKeep();
    try { autostart = await api.autostartGet(); } catch { err = true; }
  });

  const pick = (l: "en" | "pt-PT") => api.setUiLocale(setLocale, l);
  async function toggleAutostart(on: boolean) {
    err = false;
    try { autostart = await api.autostartSet(on); } catch { err = true; autostart = !on; }
  }
  async function toggleKeep(on: boolean) {
    err = false; keepAwake = on;
    try {
      await api.shellSetKeepAwake(on);
      try { globalThis.localStorage?.setItem(api.KEEP_AWAKE_KEY, on ? "1" : "0"); } catch { /* bo qua */ }
    } catch { err = true; keepAwake = !on; }
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
      <input type="checkbox" class="chk" role="switch" aria-label={$_("set.gen.autostart")} disabled={autostart === null}
        checked={autostart ?? false} onchange={(e) => toggleAutostart(e.currentTarget.checked)} />
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
      <small><Icon name="lock" /> {lockNote}</small></div>
    <div class="rr">
      <input type="checkbox" class="chk" role="switch" aria-label={$_("set.gen.notify")} disabled title={$_("set.locked.backend", { values: { cmd: "recording_notify_set" } })} />
    </div>
  </div>
  {#if err}<div class="hint" role="alert"><span class="bd rec"><Icon name="warn" />{$_("set.gen.err")}</span></div>{/if}
</div>
