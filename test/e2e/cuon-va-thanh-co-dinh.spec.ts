// M3: vung chu tu cuon xuong cuoi + thanh tren co dinh (man Ghi o cua so chinh va cua so phu de). Chay tren chromium voi lop Tauri gia;
// cua so NSPanel that khong do duoc o day (xem docs/m3-cuon-va-thanh-co-dinh-bao-cao.md).
import type { Page } from "@playwright/test";
import { test, expect, type Rlt } from "./fixtures.ts";

const metrics = (page: Page, sel: string) => page.locator(sel).evaluate(e => ({ top: e.scrollTop, h: e.scrollHeight, c: e.clientHeight, gap: e.scrollHeight - e.scrollTop - e.clientHeight }));
const box = async (page: Page, sel: string) => { const b = await page.locator(sel).first().boundingBox(); return b && { x: b.x, y: b.y, w: b.width, h: b.height }; };
const addLines = async (rlt: Rlt, from: number, n: number) => { for (let i = from; i < from + n; i++) await rlt.emit(rlt.line(i, i, `Line number ${i} of the meeting transcript`, `Linha número ${i} da transcrição da reunião`)); };

for (const loc of ["en", "pt-PT"] as const) {
  test.describe(`Cuon va thanh co dinh ${loc}`, () => {
    test(`Ghi: them nhieu dong thi vung chu cuon toi cuoi, header khong di chuyen ${loc}`, async ({ page, rlt }) => {
      await page.setViewportSize({ width: 1100, height: 600 });
      await rlt.open({ locale: loc });
      await rlt.startRecording();
      const before = { tool: await box(page, ".tool"), cfg: await box(page, ".cfg"), foot: await box(page, ".foot") };
      await addLines(rlt, 1, 40);
      await expect(page.locator(".ln:not(.live)")).toHaveCount(40);
      await expect.poll(async () => (await metrics(page, ".trs")).gap).toBeLessThanOrEqual(2);
      const m = await metrics(page, ".trs");
      expect(m.h, "co noi dung de cuon").toBeGreaterThan(m.c + 200);
      // Header, cau hinh, chan trang giu nguyen toa do; cua so chinh khong cuon.
      expect({ tool: await box(page, ".tool"), cfg: await box(page, ".cfg"), foot: await box(page, ".foot") }).toEqual(before);
      const main = await metrics(page, ".main");
      expect(main.top).toBe(0);
      expect(main.h).toBeLessThanOrEqual(main.c);
      await expect(page.locator(".jump")).toHaveCount(0);
    });

    test(`Ghi: cuon len thi khong bi keo xuong, hien nut ${loc}; bam nut hoac cuon toi cuoi thi bam lai ${loc}`, async ({ page, rlt }) => {
      await page.setViewportSize({ width: 1100, height: 600 });
      await rlt.open({ locale: loc });
      await rlt.startRecording();
      await addLines(rlt, 1, 30);
      await expect.poll(async () => (await metrics(page, ".trs")).gap).toBeLessThanOrEqual(2);
      const headerY = (await box(page, ".tool"))!.y;

      await page.locator(".trs").evaluate(e => { e.scrollTop = 0; });
      const jump = page.getByRole("button", { name: rlt.t("rec.jump.latest") });
      await expect(jump).toBeVisible();
      await addLines(rlt, 31, 5);
      await expect(page.locator(".ln:not(.live)")).toHaveCount(35);
      expect((await metrics(page, ".trs")).top, "khong bi keo xuong").toBe(0);
      await expect(jump).toBeVisible();
      expect((await box(page, ".tool"))!.y, "header giu nguyen sau khi cuon").toBe(headerY);

      await jump.click();
      await expect.poll(async () => (await metrics(page, ".trs")).gap).toBeLessThanOrEqual(2);
      await expect(jump).toHaveCount(0);
      await addLines(rlt, 36, 2);
      await expect.poll(async () => (await metrics(page, ".trs")).gap).toBeLessThanOrEqual(2);

      // Cuon len roi tu cuon xuong tay toi cuoi cung bam lai.
      await page.locator(".trs").evaluate(e => { e.scrollTop = 0; });
      await expect(jump).toBeVisible();
      await page.locator(".trs").evaluate(e => { e.scrollTop = e.scrollHeight; });
      await expect(jump).toHaveCount(0);
      await addLines(rlt, 38, 2);
      await expect.poll(async () => (await metrics(page, ".trs")).gap).toBeLessThanOrEqual(2);
    });

    test(`Ghi: prefers-reduced-motion thi nut Jump nhay thang khong cuon muot ${loc}`, async ({ page, rlt }) => {
      await page.emulateMedia({ reducedMotion: "reduce" });
      await page.setViewportSize({ width: 1100, height: 600 });
      await rlt.open({ locale: loc });
      await rlt.startRecording();
      await addLines(rlt, 1, 30);
      await expect.poll(async () => (await metrics(page, ".trs")).gap).toBeLessThanOrEqual(2);
      await page.locator(".trs").evaluate(e => { e.scrollTop = 0; });
      const jump = page.getByRole("button", { name: rlt.t("rec.jump.latest") });
      await expect(jump).toBeVisible();
      // Doc ngay trong cung tick voi cú bấm: cuon muot se chua toi cuoi, nhay thang thi da toi.
      const gap = await page.evaluate(() => {
        const b = [...document.querySelectorAll<HTMLButtonElement>(".jump")][0], t = document.querySelector<HTMLElement>(".trs")!;
        b.click();
        return t.scrollHeight - t.scrollTop - t.clientHeight;
      });
      expect(gap).toBeLessThanOrEqual(2);
    });

    test(`Cau truc layout: chi .trs la phan tu overflow-y auto trong man Ghi, co min-height 0 ${loc}`, async ({ page, rlt }) => {
      await rlt.open({ locale: loc });
      await rlt.startRecording();
      await addLines(rlt, 1, 3);
      const res = await page.evaluate(() => {
        const main = document.querySelector<HTMLElement>(".main")!;
        const scrollers = [...main.querySelectorAll<HTMLElement>("*")].filter(e => ["auto", "scroll"].includes(getComputedStyle(e).overflowY) && !e.closest(".menu")).map(e => e.className);
        const trs = document.querySelector<HTMLElement>(".trs")!;
        const rows = ["tool", "cfg", "foot"].map(c => main.querySelector(`.${c}`)!).map(e => ({ inScroller: trs.contains(e) }));
        return { scrollers, minH: getComputedStyle(trs).minHeight, wrapMinH: getComputedStyle(trs.parentElement!).minHeight, rows };
      });
      expect(res.scrollers).toEqual(["trs"]);
      expect(res.minH).toBe("0px");
      expect(res.wrapMinH).toBe("0px");
      expect(res.rows.every(r => !r.inScroller)).toBe(true);
    });

    test(`Phu de: vung chu cuon, thanh nut co dinh sat mep tren, nut Jump hien khi cuon len ${loc}`, async ({ page, rlt }) => {
      await page.setViewportSize({ width: 640, height: 150 });
      await rlt.open({ locale: loc });
      await page.goto("/subtitle.html"); // lop Tauri gia (init script) van con sau khi doi trang
      await expect(page.locator(".tools")).toBeVisible();
      const long = (n: number) => Array.from({ length: 30 }, (_, i) => `Sentence ${n}.${i} of the subtitle text`).join(" ");
      const emit = (n: number) => page.evaluate(([p]) => (window as any).__rlt.emitEvent("subtitle://line", { source: p, translation: p + " (pt)" }), [long(n)]);
      await emit(1);
      await expect.poll(async () => (await metrics(page, ".text")).gap).toBeLessThanOrEqual(2);
      const m = await metrics(page, ".text");
      expect(m.h).toBeGreaterThan(m.c + 100);

      const tools = await box(page, ".tools");
      expect(tools!.y).toBeLessThan(4);
      expect(tools!.h).toBeLessThanOrEqual(26);
      const structure = await page.evaluate(() => {
        const scrollers = [...document.querySelectorAll<HTMLElement>("*")].filter(e => ["auto", "scroll"].includes(getComputedStyle(e).overflowY) && !e.closest(".menu")).map(e => e.classList[0]);
        const text = document.querySelector<HTMLElement>(".text")!;
        return { scrollers, minH: getComputedStyle(text).minHeight, wrapMinH: getComputedStyle(text.parentElement!).minHeight, toolsInside: text.contains(document.querySelector(".tools")), dragAttr: document.querySelector(".tools")!.hasAttribute("data-drag") };
      });
      expect(structure.scrollers).toEqual(["text"]);
      expect(structure.minH).toBe("0px");
      expect(structure.wrapMinH).toBe("0px");
      expect(structure.toolsInside).toBe(false);
      expect(structure.dragAttr).toBe(true);

      await page.locator(".text").evaluate(e => { e.scrollTop = 0; });
      const jump = page.getByRole("button", { name: rlt.t("rec.jump.latest") });
      await expect(jump).toBeVisible();
      await emit(2);
      await expect.poll(async () => (await metrics(page, ".text")).h).toBeGreaterThan(m.h - 1);
      expect((await metrics(page, ".text")).top).toBe(0);
      expect(await box(page, ".tools")).toEqual(tools);
      await jump.click();
      await expect.poll(async () => (await metrics(page, ".text")).gap).toBeLessThanOrEqual(2);
      await expect(jump).toHaveCount(0);

      // Che do keo cua so: bam vao thanh (khong phai nut) van goi startDragging.
      await page.evaluate(() => { (window as any).__rlt.calls.length = 0; });
      await page.mouse.move(300, 3);
      await page.mouse.down(); await page.mouse.up();
      expect((await rlt.calls("plugin:window|start_dragging")).length).toBeGreaterThanOrEqual(1);
      rlt.pageErrors.length = 0;
    });
  });
}
