// Giao dien tren Windows: Ctrl thay Cmd, Credential Manager thay Keychain. Project chromium-win (UA Windows).
import { test, expect } from "./fixtures.ts";

const withAi = { aiKeys: { gemini: true, openai: false } };

test.describe("Windows", () => {
  test("thanh ben hien Ctrl+Shift+A, khong con ky hieu Mac", async ({ rlt }) => {
    await rlt.open(withAi);
    await expect(rlt.nav("nav.askai").locator("kbd")).toHaveText("Ctrl+Shift+A");
  });

  test("khong chuoi nao tren man Ghi con ⌘ hay macOS", async ({ page, rlt }) => {
    await rlt.open(withAi);
    const text = await page.locator("body").innerText();
    expect(text).not.toMatch(/⌘|macOS|Keychain|this Mac/);
  });
});
