<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Follow } from "../lib/follow.svelte.ts";
  import { backgroundCss, colorError, liveRows, mergeSnapshot, MODES, rowsForLines, toolbarVisible, upsertLine, type DisplayMode, type SubtitleSettings } from "./model.ts";
  import type { LineDto } from "../lib/api.ts";
  import { dismiss } from "../lib/dismiss.ts";
  import { afterFinal, type LiveText } from "../lib/live.ts";

  interface LiveMsg { kind: "partial" | "clear"; source: string; translation: string }

  let s = $state<SubtitleSettings>({
    display_mode: "bilingual", font_size_pt: 18, source_color: "#D0D6DC", translation_color: "#FFFFFF",
    opacity_percent: 0, hidden_from_capture: true,
  });
  let lines = $state<LineDto[]>([]);
  let live = $state<LiveText | null>(null);   // chu tam: moi ban thay ban truoc, dong chot den thi bo phan da chot
  let hover = $state(false);       // pointerenter/leave cua trang
  let rustHover = $state(false);   // Rust theo doi con tro (NSPanel khong nhan focus co the khong nhan hover cua trang)
  let focusIn = $state(false);     // Tab vao mot nut trong thanh
  let menuOpen = $state(false);
  let err = $state("");
  const follow = new Follow();

  const rows = $derived([...rowsForLines(s.display_mode, lines), ...liveRows(s.display_mode, live)]);
  const barOn = $derived(toolbarVisible(hover, rustHover, focusIn, menuOpen));

  onMount(() => {
    invoke<SubtitleSettings>("subtitle_get").then(v => (s = v));
    // Nghe truoc, lay ban chup sau: dong den trong luc cho ban chup khong bi mat (mergeSnapshot uu tien dong tu su kien).
    const offs = [
      listen<SubtitleSettings>("subtitle://settings", e => (s = e.payload)),
      listen<LineDto>("subtitle://line", e => {
        lines = upsertLine(lines, e.payload);
        live = afterFinal(live, e.payload.sourceText, e.payload.translatedText);
      }),
      listen<LiveMsg>("subtitle://live", e => {
        live = e.payload.kind === "partial" && e.payload.source.trim()
          ? { source: e.payload.source, translation: e.payload.translation, origin: "system", lang: "", offsetMs: 0 } : null;
      }),
      listen<boolean>("subtitle://hover", e => { rustHover = e.payload; if (!e.payload) menuOpen = false; }),
    ];
    Promise.all(offs).then(() => invoke<LineDto[]>("subtitle_lines")).then(snap => (lines = mergeSnapshot(lines, snap))).catch(() => {});
    return () => offs.forEach(p => p.then(f => f()));
  });

  // Chi tieu diem ban phim that (:focus-visible) moi giu thanh hien. Bam chuot vao nut/mau/co chu van focus nut do
  // nhung khong phai ban phim; neu tinh nhu vay thi focusIn ket true va thanh khong bao gio tu an khi con tro roi di.
  const isKeyboardFocus = (t: EventTarget | null) => t instanceof HTMLElement && t.matches(":focus-visible");

  const step = (delta: number) => invoke("subtitle_font_step", { delta });
  const setMode = (mode: DisplayMode) => invoke("subtitle_set_mode", { mode });
  const setOpacity = (e: Event) => invoke("subtitle_set_opacity", { percent: Number((e.target as HTMLInputElement).value) });
  const setHidden = (e: Event) => invoke("subtitle_set_hidden", { hidden: (e.target as HTMLInputElement).checked });

  async function setColor(which: "source" | "translation", e: Event) {
    err = "";
    try { await invoke("subtitle_set_color", { which, hex: (e.target as HTMLInputElement).value }); }
    catch (code) { const c = colorError(String(code)); err = $_(c.key, { values: c.values }); }
  }
  // Keo thanh cuon cua vung chu (phan ben phai .text vuot clientWidth) la cuon, khong phai keo cua so.
  const onScrollbar = (e: MouseEvent) => { const t = e.target as HTMLElement; return t.classList.contains("text") && e.offsetX >= t.clientWidth; };
  const startDrag = (e: MouseEvent) => { if (e.button === 0 && !onScrollbar(e) && !(e.target as HTMLElement).closest("button, select, input, label, .menu")) getCurrentWindow().startDragging(); };
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="sub" style:background={backgroundCss(s.opacity_percent)} onmousedown={startDrag}
     onpointerenter={() => (hover = true)} onpointerleave={() => { hover = false; if (!rustHover) menuOpen = false; }} role="presentation">
  <div class="tools" class:show={barOn} data-drag onfocusin={e => { if (isKeyboardFocus(e.target)) focusIn = true; }} onfocusout={e => { if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node | null)) focusIn = false; }}>
    <button class="mini" onclick={() => step(-1)} disabled={s.font_size_pt <= 12} title={$_("sub.font.decrease")} aria-label={$_("sub.font.decrease")}>{$_("sub.font.decrease.short")}</button>
    <span class="pt">{s.font_size_pt}{$_("sub.font.unit")}</span>
    <button class="mini" onclick={() => step(1)} disabled={s.font_size_pt >= 36} title={$_("sub.font.increase")} aria-label={$_("sub.font.increase")}>{$_("sub.font.increase.short")}</button>
    <select aria-label={$_("sub.mode.label")} title={$_("sub.mode.label")} value={s.display_mode} onchange={e => setMode((e.target as HTMLSelectElement).value as DisplayMode)}>
      {#each MODES as m}<option value={m.mode}>{$_(m.key)}</option>{/each}
    </select>
    <input class="sw" type="color" aria-label={$_("sub.color.source")} title={$_("sub.color.source")} value={s.source_color} onchange={e => setColor("source", e)} />
    <input class="sw" type="color" aria-label={$_("sub.color.translation")} title={$_("sub.color.translation")} value={s.translation_color} onchange={e => setColor("translation", e)} />
    <input class="rg" type="range" min="0" max="80" step="5" aria-label={$_("sub.opacity")} title={$_("sub.opacity")} value={s.opacity_percent} onchange={setOpacity} />
    <span class="grow"></span>
    <div class="more" use:dismiss={{ open: menuOpen, onClose: () => (menuOpen = false), restoreFocus: ".mini" }}>
      <button class="mini" onclick={() => (menuOpen = !menuOpen)} title={$_("sub.more")} aria-label={$_("sub.more")} aria-expanded={menuOpen}>
        <svg class="i" viewBox="0 0 16 16"><circle cx="3" cy="8" r="1"/><circle cx="8" cy="8" r="1"/><circle cx="13" cy="8" r="1"/></svg>
      </button>
      {#if menuOpen}
        <div class="menu" role="menu">
          <button role="menuitem" onclick={() => { invoke("subtitle_reset_colors"); menuOpen = false; }}>{$_("sub.color.reset")}</button>
          <button role="menuitem" onclick={() => { invoke("subtitle_reset_position"); menuOpen = false; }}>{$_("sub.resetposition")}</button>
          <label title={$_("sub.hideshare.note")}><input type="checkbox" checked={s.hidden_from_capture} onchange={setHidden} /> {$_("sub.hideshare")}</label>
        </div>
      {/if}
    </div>
    <button class="mini" onclick={() => invoke("subtitle_close")} title={$_("sub.close")} aria-label={$_("sub.close")}>{$_("sub.close.short")}</button>
  </div>
  {#if err}<p class="err" role="alert">{err}</p>{/if}
  <div class="tw">
    <div class="text" style:font-size="{s.font_size_pt}px" use:follow.action>
      {#each rows as r}
        <p class:live={r.live} style:color={r.kind === "source" ? s.source_color : s.translation_color}>{r.text}</p>
      {:else}
        <p class="wait">{$_("sub.waiting")}</p>
      {/each}
    </div>
    {#if follow.away}<button type="button" class="jump" onclick={follow.jump}>{$_("rec.jump.latest")}</button>{/if}
  </div>
</div>

<style>
  :global(html), :global(body) { margin: 0; background: transparent; overflow: hidden; font-family: -apple-system, system-ui, sans-serif; }
  .sub { box-sizing: border-box; height: 100vh; padding: 0 14px 8px; border-radius: 12px; display: flex; flex-direction: column; user-select: none; }
  .tools { flex: none; height: 24px; display: flex; flex-wrap: nowrap; gap: 6px; align-items: center; font-size: 11px; color: #d9d9de; transition: opacity 0.15s ease; position: relative; opacity: 0; }
  .tools.show { opacity: 1; }
  @media (prefers-reduced-motion: reduce) { .tools { transition: none; } }
  .tools .pt { min-width: 28px; text-align: center; }
  .tools .grow { flex: 1; min-width: 0; }
  .mini { min-width: 22px; height: 22px; padding: 0 4px; display: inline-grid; place-items: center; font-size: 11px; color: #fff; background: transparent; border: 1px solid rgba(255, 255, 255, 0.35); border-radius: 5px; cursor: pointer; }
  .mini:disabled { opacity: 0.4; cursor: not-allowed; }
  .i { width: 14px; height: 14px; fill: currentColor; }
  .tools select { font-size: 11px; height: 22px; max-width: 120px; }
  .sw { width: 22px; height: 22px; padding: 0; border: 1px solid rgba(255, 255, 255, 0.35); border-radius: 50%; background: none; cursor: pointer; }
  .rg { width: 56px; min-width: 0; }
  .more { position: relative; }
  .menu { position: absolute; right: 0; top: 26px; z-index: 5; display: flex; flex-direction: column; gap: 2px; padding: 4px; min-width: 190px; background: rgba(16, 16, 18, 0.96); border: 1px solid rgba(255, 255, 255, 0.12); border-radius: 8px; }
  .menu button, .menu label { text-align: left; font-size: 12px; color: #fff; background: transparent; border: 0; padding: 5px 8px; border-radius: 5px; cursor: pointer; display: flex; gap: 6px; align-items: center; }
  .menu button:hover, .menu label:hover { background: rgba(255, 255, 255, 0.12); }
  .tw { flex: 1; min-height: 0; position: relative; display: flex; }
  /* Chi vung chu nay cuon; thanh .tools (flex:none) nam ngoai. Dung margin-top:auto thay justify-content:flex-end de dau doan van cuon toi duoc. */
  .text { flex: 1; min-height: 0; overflow-y: auto; overflow-x: hidden; display: flex; flex-direction: column; overscroll-behavior: contain; scrollbar-width: thin; }
  .text > :global(p:first-child) { margin-top: auto; }
  .jump { position: absolute; right: 6px; bottom: 6px; height: 22px; padding: 0 10px; font-size: 11px; color: #fff; background: rgba(16, 16, 18, 0.9); border: 1px solid rgba(255, 255, 255, 0.35); border-radius: 11px; cursor: pointer; }
  .text p { margin: 0; line-height: 1.35; text-shadow: 0 1px 2px rgba(0, 0, 0, 0.6); }
  .text p.live { opacity: 0.7; font-style: italic; }
  .wait { color: #8a939c; font-size: 14px; }
  .err { color: #ffb4a8; font-size: 12px; margin: 0 0 4px; }
</style>
