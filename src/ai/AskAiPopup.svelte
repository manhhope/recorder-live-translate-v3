<script lang="ts">
  // Popup noi "Hoi AI" (phuong an A, US-46). Noi dung mo hinh luon hien bang van ban thuan
  // (`{...}` cua Svelte), KHONG dung {@html}; sanitizer o Rust da bo the va lien ket nguy hiem.
  import { onMount, tick } from "svelte";
  import { _, locale } from "svelte-i18n";
  import Icon from "../lib/Icon.svelte";
  import * as api from "./api.ts";
  import type { AiSettings, ChipKind, Conversation, Message, Provider } from "./api.ts";

  let { sessionId, onOpenSettings = () => {}, onClose = () => {} }: {
    sessionId: string; onOpenSettings?: () => void; onClose?: () => void;
  } = $props();

  const CHIPS: { kind: ChipKind; label: string }[] = [
    { kind: "summary", label: "ask.chip.summary" },
    { kind: "explain", label: "ask.chip.explain" },
    { kind: "action_items", label: "ask.chip.todo" },
  ];
  const MAX = 1000;

  let settings = $state<AiSettings | null>(null);
  let convs = $state<Conversation[]>([]);
  let convId = $state<number | null>(null);
  let msgs = $state<Message[]>([]);
  let provider = $state<Provider | null>(null);
  let input = $state("");
  let busy = $state(false);
  let showList = $state(false);
  let errCode = $state<string | null>(null);
  let scroller: HTMLElement | undefined = $state();

  const cur = $derived(settings?.providers.find(p => p.provider === provider) ?? null);
  // Endpoint tuong thich co khoa tuy chon: san sang khi da co ten model (AC-57.3, AC-57.5).
  const ready = $derived(!!cur && (cur.has_key || (cur.provider === "compat" && !!cur.model_name.trim())));
  const needConsent = $derived(ready && !!cur && !cur.consented);
  const lang = $derived($locale === "pt-PT" ? "pt-PT" : "en");
  const lastFailed = $derived(msgs.length > 0 && msgs[msgs.length - 1].role === "assistant" && msgs[msgs.length - 1].status === "failed" ? msgs[msgs.length - 1] : null);

  const pname = $derived(provider ? $_(api.providerLabelKey(provider)) : "");
  const consentTitle = $derived($_("ai.consent.title", { values: { provider: pname } }));
  const consentBody = $derived($_("ai.consent.body", { values: { provider: pname } }));
  function optLabel(p: api.ProviderSettings): string {
    const n = $_(api.providerLabelKey(p.provider));
    if (p.provider === "compat") return p.model_name.trim() ? n : n + " · " + $_("ask.model.nomodel");
    return p.has_key ? n : n + " · " + $_("ask.model.nokey");
  }

  async function reload() {
    settings = await api.getSettings();
    if (!provider) provider = settings.default_provider ?? settings.providers.find(p => p.has_key)?.provider ?? null;
    convs = await api.listConversations(sessionId);
    if (convId === null && convs.length) await open(convs[0].conversation_id);
  }
  async function open(id: number) {
    convId = id; showList = false; errCode = null;
    msgs = await api.listMessages(id);
    await scrollDown();
  }
  async function scrollDown() { await tick(); scroller?.scrollTo({ top: scroller.scrollHeight }); }
  function newChat() { convId = null; msgs = []; showList = false; errCode = null; }

  function patch(u: api.StreamUpdate) {
    const i = msgs.findIndex(m => m.message_id === u.assistant_message_id);
    if (i >= 0) msgs[i] = { ...msgs[i], content: u.text, status: "streaming" };
    scrollDown();
  }

  async function run(job: (onUpdate: (u: api.StreamUpdate) => void) => Promise<api.AskOutcome>, optimistic: Message[]) {
    if (busy) return;
    busy = true; errCode = null;
    const before = msgs;
    msgs = [...msgs, ...optimistic];
    scrollDown();
    try {
      const out = await job(patch);
      convId = out.conversation_id;
      msgs = await api.listMessages(out.conversation_id);
      convs = await api.listConversations(sessionId);
    } catch (e) {
      msgs = before; errCode = String(e);
      if (convId !== null) msgs = await api.listMessages(convId).catch(() => before);
    } finally { busy = false; scrollDown(); }
  }

  function startAsk(chip: ChipKind | null) {
    const text = chip ? null : input.trim();
    if (!chip && !text) return;
    const stubUser: Message = { message_id: -1, conversation_id: convId ?? -1, role: "user", reply_to_message_id: null, chip_kind: chip, content: chip ? $_(CHIPS.find(c => c.kind === chip)!.label) : text!, status: "completed", model_name: null, error_code: null, context_truncated: null };
    const stubAi: Message = { ...stubUser, message_id: -2, role: "assistant", content: "", status: "streaming", chip_kind: null };
    const params: api.AskParams = { session_id: sessionId, conversation_id: convId, text, chip, provider, ui_lang: lang };
    if (!chip) input = "";
    stubAi.message_id = -2;
    run(onUpdate => api.ask(params, onUpdate), [stubUser, stubAi]);
  }
  function doRetry() {
    if (!lastFailed) return;
    const id = lastFailed.message_id;
    const i = msgs.findIndex(m => m.message_id === id);
    if (i >= 0) msgs[i] = { ...msgs[i], content: "", status: "streaming", error_code: null };
    run(onUpdate => api.retry(id, provider, onUpdate), []);
  }
  async function clear() {
    if (convId === null) return;
    await api.clearHistory(convId);
    msgs = []; errCode = null;
  }
  async function agree() {
    if (!provider) return;
    await api.acceptConsent(provider);
    await reload();
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey && !e.isComposing) { e.preventDefault(); startAsk(null); }
  }
  async function copy(t: string) { try { await navigator.clipboard.writeText(t); } catch { /* khong co quyen clipboard */ } }

  onMount(() => { reload().catch(e => (errCode = String(e))); });
</script>

<section class="ask" aria-label={$_("ask.input")}>
  <header class="hd">
    <label class="mdl">
      <span class="vh">{$_("ask.model.pick")}</span>
      <select class="sel" aria-label={$_("ask.model.pick")} bind:value={provider} disabled={busy}>
        {#if !provider}<option value={null}>{$_("ask.model.none")}</option>{/if}
        {#each settings?.providers ?? [] as p (p.provider)}
          <option value={p.provider}>{optLabel(p)}</option>
        {/each}
      </select>
    </label>
    <div class="acts">
      <button type="button" class="btn sm ghost" onclick={() => (showList = !showList)} aria-expanded={showList}>{$_("ask.list.title")} ({convs.length}/20)</button>
      <button type="button" class="btn sm ghost" onclick={newChat} disabled={busy}><Icon name="plus" />{$_("ask.chat.new")}</button>
      <button type="button" class="btn sm ghost" onclick={clear} disabled={busy || convId === null || msgs.length === 0}><Icon name="trash" />{$_("ask.clear")}</button>
      <button type="button" class="btn icon sm ghost" aria-label={$_("common.close")} onclick={onClose}><Icon name="close" /></button>
    </div>
  </header>

  {#if showList}
    <ul class="list" aria-label={$_("ask.list.title")}>
      {#each convs as c (c.conversation_id)}
        <li><button type="button" class="btn sm ghost" onclick={() => open(c.conversation_id)}>{c.title ?? $_("ask.chat.new")}</button></li>
      {/each}
      {#if convs.length >= 20}<li class="hint">{$_("ask.chat.max")}</li>{/if}
    </ul>
  {/if}

  {#if settings && !ready}
    <div class="banner" role="status">
      <div><b>{$_("ask.nocfg.title")}</b><div class="hint">{$_("ask.nocfg.hint")}</div></div>
      <button type="button" class="btn sm" onclick={onOpenSettings}>{$_("ask.open.ai")}</button>
    </div>
  {:else if needConsent && provider}
    <div class="banner" role="alertdialog" aria-label={consentTitle}>
      <div><b>{consentTitle}</b>
        <div class="hint">{consentBody}</div></div>
      <button type="button" class="btn sm primary" onclick={agree}>{$_("ask.consent.ok")}</button>
    </div>
  {/if}

  <div class="log" bind:this={scroller} aria-live="polite">
    {#if msgs.length === 0}<p class="hint greet">{$_("ask.greet")}</p>{/if}
    {#each msgs as m (m.message_id)}
      <div class="msg {m.role} {m.status}">
        <div class="bub">{m.content}{#if m.status === "streaming" && !m.content}<span class="dots" aria-label={$_("ai.thinking")}>…</span>{/if}</div>
        {#if m.role === "assistant" && m.status === "completed"}
          <div class="mt"><button type="button" class="btn sm ghost" onclick={() => copy(m.content)}>{$_("common.copy")}</button></div>
        {/if}
        {#if m.status === "failed"}
          <div class="err" role="alert">{$_(api.errKey(m.error_code, provider))} {$_("ask.err.keep")}</div>
        {/if}
      </div>
    {/each}
    {#if errCode && !lastFailed}<div class="err" role="alert">{$_(api.errKey(errCode, provider))}</div>{/if}
  </div>

  <div class="chips" role="group" aria-label={$_("ai.chips")}>
    {#each CHIPS as c (c.kind)}
      <button type="button" class="btn sm" disabled={busy || !ready || needConsent} onclick={() => startAsk(c.kind)}>{$_(c.label)}</button>
    {/each}
    {#if lastFailed}<button type="button" class="btn sm primary" disabled={busy} onclick={doRetry}><Icon name="refresh" />{$_("common.retry")}</button>{/if}
    {#if busy}<button type="button" class="btn sm danger" onclick={() => api.cancelAsk()}>{$_("ai.stop")}</button>{/if}
  </div>

  <div class="row">
    <textarea class="ta" rows="2" maxlength={MAX} placeholder={$_("ask.input")} aria-label={$_("ask.input")} bind:value={input} onkeydown={onKey} disabled={!ready || needConsent}></textarea>
    <button type="button" class="btn primary" disabled={busy || !ready || needConsent || !input.trim()} onclick={() => startAsk(null)}>{$_("common.send")}</button>
  </div>
</section>

<style>
  .ask{display:flex;flex-direction:column;gap:8px;width:390px;max-width:100%;height:100%;min-height:420px;background:var(--rlt-card);border:1px solid var(--rlt-line);border-radius:var(--rlt-rc);box-shadow:var(--rlt-sh2);padding:12px;box-sizing:border-box;color:var(--rlt-text)}
  .hd{display:flex;justify-content:space-between;align-items:center;gap:8px;flex-wrap:wrap}
  .acts{display:flex;gap:2px;flex-wrap:wrap;align-items:center}
  .vh{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)}
  .list{list-style:none;margin:0;padding:6px;border:1px solid var(--rlt-line);border-radius:var(--rlt-rb);max-height:140px;overflow:auto}
  .log{flex:1;overflow:auto;display:flex;flex-direction:column;gap:10px;padding:2px}
  .greet{margin:auto;text-align:center}
  .msg{display:flex;flex-direction:column;max-width:92%}
  .msg.user{align-self:flex-end;align-items:flex-end}
  .bub{white-space:pre-wrap;overflow-wrap:anywhere;padding:8px 10px;border-radius:var(--rlt-rb);border:1px solid var(--rlt-line);background:var(--rlt-bg,transparent);font-size:var(--rlt-fs)}
  .msg.user .bub{background:var(--rlt-fill);color:var(--rlt-onfill);border-color:transparent}
  .msg.failed .bub:empty{display:none}
  .err{color:var(--rlt-rec);font-size:12px;margin-top:4px}
  .chips,.row{display:flex;gap:6px;align-items:flex-end;flex-wrap:wrap}
  .row .ta{flex:1;min-height:40px;resize:none}
  .banner{display:flex;gap:8px;align-items:center;justify-content:space-between;padding:8px 10px;border:1px solid var(--rlt-warn);border-radius:var(--rlt-rb)}
</style>
