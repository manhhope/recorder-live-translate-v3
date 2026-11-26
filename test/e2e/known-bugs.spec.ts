// Loi san pham da tim thay (xem docs/t2-playwright-bao-cao.md). Moi test dung test.fail(): test PASS khi loi con ton tai
// va se FAIL ("expected to fail but passed") ngay khi loi duoc sua, luc do go test.fail() di de no thanh test hoi quy.
// Moi test do GIA TRI that roi so sanh (khong cho theo thoi gian) de loi that bai vi dung ly do, khong vi selector sai.
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";

const note = (id: string, observed: unknown) => test.info().annotations.push({ type: id, description: JSON.stringify(observed) });

async function openSettingsTab(page: Page, rlt: any, key: string) {
  await rlt.nav("nav.settings").click();
  await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t(key) }).click();
}

test.describe("Loi da biet", () => {
  test("P-01 (da sua) tuan 0 cua Muc dung gui so tuan, khong gui ngay", async ({ page, rlt }) => {
    await rlt.open();
    await openSettingsTab(page, rlt, "set.tab.sonioxusage");
    await expect.poll(async () => (await rlt.calls("usage_week_source")).length).toBeGreaterThan(0);
    const weeksAgo = (await rlt.calls("usage_week_source"))[0].args.weeksAgo as number;
    note("P-01", { weeksAgo });
    expect(weeksAgo).toBe(0);
  });

  test("P-02 ma loi la khong sinh canh bao i18n 'missing key'", async ({ page, rlt }) => {
    test.fail(true, "P-02: errKeyOf thu khoa bang $_() nen ma la bi bao thieu khoa");
    await rlt.open({ fail: { session_start: "something_new" } });
    await rlt.btn("tool.start").click();
    await expect(page.getByRole("alert")).toContainText(rlt.t("err.generic"));
    note("P-02", rlt.consoleProblems);
    expect(rlt.consoleProblems).toEqual([]);
    rlt.consoleProblems.length = 0; // loi da biet, khong de fixture bao lai
  });

  test("P-03 (da sua) tab Dich va AI chi co mot tieu de", async ({ page, rlt }) => {
    await rlt.open();
    await openSettingsTab(page, rlt, "set.tab.translate");
    await expect(page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true })).toBeVisible();
    const n = await page.getByRole("heading", { level: 3, name: rlt.t("set.tab.translate") }).count();
    note("P-03", { headings: n });
    expect(n).toBe(1);
  });

  for (const [id, key] of [["P-04", "set.tab.translate"]] as const) {
    test(`${id} tab ${key}: the khong bi cat o cua so 820 px`, async ({ page, rlt }) => {
      await page.setViewportSize({ width: 820, height: 640 });
      await rlt.open();
      await openSettingsTab(page, rlt, key);
      await expect(page.locator(".sbody .card").first()).toBeVisible();
      const worst = await page.evaluate(() => {
        let right = 0;
        for (const el of document.querySelectorAll<HTMLElement>(".sbody .card, .sbody .seg, .sbody .rr")) right = Math.max(right, Math.round(el.getBoundingClientRect().right));
        return { right, viewport: window.innerWidth };
      });
      note(id, worst);
      expect(worst.right).toBeLessThanOrEqual(worst.viewport);
    });
  }

  test("P-05 so it: '1 lines' phai la '1 line'", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.nav("nav.sessions").click();
    const text = await page.locator(".sesrow").nth(1).innerText(); // phien co dung 1 dong
    note("P-05", text);
    expect(text).not.toMatch(/\b1 lines\b/);
  });

  test("P-06 o 820 px khi dang ghi, o ngon ngu dich khong bi nut nguon am thanh de len", async ({ page, rlt }) => {
    // Loi chi tai hien voi font macOS; tren may Windows (CI windows-latest) font hep hon nen bo cuc vua, test.fail se bao "pass bat ngo".
    // Vi vay chi dat test.fail tren nen tang khac win32; tren Windows test chay nhu test thuong (phai pass).
    test.fail(process.platform !== "win32", "P-06: thanh cau hinh qua rong o pt-PT, nut nguon de len o ngon ngu dich (chi tai hien tren macOS)");
    await page.setViewportSize({ width: 820, height: 640 });
    await rlt.open({ locale: "pt-PT" });
    await rlt.startRecording();
    const boxes = await page.evaluate(() => {
      const r = (s: string) => { const b = document.querySelector(s)!.getBoundingClientRect(); return { left: b.left, right: b.right }; };
      return { dst: r(".sw2"), src: r(".sr .srcb") };
    });
    note("P-06", boxes);
    expect(boxes.dst.right).toBeLessThanOrEqual(boxes.src.left);
  });
});
