<script lang="ts">
  import { _ } from "../i18n/index.ts";
  import Icon from "./Icon.svelte";
  import { LANGS, AUTO } from "./langs.ts";

  let { value = $bindable(), withAuto = false, locked = false, cls = "", short = false, ariaKey, tipKey }:
    { value: string; withAuto?: boolean; locked?: boolean; cls?: string; short?: boolean; ariaKey: string; tipKey: string } = $props();

  let open = $state(false);
  let q = $state("");
  const fold = (s: string) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();

  const current = $derived(value === AUTO ? null : LANGS.find(l => l.code === value) ?? null);
  const shown = $derived(LANGS.filter(l => !q || fold($_(l.key)).includes(fold(q)) || l.code.toLowerCase().includes(fold(q))));
  const popular = $derived(shown.filter(l => l.popular));
  const rest = $derived(shown.filter(l => !l.popular));
  const showAuto = $derived(withAuto && (!q || fold($_("lang.auto")).includes(fold(q))));

  function pick(code: string) { value = code; open = false; q = ""; }
  function label(key: string) { return short && key === "lang.pt" ? $_("lang.pt.short") : $_(key); }
</script>

<span class="langwrap">
  <button type="button" class="sel {cls} {open ? 'open' : ''}" aria-haspopup="listbox" aria-expanded={open}
    aria-disabled={locked} disabled={locked}
    aria-label={$_(ariaKey)} data-tip={$_(tipKey) + $_(locked ? "cfg.lock.tip" : "cfg.change.tip")}
    onclick={() => (open = !open)}>
    <span class="v">
      {#if current}<span class="fl">{current.flag}</span> {label(current.key)}
      {:else}<Icon name="globe" /> {$_("lang.auto")}{/if}
    </span>
    <Icon name={locked ? "lock" : "chev"} />
  </button>
  {#if open}
    <div class="menu" role="listbox" style="top:calc(100% + 4px);left:0">
      <div class="srch"><Icon name="search" /><input placeholder={$_("lang.search")} aria-label={$_("lang.search")} bind:value={q} /></div>
      {#if showAuto}
        <button type="button" class="mi {value === AUTO ? 'on' : ''}" role="option" aria-selected={value === AUTO} onclick={() => pick(AUTO)}>
          <Icon name="globe" /> {$_("lang.auto")}{#if value === AUTO}<Icon name="check" />{/if}
        </button>
      {/if}
      {#if popular.length}<div class="mg">{$_("lang.popular")}</div>{/if}
      {#each popular as l (l.code)}
        <button type="button" class="mi {value === l.code ? 'on' : ''}" role="option" aria-selected={value === l.code} onclick={() => pick(l.code)}>
          <span class="fl">{l.flag}</span> {$_(l.key)}<span class="cd">{l.code}</span>
        </button>
      {/each}
      {#if rest.length}<div class="mg">{$_("lang.all")}</div>{/if}
      {#each rest as l (l.code)}
        <button type="button" class="mi {value === l.code ? 'on' : ''}" role="option" aria-selected={value === l.code} onclick={() => pick(l.code)}>
          <span class="fl">{l.flag}</span> {$_(l.key)}<span class="cd">{l.code}</span>
        </button>
      {/each}
    </div>
  {/if}
</span>
