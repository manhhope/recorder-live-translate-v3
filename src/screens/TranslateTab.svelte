<script lang="ts">
  // Tab "Dich va AI": DeepL (dich chu) canh Gemini/OpenAI/Ollama (Hoi AI, OCR) tren cung mot man.
  // Cai dat AI that nam o src/ai/AiSettingsTab.svelte (khoa chi nhap vao, khong doc lai).
  import { _ } from "../i18n/index.ts";
  import AiSettingsTab from "../ai/AiSettingsTab.svelte";
  import DeeplCard from "./DeeplCard.svelte";
</script>

<h3>{$_("set.tab.translate")}</h3>
<DeeplCard />
<AiSettingsTab />
