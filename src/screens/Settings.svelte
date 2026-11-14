<script lang="ts">
  import { _ } from "../i18n/index.ts";
  import Icon from "../lib/Icon.svelte";
  import AiAssistantTab from "./AiAssistantTab.svelte";
  import UsageLogsTab from "./UsageLogsTab.svelte";
  import DriveTab from "./DriveTab.svelte";

  let { tab = $bindable("ai") }: { tab?: "ai" | "drive" | "usage" } = $props();
  const tabs = [
    { id: "ai" as const, key: "set.tab.ai", icon: "sparkle", group: "set.tab.basic" },
    { id: "drive" as const, key: "set.tab.drive", icon: "link", group: "set.tab.basic" },
    { id: "usage" as const, key: "set.tab.usagelogs", icon: "list", group: "set.tab.advanced" },
  ];
</script>

<div class="set">
  <nav class="snav" aria-label={$_("nav.settings")}>
    {#each tabs as t (t.id)}
      <div class="sg">{$_(t.group)}</div>
      <button type="button" class="sn {tab === t.id ? 'on' : ''}" aria-current={tab === t.id ? "page" : undefined} onclick={() => (tab = t.id)}>
        <Icon name={t.icon} />{$_(t.key)}
      </button>
    {/each}
  </nav>
  <div class="sbody">
    {#if tab === "ai"}<AiAssistantTab />{:else if tab === "drive"}<DriveTab />{:else}<UsageLogsTab />{/if}
  </div>
</div>
