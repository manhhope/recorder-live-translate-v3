<script lang="ts">
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import GeneralTab from "./GeneralTab.svelte";
  import TranslateTab from "./TranslateTab.svelte";
  import ShortcutsTab from "./ShortcutsTab.svelte";
  import GlossaryTab from "./GlossaryTab.svelte";
  import SubtitleTab from "./SubtitleTab.svelte";
  import UsageLogsTab from "./UsageLogsTab.svelte";
  import DriveTab from "./DriveTab.svelte";
  import AboutTab from "./AboutTab.svelte";

  type TabId = "general" | "translate" | "shortcuts" | "glossary" | "subtitle" | "usage" | "drive" | "about";
  let { tab = $bindable("general") }: { tab?: TabId } = $props();
  const tabs: { id: TabId; key: string; icon: string; group: string }[] = [
    { id: "general", key: "set.tab.general", icon: "gear", group: "set.tab.basic" },
    { id: "translate", key: "set.tab.translate", icon: "sparkle", group: "set.tab.basic" },
    { id: "shortcuts", key: "set.tab.shortcuts", icon: "bolt", group: "set.tab.basic" },
    { id: "glossary", key: "set.tab.glossary", icon: "book", group: "set.tab.basic" },
    { id: "subtitle", key: "set.tab.subtitle", icon: "caption", group: "set.tab.basic" },
    { id: "usage", key: "set.tab.sonioxusage", icon: "list", group: "set.tab.advanced" },
    { id: "drive", key: "set.tab.drive", icon: "link", group: "set.tab.advanced" },
    { id: "about", key: "set.tab.permissions", icon: "info", group: "set.tab.more" },
  ];
</script>

<div class="set">
  <nav class="snav" aria-label={$_("nav.settings")}>
    {#each tabs as t, i (t.id)}
      {#if i === 0 || tabs[i - 1].group !== t.group}<div class="sg">{$_(t.group)}</div>{/if}
      <button type="button" class="sn {tab === t.id ? 'on' : ''}" aria-current={tab === t.id ? "page" : undefined} onclick={() => (tab = t.id)}>
        <Icon name={t.icon} />{$_(t.key)}
      </button>
    {/each}
  </nav>
  <div class="sbody">
    {#if tab === "general"}<GeneralTab />
    {:else if tab === "translate"}<TranslateTab />
    {:else if tab === "shortcuts"}<ShortcutsTab />
    {:else if tab === "glossary"}<GlossaryTab />
    {:else if tab === "subtitle"}<SubtitleTab />
    {:else if tab === "usage"}<UsageLogsTab />
    {:else if tab === "drive"}<DriveTab />
    {:else}<AboutTab />{/if}
  </div>
</div>
