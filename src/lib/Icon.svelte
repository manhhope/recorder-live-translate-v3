<script lang="ts">
  import { ICONS } from "./icons.ts";
  let { name }: { name: string } = $props();
</script>

<svg class="i" viewBox="0 0 24 24" aria-hidden="true">{@html ICONS[name] ?? ""}</svg>
