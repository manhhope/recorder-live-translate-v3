// U-03 canh bao rui ro CLI chi hien 1 lan, tieu de the khong lap icon canh bao; U-25 loi cai dat co kieu loi rieng.
import { test, expect } from "./fixtures.ts";

const missing = { claude_cli: { status: "not_installed" }, codex_cli: { status: "not_installed" }, antigravity_cli: { status: "not_installed" } };

for (const loc of ["en", "pt-PT"] as const) {
  test(`U-03 canh bao CLI gop mot lan, khong icon canh bao trong tieu de the ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc });
    await rlt.nav("nav.settings").click();
    await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
    await expect(page.getByTestId("cli-risk")).toHaveCount(1);
    await expect(page.getByText(rlt.t("set.cli.risk"))).toHaveCount(1);
    for (const p of ["claude_cli", "codex_cli", "antigravity_cli"]) {
      await expect(page.getByTestId(`cli-card-${p}`).locator("h4 .i")).toHaveCount(0);
      // giu nguyen: the van co thanh phan chinh (trang thai, Kiem tra lai, Nang cao)
      await expect(page.getByTestId(`cli-st-${p}`)).toBeVisible();
      await expect(page.getByTestId(`cli-adv-${p}`)).toBeVisible();
    }
  });

  test(`U-25 loi cai dat hien trong khung loi rieng ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc, cli: missing, installFail: true } as never);
    await rlt.nav("nav.settings").click();
    await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
    const c = page.getByTestId("cli-card-codex_cli");
    await c.getByTestId("install-run-codex_cli").click();
    await c.getByTestId("install-confirm-ok-codex_cli").click();
    await expect(c.getByTestId("install-state-codex_cli")).toHaveClass(/errbox/, { timeout: 10_000 });
  });
}
