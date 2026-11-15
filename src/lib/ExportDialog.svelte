<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "./Icon.svelte";
  import * as api from "./api.ts";
  import type { ExportFormat, ExportContent } from "./api.ts";

  let { sessionId, uiLang, onClose, onDone, onError }: {
    sessionId: string; uiLang: string; onClose: () => void; onDone: (path: string) => void; onError: (code: string) => void;
  } = $props();

  const KEY = "rlt.export.prefs";
  const FORMATS: ExportFormat[] = ["md", "txt", "srt", "docx"];
  const CONTENTS: ExportContent[] = ["both", "translation", "source"];
  // Nho lua chon lan truoc; gia tri la thi bo qua (mac dinh: Ca nguon va dich, thu muc Documents).
  function loadPrefs(): { format: ExportFormat; content: ExportContent; dir: string | null } {
    try {
      const o = JSON.parse(localStorage.getItem(KEY) ?? "{}");
      return {
        format: FORMATS.includes(o.format) ? o.format : "md",
        content: CONTENTS.includes(o.content) ? o.content : "both",
        dir: typeof o.dir === "string" && o.dir ? o.dir : null,
      };
    } catch { return { format: "md", content: "both", dir: null }; }
  }
  const p0 = loadPrefs();
  let format = $state<ExportFormat>(p0.format);
  let content = $state<ExportContent>(p0.content);
  let dir = $state<string | null>(p0.dir);
  let busy = $state(false);

  onMount(async () => {
    if (!dir) { try { dir = await api.exportDefaultDir(); } catch { /* de trong: Rust tu dung Documents */ } }
  });
  async function pick() {
    try { const d = await api.exportPickDir(); if (d) dir = d; } catch (e) { onError(api.errCode(e)); }
  }
  async function go() {
    busy = true;
    try {
      const path = await api.sessionExport(sessionId, format, content, uiLang, dir);
      try { localStorage.setItem(KEY, JSON.stringify({ format, content, dir })); } catch { /* khong luu duoc thi thoi */ }
      onDone(path);
    } catch (e) { onError(api.errCode(e)); }
    busy = false;
  }
</script>

<div class="card exp" role="dialog" aria-modal="true" aria-labelledby="exp-t" style="margin:8px 18px">
  <b id="exp-t">{$_("exp.title")}</b>
  <div class="row"><span class="hint">{$_("exp.format")}</span>
    <div class="seg" role="radiogroup" aria-label={$_("exp.format")}>
      {#each FORMATS as f (f)}
        <button type="button" role="radio" aria-checked={format === f} class:on={format === f} onclick={() => (format = f)}>{f.toUpperCase()}</button>
      {/each}
    </div>
  </div>
  {#if format === "docx" || format === "srt"}<div class="hint">{$_(format === "docx" ? "exp.docx.hint" : "exp.srt.hint")}</div>{/if}
  <div class="row"><span class="hint">{$_("exp.content")}</span>
    <div class="seg" role="radiogroup" aria-label={$_("exp.content")}>
      {#each CONTENTS as c (c)}
        <button type="button" role="radio" aria-checked={content === c} class:on={content === c} onclick={() => (content = c)}>{$_("exp.content." + c)}</button>
      {/each}
    </div>
  </div>
  <div class="row"><span class="hint">{$_("exp.dir")}</span>
    <code class="dirp" title={dir ?? ""}>{dir ?? ""}</code>
    <button type="button" class="btn sm" onclick={pick}>{$_("exp.dir.change")}</button>
  </div>
  <div class="row">
    <button type="button" class="btn sm" disabled={busy} onclick={go}><Icon name="download" />{$_("exp.go")}</button>
    <button type="button" class="btn sm" onclick={onClose}>{$_("exp.cancel")}</button>
  </div>
</div>

<style>
  .exp { display: flex; flex-direction: column; gap: 8px; }
  .row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .dirp { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
