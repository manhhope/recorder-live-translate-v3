// Usage Logs gon (t_493189bdb8c9): mot hang dieu khien, mot dong audio + chi phi, khoang ngay ngan, khong tran o cua so nho.
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";
import { mkdirSync } from "node:fs";

const NEW = { costUsdMicros: 250_000, fetchedAt: Math.floor(Date.UTC(2026, 10, 15, 14, 42) / 1000) };
const SHOT = process.env.RLT_SHOT_DIR;

async function openTab(page: Page, rlt: any) {
  await rlt.nav("nav.settings").click();
  await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.sonioxusage") }).click();
}
const top = (page: Page, id: string) => page.getByTestId(id).evaluate(e => Math.round(e.getBoundingClientRect().top));

for (const [name, w, h] of [["chuan", 1100, 760], ["nho", 720, 560]] as const) {
  test(`Usage Logs gon @${name} ${w}px`, async ({ page, rlt }) => {
    await page.setViewportSize({ width: w, height: h });
    await rlt.open({ usage: { cached: NEW } });
    await openTab(page, rlt);
    await expect(page.getByTestId("usage-total")).toContainText(/0[.,]25/);

    // Bo doan giai thich UTC dai.
    await expect(page.getByText(/UTC/)).toHaveCount(0);

    // Mot hang: tuan, nhan nguon, cap nhat luc, Lam moi, Xuat CSV cung mot dong o cua so chuan.
    const refresh = page.getByRole("button", { name: rlt.t("common.refresh") });
    const exp = page.getByRole("button", { name: rlt.t("set.usage.export") });
    await expect(refresh).toHaveAttribute("title", rlt.t("common.refresh"));
    await expect(exp).toHaveAttribute("title", rlt.t("set.usage.export"));
    const week = await page.getByRole("group", { name: rlt.t("set.week.pick") }).evaluate(e => Math.round(e.getBoundingClientRect().top));
    const tops = [await top(page, "usage-source"), await top(page, "usage-updated"), await refresh.evaluate(e => Math.round(e.getBoundingClientRect().top)), await exp.evaluate(e => Math.round(e.getBoundingClientRect().top))];
    if (w >= 1000) for (const t of tops) expect(Math.abs(t - week)).toBeLessThan(12);

    // Dong tong: audio va chi phi cung mot dong; khoang ngay ngan.
    const sum = page.locator(".usum");
    const b = await sum.locator("b").evaluateAll(es => es.map(e => Math.round(e.getBoundingClientRect().top)));
    expect(Math.abs(b[0] - b[1])).toBeLessThan(6);
    await expect(sum).toContainText(rlt.t("set.usage.sub.audio"));
    await expect(sum).toContainText(rlt.t("set.usage.sub.cost"));
    await expect(page.getByTestId("usage-range")).toHaveText(/^\d{1,2} \S+ - \d{1,2} \S+$/);
    await expect(page.locator(".wk em").first()).toHaveText(rlt.t("day.mon"));

    // Khong tran ngang.
    const over = await page.evaluate(() => { const d = document.documentElement; return d.scrollWidth - d.clientWidth; });
    expect(over).toBeLessThanOrEqual(0);
    const bad = await page.evaluate(() => Array.from(document.querySelectorAll(".card *")).filter(e => e.getBoundingClientRect().right > document.documentElement.clientWidth + 0.5).length);
    expect(bad).toBe(0);
    if (SHOT) { mkdirSync(SHOT, { recursive: true }); await page.screenshot({ path: `${SHOT}/usage-gon-${name}.png` }); }
  });
}
