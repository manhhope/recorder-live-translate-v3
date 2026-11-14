import { test, expect } from "./fixtures.ts";

test.describe("Danh sach phien, tim kiem, xuat", () => {
  test("liet ke phien kem huy hieu trang thai va so dong", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.nav("nav.sessions").click();
    const rows = page.locator(".sesrow");
    await expect(rows).toHaveCount(3);
    await expect(rows.nth(0)).toContainText("Weekly product meeting");
    await expect(rows.nth(0)).toContainText(rlt.t("ses.lines", { n: 3 }));
    await expect(rows.nth(0)).toContainText("01:02:05"); // 3.725.000 ms
    await expect(rows.nth(0).locator(".bd.ok")).toContainText(rlt.t("ses.saved"));
    await expect(rows.nth(2).locator(".bd.rec")).toContainText(rlt.t("ses.saveerr"));
    await expect(rows.nth(2)).toContainText("--:--:--"); // chua co thoi luong
  });

  test("danh sach trong: thong bao huong dan", async ({ page, rlt }) => {
    await rlt.open({ sessions: [], details: {} });
    await rlt.nav("nav.sessions").click();
    await expect(page.getByText(rlt.t("ses.empty"))).toBeVisible();
  });

  test("tim kiem: ket qua theo tu khoa, khong co ket qua, xoa o tim thi ve danh sach", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.nav("nav.sessions").click();
    const box = page.getByRole("searchbox", { name: rlt.t("ses.search") });
    await box.fill("pricing");
    const hits = page.locator("button.sesrow");
    await expect(hits).toHaveCount(1);
    await expect(hits.first()).toContainText("A página de preços sai na sexta-feira.");
    await expect.poll(async () => (await rlt.calls("records_search")).at(-1)?.args.query).toBe("pricing");

    await box.fill("zzzz-nothing");
    await expect(page.getByText(rlt.t("ses.noresults"))).toBeVisible();

    await box.fill("");
    await expect(page.locator(".sesrow")).toHaveCount(3);
  });

  test("mo ket qua tim kiem vao dung phien, xem bien ban co dong gap", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.nav("nav.sessions").click();
    await page.getByRole("searchbox", { name: rlt.t("ses.search") }).fill("roadmap");
    await page.locator("button.sesrow").first().click();
    await expect(page.getByRole("button", { name: rlt.t("ses.back") })).toBeVisible();
    await expect(page.getByText("Let us review the roadmap for next quarter.")).toBeVisible();
    await expect(page.getByText(rlt.t("ses.gap"))).toBeVisible();
    await expect(page.getByText("The pricing page ships on Friday.")).toBeVisible();
    // Dong thu hai nghe bang micro.
    await expect(page.locator(".ln .meta b", { hasText: rlt.t("src.name.mic") })).toHaveCount(1);
  });

  for (const fmt of ["md", "txt", "srt"] as const) {
    test(`xuat ${fmt.toUpperCase()}: goi lenh dung dinh dang + ngon ngu va bao da luu`, async ({ page, rlt }) => {
      await rlt.open();
      await rlt.nav("nav.sessions").click();
      await page.getByRole("button", { name: "Weekly product meeting" }).click();
      await page.getByRole("button", { name: new RegExp("^" + fmt.toUpperCase() + "$") }).click();
      await expect(page.getByText(rlt.t("ses.exported"))).toBeVisible();
      const [call] = await rlt.calls("session_export");
      expect(call.args).toEqual({ sessionId: "s-weekly", format: fmt, uiLang: "en" });
    });
  }

  test("xuat that bai (loi luu tru) hien thong bao loi", async ({ page, rlt }) => {
    await rlt.open({ fail: { session_export: "store_error" } });
    await rlt.nav("nav.sessions").click();
    await page.getByRole("button", { name: "Weekly product meeting" }).click();
    await page.getByRole("button", { name: /^MD$/ }).click();
    await expect(page.getByRole("alert")).toContainText(rlt.t("err.storeerror"));
  });

  test("danh dau dong, dong bo Drive chua ket noi", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.nav("nav.sessions").click();
    await page.getByRole("button", { name: "Weekly product meeting" }).click();
    const star = page.getByRole("button", { name: rlt.t("ses.highlight") }).first();
    await expect(star).toHaveAttribute("aria-pressed", "false");
    await star.click();
    await expect(star).toHaveAttribute("aria-pressed", "true");
    await expect(page.getByText(rlt.t("ses.drive.nooauth"))).toBeVisible();
    await page.getByRole("button", { name: rlt.t("ses.drive.sync") }).click();
    await expect(page.getByText(rlt.t("ses.drive.notconnected"))).toBeVisible();
  });

  test("phien khong co dong: thong bao trong", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.nav("nav.sessions").click();
    await page.getByRole("button", { name: "Retro draft" }).click();
    await expect(page.getByText(rlt.t("ses.nolines"))).toBeVisible();
  });

  test("doi ten roi xoa phien (co xac nhan)", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.nav("nav.sessions").click();
    const rows = page.locator(".sesrow");
    await rows.nth(1).getByRole("button", { name: rlt.t("ses.rename") }).click();
    const input = rows.nth(1).getByRole("textbox", { name: rlt.t("ses.rename") });
    await input.fill("Renamed call");
    await input.press("Enter");
    await expect(rows.nth(1)).toContainText("Renamed call");
    const [rn] = await rlt.calls("session_rename");
    expect(rn.args).toEqual({ sessionId: "s-customer", title: "Renamed call" });

    await rows.nth(1).getByRole("button", { name: rlt.t("common.delete") }).click();
    await rows.nth(1).getByRole("button", { name: rlt.t("ses.delete.confirm") }).click();
    await expect(rows).toHaveCount(2);
    await expect(page.getByText("Renamed call")).toHaveCount(0);
  });

  test("nhap tu ung dung cu hien so dong da nhap", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.nav("nav.sessions").click();
    await page.getByRole("button", { name: rlt.t("ses.import") }).click();
    await expect(page.getByText(rlt.t("ses.import.done", { n: 12 }))).toBeVisible();
  });
});
