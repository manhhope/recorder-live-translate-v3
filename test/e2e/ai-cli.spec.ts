// US-58..US-62: Hoi AI qua dang nhap CLI san co. Lop Tauri la gia (tauri-mock.ts), nen day KHONG kiem chung voi CLI that.
import { test, expect, type Rlt } from "./fixtures.ts";
import type { Page } from "@playwright/test";

const askBtn = (rlt: Rlt) => rlt.page.locator(".tr2").getByRole("button", { name: rlt.t("nav.askai") });

for (const loc of ["en", "pt-PT"] as const) {
  test.describe(`Hoi AI qua CLI (${loc})`, () => {
    test("dong y lan dau: phai tick o rui ro moi bam Dong y duoc, va luc nao cung co ghi chu rui ro", async ({ page, rlt }) => {
      await rlt.open({ locale: loc, aiKeys: { gemini: false, openai: false } });
      await askBtn(rlt).click();
      // Khong co khoa thi KHONG tu chon CLI: phai chon tay (CLI co rui ro dieu khoan).
      await expect(page.getByText(rlt.t("ask.nocfg.title"))).toBeVisible();
      await page.getByLabel(rlt.t("ask.model.pick")).selectOption("claude_cli");
      const dialog = page.getByRole("alertdialog", { name: rlt.t("ai.consent.cli.title") });
      await expect(dialog).toBeVisible();
      await expect(dialog.getByText(rlt.t("ai.consent.cli.body"))).toBeVisible();
      await expect(page.getByTestId("cli-risk")).toHaveText(new RegExp(rlt.t("ask.cli.risk").slice(0, 20)));
      const ok = rlt.btn("ask.consent.ok");
      await expect(ok).toBeDisabled();
      await expect(rlt.btn("ask.chip.summary")).toBeDisabled();
      await dialog.getByLabel(rlt.t("ai.consent.cli.check")).check();
      await expect(ok).toBeEnabled();
      await ok.click();
      await expect(rlt.btn("ask.chip.summary")).toBeEnabled();
      // Ghi chu rui ro van con sau khi da dong y.
      await expect(page.getByTestId("cli-risk")).toBeVisible();
    });

    test("Codex TAT mac dinh: khong co the Codex trong cai dat va khong co trong o chon mac dinh, Claude van co", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await rlt.nav("nav.settings").click();
      await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
      await expect(page.getByTestId("cli-card-claude_cli")).toBeVisible();
      await expect(page.getByTestId("cli-card-codex_cli")).toHaveCount(0);
      const sel = page.getByRole("combobox", { name: rlt.t("ask.model.pick") }).first();
      await expect(sel.locator("option", { hasText: rlt.t("ai.provider.claudecli") })).toHaveCount(1);
      await expect(sel.locator("option", { hasText: rlt.t("ai.provider.codexcli") })).toHaveCount(0);
    });

    test("cai dat: the Claude CLI co ghi chu rui ro, duong dan tuong doi bi chan Luu, thu ket noi chay", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await rlt.nav("nav.settings").click();
      await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
      const c = page.getByTestId("cli-card-claude_cli");
      await expect(c.getByTestId("cli-risk-claude_cli")).toHaveText(rlt.t("set.cli.risk"));
      const path = c.getByLabel(rlt.t("set.cli.path"));
      const save = page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true });
      await path.fill("claude");
      await expect(c.getByRole("alert").filter({ hasText: rlt.t("set.cli.path.bad") })).toBeVisible();
      await expect(save).toBeDisabled();
      await path.fill("/opt/homebrew/bin/claude");
      await expect(c.getByText(rlt.t("set.cli.path.bad"))).toHaveCount(0);
      await expect(save).toBeEnabled();
      await c.getByRole("button", { name: rlt.t("set.test") }).click();
      await expect(c.getByRole("status").filter({ hasText: rlt.t("ai.test.ok") })).toBeVisible();
    });
  });
}
