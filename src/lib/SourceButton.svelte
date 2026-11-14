<script lang="ts">
  import { _ } from "../i18n/index.ts";
  import Icon from "./Icon.svelte";
  import type { Level } from "./app.svelte.ts";

  let { kind, on = $bindable(), level }: { kind: "sys" | "mic"; on: boolean; level: Level } = $props();

  const mark = $derived(level === "clip" ? "!" : level === "none" ? "×" : "");
  const stateKey = $derived(on ? level : "idle");
  const tip = $derived(
    $_("cfg.audio.group") + ": " + $_(kind === "sys" ? "src.name.sys" : "src.name.mic") + $_("src.tip.on") + " " +
    $_("src.state." + stateKey) + $_("src.tip.toggle"),
  );
</script>

<button type="button" class="srcb tipr s-{stateKey} {on ? 'on' : ''}" aria-pressed={on} aria-label={tip} data-tip={tip}
  onclick={() => (on = !on)}>
  <Icon name={kind === "sys" ? "display" : "mic"} />
  <span class="mt" aria-hidden="true"><i></i><i></i><i></i><i></i></span>
  <span class="mk" aria-hidden="true">{mark}</span>
</button>
