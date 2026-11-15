<script lang="ts">
  // Modal What's New (phuong an A): header co dinh (tieu de + phien ban + X), vung giua cuon, footer co dinh (Got it).
  // Got it, X, Esc deu dong va ghi phien ban da xem (closeWhatsNew). Bam ra ngoai khong dong.
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "./Icon.svelte";
  import { whatsNew, closeWhatsNew } from "./whats-new.svelte.ts";

  let okBtn = $state<HTMLButtonElement>();
  let xBtn = $state<HTMLButtonElement>();
  onMount(() => {
    const prev = document.activeElement as HTMLElement | null;
    okBtn?.focus();
    return () => { try { prev?.focus?.(); } catch { /* bo qua */ } };
  });
  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") { e.preventDefault(); closeWhatsNew(); return; }
    if (e.key === "Tab") { // xoay vong trong hop thoai: X <-> Got it
      const first = xBtn, last = okBtn;
      if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last?.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="tdlg wn-scrim">
  <div class="card wn" role="dialog" aria-modal="true" aria-labelledby="wn-t">
    <header class="wn-h">
      <h3 id="wn-t"><Icon name="sparkle" />{$_("wn.title")} <span class="wn-v">{"v" + whatsNew.version}</span></h3>
      <button type="button" class="btn icon ghost" bind:this={xBtn} aria-label={$_("wn.close")} title={$_("wn.close")} onclick={closeWhatsNew}><Icon name="close" /></button>
    </header>
    <div class="wn-b" tabindex="0">
      {#each whatsNew.notes as g (g.group)}
        <h4 class="wn-g">{$_("wn.group." + g.group)}</h4>
        <ul>{#each g.items as it (it)}<li>{it}</li>{/each}</ul>
      {:else}
        <p class="hint">{$_("wn.empty")}</p>
      {/each}
    </div>
    <footer class="wn-f">
      <button type="button" class="btn primary" bind:this={okBtn} onclick={closeWhatsNew}>{$_("wn.gotit")}</button>
    </footer>
  </div>
</div>
