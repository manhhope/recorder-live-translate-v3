<script lang="ts">
  import { platformize } from "./lib/platform.ts";
  import { onMount } from "svelte";
  import { _, currentLocale, locale, setLocale } from "./i18n/index.ts";
  import * as api from "./lib/api.ts";
  import Icon from "./lib/Icon.svelte";
  import Record from "./screens/Record.svelte";
  import Sessions from "./screens/Sessions.svelte";
  import Settings from "./screens/Settings.svelte";
  import AskAi from "./screens/AskAi.svelte";
  import WhatsNew from "./lib/WhatsNew.svelte";
  import PermWelcome from "./lib/PermWelcome.svelte";
  import { permWelcome, evaluatePermWelcome } from "./lib/perm-welcome.svelte.ts";
  import { whatsNew, evaluateWhatsNew } from "./lib/whats-new.svelte.ts";
  import { listen } from "@tauri-apps/api/event";
  import { upd, startAutoUpdate } from "./lib/update.svelte.ts";
  import { displayAccel } from "./lib/accel.ts";
  import { app, applyTheme, openAsk, type Screen } from "./lib/app.svelte.ts";

  const items: { id: Screen; icon: string; key: string; kbd: string }[] = [
    { id: "rec", icon: "mic", key: "nav.record", kbd: platformize("⌘1") },
    { id: "ses", icon: "clock", key: "nav.sessions", kbd: platformize("⌘2") },
    { id: "ask", icon: "sparkle", key: "nav.askai", kbd: "" },
  ];
  // Phim tat Hoi AI that (nguoi dung doi duoc o Cai dat), hien canh nhan thanh ben; tat thi khong hien.
  let askKbd = $state("");
  async function loadAskKbd() {
    try {
      const a = (await api.shortcutsStatus()).find(s => s.action === "ask_ai");
      askKbd = a?.enabled ? displayAccel(a.accelerator) : "";
    } catch { askKbd = ""; }
  }
  $effect(() => { void app.screen; void loadAskKbd(); });
  // Dong bo khay he thong voi ngon ngu hien tai va ap dung tuy chon giu may thuc da luu (loi thi bo qua: vo chua san sang).
  onMount(() => {
    void api.shellSetLocale(currentLocale()).catch(() => {});
    if (localStorage.getItem(api.KEEP_AWAKE_KEY) === "1") void api.shellSetKeepAwake(true).catch(() => {});
    // Phim tat toan cuc Hoi AI (Rust: shell/mod.rs AskAiOpen) phat shell://deeplink {action: ask_ai}; deeplink rlt://ask-ai cung vay.
    // What's New: sau khi giao dien da ve xong (khung hinh ke tiep), hien mot lan neu phien ban doi so voi lan xem truoc.
    // Thu tu: Welcome cap quyen truoc (chi khi con quyen bat buoc chua cap), What's New sau; Welcome dong xong thi tu goi What's New.
    requestAnimationFrame(() => { void evaluatePermWelcome().then(shown => { if (!shown) evaluateWhatsNew(); }, () => evaluateWhatsNew()); });
    const stopUpd = startAutoUpdate(); // US-33: kiem nen sau vai giay, chi khi cong tu dong bat
    const un = listen<{ action?: string }>("shell://deeplink", e => { if (e.payload?.action === "ask_ai") openAsk(); });
    return () => { stopUpd(); void un.then(f => f()).catch(() => {}); };
  });
  const bannerText = $derived($_("set.upd.banner", { values: { version: upd.info?.version ?? "" } }));
  const isPt = $derived($locale === "pt-PT");
  const dark = $derived(app.theme === "dark" || (app.theme === "system" && matchMedia("(prefers-color-scheme: dark)").matches));
</script>

<div class="app">
  <nav class="nav" aria-label={$_("nav.tools")}>
    {#each items as n (n.id)}
      <button type="button" class="ni {app.screen === n.id ? 'on' : ''}" aria-current={app.screen === n.id ? "page" : undefined} onclick={() => (app.screen = n.id)}>
        <Icon name={n.icon} /><span>{$_(n.key)}</span>{#if (n.id === "ask" ? askKbd : n.kbd)}<kbd>{n.id === "ask" ? askKbd : n.kbd}</kbd>{/if}
      </button>
    {/each}
    <div class="nsp"></div>
    <button type="button" class="ni {app.screen === 'set' ? 'on' : ''}" aria-current={app.screen === "set" ? "page" : undefined} onclick={() => (app.screen = "set")}>
      <Icon name="gear" /><span>{$_("nav.settings")}</span><kbd>{platformize("⌘,")}</kbd>
    </button>
    <div class="ntools">
      <button type="button" class="btn sm" aria-label={$_("ui.lang.aria")} title={$_("ui.lang.btn")} onclick={() => api.setUiLocale(setLocale, isPt ? "en" : "pt-PT")}>
        <Icon name="globe" />{isPt ? "PT" : "EN"}
      </button>
      <button type="button" class="btn sm icon" aria-label={$_(dark ? "theme.dark.hint" : "theme.light.hint")} title={$_(dark ? "theme.dark.hint" : "theme.light.hint")}
        onclick={() => applyTheme(dark ? "light" : "dark")}>
        <Icon name={dark ? "moon" : "sun"} />
      </button>
    </div>
  </nav>
  <main class="main">
    {#if app.screen === "rec"}<Record />{:else if app.screen === "ses"}<Sessions />{:else if app.screen === "ask"}<AskAi />{:else}<Settings bind:tab={app.settingsTab} />{/if}
  </main>
</div>
{#if upd.phase === "ready" && !upd.dismissed && app.screen !== "set"}
  <div class="banner" role="status" data-testid="update-banner" style="position:fixed;right:12px;bottom:12px;z-index:30">
    <span>{bannerText}</span>
    <button type="button" class="btn sm primary" data-testid="update-banner-open" onclick={() => (app.screen = "set")}>{$_("set.upd.bannergo")}</button>
    <button type="button" class="btn sm" data-testid="update-banner-later" onclick={() => (upd.dismissed = true)}>{$_("set.upd.bannerlater")}</button>
  </div>
{/if}
{#if permWelcome.open}<PermWelcome />{/if}
{#if whatsNew.open}<WhatsNew />{/if}
