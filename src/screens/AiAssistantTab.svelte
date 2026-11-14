<script lang="ts">
  import { _, formatNumber, locale, currentLocale } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import { PROVIDERS, OLLAMA, PROMPT_KEYS, CHIP_LABELS } from "../lib/mock.ts";

  const loc = $derived(($locale as "en" | "pt-PT") ?? currentLocale());
  let defaultModel = $state("gemini");
  let prompts = $state(PROMPT_KEYS.map(k => $_(k)));
  const limit = "60000";
  const turns = "10";
</script>

<h3>{$_("set.tab.ai")}</h3>

<div class="card">
  <h4>{$_("set.ai.default")}</h4>
  <div class="rw">
    <div><b>{$_("set.ai.default.hint")}</b><small>{$_("set.ai.default.note")}</small></div>
    <div class="rr">
      <select class="sel" aria-label={$_("ask.model.pick")} bind:value={defaultModel}>
        {#each PROVIDERS as p (p.id)}<option value={p.id}>{p.name}</option>{/each}
      </select>
    </div>
  </div>
  <div class="two" style="margin-top:6px">
    {#each PROVIDERS as p (p.id)}
      <div class="card">
        <h4>{p.name}</h4>
        <div class="rw"><b>{$_("set.apikey")}</b>
          <div class="rr">
            <label class="fld {p.hasKey ? '' : 'err'}"><Icon name="key" /><input readonly aria-label={$_("set.apikey")} value={p.hasKey ? p.keyShown : ""} placeholder={$_("set.key.none")} /></label>
            {#if p.hasKey}<span class="bd acc">{$_("set.ai.shared")}</span>{/if}
          </div>
        </div>
        <div class="rw"><b>{$_("set.model.name")}</b>
          <div class="rr"><label class="fld"><input aria-label={$_("set.model.name")} value={p.model} /></label>
            <button type="button" class="btn sm ghost"><Icon name="refresh" />{$_("common.default")}</button></div>
        </div>
        <div class="rr" style="padding-top:6px"><button type="button" class="btn sm">{$_("set.test")}</button></div>
      </div>
    {/each}
  </div>
  <div class="card" style="margin-top:14px">
    <h4>{$_("set.ollama.local")}</h4>
    <div class="rw"><b>{"URL"}</b><div class="rr"><label class="fld"><input aria-label={"URL"} value={OLLAMA.url} /></label></div></div>
    <div class="rw"><b>{$_("set.model.name")}</b>
      <div class="rr"><label class="fld"><input aria-label={$_("set.model.name")} value={OLLAMA.model} /></label>
        <button type="button" class="btn sm ghost"><Icon name="refresh" />{$_("common.default")}</button></div></div>
    <div class="rr" style="padding-top:6px"><button type="button" class="btn sm">{$_("set.test")}</button></div>
    <p class="hint">{$_("set.ollama.need")}</p>
  </div>
</div>

<div class="card">
  <h4>{$_("set.ai.prompts")}</h4>
  {#each CHIP_LABELS as c, i (c)}
    <div class="lab"><span>{$_(c)}</span><button type="button" class="btn sm ghost" onclick={() => (prompts[i] = $_(PROMPT_KEYS[i]))}>{$_("common.restore")}</button></div>
    <textarea class="ta" rows="2" aria-label={$_(c)} bind:value={prompts[i]}></textarea>
    <div class="cnt">{formatNumber(prompts[i].length, loc)} / {formatNumber(2000, loc)}</div>
  {/each}
</div>

<div class="card">
  <h4>{$_("set.ai.ctx")}</h4>
  <div class="rw"><b>{$_("set.ai.chars")}</b><div class="rr"><label class="fld"><input aria-label={$_("set.ai.chars")} value={$_("set.limit.60000")} /></label><span class="hint">{$_("set.ai.chars.range")}</span></div></div>
  <div class="rw"><b>{$_("set.ai.turns")}</b><div class="rr"><label class="fld"><input aria-label={$_("set.ai.turns")} value={turns} /></label><span class="hint">{$_("set.ai.turns.range")}</span></div></div>
</div>

<div class="tl2"><button type="button" class="btn"><Icon name="refresh" />{$_("set.reset.tab")}</button><span class="hint">{$_("set.reset.nokey")}</span></div>
