<script lang="ts">
  // Nhom tuy chinh hien thi transcript o goc phai thanh trang thai. Rong: nam thang hang voi chu Ready; hep: gom vao popover "Aa".
  // Chi ap cho man Ghi (xem transcript-prefs.ts); cua so phu de co cai dat rieng.
  import { _ } from "../i18n/index.ts";
  import Icon from "./Icon.svelte";
  import { COLOR_IDS, DEFAULTS, SIZES, resolveColor, showsSource, stepSize, withShowSource, type ColorId, type TranscriptPrefs, type View } from "./transcript-prefs.ts";

  let { prefs = $bindable(), view = $bindable(), dark }: { prefs: TranscriptPrefs; view: View; dark: boolean } = $props();
  let open = $state(false);
  let root: HTMLElement | undefined = $state();

  $effect(() => {
    if (!open) return;
    const away = (e: Event) => { if (root && !root.contains(e.target as Node)) open = false; };
    const esc = (e: KeyboardEvent) => { if (e.key === "Escape") { open = false; (root?.querySelector(".tp-aa") as HTMLElement | null)?.focus(); } };
    document.addEventListener("pointerdown", away);
    document.addEventListener("keydown", esc);
    return () => { document.removeEventListener("pointerdown", away); document.removeEventListener("keydown", esc); };
  });

  const colorRows = [["srcColor", "tp.color.src", "tp.short.src"], ["dstColor", "tp.color.dst", "tp.short.dst"]] as const;
  const sizeLbl = $derived($_("tp.size.value", { values: { n: SIZES[prefs.size] } }));
  const levelLbl = $derived($_("tp.size.level", { values: { n: prefs.size + 1, max: SIZES.length } }));
  const step = (d: number) => (prefs = stepSize(prefs, d));
  const pick = (k: "srcColor" | "dstColor", id: ColorId) => (prefs = { ...prefs, [k]: id });
  const reset = () => { prefs = { ...DEFAULTS }; view = DEFAULTS.view; };
  const swatch = (id: ColorId, k: "srcColor" | "dstColor") => resolveColor(id, dark) ?? (k === "srcColor" ? "var(--rlt-text2)" : "var(--rlt-text)");
</script>

<div class="tp" bind:this={root} role="group" aria-label={$_("tp.group")}>
  <button type="button" class="btn sm icon tp-aa" aria-haspopup="true" aria-expanded={open} aria-controls="tp-panel" aria-label={$_("tp.open")} title={$_("tp.open")} onclick={() => (open = !open)}>{"Aa"}</button>
  <div class="tp-panel {open ? 'open' : ''}" id="tp-panel">
    <div class="tp-row" role="group" aria-label={$_("tp.size")}>
      <span class="tp-lbl">{$_("tp.size")}</span>
      <button type="button" class="btn sm icon" aria-label={$_("tp.size.decrease")} title={$_("tp.size.decrease")} disabled={prefs.size === 0} onclick={() => step(-1)}><Icon name="minus" /></button>
      <output class="tp-val" aria-label={levelLbl}>{sizeLbl}</output>
      <button type="button" class="btn sm icon" aria-label={$_("tp.size.increase")} title={$_("tp.size.increase")} disabled={prefs.size === SIZES.length - 1} onclick={() => step(1)}><Icon name="plus" /></button>
    </div>
    {#each colorRows as [k, label, short] (k)}
      <div class="tp-row" role="radiogroup" aria-label={$_(label)}>
        <span class="tp-lbl" aria-hidden="true"><span class="tp-s">{$_(short)}</span><span class="tp-l">{$_(label)}</span></span>
        {#each COLOR_IDS as id (id)}
          <button type="button" class="sw {prefs[k] === id ? 'on' : ''}" role="radio" aria-checked={prefs[k] === id} aria-label={$_("tp.color." + id)} title={$_("tp.color." + id)}
            style:background={swatch(id, k)} data-color={id} onclick={() => pick(k, id)}></button>
        {/each}
      </div>
    {/each}
    <label class="tp-row tp-sw">
      <span class="tp-lbl">{$_("tp.showsrc")}</span>
      <input type="checkbox" class="chk" role="switch" checked={showsSource(view)} onchange={(e) => (view = withShowSource(view, e.currentTarget.checked))} />
    </label>
    <div class="tp-note">
      <span>{$_("tp.scope")}</span>
      <button type="button" class="btn sm" onclick={reset}><Icon name="refresh" />{$_("tp.reset")}</button>
    </div>
  </div>
</div>
