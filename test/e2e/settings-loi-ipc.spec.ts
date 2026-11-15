// Windows: lenh IPC tai cai dat loi hoac treo thi tab phai bao loi kem nut thu lai, khong duoc trong im lang.
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";

async function openTab(page: Page, rlt: any, tabKey: string) {
  await rlt.nav("nav.settings").click();
  await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t(tabKey) }).click();
}
const retry = (page: Page, rlt: any) => page.getByRole("button", { name: rlt.t("common.retry") });

test.describe("Cai dat: lenh IPC loi hoac cham", () => {
  test("Dich va AI: ai_get_settings loi -> bao loi + Thu lai, bam la hien ra form day du", async ({ page, rlt }) => {
    await rlt.open({ fail: { ai_get_settings: "store_busy" } });
    await openTab(page, rlt, "set.tab.translate");
    await expect(page.getByRole("alert").filter({ hasText: rlt.t("set.load.err") })).toBeVisible();
    await expect(page.getByText(rlt.t("set.ollama.local"))).toHaveCount(0);
    await rlt.setFail("ai_get_settings", null);
    await retry(page, rlt).click();
    await expect(page.getByText(rlt.t("set.ollama.local"))).toBeVisible();
    await expect(page.getByRole("alert").filter({ hasText: rlt.t("set.load.err") })).toHaveCount(0);
    expect((await rlt.calls("ai_get_settings")).length).toBe(2);
  });

  test("Dich va AI: ai_get_settings cham van hien Dang tai roi hien form", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.setDelay("ai_get_settings", 1200);
    await openTab(page, rlt, "set.tab.translate");
    await expect(page.getByRole("status").filter({ hasText: rlt.t("common.loading") })).toBeVisible();
    await expect(page.getByText(rlt.t("set.ollama.local"))).toBeVisible();
  });

  test("Dich va AI: ai_get_settings treo han -> het 8 giay bao loi timeout + Thu lai", async ({ page, rlt }) => {
    await page.clock.install();
    await rlt.open();
    await rlt.setDelay("ai_get_settings", -1);
    await openTab(page, rlt, "set.tab.translate");
    await expect(page.getByRole("status").filter({ hasText: rlt.t("common.loading") })).toBeVisible();
    await page.clock.runFor(8100);
    await expect(page.getByRole("alert").filter({ hasText: rlt.t("set.load.timeout") })).toBeVisible();
    await expect(retry(page, rlt)).toBeEnabled();
  });

  test("Dich va AI: luu cai dat loi thi hien loi, khong im lang", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.translate");
    await expect(page.getByText(rlt.t("set.ollama.local"))).toBeVisible();
    await rlt.setFail("ai_save_settings", "store_busy");
    await page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true }).click();
    await expect(page.getByRole("alert").filter({ hasText: rlt.t("ai.err.generic") })).toBeVisible();
  });

  test("Phu de: subtitle_get loi -> bao loi + Thu lai, bam la hien cai dat", async ({ page, rlt }) => {
    await rlt.open({ fail: { subtitle_get: "store_busy" } });
    await openTab(page, rlt, "set.tab.subtitle");
    await expect(page.getByRole("alert").filter({ hasText: rlt.t("set.load.err") })).toBeVisible();
    await expect(page.getByText(rlt.t("sub.hideshare"))).toHaveCount(0);
    await rlt.setFail("subtitle_get", null);
    await retry(page, rlt).click();
    await expect(page.getByRole("switch", { name: rlt.t("sub.hideshare") })).toBeVisible();
  });

  test("Phu de: subtitle_get treo han -> timeout + Thu lai", async ({ page, rlt }) => {
    await page.clock.install();
    await rlt.open();
    await rlt.setDelay("subtitle_get", -1);
    await openTab(page, rlt, "set.tab.subtitle");
    await expect(page.getByRole("status").filter({ hasText: rlt.t("common.loading") })).toBeVisible();
    await page.clock.runFor(8100);
    await expect(page.getByRole("alert").filter({ hasText: rlt.t("set.load.timeout") })).toBeVisible();
    await rlt.setDelay("subtitle_get", null);
    await retry(page, rlt).click();
    await expect(page.getByRole("switch", { name: rlt.t("sub.hideshare") })).toBeVisible();
  });

  test("Khoi dong cung may: autostart_get loi -> cong tat khoa + bao loi + Thu lai mo lai cong tat", async ({ page, rlt }) => {
    await rlt.open({ fail: { autostart_get: "registry_denied" } });
    await openTab(page, rlt, "set.tab.general");
    const sw = page.getByRole("switch", { name: rlt.t("set.gen.autostart") });
    await expect(page.getByRole("alert").filter({ hasText: rlt.t("set.gen.err") })).toBeVisible();
    await expect(sw).toBeDisabled();
    await rlt.setFail("autostart_get", null);
    await retry(page, rlt).click();
    await expect(sw).toBeEnabled();
    await expect(page.getByRole("alert")).toHaveCount(0);
  });

  test("Khoi dong cung may: autostart_get treo han -> timeout + Thu lai", async ({ page, rlt }) => {
    await page.clock.install();
    await rlt.open();
    await rlt.setDelay("autostart_get", -1);
    await openTab(page, rlt, "set.tab.general");
    await page.clock.runFor(8100);
    await expect(page.getByRole("alert").filter({ hasText: rlt.t("set.load.timeout") })).toBeVisible();
    await rlt.setDelay("autostart_get", null);
    await retry(page, rlt).click();
    await expect(page.getByRole("switch", { name: rlt.t("set.gen.autostart") })).toBeEnabled();
  });

  test("Khoi dong cung may: autostart_set loi -> bao loi va cong tat tro ve trang thai that", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.general");
    const sw = page.getByRole("switch", { name: rlt.t("set.gen.autostart") });
    await expect(sw).toBeEnabled();
    await rlt.setFail("autostart_set", "registry_denied");
    await sw.click();
    await expect(page.getByRole("alert").filter({ hasText: rlt.t("set.gen.err") })).toBeVisible();
    await expect(sw).not.toBeChecked();
    await expect(sw).toBeEnabled();
  });

  test("Thong bao ghi am: cong tat bi khoa kem ghi chu ro rang (chua co o backend), khong im lang", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.general");
    await expect(page.getByRole("switch", { name: rlt.t("set.gen.notify") })).toBeDisabled();
    await expect(page.getByText(rlt.t("set.gen.notify.na"))).toBeVisible();
  });
});
