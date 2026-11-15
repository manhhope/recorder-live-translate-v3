<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "./Icon.svelte";
  import * as api from "./api.ts";
  import type { ExportFormat, ExportContent } from "./api.ts";
  import { loadExportPrefs, saveExportPrefs, rememberDir } from "./export-dir.ts";

  let { sessionId, uiLang, onClose, onDone, onError }: {
    sessionId: string; uiLang: string; onClose: () => void; onDone: (out: api.ExportOut) => void; onError: (code: string) => void;
  } = $props();

  const FORMATS: ExportFormat[] = ["md", "txt", "srt", "docx"];
  const CONTENTS: ExportContent[] = ["both", "translation", "source"];
  // Nho lua chon lan truoc (logic o export-dir.ts); mac dinh: Ca nguon va dich; thu muc khoi dau cua hop thoai luu = thu muc lan truoc hoac Documents (Rust).
  const p0 = loadExportPrefs(localStorage);
  let format = $state<ExportFormat>(p0.format);
  let content = $state<ExportContent>(p0.content);
  let dir = p0.dir;
  let busy = $state(false);
  const fmtHint = (f: ExportFormat) => (f === "docx" ? $_("exp.docx.hint") : f === "srt" ? $_("exp.srt.hint") : undefined);

  async function go() {
    busy = true;
    saveExportPrefs(localStorage, { format, content, dir });
    try {
      const out = await api.sessionExport(sessionId, format, content, uiLang, dir);
      if (out) { // null = huy hop thoai luu: khong ghi, khong bao loi, dialog van mo
        dir = rememberDir(dir, out.path);
        saveExportPrefs(localStorage, { format, content, dir });
        onDone(out);
      }
    } catch (e) { onError(api.errCode(e)); }
    busy = false;
  }
</script>

<div class="card exp" role="dialog" aria-modal="true" aria-labelledby="exp-t" style="margin:8px 12px">
  <b id="exp-t">{$_("exp.title")}</b>
  <div class="row">
    <div class="grp"><span class="hint">{$_("exp.format")}</span>
      <div class="seg" role="radiogroup" aria-label={$_("exp.format")}>
        {#each FORMATS as f (f)}
          <button type="button" role="radio" aria-checked={format === f} class:on={format === f} title={fmtHint(f)} onclick={() => (format = f)}>{f.toUpperCase()}</button>
        {/each}
      </div>
    </div>
    <div class="grp"><span class="hint">{$_("exp.content")}</span>
      <div class="seg" role="radiogroup" aria-label={$_("exp.content")}>
        {#each CONTENTS as c (c)}
          <button type="button" role="radio" aria-checked={content === c} class:on={content === c} onclick={() => (content = c)}>{$_("exp.content." + c)}</button>
        {/each}
      </div>
    </div>
    <div class="grp acts">
      <button type="button" class="btn sm" disabled={busy} onclick={go}><Icon name="download" />{$_("exp.go")}</button>
      <button type="button" class="btn sm" onclick={onClose}>{$_("exp.cancel")}</button>
    </div>
  </div>
</div>

<style>
  .exp { display: flex; flex-direction: column; gap: 8px; }
  .row { display: flex; align-items: center; gap: 8px 10px; flex-wrap: wrap; }
  .exp .seg > * { padding: 3px 5px; }
  .grp { display: flex; align-items: center; gap: 5px; }
  .acts { margin-left: auto; }
</style>
