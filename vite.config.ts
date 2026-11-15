import { defineConfig } from "vite";
import { resolve } from "node:path";
import { readFileSync } from "node:fs";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Phien ban ung dung: tauri.conf.json la nguon duy nhat (test version-sync bat lech voi package.json).
const appVersion: string = JSON.parse(readFileSync(resolve(__dirname, "src-tauri/tauri.conf.json"), "utf8")).version;

export default defineConfig({
  define: { __APP_VERSION__: JSON.stringify(appVersion) },
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
