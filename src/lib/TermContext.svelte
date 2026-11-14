<script lang="ts">
  // Menu chuot phai "Them vao thuat ngu" tren chu boi chon trong bien ban (US-27). Bao quanh vung bien ban;
  // chu boi chon dien san vao hop thoai (chon o dong dich thi dien vao o Ban dich). Khong co chu boi chon thi de menu mac dinh.
  import type { Snippet } from "svelte";
  import { _ } from "../i18n/index.ts";
  import Icon from "./Icon.svelte";
  import * as api from "./api.ts";
  import { DEFAULT_DST, DEFAULT_SRC, LANGS } from "./langs.ts";

  let { dstLang = DEFAULT_DST, children }: { dstLang?: string; children: Snippet } = $props();

  let menu = $state<{ x: number; y: number; text: string; inDst: boolean; lang: string } | null>(null);
  let dlg = $state<{ src: string; dst: string; keep: boolean; srcLang: string; dstLang: string } | null>(null);
  let err = $state<string | null>(null);
  let done = $state(false);

  const errKey = (c: string) => api.errKeyOf(c, k => $_(k) !== k);
  const MAX = 200;

  function onContext(e: MouseEvent) {
    const sel = window.getSelection();
    const text = sel?.toString().trim().replace(/\s+/g, " ") ?? "";
    const el = (sel?.anchorNode?.nodeType === 3 ? sel.anchorNode.parentElement : (sel?.anchorNode as Element | null)) ?? null;
    if (!text || !el || !(e.currentTarget as HTMLElement).contains(el)) return;
    e.preventDefault();
    const lang = el.closest<HTMLElement>("[data-lang]")?.dataset.lang;
    menu = { x: e.clientX, y: e.clientY, text: text.slice(0, MAX), inDst: !!el.closest(".dst"), lang: lang && LANGS.some(l => l.code === lang) ? lang : DEFAULT_SRC };
    done = false;
  }
  function openDialog() {
    if (!menu) return;
    dlg = { src: menu.inDst ? "" : menu.text, dst: menu.inDst ? menu.text : "", keep: false, srcLang: menu.lang, dstLang };
    menu = null; err = null; done = false;
  }
  async function submit() {
    if (!dlg) return;
    if (!dlg.src.trim()) { err = "set.gl.needtext"; return; }
    if (!dlg.keep && !dlg.dst.trim()) { err = "set.gl.need.tr"; return; }
    try {
      await api.glossaryAdd({ termSource: dlg.src.trim(), termTarget: dlg.keep ? null : dlg.dst.trim(), keepOriginal: dlg.keep, sourceLang: dlg.srcLang, targetLang: dlg.dstLang });
      done = true; dlg = null; err = null;
    } catch (e) { err = errKey(api.errCode(e)); }
  }
  function onWindowKey(e: KeyboardEvent) { if (e.key === "Escape") { menu = null; dlg = null; } }
</script>

<svelte:window onkeydown={onWindowKey} onclick={() => (menu = null)} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div oncontextmenu={onContext}>
  {@render children()}
</div>

{#if done}<div class="hint" role="status" style="padding:0 18px">{$_("set.gl.added")}</div>{/if}

{#if menu}
  <div class="ctxmenu" role="menu" style="left:{menu.x}px;top:{menu.y}px">
    <button type="button" role="menuitem" onclick={(e) => { e.stopPropagation(); openDialog(); }}><Icon name="book" />{$_("term.menu.add")}</button>
  </div>
{/if}

{#if dlg}
  <div class="tdlg" role="presentation">
    <div class="card" role="dialog" aria-modal="true" aria-labelledby="term-dlg-t">
      <h4 id="term-dlg-t">{$_("term.dlg.title")}</h4>
      <div class="hint" style="margin:0 0 6px">{$_("term.dlg.hint")}</div>
      <div class="rw"><b>{$_("set.gl.source")}</b><div class="rr"><label class="fld"><input aria-label={$_("set.gl.source")} bind:value={dlg.src} /></label></div></div>
      <div class="rw"><b>{$_("set.gl.target")}</b><div class="rr"><label class="fld"><input aria-label={$_("set.gl.target")} disabled={dlg.keep} bind:value={dlg.dst} /></label></div></div>
      <div class="rw"><b>{$_("set.gl.keep")}</b><div class="rr"><input type="checkbox" class="chk" role="switch" aria-label={$_("set.gl.keep")} bind:checked={dlg.keep} /></div></div>
      <div class="rw"><b>{$_("set.gl.srclang")}</b>
        <div class="rr">
          <select class="sel" aria-label={$_("set.gl.srclang")} bind:value={dlg.srcLang}>{#each LANGS as l (l.code)}<option value={l.code}>{$_(l.key)}</option>{/each}</select>
          <select class="sel" aria-label={$_("set.gl.dstlang")} bind:value={dlg.dstLang}>{#each LANGS as l (l.code)}<option value={l.code}>{$_(l.key)}</option>{/each}</select>
        </div>
      </div>
      {#if err}<div class="hint" role="alert"><span class="bd rec"><Icon name="warn" />{$_(err)}</span></div>{/if}
      <div class="tl2" style="margin-top:10px">
        <button type="button" class="btn primary" onclick={submit}>{$_("set.gl.save")}</button>
        <button type="button" class="btn" onclick={() => (dlg = null)}>{$_("common.cancel")}</button>
      </div>
    </div>
  </div>
{/if}
