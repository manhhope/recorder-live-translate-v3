// What's New: hien mot lan sau khi doi phien ban va o lan mo dau tien, dong bang Got it/X/Esc, mo lai tu Cai dat > About, en va pt-PT, cua so nho.
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";
import { readFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";

const VERSION: string = JSON.parse(readFileSync(resolve(import.meta.dirname, "../../src-tauri/tauri.conf.json"), "utf8")).version;
const SHOT = process.env.RLT_SHOT_DIR;
const KEY = "rlt.ui.whatsNewSeen";
const seen = (page: Page) => page.evaluate(k => localStorage.getItem(k), KEY);
const dlg = (page: Page) => page.getByRole("dialog");
const shot = async (page: Page, name: string) => { if (SHOT) { mkdirSync(SHOT, { recursive: true }); await page.screenshot({ path: `${SHOT}/${name}.png` }); } };

for (const loc of ["en", "pt-PT"] as const) {
  test(`doi phien ban: hien mot lan, Got it dong va ghi nho, mo lai khong hien ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc, whatsNewSeen: "0.0.1" });
    const d = dlg(page);
    await expect(d).toBeVisible();
    await expect(d).toContainText(rlt.t("wn.title"));
    await expect(d).toContainText("v" + VERSION);
    for (const g of ["new", "improved", "fixed"]) await expect(d.getByRole("heading", { name: rlt.t("wn.group." + g), exact: true })).toBeVisible();
    await expect(d.locator("li").first()).toBeVisible();
    // Khong co dong xin danh gia, khong co hang nen tang.
    await expect(d).not.toContainText(/rate the app|avaliar|platforms|plataformas/i);
    await expect(rlt.btn("wn.gotit", d)).toBeFocused();
    await shot(page, `whats-new-${loc}`);
    await rlt.btn("wn.gotit", d).click();
    await expect(d).toHaveCount(0);
    expect(await seen(page)).toBe(VERSION);
    await page.reload();
    await expect(rlt.nav("nav.record")).toBeVisible();
    await expect(dlg(page)).toHaveCount(0);
  });
}

test("nut X dong va ghi phien ban da xem", async ({ page, rlt }) => {
  await rlt.open({ whatsNewSeen: "0.0.1" });
  await dlg(page).getByRole("button", { name: rlt.t("wn.close") }).click();
  await expect(dlg(page)).toHaveCount(0);
  expect(await seen(page)).toBe(VERSION);
});

test("Esc dong va ghi phien ban da xem", async ({ page, rlt }) => {
  await rlt.open({ whatsNewSeen: "0.0.1" });
  await expect(dlg(page)).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(dlg(page)).toHaveCount(0);
  expect(await seen(page)).toBe(VERSION);
});

test("bam ra ngoai khong dong; Tab xoay vong trong hop thoai", async ({ page, rlt }) => {
  await rlt.open({ whatsNewSeen: "0.0.1" });
  await page.mouse.click(5, 5);
  await expect(dlg(page)).toBeVisible();
  await page.keyboard.press("Tab");
  await expect(dlg(page).getByRole("button", { name: rlt.t("wn.close") })).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(rlt.btn("wn.gotit", dlg(page))).toBeFocused();
});

for (const loc of ["en", "pt-PT"] as const) {
  test(`lan mo dau tien (chua co phien ban da xem): hien What's New cua ban hien tai, Got it ghi nho, mo lai khong hien ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc, whatsNewSeen: null });
    expect(await seen(page)).toBeNull();
    const d = dlg(page);
    await expect(d).toBeVisible();
    await expect(d).toContainText("v" + VERSION);
    await expect(d.locator("li").first()).toBeVisible();
    await rlt.btn("wn.gotit", d).click();
    await expect(d).toHaveCount(0);
    expect(await seen(page)).toBe(VERSION);
    await page.reload();
    await expect(rlt.nav("nav.record")).toBeVisible();
    await page.waitForTimeout(1200);
    await expect(dlg(page)).toHaveCount(0);
  });
}

test("lan mo dau tien: dong bang Esc cung ghi nho; dong y/xin quyen dang mo thi doi xong moi hien", async ({ page, rlt }) => {
  await rlt.open({ whatsNewSeen: "0.0.1" });
  await page.keyboard.press("Escape");
  await expect(dlg(page)).toHaveCount(0);
  await page.evaluate(() => localStorage.removeItem("rlt.ui.whatsNewSeen"));
  // Gia lap hop thoai khac dang mo luc khoi dong: What's New phai doi.
  await page.addInitScript(() => {
    const d = document.createElement("div"); d.id = "other"; d.setAttribute("role", "dialog"); d.setAttribute("aria-modal", "true");
    document.addEventListener("DOMContentLoaded", () => document.body.appendChild(d));
    (window as any).__closeOther = () => d.remove();
  });
  await page.reload();
  await expect(rlt.nav("nav.record")).toBeVisible();
  await page.waitForTimeout(900);
  await expect(page.locator(".wn")).toHaveCount(0);
  await page.evaluate(() => (window as any).__closeOther());
  await expect(page.locator(".wn")).toBeVisible();
});

test("cung phien ban da xem: khong hien", async ({ page, rlt }) => {
  await rlt.open({ whatsNewSeen: VERSION });
  await expect(dlg(page)).toHaveCount(0);
});

for (const loc of ["en", "pt-PT"] as const) {
  test(`mo lai thu cong tu Cai dat > About ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc });
    await rlt.nav("nav.settings").click();
    await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.permissions") }).click();
    await expect(page.getByText(rlt.t("set.about.whatsnew"), { exact: true })).toBeVisible();
    await page.getByTestId("whatsnew-open").click();
    await expect(dlg(page)).toContainText("v" + VERSION);
    await expect(dlg(page).locator("li").first()).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(dlg(page)).toHaveCount(0);
  });
}

for (const loc of ["en", "pt-PT"] as const) {
  test(`cua so nho 720x520: noi dung cuon, Got it va X khong bi cat ${loc}`, async ({ page, rlt }) => {
    await page.setViewportSize({ width: 720, height: 520 });
    await rlt.open({ locale: loc, whatsNewSeen: "0.0.1" });
    const d = dlg(page);
    await expect(d).toBeVisible();
    const ok = rlt.btn("wn.gotit", d), x = d.getByRole("button", { name: rlt.t("wn.close") });
    for (const b of [ok, x]) {
      const r = (await b.boundingBox())!;
      expect(r.y).toBeGreaterThanOrEqual(0);
      expect(r.y + r.height).toBeLessThanOrEqual(520);
      expect(r.x + r.width).toBeLessThanOrEqual(720);
    }
    const body = d.locator(".wn-b");
    const m = await body.evaluate(e => ({ sh: e.scrollHeight, ch: e.clientHeight, ov: getComputedStyle(e).overflowY }));
    expect(m.ov).toBe("auto");
    expect(m.sh).toBeGreaterThan(m.ch); // noi dung dai hon khung: phai cuon
    await body.evaluate(e => { e.scrollTop = e.scrollHeight; });
    expect(await body.evaluate(e => e.scrollTop)).toBeGreaterThan(0);
    await expect(ok).toBeVisible();
    await shot(page, `whats-new-small-${loc}`);
    const box = (await d.boundingBox())!;
    expect(box.y + box.height).toBeLessThanOrEqual(520);
  });
}
