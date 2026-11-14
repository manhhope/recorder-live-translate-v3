<script lang="ts">
  import { _, formatCurrency, locale, currentLocale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import { WEEKS, WEEK_DAYS, SONIOX_KEY_SHOWN } from "../lib/mock.ts";

  const loc = $derived(($locale as "en" | "pt-PT") ?? currentLocale());
  let week = $state(0);
  let showKey = $state(false);
  const data = $derived(WEEKS[week]);
  const weekKeys = ["set.week.0", "set.week.1", "set.week.2", "set.week.3"];
</script>

<h3>{$_("set.tab.usagelogs")}</h3>

<div class="card">
  <h4>{$_("set.soniox.title")}</h4>
  <div class="rw">
    <div><b>{$_("set.soniox.key")}</b><small>{$_("set.soniox.store")}</small></div>
    <div class="rr">
      <label class="fld"><Icon name="key" /><input readonly aria-label={$_("set.soniox.key")} type={showKey ? "text" : "password"} value={SONIOX_KEY_SHOWN} /></label>
      <button type="button" class="btn sm icon" aria-label={$_("set.key.show")} title={$_("set.key.show")} onclick={() => (showKey = !showKey)}><Icon name="eye" /></button>
      <button type="button" class="btn sm">{$_("set.test")}</button>
      <span class="bd ok"><Icon name="check" />{$_("set.soniox.ok")}</span>
    </div>
  </div>
</div>

<div class="card">
  <h4>{$_("set.tab.usagelogs")}</h4>
  <div class="tl2" style="justify-content:space-between;margin-bottom:8px">
    <span class="seg" role="group" aria-label={$_("set.week.pick")}>
      {#each weekKeys as k, i (k)}<button type="button" class={week === i ? "on" : ""} aria-pressed={week === i} onclick={() => (week = i)}>{$_(k)}</button>{/each}
    </span>
    <button type="button" class="btn sm"><Icon name="refresh" />{$_("common.refresh")}</button>
  </div>
  {#if data.bars.length}
    <div class="hint" style="margin:0">{$_("set.usage.week")}</div>
    <div><b>{$_("set.usage.audio")}</b> {$_("set.usage.cost")} <b>{"≈"} {formatCurrency(data.cost, loc)}</b> <span class="hint">{$_("common.estimate")}</span></div>
    <div class="wkw">
      <div class="wk">
        {#each data.bars as h, i (i)}
          <div><div class="st" style="height:{h * 0.85}%"></div><em>{$_(WEEK_DAYS[i])}</em></div>
        {/each}
      </div>
    </div>
    <div class="hint">{$_("set.usage.note")}</div>
  {:else}
    <div class="empty" style="padding:30px"><Icon name="list" /><b>{$_("set.usage.none")}</b><span>{$_("set.usage.none.hint")}</span>
      <button type="button" class="btn sm"><Icon name="refresh" />{$_("common.refresh")}</button></div>
  {/if}
</div>
