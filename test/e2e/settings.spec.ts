import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";

async function openTab(page: Page, rlt: any, tabKey: string) {
  await rlt.nav("nav.settings").click();
  await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t(tabKey) }).click();
}

test.describe("Cai dat: Soniox (trong Usage Logs)", () => {
  test("co khoa: hien Da luu, Thu ket noi ok, Go khoa roi nhap lai", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.sonioxusage");
    await expect(page.getByText(rlt.t("set.soniox.title"))).toBeVisible();
    await expect(page.getByText(rlt.t("set.soniox.saved"))).toBeVisible();

    await page.getByRole("button", { name: rlt.t("set.test") }).click();
    await expect(page.locator(".bd.ok")).toContainText(rlt.t("set.soniox.ok"));
    expect(await rlt.calls("soniox_test_connection")).toHaveLength(1);

    await page.getByRole("button", { name: rlt.t("set.soniox.remove") }).click();
    const input = page.getByLabel(rlt.t("set.soniox.key"));
    await expect(input).toBeVisible();
    const save = page.getByRole("button", { name: rlt.t("rec.key.save") });
    await expect(save).toBeDisabled();
    await input.fill("  sk-new-999  ");
    await save.click();
    await expect(page.getByText(rlt.t("set.soniox.saved"))).toBeVisible();
    const [call] = await rlt.calls("soniox_save_api_key");
    expect(call.args.value).toBe("sk-new-999"); // cat khoang trang
    // Khoa khong bao gio duoc doc lai ve giao dien.
    await expect(page.locator("input[type=password]")).toHaveCount(0);
  });

  test("chua co khoa: o nhap bao loi mau do, o la mat khau", async ({ page, rlt }) => {
    await rlt.open({ hasKey: false });
    await openTab(page, rlt, "set.tab.sonioxusage");
    const input = page.getByLabel(rlt.t("set.soniox.key"));
    await expect(input).toHaveAttribute("type", "password");
    await expect(page.locator("label.fld.err")).toBeVisible();
  });
});

test.describe("Cai dat: nhat ky chan doan", () => {
  test("nut Mo thu muc nhat ky goi lenh diag_open_logs va hien duong dan", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.sonioxusage");
    await page.getByTestId("diag-open-logs").click();
    expect(await rlt.calls("diag_open_logs")).toHaveLength(1);
    await expect(page.getByTestId("diag-msg")).toContainText("com.mmola.recorderlivetranslate");
    await expect(page.getByTestId("diag-msg")).toContainText("logs");
  });

  test("mo thu muc that bai: hien loi co khoa i18n, khong vang loi chung", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.sonioxusage");
    await rlt.setFail("diag_open_logs", "logs_open_failed");
    await page.getByTestId("diag-open-logs").click();
    await expect(page.getByTestId("diag-msg")).toContainText(rlt.t("err.logsopenfailed"));
  });
});

test.describe("Cai dat: Usage Logs", () => {
  test("tuan nay co du lieu: tong thoi luong, chi phi uoc tinh, bieu do 7 cot", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.sonioxusage");
    await expect(page.getByText(rlt.t("set.usage.audio", { h: 3, m: 15 }))).toBeVisible(); // 11.700.000 ms = 195 phut
    await expect(page.getByTestId("usage-source")).toBeVisible();
    await expect(page.locator(".wk > div")).toHaveCount(7);
    await expect(page.locator(".wk .st").nth(4)).toHaveAttribute("style", /height:\s*85%/); // cot cao nhat
  });

  test("tuan khong co du lieu: khong hien $0 nhu da do", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.sonioxusage");
    await page.getByRole("button", { name: rlt.t("set.week.1") }).click();
    await expect(page.getByText(rlt.t("set.usage.none"))).toBeVisible();
    await expect(page.getByText(rlt.t("set.usage.none.hint"))).toBeVisible();
    await expect(page.locator(".wk")).toHaveCount(0);
    await page.getByRole("button", { name: rlt.t("set.week.0") }).click();
    await expect(page.locator(".wk")).toBeVisible();
  });

  test("xuat CSV tai file usage-logs.csv", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.sonioxusage");
    const dl = page.waitForEvent("download");
    await page.getByRole("button", { name: rlt.t("set.usage.export") }).click();
    expect((await dl).suggestedFilename()).toBe("usage-logs.csv");
  });

  test("xoa du lieu muc dung co xac nhan, sau do hien trang thai trong", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.sonioxusage");
    await page.getByRole("button", { name: rlt.t("set.usage.clear") }).click();
    await page.getByRole("button", { name: rlt.t("set.usage.clear.confirm") }).click();
    await expect(page.getByText(rlt.t("set.usage.none"))).toBeVisible();
    expect(await rlt.calls("usage_clear")).toHaveLength(1);
  });
});

test.describe("Cai dat: AI Assistant", () => {
  test("man hinh mac dinh: chua co khoa, nut Luu khoa bi khoa, Thu bi khoa", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.translate");
    const gemini = page.locator(".card", { has: page.getByRole("heading", { name: rlt.t("ai.provider.gemini") }) }).last();
    await expect(gemini.getByRole("button", { name: rlt.t("ai.key.save") })).toBeDisabled();
    await expect(gemini.getByRole("button", { name: rlt.t("set.test") })).toBeDisabled();
    await expect(gemini.getByLabel(rlt.t("set.apikey"))).toHaveAttribute("type", "password");
    await expect(page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true })).toBeVisible();
  });

  test("luu khoa Gemini: o nhap bi xoa, hien Da luu, thu ket noi ok", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.translate");
    const gemini = page.locator(".card", { has: page.getByRole("heading", { name: rlt.t("ai.provider.gemini") }) }).last();
    const key = gemini.getByLabel(rlt.t("set.apikey"));
    await key.fill("AIza-secret");
    await gemini.getByRole("button", { name: rlt.t("ai.key.save") }).click();
    await expect(gemini.locator(".bd.ok")).toContainText(rlt.t("ai.key.saved"));
    await expect(key).toHaveValue("");
    await expect(key).toHaveAttribute("placeholder", rlt.t("ai.key.saved"));

    await gemini.getByRole("button", { name: rlt.t("set.test") }).click();
    await expect(gemini.getByRole("status")).toHaveText(rlt.t("ai.test.ok"));
    // Thu ket noi luu cai dat truoc roi moi thu.
    expect((await rlt.calls("ai_save_settings")).length).toBeGreaterThan(0);

    await gemini.getByRole("button", { name: rlt.t("ai.key.remove") }).click();
    await expect(gemini.locator(".bd.ok")).toHaveCount(0);
  });

  test("thu ket noi AI that bai: khoa bi tu choi", async ({ page, rlt }) => {
    await rlt.open({ aiKeys: { gemini: true, openai: false }, fail: { ai_test_connection: "provider_unauthorized" } });
    await openTab(page, rlt, "set.tab.translate");
    const gemini = page.locator(".card", { has: page.getByRole("heading", { name: rlt.t("ai.provider.gemini") }) }).last();
    await gemini.getByRole("button", { name: rlt.t("set.test") }).click();
    await expect(gemini.getByRole("status")).toHaveText(rlt.t("ai.err.unauthorized"));
  });

  test("luu cai dat: chon mo hinh mac dinh, doi nguong ngu canh, bao Da luu", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.translate");
    await page.getByRole("combobox", { name: rlt.t("ask.model.pick") }).selectOption("openai");
    await page.getByLabel(rlt.t("set.ai.turns")).fill("5");
    await page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true }).click();
    await expect(page.getByRole("status").filter({ hasText: rlt.t("ai.settings.saved") })).toBeVisible();
    const saves = await rlt.calls("ai_save_settings");
    expect(saves.at(-1)!.args.input).toMatchObject({ default_provider: "openai", context_max_turns: 5, context_max_chars: 60000 });
  });

  test("Ollama: khong can khoa, nut Thu ket noi dung duoc", async ({ page, rlt }) => {
    await rlt.open();
    await openTab(page, rlt, "set.tab.translate");
    const card = page.locator(".card", { has: page.getByRole("heading", { name: rlt.t("set.ollama.local") }) }).last();
    await expect(card.getByRole("button", { name: rlt.t("set.test") })).toBeEnabled();
    await expect(card.getByLabel("URL")).toHaveValue("http://localhost:11434");
  });
});
