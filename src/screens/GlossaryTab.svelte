<script lang="ts">
  // Tab "Thuat ngu": them, xoa, nhap CSV. Sua chua co lenh `glossary_update` o backend nen nut Sua bi khoa kem chu thich.
  import { onMount } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import * as api from "../lib/api.ts";
  import type { TermRow } from "../lib/api.ts";
  import { DEFAULT_DST, DEFAULT_SRC, LANGS } from "../lib/langs.ts";

  let terms = $state<TermRow[]>([]);
  let loading = $state(true);
  let src = $state("");
  let dst = $state("");
  let keep = $state(false);
  let srcLang = $state(DEFAULT_SRC);
  let dstLang = $state(DEFAULT_DST);
  let err = $state<string | null>(null);
  let note = $state<string | null>(null);
  let confirmDel = $state<number | null>(null);
  let fileEl = $state<HTMLInputElement | null>(null);

  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
  const lockNote = $derived($_("set.locked.backend", { values: { cmd: "glossary_update" } }));

  async function load() {
    loading = true;
    try { terms = await api.glossaryList(); } catch (e) { err = api.errCode(e); }
    loading = false;
  }
  onMount(load);

  async function add() {
    err = null; note = null;
    if (!src.trim()) { note = $_("set.gl.needtext"); return; }
    if (!keep && !dst.trim()) { note = $_("set.gl.need.tr"); return; }
    try {
      await api.glossaryAdd({ termSource: src.trim(), termTarget: keep ? null : dst.trim(), keepOriginal: keep, sourceLang: srcLang, targetLang: dstLang });
      src = ""; dst = ""; keep = false; note = $_("set.gl.added");
      await load();
    } catch (e) { err = api.errCode(e); }
  }
  async function remove(id: number) {
    err = null; note = null;
    try { await api.glossaryDelete(id); confirmDel = null; await load(); } catch (e) { err = api.errCode(e); }
  }
  async function onFile(e: Event & { currentTarget: HTMLInputElement }) {
    const f = e.currentTarget.files?.[0];
    e.currentTarget.value = "";
    if (!f) return;
    err = null; note = null;
    try {
      const [n, d, i] = await api.glossaryImportCsv(await f.text(), srcLang, dstLang);
      note = $_("set.gl.import.done", { values: { n, d, i } });
      await load();
    } catch (er) { err = api.errCode(er); }
  }
</script>

<h3>{$_("set.tab.glossary")}</h3>

<div class="card">
  <h4>{$_("set.gl.add")}</h4>
  <div class="rw">
    <b>{$_("set.gl.srclang")}</b>
    <div class="rr">
      <select class="sel" aria-label={$_("set.gl.srclang")} bind:value={srcLang}>{#each LANGS as l (l.code)}<option value={l.code}>{$_(l.key)}</option>{/each}</select>
      <select class="sel" aria-label={$_("set.gl.dstlang")} bind:value={dstLang}>{#each LANGS as l (l.code)}<option value={l.code}>{$_(l.key)}</option>{/each}</select>
    </div>
  </div>
  <div class="rw">
    <b>{$_("set.gl.source")}</b>
    <div class="rr"><label class="fld"><input aria-label={$_("set.gl.source")} bind:value={src} /></label></div>
  </div>
  <div class="rw">
    <b>{$_("set.gl.target")}</b>
    <div class="rr"><label class="fld"><input aria-label={$_("set.gl.target")} disabled={keep} bind:value={dst} /></label></div>
  </div>
  <div class="rw">
    <b>{$_("set.gl.keep")}</b>
    <div class="rr"><input type="checkbox" class="chk" role="switch" aria-label={$_("set.gl.keep")} bind:checked={keep} /></div>
  </div>
  <div class="tl2" style="margin-top:8px">
    <button type="button" class="btn sm primary" onclick={add}><Icon name="plus" />{$_("set.gl.save")}</button>
    <button type="button" class="btn sm" onclick={() => fileEl?.click()}><Icon name="download" />{$_("set.gl.import")}</button>
    <input bind:this={fileEl} type="file" accept=".csv,text/csv" hidden aria-label={$_("set.gl.import")} onchange={onFile} />
    {#if note}<span class="hint" role="status" style="margin:0">{note}</span>{/if}
    {#if err}<span class="bd rec" role="alert"><Icon name="warn" />{$_(errKey(err))}</span>{/if}
  </div>
</div>

<div class="card">
  <h4>{$_("nav.glossary")}</h4>
  {#if loading}
    <div class="empty" style="padding:24px"><Icon name="refresh" /><span>{$_("common.loading")}</span></div>
  {:else if terms.length}
    {#each terms as t (t.id)}
      <div class="rw">
        <div><b>{t.termSource}</b><small>{t.keepOriginal ? $_("set.gl.keeporig") : (t.termTarget ?? "")} {"·"} {t.sourceLang} {"→"} {t.targetLang}</small></div>
        <div class="rr">
          <button type="button" class="btn sm" disabled title={lockNote} aria-label={$_("set.gl.edit")}><Icon name="lock" />{$_("set.gl.edit")}</button>
          {#if confirmDel === t.id}
            <button type="button" class="btn sm stop" onclick={() => remove(t.id)}>{$_("set.gl.delete.confirm")}</button>
            <button type="button" class="btn sm" onclick={() => (confirmDel = null)}>{$_("common.cancel")}</button>
          {:else}
            <button type="button" class="btn sm icon" aria-label={$_("common.delete")} title={$_("common.delete")} onclick={() => (confirmDel = t.id)}><Icon name="trash" /></button>
          {/if}
        </div>
      </div>
    {/each}
    <div class="hint"><Icon name="lock" /> {lockNote}</div>
  {:else}
    <div class="empty" style="padding:24px"><Icon name="list" /><b>{$_("set.gl.empty")}</b><span>{$_("set.gl.empty.hint")}</span></div>
  {/if}
</div>
