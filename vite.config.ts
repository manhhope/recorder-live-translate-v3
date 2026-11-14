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
        main: resolve(__dirname, "index.html"),
        "ask-ai": resolve(__dirname, "ask-ai.html"),
        subtitle: resolve(__dirname, "subtitle.html"),
        capture: resolve(__dirname, "capture.html"),
        quick: resolve(__dirname, "quick.html"),
      },
    },
  },
});
