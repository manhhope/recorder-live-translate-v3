<script lang="ts">
  // Khung loi tai cai dat: thong bao + nut thu lai, thay cho man trong im lang.
  import { _ } from "../i18n/index.ts";
  import Icon from "./Icon.svelte";
  let { message, busy = false, onretry }: { message: string; busy?: boolean; onretry: () => void } = $props();
</script>

<div class="card" role="alert">
  <div class="rw">
    <div><span class="bd rec"><Icon name="warn" />{message}</span></div>
    <div class="rr"><button type="button" class="btn sm" disabled={busy} onclick={onretry}><Icon name="refresh" />{$_("common.retry")}</button></div>
  </div>
</div>
