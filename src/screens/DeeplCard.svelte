<script lang="ts">
  // Cau hinh DeepL (BYOK). Khoa chi NHAP VAO: o la mat khau, bi xoa ngay sau khi luu; giao dien chi biet "da co / chua co".
  import { onMount } from "svelte";
  import { _, formatNumber, locale, currentLocale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import type { DeeplUsage } from "../lib/api.ts";

  let hasKey = $state(false);
  let keyInput = $state("");
  let busy = $state(false);
  let testing = $state(false);
  let usage = $state<DeeplUsage | null>(null);
  const usageLbl = $derived(usage ? $_("set.deepl.usage", { values: { used: formatNumber(usage.character_count, loc), limit: formatNumber(usage.character_limit, loc) } }) : "");
  let err = $state<string | null>(null);

  const loc = $derived(($locale as "en" | "pt-PT") ?? currentLocale());
  const errKey = (c: string) => { const k = "quick.err." + c.replace(/_/g, ""); return $_(k) !== k ? k : "err.generic"; };

  onMount(async () => { try { hasKey = await api.deeplHasKey(); } catch (e) { err = api.errCode(e); } });

  async function save() {
    const v = keyInput.trim();
    if (!v) return;
    busy = true; err = null; usage = null;
    try { await api.deeplSaveKey(v); keyInput = ""; hasKey = true; } catch (e) { err = api.errCode(e); }
    busy = false;
  }
  async function remove() {
    busy = true; err = null; usage = null;
    try { await api.deeplClearKey(); hasKey = false; } catch (e) { err = api.errCode(e); }
    busy = false;
  }
  async function test() {
    testing = true; err = null; usage = null;
    try { usage = await api.deeplTestConnection(); } catch (e) { err = api.errCode(e); }
    testing = false;
  }
</script>

<div class="card">
  <h4>{$_("set.deepl.title")}</h4>
  <div class="hint" style="margin:0 0 8px">{$_("set.deepl.desc")}</div>
  <div class="rw">
    <b>{$_("set.deepl.key")}</b>
    <div class="rr">
      {#if hasKey}
        <span class="bd acc"><Icon name="key" />{$_("set.deepl.saved")}</span>
        <button type="button" class="btn sm ghost" disabled={busy} onclick={remove}>{$_("set.deepl.remove")}</button>
      {:else}
        <label class="fld err"><Icon name="key" /><input type="password" autocomplete="off" spellcheck="false" aria-label={$_("set.deepl.key")}
          placeholder={$_("set.key.none")} bind:value={keyInput} /></label>
        <button type="button" class="btn sm primary" disabled={busy || !keyInput.trim()} onclick={save}>{$_("set.deepl.save")}</button>
      {/if}
    </div>
  </div>
  <div class="rr" style="padding-top:6px;justify-content:flex-start">
    <button type="button" class="btn sm" disabled={!hasKey || testing} onclick={test}>{$_("set.test")}</button>
    <span class="hint" role="status" style="margin:0">
      {#if testing}{$_("set.deepl.testing")}
      {:else if usage}<span class="bd ok"><Icon name="check" />{usageLbl}</span>
      {:else if err}<span class="bd rec"><Icon name="warn" />{$_(errKey(err))}</span>{/if}
    </span>
  </div>
</div>
