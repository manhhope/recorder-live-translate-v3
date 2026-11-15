<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "./Icon.svelte";
  import * as api from "./api.ts";
  import type { ExportFormat, ExportContent } from "./api.ts";
  import { loadExportPrefs, saveExportPrefs, applyPickedDir, pickErrorDetail } from "./export-dir.ts";

  let { sessionId, uiLang, onClose, onDone, onError }: {
    sessionId: string; uiLang: string; onClose: () => void; onDone: (out: api.ExportOut) => void; onError: (code: string) => void;
  } = $props();

  const FORMATS: ExportFormat[] = ["md", "txt", "srt", "docx"];
  const CONTENTS: ExportContent[] = ["both", "translation", "source"];
  // Nho lua chon lan truoc (logic o export-dir.ts); mac dinh: Ca nguon va dich, thu muc Documents.
  const p0 = loadExportPrefs(localStorage);
  let format = $state<ExportFormat>(p0.format);
  let content = $state<ExportContent>(p0.content);
  let dir = $state<string | null>(p0.dir);
  let busy = $state(false);
  let pickErr = $state<string | null>(null);
  const pickMsg = $derived(pickErr ? $_("exp.dir.error", { values: { detail: pickErr } }) : "");

  onMount(async () => {
    if (!dir) { try { dir = await api.exportDefaultDir(); } catch { /* o "Save to" hien Documents mac dinh */ } }
  });
  async function pick() {
    pickErr = null;
    try {
      const d = applyPickedDir(dir, await api.exportPickDir());
      if (d !== dir) { dir = d; saveExportPrefs(localStorage, { format, content, dir }); }
    } catch (e) { pickErr = pickErrorDetail(e); }
  }
  async function go() {
    busy = true;
    try {
      const out = await api.sessionExport(sessionId, format, content, uiLang, dir);
      saveExportPrefs(localStorage, { format, content, dir });
      onDone(out);
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
    <code class="dirp" title={dir ?? ""}>{dir ?? $_("exp.dir.default")}</code>
    <button type="button" class="btn sm" onclick={pick}>{$_("exp.dir.change")}</button>
  </div>
  {#if pickErr}<div class="row" role="alert"><span class="bd rec"><Icon name="warn" />{pickMsg}</span></div>{/if}
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
