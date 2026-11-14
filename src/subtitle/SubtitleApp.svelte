<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { backgroundCss, colorError, MODES, rowsFor, type DisplayMode, type Line, type SubtitleSettings } from "./model.ts";

  let s = $state<SubtitleSettings>({
    display_mode: "bilingual", font_size_pt: 18, source_color: "#D0D6DC", translation_color: "#FFFFFF",
    opacity_percent: 0, hidden_from_capture: true,
  });
  let line = $state<Line | null>(null);
  let toolsOpen = $state(false);
  let err = $state("");

  const rows = $derived(rowsFor(s.display_mode, line));

  onMount(() => {
    invoke<SubtitleSettings>("subtitle_get").then(v => (s = v));
    const offs = [
      listen<SubtitleSettings>("subtitle://settings", e => (s = e.payload)),
      listen<Line>("subtitle://line", e => (line = e.payload)),
    ];
    return () => offs.forEach(p => p.then(f => f()));
  });

  const step = (delta: number) => invoke("subtitle_font_step", { delta });
  const setMode = (mode: DisplayMode) => invoke("subtitle_set_mode", { mode });
  const setOpacity = (e: Event) => invoke("subtitle_set_opacity", { percent: Number((e.target as HTMLInputElement).value) });
  const setHidden = (e: Event) => invoke("subtitle_set_hidden", { hidden: (e.target as HTMLInputElement).checked });

  async function setColor(which: "source" | "translation", e: Event) {
    err = "";
    try { await invoke("subtitle_set_color", { which, hex: (e.target as HTMLInputElement).value }); }
    catch (code) { const c = colorError(String(code)); err = $_(c.key, { values: c.values }); }
  }
  const startDrag = (e: MouseEvent) => { if (e.button === 0 && !(e.target as HTMLElement).closest(".tools")) getCurrentWindow().startDragging(); };
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="sub" style:background={backgroundCss(s.opacity_percent)} onmousedown={startDrag}
     onmouseenter={() => (toolsOpen = true)} onmouseleave={() => (toolsOpen = false)} role="presentation">
  {#if toolsOpen}
    <div class="tools">
      <button onclick={() => step(-1)} disabled={s.font_size_pt <= 12} title={$_("sub.font.decrease")}>{$_("sub.font.decrease.short")}</button>
      <span>{s.font_size_pt}{$_("sub.font.unit")}</span>
      <button onclick={() => step(1)} disabled={s.font_size_pt >= 36} title={$_("sub.font.increase")}>{$_("sub.font.increase.short")}</button>
      <select aria-label={$_("sub.mode.label")} value={s.display_mode} onchange={e => setMode((e.target as HTMLSelectElement).value as DisplayMode)}>
        {#each MODES as m}<option value={m.mode}>{$_(m.key)}</option>{/each}
      </select>
      <input type="color" aria-label={$_("sub.color.source")} title={$_("sub.color.source")} value={s.source_color} onchange={e => setColor("source", e)} />
      <input type="color" aria-label={$_("sub.color.translation")} title={$_("sub.color.translation")} value={s.translation_color} onchange={e => setColor("translation", e)} />
      <button onclick={() => invoke("subtitle_reset_colors")}>{$_("sub.color.reset")}</button>
      <input type="range" min="0" max="80" step="5" aria-label={$_("sub.opacity")} title={$_("sub.opacity")} value={s.opacity_percent} onchange={setOpacity} />
      <label title={$_("sub.hideshare.note")}><input type="checkbox" checked={s.hidden_from_capture} onchange={setHidden} /> {$_("sub.hideshare")}</label>
      <button onclick={() => invoke("subtitle_reset_position")}>{$_("sub.resetposition")}</button>
      <button onclick={() => invoke("subtitle_close")} aria-label={$_("sub.close")}>{$_("sub.close.short")}</button>
    </div>
  {/if}
  {#if err}<p class="err" role="alert">{err}</p>{/if}
  <div class="text" style:font-size="{s.font_size_pt}px">
    {#each rows as r}
      <p style:color={r.kind === "source" ? s.source_color : s.translation_color}>{r.text}</p>
    {:else}
      <p class="wait">{$_("sub.waiting")}</p>
    {/each}
  </div>
</div>

<style>
  :global(html), :global(body) { margin: 0; background: transparent; overflow: hidden; font-family: -apple-system, system-ui, sans-serif; }
  .sub { box-sizing: border-box; height: 100vh; padding: 10px 16px; border-radius: 10px; display: flex; flex-direction: column; justify-content: flex-end; user-select: none; }
  .tools { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 12px; color: #d0d6dc; margin-bottom: 6px; }
  .tools button, .tools select { font-size: 12px; }
  .text p { margin: 0; line-height: 1.35; text-shadow: 0 1px 2px rgba(0, 0, 0, 0.6); }
  .wait { color: #8a939c; font-size: 14px; }
  .err { color: #ffb4a8; font-size: 12px; margin: 0 0 4px; }
</style>
