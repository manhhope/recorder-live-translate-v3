// M3: cac bo chon ngon ngu khong cat chu o en va pt-PT, o be rong cua so toi thieu (720).
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";

const LOCALES = ["en", "pt-PT"] as const;
const MIN_W = 720;

/** Nut LangSelect: phan chu .v phai hien du (scrollWidth <= clientWidth) va nut khong tran ra ngoai thanh cau hinh. */
async function langButtons(page: Page) {
  return page.evaluate(() => Array.from(document.querySelectorAll<HTMLElement>(".cfg .sel")).map(b => {
    const v = b.querySelector<HTMLElement>(".v")!;
    const chev = b.querySelector<HTMLElement>(":scope > .i")!;
    const cfg = b.closest(".cfg")!.getBoundingClientRect();
    const br = b.getBoundingClientRect();
    return {
      text: v.textContent!.trim(), vScroll: v.scrollWidth, vClient: v.clientWidth,
      overlap: chev.getBoundingClientRect().left < v.getBoundingClientRect().left + v.scrollWidth - 0.5,
      inside: br.right <= cfg.right + 0.5,
    };
  }));
}

/** Select goc: moi option do bang canvas phai vua trong o chon (tru phan padding + mui ten). */
async function nativeSelects(page: Page) {
  return page.evaluate(() => {
    const ctx = document.createElement("canvas").getContext("2d")!;
    return Array.from(document.querySelectorAll<HTMLSelectElement>("select")).filter(s => s.clientWidth > 0).map(s => {
      const cs = getComputedStyle(s);
      ctx.font = `${cs.fontWeight} ${cs.fontSize} ${cs.fontFamily}`;
      const pad = parseFloat(cs.paddingLeft) + parseFloat(cs.paddingRight) + 20;
      const longest = Math.max(...Array.from(s.options).map(o => ctx.measureText(o.text).width));
      return { label: s.getAttribute("aria-label") ?? s.parentElement?.textContent?.trim().slice(0, 20) ?? "", need: Math.ceil(longest + pad), have: s.clientWidth };
    });
  });
}

for (const loc of LOCALES) {
  test(`Ghi: bo chon ngon ngu nguon va dich khong cat chu ${loc} @${MIN_W}`, async ({ page, rlt }) => {
    await page.setViewportSize({ width: MIN_W, height: 700 });
    await rlt.open({ locale: loc });
    const check = async (tag: string) => {
      const r = await langButtons(page);
      expect(r.length, tag).toBe(2);
      for (const b of r) {
        expect(b.vScroll, `${tag}: "${b.text}" bi cat (${b.vScroll}>${b.vClient})`).toBeLessThanOrEqual(b.vClient);
        expect(b.overlap, `${tag}: mui ten de len chu "${b.text}"`).toBe(false);
        expect(b.inside, `${tag}: "${b.text}" tran khoi thanh cau hinh`).toBe(true);
      }
    };
    await check("mac dinh (Auto-detect)");
    // Chon lan luot chuoi dai nhat cho ca hai o.
    const src = page.locator(".cfg .sel").nth(0), dst = page.locator(".cfg .sel").nth(1);
    for (const key of ["lang.pt", "lang.zh", "lang.vi"]) {
      for (const btn of [src, dst]) {
        await btn.click();
        await page.locator(".menu .mi", { hasText: rlt.t(key) }).first().click();
        await check(`${key}`);
      }
    }
    await expect(page.locator("html")).not.toHaveJSProperty("scrollWidth", MIN_W + 1);
  });

  test(`Select goc (Tu dien/Quick/Phu de) khong cat chu ${loc}`, async ({ page, rlt }) => {
    await page.setViewportSize({ width: MIN_W, height: 700 });
    await rlt.open({ locale: loc });
    await rlt.nav("nav.settings").click();
    await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.glossary") }).click();
    for (const s of await nativeSelects(page)) expect(s.have, `Cai dat ${s.label}`).toBeGreaterThanOrEqual(s.need);

    for (const path of ["/quick.html", "/subtitle.html"]) {
      await page.goto(path);
      await page.waitForTimeout(400);
      // Hai cua so rieng dang ky su kien Tauri ma lop gia chua co; khong lien quan den phep do nay.
      await page.evaluate(() => { (window as any).__rlt.unhandled.length = 0; });
      for (const s of await nativeSelects(page)) expect(s.have, `${path} ${s.label}`).toBeGreaterThanOrEqual(s.need);
    }
    await page.evaluate(() => { (window as any).__rlt.unhandled.length = 0; });
    rlt.pageErrors.length = 0;
  });
}
