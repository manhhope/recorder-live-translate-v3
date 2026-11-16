// Bo test e2e (T2): chay giao dien that tren chromium headless, lop invoke/Channel cua Tauri la gia (test/e2e/tauri-mock.ts).
// Khong sua src/. Ban build rieng vao .e2e-dist de khong dung vao dist/ cua san pham va tranh loi tai lai cua `vite dev`.
import { defineConfig, devices } from "@playwright/test";

const PORT = 4173;

export default defineConfig({
  testDir: "./test/e2e",
  testMatch: /.*\.spec\.ts/,
  outputDir: "test-results",
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  workers: 4,
  reporter: [["list"]],
  timeout: 30_000,
  expect: { timeout: 7_000 },
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "off",
    viewport: { width: 1100, height: 760 },
    deviceScaleFactor: 1,
  },
  // Desktop Chrome mac dinh co UA Windows; cac test cu kiem chuoi macOS nen ghim UA Mac. Windows co project rieng.
  projects: [
    {
      name: "chromium",
      testIgnore: /windows\.spec\.ts/,
      use: { ...devices["Desktop Chrome"], userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36", viewport: { width: 1100, height: 760 }, headless: true },
    },
    {
      name: "chromium-win",
      testMatch: /windows\.spec\.ts/,
      use: { ...devices["Desktop Chrome"], userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36", viewport: { width: 1100, height: 760 }, headless: true },
    },
  ],
  webServer: {
    command: `npx vite build --outDir .e2e-dist --emptyOutDir && npx vite preview --outDir .e2e-dist --host 127.0.0.1 --port ${PORT} --strictPort`,
    url: `http://127.0.0.1:${PORT}/index.html`,
    reuseExistingServer: false,
    timeout: 180_000,
  },
});
