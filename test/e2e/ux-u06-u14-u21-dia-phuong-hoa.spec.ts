// U-06 so it/so nhieu + nhan dong meta phien, U-14 trang thai phien rong co nut bat dau, U-21 gio kiem tra cap nhat theo ngon ngu UI.
import { test, expect } from "./fixtures.ts";

for (const loc of ["en", "pt-PT"] as const) {
  test(`U-06 dong meta phien: 1 dong la so it, co nhan cho dong meta ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc });
    await rlt.nav("nav.sessions").click();
    const rows = page.locator(".sesrow");
    await expect(rows.nth(1).locator("small")).toContainText(rlt.t("ses.lines.one", { n: 1 }));
    await expect(rows.nth(1).locator("small")).not.toContainText(rlt.t("ses.lines", { n: 1 }));
    await expect(rows.nth(0).locator("small")).toContainText(rlt.t("ses.lines", { n: 3 }));
    await expect(rows.nth(0).locator("small")).toHaveAttribute("title", rlt.t("ses.meta.label"));
    await expect(rows.nth(0).locator("small .vh")).toHaveText(rlt.t("ses.meta.label") + ":");
  });

  test(`U-14 phien rong co nut Bat dau phien dau tien, dua ve man Ghi ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc, sessions: [], details: {} });
    await rlt.nav("nav.sessions").click();
    await expect(page.getByText(rlt.t("ses.empty"))).toBeVisible();
    await page.getByTestId("ses-empty-start").click();
    await expect(page.getByTestId("ses-empty-start")).toHaveCount(0);
    await expect(rlt.btn("tool.start")).toBeVisible();
  });

  test(`U-21 gio kiem tra cap nhat theo dinh dang 24h cua ngon ngu UI, khong AM/PM ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc });
    await rlt.nav("nav.settings").click();
    await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.permissions") }).click();
    await page.getByTestId("update-check").click();
    const txt = page.getByText(new RegExp("^" + rlt.t("set.upd.lastcheck", { time: "" }).trim()));
    await expect(txt).toBeVisible();
    const t = (await txt.innerText()).split(": ").pop()!;
    expect(t).toMatch(/^\d{2}:\d{2}$/);
  });
}
