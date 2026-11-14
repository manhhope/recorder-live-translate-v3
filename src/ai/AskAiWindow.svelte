<script lang="ts">
  // Noi dung cua cua so Tauri "ask-ai": lay ma phien tu `?session=<uuid>`.
  import { _ } from "svelte-i18n";
  import AskAiPopup from "./AskAiPopup.svelte";
  const sessionId = new URLSearchParams(location.search).get("session") ?? "";
</script>

{#if sessionId}
  <AskAiPopup {sessionId} />
{:else}
  <p class="hint">{$_("ai.no.session")}</p>
{/if}
