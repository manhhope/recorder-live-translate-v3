// Nut sao chep dang icon o goc duoi phai moi bong bong phan hoi AI (thay nut chu "Copy"), man Hoi AI va panel Hoi AI o Sessions.
// SHOT=truoc|sau: chup anh vao docs/ask-ai-copy-img/ de dinh vao bao cao (khong dung vao test/e2e/screenshots).
import { test, expect, type Rlt } from "./fixtures.ts";
import type { Page } from "@playwright/test";

const withAi = { aiKeys: { gemini: true, openai: false } };
const SHOT = process.env.SHOT;

async function twoReplies(rlt: Rlt, chat: ReturnType<Page["locator"]>) {
  await rlt.btn("ask.consent.ok").click();
  await rlt.btn("ask.chip.summary", chat).click();
  await expect(chat.locator(".msg.assistant")).toHaveCount(1);
  await rlt.btn("ask.chip.explain", chat).click();
  await expect(chat.locator(".msg.assistant")).toHaveCount(2);
}

for (const locale of ["en", "pt-PT"] as const) {
  test.describe(`Nut sao chep bong bong AI (${locale})`, () => {
    test.beforeEach(async ({ context }) => {
      await context.grantPermissions(["clipboard-read", "clipboard-write"]);
    });

    test("man Hoi AI: icon o goc duoi phai, khong con nut chu, bam chep dung bong bong, doi dau tich roi tro lai", async ({ page, rlt }) => {
      await rlt.open({ ...withAi, locale });
      await rlt.nav("nav.askai").click();
      const chat = page.getByRole("region", { name: rlt.t("ask.input") });
      await twoReplies(rlt, chat);
      if (SHOT) await page.screenshot({ path: `docs/ask-ai-copy-img/${SHOT}-${locale}.png` });

      await expect(chat.getByRole("button", { name: rlt.t("common.copy"), exact: true })).toHaveCount(0);
      const label = rlt.t("ask.copy");
      const btns = chat.getByRole("button", { name: label, exact: true });
      await expect(btns).toHaveCount(2);
      await expect(btns.first()).toHaveAttribute("title", label);

      const bub = chat.locator(".msg.assistant .bub").first();
      const bb = (await bub.boundingBox())!;
      const ib = (await btns.first().boundingBox())!;
      expect(ib.x + ib.width).toBeLessThanOrEqual(bb.x + bb.width + 0.5);
      expect(ib.x + ib.width).toBeGreaterThan(bb.x + bb.width - 16);
      expect(ib.y + ib.height).toBeLessThanOrEqual(bb.y + bb.height + 0.5);
      expect(ib.y + ib.height).toBeGreaterThan(bb.y + bb.height - 16);

      for (const i of [0, 1]) {
        const expected = (await chat.locator(".msg.assistant .bub").nth(i).innerText()).trim();
        expect(expected).toContain("Reply to:");
        await btns.nth(i).click();
        await expect(chat.getByRole("button", { name: rlt.t("ask.copied"), exact: true })).toHaveCount(1);
        await expect(chat.locator(".msg.assistant").nth(i).locator(".cp.done")).toHaveCount(1);
        expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(expected);
        await expect(chat.locator(".cp.done")).toHaveCount(0, { timeout: 3000 });
      }
      expect(await rlt.unhandled()).toEqual([]);
      expect(rlt.pageErrors).toEqual([]);
    });

    test("cua so nho nhat 720x520: icon trong khung nhin, khong che chu (chua padding duoi)", async ({ page, rlt }) => {
      await page.setViewportSize({ width: 720, height: 520 });
      await rlt.open({ ...withAi, locale });
      await rlt.nav("nav.askai").click();
      const chat = page.getByRole("region", { name: rlt.t("ask.input") });
      await twoReplies(rlt, chat);
      if (SHOT) await page.screenshot({ path: `docs/ask-ai-copy-img/${SHOT}-720x520-${locale}.png` });
      const btn = chat.getByRole("button", { name: rlt.t("ask.copy"), exact: true }).last();
      await btn.scrollIntoViewIfNeeded();
      await expect(btn).toBeVisible();
      const box = (await btn.boundingBox())!;
      expect(box.x + box.width).toBeLessThanOrEqual(720);
      expect(box.y + box.height).toBeLessThanOrEqual(520);
      expect(box.width).toBeGreaterThanOrEqual(24);
      // Dong chu cuoi cua bong bong nam phia tren icon: icon khong chong len chu.
      const textBottom = await chat.locator(".msg.assistant .bub").last().evaluate(el => {
        const r = document.createRange(); r.selectNodeContents(el); const rs = r.getClientRects(); return rs[rs.length - 1].bottom;
      });
      expect(textBottom).toBeLessThanOrEqual(box.y + 0.5);
    });

    test("panel Hoi AI o chi tiet bien ban (Sessions): cung nut icon, chep dung noi dung", async ({ page, rlt }) => {
      await rlt.open({ ...withAi, locale });
      await rlt.nav("nav.sessions").click();
      await page.locator(".sesrow button").first().click();
      await rlt.btn("nav.askai", page.locator(".tr2")).click();
      const chat = page.getByRole("region", { name: rlt.t("ask.input") });
      await twoReplies(rlt, chat);
      if (SHOT) await page.screenshot({ path: `docs/ask-ai-copy-img/${SHOT}-sessions-${locale}.png` });
      await expect(chat.getByRole("button", { name: rlt.t("common.copy"), exact: true })).toHaveCount(0);
      const btns = chat.getByRole("button", { name: rlt.t("ask.copy"), exact: true });
      await expect(btns).toHaveCount(2);
      const expected = (await chat.locator(".msg.assistant .bub").nth(1).innerText()).trim();
      const second = chat.locator(".msg.assistant .cp").nth(1);
      await second.click();
      await expect(second).toHaveAccessibleName(rlt.t("ask.copied"));
      expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(expected);
    });

    test("focus-visible va mau tuong phan AA o nen sang va toi", async ({ page, rlt }) => {
      await rlt.open({ ...withAi, locale });
      await rlt.nav("nav.askai").click();
      const chat = page.getByRole("region", { name: rlt.t("ask.input") });
      await twoReplies(rlt, chat);
      const btn = chat.getByRole("button", { name: rlt.t("ask.copy"), exact: true }).first();
      await btn.focus();
      await page.keyboard.press("Tab"); await page.keyboard.press("Shift+Tab");
      await expect(btn).toBeFocused();
      expect(await btn.evaluate(el => getComputedStyle(el).boxShadow)).not.toBe("none");
      for (const theme of ["light", "dark"]) {
        await page.evaluate(t => document.documentElement.setAttribute("data-theme", t), theme);
        const r = await btn.evaluate(el => {
          const lum = (c: string) => {
            const m = c.match(/[\d.]+/g)!.map(Number).slice(0, 3).map(v => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; });
            return 0.2126 * m[0] + 0.7152 * m[1] + 0.0722 * m[2];
          };
          const fg = getComputedStyle(el).color;
          let bgEl: Element | null = el, bg = "rgba(0, 0, 0, 0)";
          while (bgEl && /rgba\(.*, 0\)$|transparent/.test(bg)) { bg = getComputedStyle(bgEl).backgroundColor; bgEl = bgEl.parentElement; }
          const a = lum(fg), b = lum(bg);
          return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
        });
        expect(r, `${theme} idle`).toBeGreaterThanOrEqual(4.5);
        await btn.click();
        const rd = await btn.evaluate(el => {
          const lum = (c: string) => {
            const m = c.match(/[\d.]+/g)!.map(Number).slice(0, 3).map(v => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; });
            return 0.2126 * m[0] + 0.7152 * m[1] + 0.0722 * m[2];
          };
          const fg = getComputedStyle(el).color;
          let bgEl: Element | null = el, bg = "rgba(0, 0, 0, 0)";
          while (bgEl && /rgba\(.*, 0\)$|transparent/.test(bg)) { bg = getComputedStyle(bgEl).backgroundColor; bgEl = bgEl.parentElement; }
          const a = lum(fg), b = lum(bg);
          return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
        });
        expect(rd, `${theme} done`).toBeGreaterThanOrEqual(4.5);
        await expect(chat.locator(".cp.done")).toHaveCount(0, { timeout: 3000 });
      }
    });
  });
}
