<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import { invoke } from "@tauri-apps/api/core";
  import { dragBox, errorKey, toGlobal, type Pt } from "./model.ts";

  // Phien dang ghi do trang chinh dat vao localStorage khi mo lop phu; loi cua khong co phien duoc bao ro.
  let start = $state<Pt | null>(null);
  let cur = $state<Pt | null>(null);
  let err = $state("");
  const box = $derived(start && cur ? dragBox(start, cur) : null);

  const winOrigin = (): Pt => ({ x: window.screenX, y: window.screenY });
  const cancel = () => invoke("capture_cancel");

  onMount(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") cancel(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  function down(e: MouseEvent) { err = ""; start = { x: e.clientX, y: e.clientY }; cur = start; }
  function move(e: MouseEvent) { if (start) cur = { x: e.clientX, y: e.clientY }; }
  async function up(e: MouseEvent) {
    if (!start) return;
    const a = toGlobal(winOrigin(), start), b = toGlobal(winOrigin(), { x: e.clientX, y: e.clientY });
    start = cur = null;
    try {
      const session_id = localStorage.getItem("rlt.capture.session") ?? "";
      const offset_ms = Number(localStorage.getItem("rlt.capture.offset_ms") ?? "0");
      await invoke("capture_commit", { sessionId: session_id, offsetMs: offset_ms, start: [a.x, a.y], end: [b.x, b.y] });
    } catch (code) { err = $_(errorKey(String(code))); }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="layer" onmousedown={down} onmousemove={move} onmouseup={up} role="presentation">
  <p class="hint">{err || $_("cap.hint")}</p>
  {#if box}<div class="box" style:left="{box.x}px" style:top="{box.y}px" style:width="{box.w}px" style:height="{box.h}px"></div>{/if}
</div>

<style>
  :global(html), :global(body) { margin: 0; background: transparent; overflow: hidden; }
  .layer { position: fixed; inset: 0; background: rgba(0, 0, 0, 0.25); cursor: crosshair; font-family: -apple-system, system-ui, sans-serif; }
  .hint { position: absolute; top: 16px; left: 50%; transform: translateX(-50%); margin: 0; padding: 6px 12px; border-radius: 8px; background: rgba(16, 20, 24, 0.85); color: #fff; font-size: 14px; }
  .box { position: absolute; border: 2px solid #4fc3f7; background: rgba(79, 195, 247, 0.12); }
</style>
