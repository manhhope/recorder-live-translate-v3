<script lang="ts">
  // Thanh nho khi boi chon chu o app khac (US-41). Bam "Dich" chay luong Inline Translate.
  import { _ } from "svelte-i18n";
  import * as api from "./api.ts";
  let busy = $state(false);
  async function go() {
    busy = true;
    try { await api.inlineRun(); } finally { busy = false; }
  }
</script>

<div class="bar" role="toolbar" aria-label={$_("quick.toolbar.aria")}>
  <button type="button" onclick={go} disabled={busy}>{busy ? $_("quick.working") : $_("quick.toolbar.translate")}</button>
</div>

<style>
  .bar { display: flex; align-items: center; justify-content: center; height: 100vh; padding: 4px; box-sizing: border-box; }
  button { width: 100%; height: 100%; border-radius: 8px; border: 1px solid var(--border, #8884); cursor: pointer; font: inherit; }
</style>
