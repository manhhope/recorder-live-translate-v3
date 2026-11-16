// US-57: form "Endpoint tuong thich OpenAI" (LM Studio) trong tab Dich va AI. Lop invoke la gia (tauri-mock.ts).
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";

async function openTab(page: Page, rlt: any) {
  await rlt.nav("nav.settings").click();
  await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
}
const card = (page: Page, rlt: any) => page.getByTestId("compat-card").filter({ has: page.getByRole("heading", { name: rlt.t("set.compat.title") }) });

for (const loc of ["en", "pt-PT"] as const) {
  test.describe(`Endpoint tuong thich OpenAI (${loc})`, () => {
    test("mac dinh LM Studio, khong co model gia, Thu ket noi bi khoa den khi nhap model", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await openTab(page, rlt);
      const c = card(page, rlt);
      await expect(c.getByLabel(rlt.t("set.compat.url"))).toHaveValue("http://localhost:1234/v1");
      await expect(c.getByLabel(rlt.t("set.model.name"))).toHaveValue("");
      await expect(c.getByText(rlt.t("set.compat.model.required"))).toBeVisible();
      await expect(c.getByRole("button", { name: rlt.t("set.test") })).toBeDisabled();
      // Khoa de trong la hop le: o khoa la o mat khau, khong co "da luu".
      await expect(c.getByLabel(rlt.t("set.compat.key"))).toHaveAttribute("type", "password");
      await expect(c.locator(".bd.ok")).toHaveCount(0);
      await c.getByLabel(rlt.t("set.model.name")).fill("qwen2.5-7b-instruct");
      await expect(c.getByRole("button", { name: rlt.t("set.test") })).toBeEnabled();
    });

    test("URL sai: bao loi, khoa Thu ket noi, Tai model va Luu", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await openTab(page, rlt);
      const c = card(page, rlt);
      const url = c.getByLabel(rlt.t("set.compat.url"));
      for (const bad of ["localhost:1234/v1", "ftp://x/v1", "http://user:pw@localhost:1234/v1", "http://localhost:1234/v1?key=abc", "http://a b/v1", ""]) {
        await url.fill(bad);
        await expect(c.getByRole("alert").filter({ hasText: rlt.t("set.compat.url.bad") }), bad).toBeVisible();
        await expect(c.getByRole("button", { name: rlt.t("set.compat.models.load") })).toBeDisabled();
        await expect(page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true })).toBeDisabled();
      }
      for (const ok of ["http://localhost:1234/v1", "http://127.0.0.1:1234/v1", "http://192.168.1.20:1234/v1", "https://api.example.com/v1", "http://localhost:1234"]) {
        await url.fill(ok);
        await expect(c.getByText(rlt.t("set.compat.url.bad"))).toHaveCount(0);
        await expect(page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true })).toBeEnabled();
      }
    });

    test("Tai model: liet ke tu may chu, chon xong luu dung URL va model", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await openTab(page, rlt);
      const c = card(page, rlt);
      await c.getByLabel(rlt.t("set.compat.url")).fill("http://192.168.1.20:1234/v1");
      await c.getByRole("button", { name: rlt.t("set.compat.models.load") }).click();
      await expect(c.getByText(rlt.t("set.compat.models.count", { count: 2 }))).toBeVisible();
      await expect(c.locator("datalist#compat-models option")).toHaveCount(2);
      await c.getByLabel(rlt.t("set.model.name")).fill("qwen2.5-7b-instruct");
      await page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true }).click();
      await expect(page.getByRole("status").filter({ hasText: rlt.t("ai.settings.saved") })).toBeVisible();
      const input = (await rlt.calls("ai_save_settings")).at(-1)!.args.input;
      expect(input.providers.find((p: any) => p.provider === "compat")).toEqual({ provider: "compat", model_name: "qwen2.5-7b-instruct", base_url: "http://192.168.1.20:1234/v1" });
      // Lenh nhan ten nha cung cap, khong nhan khoa.
      expect((await rlt.calls("ai_list_models")).map(x => x.args)).toEqual([{ provider: "compat" }]);
    });

    test("Tai model loi: khong bao do, cho nhap tay", async ({ page, rlt }) => {
      await rlt.open({ locale: loc, fail: { ai_list_models: "provider_unreachable" } });
      await openTab(page, rlt);
      const c = card(page, rlt);
      await c.getByRole("button", { name: rlt.t("set.compat.models.load") }).click();
      await expect(c.getByText(rlt.t("set.compat.models.manual"))).toBeVisible();
      await expect(c.locator(".bd.rec")).toHaveCount(0);
      await expect(page.getByRole("alert").filter({ hasText: rlt.t("ai.err.unreachable.compat") })).toHaveCount(0);
      await c.getByLabel(rlt.t("set.model.name")).fill("my-model");
      await expect(c.getByRole("button", { name: rlt.t("set.test") })).toBeEnabled();
    });

    test("Thu ket noi loi: cau rieng cua endpoint, khong phai cau chung cua dich vu AI", async ({ page, rlt }) => {
      await rlt.open({ locale: loc, fail: { ai_test_connection: "provider_unreachable" } });
      await openTab(page, rlt);
      const c = card(page, rlt);
      await c.getByLabel(rlt.t("set.model.name")).fill("m");
      await c.getByRole("button", { name: rlt.t("set.test") }).click();
      await expect(c.getByRole("status").filter({ hasText: rlt.t("ai.err.unreachable.compat") })).toBeVisible();
      expect(rlt.t("ai.err.unreachable.compat")).not.toBe(rlt.t("ai.err.unreachable"));
      await rlt.setFail("ai_test_connection", null);
      await c.getByRole("button", { name: rlt.t("set.test") }).click();
      await expect(c.getByRole("status").filter({ hasText: rlt.t("ai.test.ok") })).toBeVisible();
    });

    test("khoa tuy chon: luu chi gui di, o nhap bi xoa, go duoc; khong doc lai khoa", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await openTab(page, rlt);
      const c = card(page, rlt);
      const key = c.getByLabel(rlt.t("set.compat.key"));
      await expect(c.getByRole("button", { name: rlt.t("ai.key.save") })).toBeDisabled();
      await key.fill("lm-studio-secret");
      await c.getByRole("button", { name: rlt.t("ai.key.save") }).click();
      await expect(c.locator(".bd.ok")).toContainText(rlt.t("ai.key.saved"));
      await expect(key).toHaveValue("");
      const [call] = await rlt.calls("ai_set_api_key");
      expect(call.args).toEqual({ provider: "compat", value: "lm-studio-secret" });
      await expect(page.locator("body")).not.toContainText("lm-studio-secret");
      await c.getByRole("button", { name: rlt.t("ai.key.remove") }).click();
      await expect(c.locator(".bd.ok")).toHaveCount(0);
    });

    test("khong lo khoa vao cai dat luu: input.providers khong chua truong khoa", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await openTab(page, rlt);
      const c = card(page, rlt);
      await c.getByLabel(rlt.t("set.compat.key")).fill("lm-studio-secret");
      await c.getByLabel(rlt.t("set.model.name")).fill("m");
      await page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true }).click();
      await expect(page.getByRole("status").filter({ hasText: rlt.t("ai.settings.saved") })).toBeVisible();
      for (const call of await rlt.calls("ai_save_settings")) expect(JSON.stringify(call.args)).not.toContain("lm-studio-secret");
    });

    test("Dat lai tab: URL ve mac dinh LM Studio, model ve trong", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await openTab(page, rlt);
      const c = card(page, rlt);
      await c.getByLabel(rlt.t("set.compat.url")).fill("https://api.example.com/v1");
      await c.getByLabel(rlt.t("set.model.name")).fill("x");
      await page.getByRole("button", { name: rlt.t("set.reset.tab") }).click();
      await expect(c.getByLabel(rlt.t("set.compat.url"))).toHaveValue("http://localhost:1234/v1");
      await expect(c.getByLabel(rlt.t("set.model.name"))).toHaveValue("");
    });

    test("co mat o danh sach mo hinh mac dinh, hien that Hoi AI dung nhan", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await openTab(page, rlt);
      const options = page.getByRole("combobox", { name: rlt.t("ask.model.pick") }).locator("option");
      await expect(options.filter({ hasText: rlt.t("ai.provider.compat") })).toHaveCount(1);
    });
  });
}
