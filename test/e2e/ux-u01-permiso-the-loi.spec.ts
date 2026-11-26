// U-01: loi quyen micro/am thanh he thong co nut "Mo Cai dat He thong" ngay tai the loi o man Ghi, kem mot cau noi can lam gi.
import { test, expect } from "./fixtures.ts";

const panes = (page: import("@playwright/test").Page) =>
  page.evaluate(() => (window as any).__rlt.calls.filter((c: any) => c.cmd === "open_system_settings").map((c: any) => c.args.pane));

for (const loc of ["en", "pt-PT"] as const) {
  test(`U-01 the loi thieu quyen man hinh co nut mo Cai dat va cau huong dan ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc, permWelcomeOff: true, fail: { session_start: "permission_screen" } });
    await rlt.btn("tool.start").click();
    const card = page.getByRole("alert");
    await expect(card).toContainText(rlt.t("err.permissionscreen"));
    await expect(card.getByTestId("perm-todo")).toHaveText(rlt.t("rec.perm.todo"));
    await card.getByTestId("perm-open-settings").click();
    expect(await panes(page)).toEqual(["screen_recording"]);
    await expect(card.getByTestId("perm-review")).toBeVisible();
  });

  test(`U-01 the loi thieu micro mo ngan Micro ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc, permWelcomeOff: true, fail: { session_start: "permission_mic" } });
    await rlt.btn("tool.start").click();
    await page.getByTestId("perm-open-settings").click();
    expect(await panes(page)).toEqual(["microphone"]);
  });
}

test("U-01 loi khong phai quyen thi khong hien nut mo Cai dat", async ({ page, rlt }) => {
  await rlt.open({ fail: { session_start: "network" } });
  await rlt.btn("tool.start").click();
  await expect(page.getByRole("alert")).toBeVisible();
  await expect(page.getByTestId("perm-open-settings")).toHaveCount(0);
});
