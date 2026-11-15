<script lang="ts">
  // Modal Welcome: cap quyen (phuong an A). Header co dinh, vung giua cuon (ba hang quyen), footer co dinh (Continue luon bam duoc).
  // Esc, X, Skip for now, Continue deu dong. Bam ra ngoai khong dong. Tu doc lai quyen khi cua so lay lai tieu diem.
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "./Icon.svelte";
  import * as api from "./api.ts";
  import { isAllowed, type PermKey } from "./perm-welcome.ts";
  import { permWelcome, closePermWelcome, refreshPerms, allowMic, openPane, restartNow } from "./perm-welcome.svelte.ts";

  let root = $state<HTMLElement>();
  let okBtn = $state<HTMLButtonElement>();
  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
  const NAME: Record<PermKey, string> = { mic: "pw.mic.name", screen: "pw.screen.name", ax: "pw.ax.name" };
  const WHY: Record<PermKey, string> = { mic: "pw.mic.why", screen: "pw.screen.why", ax: "pw.ax.why" };

  onMount(() => {
    const prev = document.activeElement as HTMLElement | null;
    okBtn?.focus();
    const onFocus = () => { void refreshPerms(); };
    const onVis = () => { if (document.visibilityState === "visible") onFocus(); };
    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", onVis);
    return () => {
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("visibilitychange", onVis);
      try { prev?.focus?.(); } catch { /* bo qua */ }
    };
  });
  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); closePermWelcome(); return; }
    if (e.key === "Tab" && root) { // xoay vong trong hop thoai
      const f = [...root.querySelectorAll<HTMLElement>("button:not([disabled]), input:not([disabled])")];
      if (!f.length) return;
      const first = f[0], last = f[f.length - 1];
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="tdlg wn-scrim">
  <div class="card wn pw" role="dialog" aria-modal="true" aria-labelledby="pw-t" bind:this={root} data-testid="perm-welcome">
    <header class="wn-h">
      <h3 id="pw-t"><Icon name="shield" />{$_("pw.title")}</h3>
      <button type="button" class="btn icon ghost" aria-label={$_("pw.close")} title={$_("pw.close")} onclick={closePermWelcome}><Icon name="close" /></button>
    </header>
    <div class="wn-b" tabindex="0">
      <p class="hint pw-intro">{$_("pw.intro")}</p>
      {#each permWelcome.keys as k (k)}
        {@const st = permWelcome.states[k]}
        <div class="pw-row {permWelcome.focus === k ? 'hot' : ''}" data-testid={"pw-" + k} data-state={st}>
          <div class="pw-main">
            <b>{$_(NAME[k])}{#if k === "ax"} <span class="bd">{$_("pw.optional")}</span>{/if}</b>
            <span class="hint">{$_(WHY[k])}</span>
            {#if permWelcome.focus === k && !isAllowed(st)}<span class="hint pw-miss" role="status"><Icon name="warn" /> {$_("pw.missing")}</span>{/if}
            {#if k === "mic" && st === "denied"}<span class="hint">{$_(permWelcome.keys.length === 1 ? "pw.mic.denied.win" : "pw.mic.denied")}</span>{/if}
            {#if k === "screen" && st === "needsrestart"}<span class="hint">{$_("pw.screen.restart")}</span>{/if}
            {#if k === "screen" && st !== "allowed" && st !== "needsrestart"}<span class="hint">{$_("pw.screen.settings")}</span>{/if}
          </div>
          <div class="pw-act">
            <span class="bd {isAllowed(st) ? 'ok' : st === "denied" || st === "needsrestart" ? 'warn' : ''}"><Icon name={isAllowed(st) ? "check" : st === "notasked" ? "info" : "warn"} />{$_("pw.s." + st)}</span>
            {#if !isAllowed(st)}
              {#if k === "mic" && st === "notasked"}
                <button type="button" class="btn sm primary" disabled={permWelcome.busy} onclick={allowMic}>{$_("pw.mic.allow")}</button>
              {:else}
                <button type="button" class="btn sm" onclick={() => openPane(k)}><Icon name="link" />{$_("set.perm.opensys")}</button>
              {/if}
              {#if k === "screen" && st === "needsrestart"}
                <button type="button" class="btn sm primary" onclick={restartNow}><Icon name="refresh" />{$_("set.perm.restart")}</button>
              {/if}
            {/if}
          </div>
        </div>
      {/each}
      {#if permWelcome.errCode}<span class="bd rec" role="alert"><Icon name="warn" />{$_(errKey(permWelcome.errCode))}</span>{/if}
      {#if permWelcome.err}<span class="bd rec" role="alert"><Icon name="warn" />{$_("pw.err")}</span>{/if}
    </div>
    <footer class="wn-f pw-f">
      <label class="pw-off"><input type="checkbox" bind:checked={permWelcome.off} />{$_("pw.dontshow")}</label>
      <span class="pw-btns">
        <button type="button" class="btn" onclick={closePermWelcome}>{$_("pw.skip")}</button>
        <button type="button" class="btn primary" bind:this={okBtn} onclick={closePermWelcome}>{$_("pw.continue")}</button>
      </span>
    </footer>
  </div>
</div>
