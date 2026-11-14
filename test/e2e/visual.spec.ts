// Chup 5 man (Ghi, Phien, Cai dat tab AI, Usage) o cua so 820 x 640 (do rong toi thieu theo thiet ke, M-01) bang pt-PT,
// ngon ngu mac dinh cua san pham. Anh luu vao test/e2e/screenshots/ de so voi design-system-v3/anh/ban-cuoi/ (phuong an A).
// Day la anh de nguoi doi chieu (khong phai baseline pixel): test chi kiem anh duoc ghi ra va khong co cuon ngang.
import { test, expect } from "./fixtures.ts";
import { statSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "@playwright/test";

const DIR = resolve(import.meta.dirname, "screenshots");

async function shot(page: Page, name: string) {
  await page.evaluate(() => document.fonts.ready);
  const path = resolve(DIR, `${name}.png`);
  await page.screenshot({ path, animations: "disabled", caret: "hide" });
  expect(statSync(path).size).toBeGreaterThan(5_000);
  // Khong cuon ngang o chieu rong toi thieu 820.
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  expect(overflow, "noi dung tran ngang").toBeLessThanOrEqual(0);
}

test.describe("Chup man hinh de doi chieu thiet ke (pt-PT, 820 x 640)", () => {
  test.use({ viewport: { width: 820, height: 640 } });

  test("Ghi: cho va dang ghi", async ({ page, rlt }) => {
    await rlt.open({ locale: "pt-PT" });
    await shot(page, "ghi-cho");
    await rlt.startRecording();
    await rlt.emit(rlt.line(1, 1, "Let us review the roadmap for next quarter.", "Vamos rever o roteiro do próximo trimestre."));
    await rlt.emit(rlt.line(2, 2, "The pricing page ships on Friday.", null));
    await rlt.emit({ kind: "level", level: 0.4 });
    await expect(page.locator(".ln:not(.live)")).toHaveCount(2);
    await shot(page, "ghi-dang-ghi");
  });

  test("Phien: danh sach va chi tiet", async ({ page, rlt }) => {
    await rlt.open({ locale: "pt-PT" });
    await rlt.nav("nav.sessions").click();
    await expect(page.locator(".sesrow")).toHaveCount(3);
    await shot(page, "phien-danh-sach");
    await page.getByRole("button", { name: "Weekly product meeting" }).click();
    await expect(page.getByText(rlt.t("ses.gap"))).toBeVisible();
    await shot(page, "phien-chi-tiet");
  });

  test("Cai dat: tab AI", async ({ page, rlt }) => {
    await rlt.open({ locale: "pt-PT", aiKeys: { gemini: true, openai: false } });
    await rlt.nav("nav.settings").click();
    await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.translate") }).click();
    await expect(page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true })).toBeVisible();
    await shot(page, "cai-dat-ai");
  });

  test("Cai dat: tab Usage Logs", async ({ page, rlt }) => {
    await rlt.open({ locale: "pt-PT" });
    await rlt.nav("nav.settings").click();
    await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.sonioxusage") }).click();
    await expect(page.locator(".wk > div")).toHaveCount(7);
    await shot(page, "cai-dat-usage");
  });
});
