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

    test("Codex BAT mac dinh (3.0.6): co the Codex trong cai dat va trong o chon mac dinh, canh Claude", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await rlt.nav("nav.settings").click();
      await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
      await expect(page.getByTestId("cli-card-claude_cli")).toBeVisible();
      await expect(page.getByTestId("cli-card-codex_cli")).toBeVisible();
      const sel = page.getByRole("combobox", { name: rlt.t("ask.model.pick") }).first();
      await expect(sel.locator("option", { hasText: rlt.t("ai.provider.claudecli") })).toHaveCount(1);
      await expect(sel.locator("option", { hasText: rlt.t("ai.provider.codexcli") })).toHaveCount(1);
    });

    test("tu do CLI khi mo tab: hien duong dan, phien ban, trang thai; o duong dan nam trong muc Nang cao dang dong", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await rlt.nav("nav.settings").click();
      await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
      const c = page.getByTestId("cli-card-claude_cli");
      await expect(c.getByTestId("cli-st-claude_cli")).toHaveText(rlt.t("set.cli.st.notloggedin"));
      await expect(c.getByTestId("cli-found-claude_cli")).toContainText("/opt/homebrew/bin/claude");
      await expect(c.getByTestId("cli-found-claude_cli")).toContainText("2.1.332");
      await expect(c.getByRole("button", { name: rlt.t("set.login.btn"), exact: true })).toBeVisible();
      await expect(c.getByTestId("cli-help-claude_cli")).toHaveText(rlt.t("set.cli.notloggedin.help"));
      // O duong dan: co trong muc Nang cao nhung dang dong.
      await expect(c.getByTestId("cli-adv-claude_cli")).not.toHaveAttribute("open", "");
      await expect(c.getByLabel(rlt.t("set.cli.path"))).toBeHidden();
      await c.getByText(rlt.t("set.cli.adv")).click();
      await expect(c.getByLabel(rlt.t("set.cli.path"))).toBeVisible();
      expect((await rlt.calls("ai_cli_status")).length).toBeGreaterThanOrEqual(3);
      await page.screenshot({ path: `test/e2e/screenshots/ai-cli-detect-${loc}.png`, fullPage: true });
    });

    test("cai dat: the Claude CLI co ghi chu rui ro, duong dan tuong doi bi chan Luu, thu ket noi chay", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await rlt.nav("nav.settings").click();
      await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
      const c = page.getByTestId("cli-card-claude_cli");
      await expect(c.getByTestId("cli-risk-claude_cli")).toHaveText(rlt.t("set.cli.risk"));
      await c.getByText(rlt.t("set.cli.adv")).click();
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

    test("dang nhap Claude trong app: hien link gia, mo/chep, dan ma, xong; ma khong hien lai; Huy duoc", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await rlt.nav("nav.settings").click();
      await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
      const c = page.getByTestId("cli-card-claude_cli");
      await c.getByRole("button", { name: rlt.t("set.login.btn"), exact: true }).click();
      await expect(c.getByTestId("login-url")).toHaveText("https://claude.ai/oauth/authorize?fake=1");
      await c.getByRole("button", { name: rlt.t("set.login.open") }).click();
      expect((await rlt.calls("ai_login_open")).length).toBe(1);
      // Huy: dung va bao da huy.
      await c.getByRole("button", { name: rlt.t("set.login.cancel") }).click();
      await expect(c.getByText(rlt.t("set.login.cancelled"))).toBeVisible();
      expect((await rlt.calls("ai_login_cancel")).length).toBe(1);
      // Lam lai va dan ma gia.
      await c.getByRole("button", { name: rlt.t("set.login.btn"), exact: true }).click();
      await page.screenshot({ path: `test/e2e/screenshots/ai-cli-claude-link-${loc}.png`, fullPage: true });
      const box = c.getByLabel(rlt.t("set.login.code"));
      await expect(box).toHaveAttribute("type", "password");
      await box.fill("FAKE-CODE-0000");
      await c.getByRole("button", { name: rlt.t("set.login.verify") }).click();
      await expect(c.getByText(rlt.t("set.login.done"))).toBeVisible();
      expect((await rlt.calls("ai_login_code")).length).toBe(1);
      await expect(page.getByText("FAKE-CODE-0000")).toHaveCount(0);
      // Xong thi tu kiem lai: chuyen sang San sang, nut Sign in khong con la nut chinh.
      await expect(c.getByTestId("cli-st-claude_cli")).toHaveText(rlt.t("set.cli.st.ready"));
      await expect(c.getByRole("button", { name: rlt.t("set.login.btn"), exact: true })).toHaveCount(0);
    });

    test("dang nhap Codex trong app: ma 9 ky tu chu to co nut chep, link trang thiet bi co nut mo, roi Da dang nhap", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await rlt.nav("nav.settings").click();
      await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
      const c = page.getByTestId("cli-card-codex_cli");
      await expect(c.getByTestId("cli-found-codex_cli")).toContainText("codex-cli 0.166.0");
      await c.getByRole("button", { name: rlt.t("set.login.btn"), exact: true }).click();
      await expect(c.getByTestId("login-device-code")).toHaveText("ABCD-12345");
      await expect(c.getByRole("button", { name: rlt.t("set.login.copycode") })).toBeVisible();
      await expect(c.getByTestId("login-url")).toHaveText("https://auth.openai.com/codex/device");
      await expect(c.getByLabel(rlt.t("set.login.code"))).toHaveCount(0);
      await page.screenshot({ path: `test/e2e/screenshots/ai-cli-codex-code-${loc}.png`, fullPage: true });
      await c.getByRole("button", { name: rlt.t("set.login.opendevice") }).click();
      expect((await rlt.calls("ai_login_open")).length).toBe(1);
      await expect(c.getByText(rlt.t("set.login.done"))).toBeVisible({ timeout: 10_000 });
      await expect(c.getByTestId("cli-st-codex_cli")).toHaveText(rlt.t("set.cli.st.ready"));
    });

    test("the Antigravity (agy): chua cai thi hien huong dan cai, khong co nut dang nhap trong app, co ghi chu dang nhap o terminal", async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await rlt.nav("nav.settings").click();
      await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
      const c = page.getByTestId("cli-card-antigravity_cli");
      await expect(c).toBeVisible();
      await expect(c.getByRole("button", { name: rlt.t("set.login.btn") })).toHaveCount(0);
      await expect(c.getByTestId("agy-signin-note")).toHaveText(rlt.t("set.cli.signin.agy"));
      await expect(c.getByTestId("cli-risk-antigravity_cli")).toHaveText(rlt.t("set.cli.risk"));
      await expect(c.getByTestId("cli-st-antigravity_cli")).toHaveText(rlt.t("set.cli.st.notinstalled"));
      await expect(c.getByTestId("cli-install-antigravity_cli")).toHaveText(rlt.t("set.cli.install.agy"));
      await c.getByText(rlt.t("set.cli.adv")).click();
      await c.getByRole("button", { name: rlt.t("set.test") }).click();
      await expect(c.getByRole("status").filter({ hasText: rlt.t("ai.test.ok") })).toBeVisible();
    });
  });
}
