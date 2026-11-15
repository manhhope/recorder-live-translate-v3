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

  const openWeekly = async (page: any, rlt: any, over: any = {}) => {
    await rlt.open(over);
    await rlt.nav("nav.sessions").click();
    await page.getByRole("button", { name: "Weekly product meeting" }).click();
    await page.getByRole("button", { name: rlt.t("exp.open") }).click();
    await expect(page.getByRole("dialog", { name: rlt.t("exp.title") })).toBeVisible();
  };

  for (const fmt of ["md", "txt", "srt", "docx"] as const) {
    for (const content of ["both", "translation", "source"] as const) {
      test(`xuat ${fmt.toUpperCase()} / ${content}: goi lenh dung dinh dang, noi dung, ngon ngu, thu muc Documents`, async ({ page, rlt }) => {
        await openWeekly(page, rlt);
        await page.getByRole("radio", { name: new RegExp("^" + fmt.toUpperCase() + "$") }).click();
        await page.getByRole("radio", { name: rlt.t("exp.content." + content) }).click();
        await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
        await expect(page.getByText(rlt.t("exp.saved", { path: `/Users/test/Documents/s-weekly.${fmt}` }))).toBeVisible();
        const [call] = await rlt.calls("session_export");
        expect(call.args).toEqual({ sessionId: "s-weekly", format: fmt, content, uiLang: "en", dir: "/Users/test/Documents" });
      });
    }
  }

  test("mac dinh Md + Ca nguon va dich; nho lua chon lan truoc (dinh dang, noi dung, thu muc)", async ({ page, rlt }) => {
    await openWeekly(page, rlt);
    await expect(page.getByRole("radio", { name: /^MD$/ })).toHaveAttribute("aria-checked", "true");
    await expect(page.getByRole("radio", { name: rlt.t("exp.content.both") })).toHaveAttribute("aria-checked", "true");
    await page.getByRole("radio", { name: /^DOCX$/ }).click();
    await page.getByRole("radio", { name: rlt.t("exp.content.translation") }).click();
    await page.getByRole("button", { name: rlt.t("exp.dir.change") }).click();
    await expect(page.getByRole("dialog").locator("code")).toHaveText("/Users/test/Exports");
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    await expect(page.getByText(rlt.t("exp.saved", { path: "/Users/test/Exports/s-weekly.docx" }))).toBeVisible();
    // Mo lai dialog: giu lua chon lan truoc.
    await page.getByRole("button", { name: rlt.t("exp.open") }).click();
    await expect(page.getByRole("radio", { name: /^DOCX$/ })).toHaveAttribute("aria-checked", "true");
    await expect(page.getByRole("radio", { name: rlt.t("exp.content.translation") })).toHaveAttribute("aria-checked", "true");
    await expect(page.getByRole("dialog").locator("code")).toHaveText("/Users/test/Exports");
  });

  test("Change: chon thu muc -> o Save to hien duong dan va luu cho lan sau; chua chon thi hien Documents", async ({ page, rlt }) => {
    await openWeekly(page, rlt);
    const code = page.getByRole("dialog").locator("code");
    await expect(code).toHaveText("/Users/test/Documents");
    await page.getByRole("button", { name: rlt.t("exp.dir.change") }).click();
    await expect(code).toHaveText("/Users/test/Exports");
    expect((await rlt.calls("export_pick_dir")).length).toBe(1);
    expect(await page.evaluate(() => JSON.parse(localStorage.getItem("rlt.export.prefs") ?? "{}").dir)).toBe("/Users/test/Exports");
    await expect(page.getByRole("alert")).toHaveCount(0);
  });

  test("Change: huy hop thoai khong bao loi va giu thu muc cu", async ({ page, rlt }) => {
    await openWeekly(page, rlt);
    await page.evaluate(() => { (window as any).__rlt.state.pickDir = null; });
    await page.getByRole("button", { name: rlt.t("exp.dir.change") }).click();
    await expect.poll(async () => (await rlt.calls("export_pick_dir")).length).toBe(1);
    await expect(page.getByRole("dialog").locator("code")).toHaveText("/Users/test/Documents");
    await expect(page.getByRole("alert")).toHaveCount(0);
  });

  test("Change: lenh loi (vd ACL tu choi) hien loi co chi tiet, khong phai 'Something went wrong', thu muc khong doi", async ({ page, rlt }) => {
    await openWeekly(page, rlt);
    await rlt.setFail("export_pick_dir", "export_pick_dir not allowed by ACL");
    await page.getByRole("button", { name: rlt.t("exp.dir.change") }).click();
    const alert = page.getByRole("alert");
    await expect(alert).toContainText("export_pick_dir not allowed by ACL");
    await expect(alert).not.toContainText(rlt.t("err.generic"));
    await expect(page.getByRole("dialog").locator("code")).toHaveText("/Users/test/Documents");
  });

  test("Change: loi hien theo pt-PT", async ({ page, rlt }) => {
    await openWeekly(page, rlt, { locale: "pt-PT", fail: { export_pick_dir: "export_pick_failed" } });
    await page.getByRole("button", { name: rlt.t("exp.dir.change") }).click();
    await expect(page.getByRole("alert")).toContainText(rlt.t("exp.dir.error", { detail: "export_pick_failed" }));
  });

  test("dialog xuat hien nhan theo pt-PT", async ({ page, rlt }) => {
    await openWeekly(page, rlt, { locale: "pt-PT" });
    await expect(page.getByRole("radio", { name: rlt.t("exp.content.source") })).toBeVisible();
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    const [call] = await rlt.calls("session_export");
    expect(call.args.uiLang).toBe("pt-PT");
  });

  test("xuat docx thieu anh: hien canh bao export_image_missing voi so anh thieu (en)", async ({ page, rlt }) => {
    await openWeekly(page, rlt, { exportMissing: 1 });
    await page.getByRole("radio", { name: /^DOCX$/ }).click();
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    await expect(page.getByText(rlt.t("exp.saved", { path: "/Users/test/Documents/s-weekly.docx" }))).toBeVisible();
    await expect(page.getByText(rlt.t("err.exportimagemissing", { n: 1 }))).toBeVisible();
  });

  test("xuat docx thieu anh: canh bao theo pt-PT", async ({ page, rlt }) => {
    await openWeekly(page, rlt, { exportMissing: 2, locale: "pt-PT" });
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    await expect(page.getByText(rlt.t("err.exportimagemissing", { n: 2 }))).toBeVisible();
  });

  test("xuat that bai (loi luu tru) hien thong bao loi", async ({ page, rlt }) => {
    await openWeekly(page, rlt, { fail: { session_export: "store_error" } });
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    await expect(page.getByRole("alert")).toContainText(rlt.t("err.storeerror"));
  });

  test("thu muc xuat khong ton tai hien loi rieng", async ({ page, rlt }) => {
    await openWeekly(page, rlt, { fail: { session_export: "export_dir_missing" } });
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    await expect(page.getByRole("alert")).toContainText(rlt.t("err.exportdirmissing"));
  });

  test("dinh kem / go dinh kem anh chup goi lenh dung va doi nhan nut", async ({ page, rlt }) => {
    const cap = { screenCaptureId: 7, offsetMs: 5000, absPath: "/x.png", widthPx: 64, heightPx: 60, byteSize: 10, isAttached: false, thumbDataUrl: "" };
    await rlt.open({ captures: { "s-weekly": [cap] } });
    await rlt.nav("nav.sessions").click();
    await page.getByRole("button", { name: "Weekly product meeting" }).click();
    await expect(page.getByText(rlt.t("ses.attach.none"))).toBeVisible();
    await page.getByRole("button", { name: rlt.t("ses.attach") }).click();
    await expect(page.getByRole("button", { name: rlt.t("ses.detach") })).toHaveAttribute("aria-pressed", "true");
    const [call] = await rlt.calls("session_capture_attach");
    expect(call.args).toEqual({ sessionId: "s-weekly", screenCaptureId: 7, attached: true });
    await expect(page.getByText(rlt.t("ses.attach.none"))).toHaveCount(0);
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
