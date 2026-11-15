import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";
import { mkdirSync } from "node:fs";

const SHOT = process.env.RLT_SHOT_DIR;
const shot = async (page: Page, name: string) => { if (SHOT) { mkdirSync(SHOT, { recursive: true }); await page.screenshot({ path: `${SHOT}/${name}.png` }); } };

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
      test(`xuat ${fmt.toUpperCase()} / ${content}: goi lenh dung dinh dang, noi dung, ngon ngu, thu muc khoi dau mac dinh (chua nho)`, async ({ page, rlt }) => {
        await openWeekly(page, rlt);
        await page.getByRole("radio", { name: new RegExp("^" + fmt.toUpperCase() + "$") }).click();
        await page.getByRole("radio", { name: rlt.t("exp.content." + content) }).click();
        await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
        await expect(page.getByText(rlt.t("exp.saved", { path: `/Users/test/Documents/s-weekly.${fmt}` }))).toBeVisible();
        const [call] = await rlt.calls("session_export");
        expect(call.args).toEqual({ sessionId: "s-weekly", format: fmt, content, uiLang: "en", dir: null });
      });
    }
  }

  test("mac dinh Md + Ca nguon va dich; nho lua chon va thu muc luu lan truoc", async ({ page, rlt }) => {
    await openWeekly(page, rlt);
    await expect(page.getByRole("radio", { name: /^MD$/ })).toHaveAttribute("aria-checked", "true");
    await expect(page.getByRole("radio", { name: rlt.t("exp.content.both") })).toHaveAttribute("aria-checked", "true");
    await page.getByRole("radio", { name: /^DOCX$/ }).click();
    await page.getByRole("radio", { name: rlt.t("exp.content.translation") }).click();
    await page.evaluate(() => { (window as any).__rlt.state.savePick = "/Users/test/Exports"; });
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    await expect(page.getByText(rlt.t("exp.saved", { path: "/Users/test/Exports/s-weekly.docx" }))).toBeVisible();
    expect(await page.evaluate(() => JSON.parse(localStorage.getItem("rlt.export.prefs") ?? "{}").dir)).toBe("/Users/test/Exports");
    // Mo lai dialog: giu lua chon lan truoc, hop thoai luu mo o thu muc da nho.
    await page.getByRole("button", { name: rlt.t("exp.open") }).click();
    await expect(page.getByRole("radio", { name: /^DOCX$/ })).toHaveAttribute("aria-checked", "true");
    await expect(page.getByRole("radio", { name: rlt.t("exp.content.translation") })).toHaveAttribute("aria-checked", "true");
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    const calls = await rlt.calls("session_export");
    expect(calls[1].args.dir).toBe("/Users/test/Exports");
    expect(await page.evaluate(() => (window as any).__rlt.state.lastSuggest)).toEqual({ name: "s-weekly.docx", dir: "/Users/test/Exports" });
  });

  test("dialog gon: khong con dong Save to / nut Change", async ({ page, rlt }) => {
    await openWeekly(page, rlt);
    const d = page.getByRole("dialog");
    await expect(d.locator("code")).toHaveCount(0);
    await expect(d.getByRole("button", { name: /^(Change|Alterar)$/ })).toHaveCount(0);
    await expect(d).not.toContainText(/Save to|Guardar em/);
  });

  // Cua so mac dinh cua app la 1000x700 (tauri.conf.json): mot hang, ca en va pt-PT.
  for (const loc of ["en", "pt-PT"] as const) {
    test(`Format, Content va Export/Cancel nam tren mot hang o cua so chuan 1000x700 ${loc}`, async ({ page, rlt }) => {
      await page.setViewportSize({ width: 1000, height: 700 });
      await openWeekly(page, rlt, { locale: loc });
      await page.getByRole("radio", { name: /^DOCX$/ }).click(); // tooltip mo ta khong them dong
      const d = page.getByRole("dialog", { name: rlt.t("exp.title") });
      const ys = async (l: import("@playwright/test").Locator) => { const b = (await l.boundingBox())!; return b.y + b.height / 2; };
      const y0 = await ys(d.getByRole("radiogroup", { name: rlt.t("exp.format") }));
      const y1 = await ys(d.getByRole("radiogroup", { name: rlt.t("exp.content") }));
      const y2 = await ys(rlt.btn("exp.go", d));
      const y3 = await ys(rlt.btn("exp.cancel", d));
      for (const y of [y1, y2, y3]) expect(Math.abs(y - y0)).toBeLessThan(6);
      const box = (await d.boundingBox())!;
      expect(box.x + box.width).toBeLessThanOrEqual(1000);
      await shot(page, `export-chuan-${loc}`);
    });
  }

  test("cua so nho toi thieu 720x520: cho phep gap dong, khong tran ngang, nut van bam duoc", async ({ page, rlt }) => {
    await page.setViewportSize({ width: 720, height: 520 });
    await openWeekly(page, rlt);
    const d = page.getByRole("dialog", { name: rlt.t("exp.title") });
    const box = (await d.boundingBox())!;
    expect(box.x).toBeGreaterThanOrEqual(0);
    expect(box.x + box.width).toBeLessThanOrEqual(720);
    await expect(rlt.btn("exp.go", d)).toBeVisible();
    await expect(rlt.btn("exp.cancel", d)).toBeVisible();
    await shot(page, "export-nho-en");
  });

  test("Export mo hop thoai luu, chon xong moi ghi: hien Saved to duong dan da chon", async ({ page, rlt }) => {
    await openWeekly(page, rlt);
    await page.evaluate(() => { (window as any).__rlt.state.savePick = "/Users/test/Desktop"; });
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    await expect(page.getByText(rlt.t("exp.saved", { path: "/Users/test/Desktop/s-weekly.md" }))).toBeVisible();
    await expect(page.getByRole("dialog", { name: rlt.t("exp.title") })).toHaveCount(0);
    expect((await rlt.calls("session_export")).length).toBe(1);
  });

  test("huy hop thoai luu: khong ghi, khong bao loi, khong luu thu muc, dialog van mo", async ({ page, rlt }) => {
    await openWeekly(page, rlt);
    await page.evaluate(() => { (window as any).__rlt.state.savePick = null; });
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    await expect.poll(async () => (await rlt.calls("session_export")).length).toBe(1);
    await expect(page.getByRole("alert")).toHaveCount(0);
    await expect(page.getByText(/Saved to|Guardado em/)).toHaveCount(0);
    await expect(page.getByRole("dialog", { name: rlt.t("exp.title") })).toBeVisible();
    await expect(rlt.btn("exp.go", page.getByRole("dialog"))).toBeEnabled();
    expect(await page.evaluate(() => JSON.parse(localStorage.getItem("rlt.export.prefs") ?? "{}").dir ?? null)).toBeNull();
  });

  test("hop thoai luu bao loi (vd ACL tu choi): hien loi, khong ghi nho thu muc", async ({ page, rlt }) => {
    await openWeekly(page, rlt, { fail: { session_export: "export_failed" } });
    await page.getByRole("dialog").getByRole("button", { name: rlt.t("exp.go") }).click();
    await expect(page.getByRole("alert")).toBeVisible();
    expect(await page.evaluate(() => JSON.parse(localStorage.getItem("rlt.export.prefs") ?? "{}").dir ?? null)).toBeNull();
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
