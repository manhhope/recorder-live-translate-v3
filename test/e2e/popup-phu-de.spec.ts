// M3: popup phu de hien chu (replay + realtime) va thanh nut chi hien khi re chuot. Chay tren chromium voi lop Tauri gia;
// NSPanel that khong do duoc o day (xem docs/m3-popup-phu-de-bao-cao.md).
import type { Page } from "@playwright/test";
import { test, expect, type Rlt } from "./fixtures.ts";

const dto = (id: number, src: string, dst: string | null) => ({ captionLineId: id, seq: id, offsetMs: id * 1000, sourceLang: "en", sourceText: src, translatedText: dst });
const emitLine = (page: Page, l: ReturnType<typeof dto>) => page.evaluate(x => (window as any).__rlt.emitEvent("subtitle://line", x), l);
const emitHover = (page: Page, v: boolean) => page.evaluate(x => (window as any).__rlt.emitEvent("subtitle://hover", x), v);
const opacity = (page: Page) => page.locator(".tools").evaluate(e => Number(getComputedStyle(e).opacity));
const openPopup = async (page: Page, rlt: Rlt, loc: "en" | "pt-PT", subtitleLines: unknown[] = []) => {
  await page.setViewportSize({ width: 640, height: 160 });
  await rlt.open({ locale: loc, subtitleLines });
  await page.goto("/subtitle.html"); // lop Tauri gia (init script) van con sau khi doi trang
  await expect(page.locator(".tools")).toBeAttached();
};

for (const loc of ["en", "pt-PT"] as const) {
  test.describe(`Popup phu de ${loc}`, () => {
    test(`mo giua chung: hien cac dong da co (replay) roi nhan dong moi realtime, chu chot thay chu tam ${loc}`, async ({ page, rlt }) => {
      await openPopup(page, rlt, loc, [dto(1, "First sentence", "Primeira frase"), dto(2, "Second so far", null)]);
      await expect(page.locator(".text p")).toHaveText(["First sentence", "Primeira frase", "Second so far"]);
      await expect(page.locator(".wait")).toHaveCount(0);
      await emitLine(page, dto(2, "Second sentence final", "Segunda frase"));
      await emitLine(page, dto(3, "Third", "Terceira"));
      await expect(page.locator(".text p")).toHaveText(["First sentence", "Primeira frase", "Second sentence final", "Segunda frase", "Third", "Terceira"]);
      // Chu phai nhin thay duoc: kich thuoc > 0 va mau chu khac nen.
      const vis = await page.locator(".text p").first().evaluate(e => { const r = e.getBoundingClientRect(); return { w: r.width, h: r.height, color: getComputedStyle(e).color }; });
      expect(vis.w).toBeGreaterThan(0); expect(vis.h).toBeGreaterThan(0);
      expect((await rlt.calls("subtitle_lines")).length).toBe(1);
    });

    test(`popup mo luc chua co dong nao: hien "cho chu", dong dau tien den thi thay cho ${loc}`, async ({ page, rlt }) => {
      await openPopup(page, rlt, loc);
      await expect(page.locator(".wait")).toHaveText(rlt.t("sub.waiting"));
      await emitLine(page, dto(1, "Hello", "Ola"));
      await expect(page.locator(".wait")).toHaveCount(0);
      await expect(page.locator(".text p")).toHaveText(["Hello", "Ola"]);
    });

    test(`thanh nut: an bang opacity, hien khi re chuot, an lai khi roi ra, khong nhay layout ${loc}`, async ({ page, rlt }) => {
      await openPopup(page, rlt, loc);
      const tools = page.locator(".tools");
      const box0 = await tools.boundingBox();
      expect(await opacity(page)).toBe(0);
      expect(await tools.evaluate(e => getComputedStyle(e).display)).not.toBe("none");
      expect(await tools.evaluate(e => getComputedStyle(e).visibility)).toBe("visible");
      expect(await tools.evaluate(e => getComputedStyle(e).transitionProperty)).toContain("opacity");
      expect(await tools.evaluate(e => parseFloat(getComputedStyle(e).transitionDuration))).toBeGreaterThan(0);

      await page.mouse.move(300, 80);
      await expect.poll(() => opacity(page)).toBe(1);
      expect(await tools.boundingBox()).toEqual(box0);

      // Roi khoi cua so (khong con tro trong trang): pointerleave.
      await page.mouse.move(300, 400);
      await expect.poll(() => opacity(page)).toBe(0);
      expect(await tools.boundingBox()).toEqual(box0);
    });

    test(`thanh nut: bam duoc nut va keo duoc cua so khi dang hien ${loc}`, async ({ page, rlt }) => {
      await openPopup(page, rlt, loc);
      await page.mouse.move(300, 80);
      await expect.poll(() => opacity(page)).toBe(1);
      await page.getByRole("button", { name: rlt.t("sub.font.increase") }).click();
      expect((await rlt.calls("subtitle_font_step")).map(c => c.args.delta)).toEqual([1]);
      await page.evaluate(() => { (window as any).__rlt.calls.length = 0; });
      await page.mouse.move(300, 3);
      await page.mouse.down(); await page.mouse.up();
      expect((await rlt.calls("plugin:window|start_dragging")).length).toBeGreaterThanOrEqual(1);
    });

    test(`thanh nut: Rust bao con tro (NSPanel khong nhan hover cua trang) cung hien/an thanh ${loc}`, async ({ page, rlt }) => {
      await openPopup(page, rlt, loc);
      expect(await opacity(page)).toBe(0);
      await emitHover(page, true);
      await expect.poll(() => opacity(page)).toBe(1);
      await emitHover(page, false);
      await expect.poll(() => opacity(page)).toBe(0);
    });

    test(`thanh nut: Tab vao nut lam thanh hien, roi tieu diem ra ngoai thi an ${loc}`, async ({ page, rlt }) => {
      await openPopup(page, rlt, loc);
      expect(await opacity(page)).toBe(0);
      await page.keyboard.press("Tab");
      await expect(page.locator(".tools :focus")).toHaveCount(1);
      await expect.poll(() => opacity(page)).toBe(1);
      await page.locator(".tools :focus").evaluate(e => (e as HTMLElement).blur());
      await expect.poll(() => opacity(page)).toBe(0);
    });

    test(`thanh nut: moi nut co tooltip theo ${loc}`, async ({ page, rlt }) => {
      await openPopup(page, rlt, loc);
      const keys = ["sub.font.decrease", "sub.font.increase", "sub.mode.label", "sub.color.source", "sub.color.translation", "sub.opacity", "sub.more", "sub.close"];
      for (const k of keys) await expect(page.locator(`.tools [title="${rlt.t(k)}"]`)).toHaveCount(1);
    });

    test(`prefers-reduced-motion: thanh doi opacity ngay, khong chuyen ${loc}`, async ({ page, rlt }) => {
      await page.emulateMedia({ reducedMotion: "reduce" });
      await openPopup(page, rlt, loc);
      expect(await page.locator(".tools").evaluate(e => parseFloat(getComputedStyle(e).transitionDuration))).toBe(0);
      await page.mouse.move(300, 80);
      await expect.poll(() => opacity(page)).toBe(1);
    });
  });
}
