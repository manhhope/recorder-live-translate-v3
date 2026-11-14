<script lang="ts">
  import { _, locale, setLocale } from "./i18n/index.ts";
  import Icon from "./lib/Icon.svelte";
  import Record from "./screens/Record.svelte";
  import Sessions from "./screens/Sessions.svelte";
  import Settings from "./screens/Settings.svelte";
  import { app, applyTheme, type Screen } from "./lib/app.svelte.ts";

  const items: { id: Screen; icon: string; key: string; kbd: string }[] = [
    { id: "rec", icon: "mic", key: "nav.record", kbd: "⌘1" },
    { id: "ses", icon: "clock", key: "nav.sessions", kbd: "⌘2" },
  ];
  const isPt = $derived($locale === "pt-PT");
  const dark = $derived(app.theme === "dark" || (app.theme === "system" && matchMedia("(prefers-color-scheme: dark)").matches));
</script>

<div class="app">
  <nav class="nav" aria-label={$_("nav.tools")}>
    {#each items as n (n.id)}
      <button type="button" class="ni {app.screen === n.id ? 'on' : ''}" aria-current={app.screen === n.id ? "page" : undefined} onclick={() => (app.screen = n.id)}>
        <Icon name={n.icon} /><span>{$_(n.key)}</span><kbd>{n.kbd}</kbd>
      </button>
    {/each}
    <div class="nsp"></div>
    <button type="button" class="ni {app.screen === 'set' ? 'on' : ''}" aria-current={app.screen === "set" ? "page" : undefined} onclick={() => (app.screen = "set")}>
      <Icon name="gear" /><span>{$_("nav.settings")}</span><kbd>{"⌘,"}</kbd>
    </button>
    <div class="ntools">
      <button type="button" class="btn sm" aria-label={$_("ui.lang.aria")} title={$_("ui.lang.btn")} onclick={() => setLocale(isPt ? "en" : "pt-PT")}>
        <Icon name="globe" />{isPt ? "PT" : "EN"}
      </button>
      <button type="button" class="btn sm icon" aria-label={$_(dark ? "theme.dark.hint" : "theme.light.hint")} title={$_(dark ? "theme.dark.hint" : "theme.light.hint")}
        onclick={() => applyTheme(dark ? "light" : "dark")}>
        <Icon name={dark ? "moon" : "sun"} />
      </button>
    </div>
  </nav>
  <main class="main">
    {#if app.screen === "rec"}<Record />{:else if app.screen === "ses"}<Sessions />{:else}<Settings />{/if}
  </main>
</div>
