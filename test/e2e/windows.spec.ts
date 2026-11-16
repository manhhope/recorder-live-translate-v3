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

test.describe("Windows: cap nhat (US-33)", () => {
  test("ban moi tai va cai duoc, khong co loi nhac TCC/Keychain cua macOS", async ({ page, rlt }) => {
    await rlt.open({ update: { version: "9.9.9", notes: "Win notes" } });
    await page.getByRole("navigation", { name: rlt.t("nav.tools") }).getByRole("button", { name: new RegExp("^" + rlt.t("nav.settings")) }).click();
    await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.permissions") }).click();
    await expect(page.getByTestId("update-maccaveat")).toHaveCount(0);
    await page.getByTestId("update-check").click();
    await expect(page.getByTestId("update-new")).toHaveText("9.9.9");
    await page.getByTestId("update-download").click();
    await expect(page.getByTestId("update-ready")).toBeVisible();
    await page.getByTestId("update-install").click();
    await expect.poll(() => page.evaluate(() => (window as any).__rlt.calls.filter((x: any) => x.cmd === "update_install").length)).toBe(1);
  });
  test("chu ky sai: khong cai", async ({ page, rlt }) => {
    await rlt.open({ update: { version: "9.9.9" }, fail: { update_download: "update_signature" } });
    await page.getByRole("navigation", { name: rlt.t("nav.tools") }).getByRole("button", { name: new RegExp("^" + rlt.t("nav.settings")) }).click();
    await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.permissions") }).click();
    await page.getByTestId("update-check").click();
    await page.getByTestId("update-download").click();
    await expect(page.getByTestId("update-error")).toContainText(rlt.t("err.updatesignature"));
    await expect(page.getByTestId("update-install")).toHaveCount(0);
  });
});
