<script lang="ts">
  let version = $state("...");
  (async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      version = await invoke<string>("core_version");
    } catch {
      version = "(ngoai Tauri)";
    }
  })();
</script>

<main>
  <h1>Recorder Live Translate v3</h1>
  <p>rlt-core {version}</p>
</main>
