import { defineConfig } from "vite";
import { resolve } from "node:path";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    rollupOptions: {
      input: {
        main: resolve(import.meta.dirname, "index.html"),
        "ask-ai": resolve(import.meta.dirname, "ask-ai.html"),
      },
    },
  },
});
