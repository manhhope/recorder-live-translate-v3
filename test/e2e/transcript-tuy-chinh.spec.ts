// Tuy chinh hien thi transcript o goc phai thanh trang thai man Ghi: co chu, mau chu, hien ban nguon, popover Aa khi hep.
import { test, expect, esc, type Rlt } from "./fixtures.ts";
import type { Page } from "@playwright/test";
import { mkdirSync } from "node:fs";

const SHOT = process.env.RLT_SHOT_DIR;
const KEY = "rlt.ui.transcript";

async function seed(rlt: Rlt) {
  await rlt.startRecording();
  await rlt.emit(rlt.line(1, 1, "Hello everyone", "Olá a todos"));
  await rlt.emit(rlt.line(2, 2, "Let us begin", "Vamos começar"));
  await expect(rlt.page.locator(".ln .dst").first()).toBeVisible();
}
const shot = async (page: Page, name: string) => { if (SHOT) { mkdirSync(SHOT, { recursive: true }); await page.screenshot({ path: `${SHOT}/${name}.png` }); } };
const box = async (page: Page, sel: string) => (await page.locator(sel).first().boundingBox())!;
const css = (page: Page, sel: string, prop: string) => page.locator(sel).first().evaluate((e, p) => getComputedStyle(e).getPropertyValue(p), prop);
const stored = (page: Page) => page.evaluate(k => JSON.parse(localStorage.getItem(k) ?? "null"), KEY);
const wide = (page: Page) => page.locator(".tp-aa");

test.describe("Tuy chinh transcript: man rong (nhom nam ngang thanh duoi)", () => {
  test("nhom nam cung hang voi chu Ready, khong them thanh, khong lam thanh cao hon", async ({ page, rlt }) => {
    await rlt.open();
    await expect(page.locator(".foot")).toHaveCount(1);
    const foot = await box(page, ".foot"), sb = await box(page, ".foot .sb"), tp = await box(page, ".tp .tp-panel");
    await expect(page.locator(".foot .sb")).toContainText(rlt.t("status.ready"));
    await expect(wide(page)).toBeHidden();
    // cung hang: tam doc cua nhom trung tam doc cua chu Ready (lech <= 2px), nhom nam ben phai, trong thanh
    expect(Math.abs(tp.y + tp.height / 2 - (sb.y + sb.height / 2))).toBeLessThanOrEqual(2);
    expect(tp.x).toBeGreaterThan(sb.x + sb.width);
    expect(tp.x + tp.width).toBeLessThanOrEqual(foot.x + foot.width);
    // chieu cao thanh = chieu cao chu + padding 8+8 (+ vien tren 1px): khong tang do nhom tuy chinh
    expect(foot.height).toBeLessThanOrEqual(sb.height + 16 + 1.01);
    await shot(page, "rong-sang-idle");
    const noteH = await box(page, ".foot");
    await seed(rlt);
    expect((await box(page, ".foot")).height).toBe(noteH.height);
  });

  test("co chu: tang/giam doi ngay chu dang chay, hien muc, chan o hai dau, va duoc luu", async ({ page, rlt }) => {
    await rlt.open();
    await seed(rlt);
    const out = page.locator(".tp-val");
    await expect(out).toHaveText(rlt.t("tp.size.value", { n: 15 }));
    expect(await css(page, ".ln .dst", "font-size")).toBe("15px");
    expect(await css(page, ".ln .src", "font-size")).toBe("13px");
    await rlt.btn("tp.size.increase").click();
    await expect(out).toHaveText(rlt.t("tp.size.value", { n: 17 }));
    expect(await css(page, ".ln .dst", "font-size")).toBe("17px");
    expect(await css(page, ".ln .src", "font-size")).toBe("15px");
    await expect(out).toHaveAccessibleName(rlt.t("tp.size.level", { n: 5, max: 7 }));
    // dong moi den sau khi doi cung dung co moi
    await rlt.emit(rlt.line(3, 3, "New line", "Nova linha"));
    expect(await page.locator(".ln .dst").last().evaluate(e => getComputedStyle(e).fontSize)).toBe("17px");
    for (let i = 0; i < 8; i++) if (await rlt.btn("tp.size.increase").isEnabled()) await rlt.btn("tp.size.increase").click();
    await expect(rlt.btn("tp.size.increase")).toBeDisabled();
    await expect(out).toHaveText(rlt.t("tp.size.value", { n: 22 }));
    for (let i = 0; i < 8; i++) if (await rlt.btn("tp.size.decrease").isEnabled()) await rlt.btn("tp.size.decrease").click();
    await expect(rlt.btn("tp.size.decrease")).toBeDisabled();
    await expect(out).toHaveText(rlt.t("tp.size.value", { n: 12 }));
    expect(await stored(page)).toMatchObject({ size: 0 });
  });

  test("mau chu nguon va dich: chon ma, 'Mac dinh' tra ve mau he thong, dat AA o nen sang va toi", async ({ page, rlt }) => {
    await page.emulateMedia({ colorScheme: "light" });
    await rlt.open();
    await seed(rlt);
    const src = page.getByRole("radiogroup", { name: rlt.t("tp.color.src") }), dst = page.getByRole("radiogroup", { name: rlt.t("tp.color.dst") });
    const defDst = await css(page, ".ln .dst", "color");
    await dst.getByRole("radio", { name: rlt.t("tp.color.blue") }).click();
    await src.getByRole("radio", { name: rlt.t("tp.color.green") }).click();
    expect(await css(page, ".ln .dst", "color")).toBe("rgb(10, 91, 196)");
    expect(await css(page, ".ln .src", "color")).toBe("rgb(26, 116, 54)");
    await expect(dst.getByRole("radio", { name: rlt.t("tp.color.blue") })).toHaveAttribute("aria-checked", "true");
    await shot(page, "rong-sang-mau");
    // doi sang nen toi: cung ma mau, hex sang hon (AA tren nen toi)
    await page.emulateMedia({ colorScheme: "dark" });
    await expect.poll(() => css(page, ".ln .dst", "color")).toBe("rgb(106, 166, 255)");
    expect(await css(page, ".ln .src", "color")).toBe("rgb(94, 201, 119)");
    await shot(page, "rong-toi-mau");
    await dst.getByRole("radio", { name: rlt.t("tp.color.default") }).click();
    await page.emulateMedia({ colorScheme: "light" });
    await expect.poll(() => css(page, ".ln .dst", "color")).toBe(defDst);
    expect(await stored(page)).toMatchObject({ srcColor: "green", dstColor: "default" });
  });

  test("luu ben: mo lai van giu lua chon (co chu, mau, che do xem)", async ({ page, rlt }) => {
    await rlt.open();
    await seed(rlt);
    await rlt.btn("tp.size.increase").click();
    await page.getByRole("radiogroup", { name: rlt.t("tp.color.dst") }).getByRole("radio", { name: rlt.t("tp.color.amber") }).click();
    await page.getByRole("switch", { name: rlt.t("tp.showsrc") }).uncheck();
    await page.reload();
    await expect(rlt.btn("tool.start")).toBeEnabled();
    await expect(page.locator(".tp-val")).toHaveText(rlt.t("tp.size.value", { n: 17 }));
    await expect(page.getByRole("radiogroup", { name: rlt.t("tp.color.dst") }).getByRole("radio", { name: rlt.t("tp.color.amber") })).toHaveAttribute("aria-checked", "true");
    await expect(page.getByRole("switch", { name: rlt.t("tp.showsrc") })).not.toBeChecked();
    await expect(page.getByRole("button", { name: rlt.t("rec.view.dst"), pressed: true })).toHaveCount(0); // chua co dong: bo chuyen chua hien
    await rlt.startRecording();
    await rlt.emit(rlt.line(1, 1, "Hello everyone", "Olá a todos"));
    await expect(page.getByRole("button", { name: rlt.t("rec.view.dst") })).toHaveAttribute("aria-pressed", "true");
    await expect(page.locator(".ln .src")).toHaveCount(0);
    expect(await css(page, ".ln .dst", "font-size")).toBe("17px");
  });

  test("cong tac 'Hien ban nguon' dong bo hai chieu voi bo chuyen Both/Dich/Nguon", async ({ page, rlt }) => {
    await rlt.open();
    await seed(rlt);
    const sw = page.getByRole("switch", { name: rlt.t("tp.showsrc") });
    await expect(sw).toBeChecked();
    await expect(page.locator(".ln .src").first()).toBeVisible();
    await sw.uncheck();
    await expect(page.getByRole("button", { name: rlt.t("rec.view.dst") })).toHaveAttribute("aria-pressed", "true");
    await expect(page.locator(".ln .src")).toHaveCount(0);
    await page.getByRole("button", { name: rlt.t("rec.view.src") }).click();
    await expect(sw).toBeChecked();
    await expect(page.locator(".ln:not(.live) .dst")).toHaveCount(0);
    await sw.uncheck();
    await sw.check();
    await expect(page.getByRole("button", { name: rlt.t("rec.view.both") })).toHaveAttribute("aria-pressed", "true");
    await expect(page.locator(".ln .src").first()).toBeVisible();
  });

  test("pham vi: khong goi lenh phu de, khong dong vao cai dat phu de", async ({ page, rlt }) => {
    await rlt.open();
    await seed(rlt);
    await rlt.btn("tp.size.increase").click();
    await page.getByRole("radiogroup", { name: rlt.t("tp.color.dst") }).getByRole("radio", { name: rlt.t("tp.color.red") }).click();
    await page.getByRole("switch", { name: rlt.t("tp.showsrc") }).uncheck();
    for (const c of ["subtitle_font_step", "subtitle_set_color", "subtitle_set_mode", "subtitle_reset_colors"]) expect(await rlt.calls(c)).toHaveLength(0);
  });
});

test.describe("Tuy chinh transcript: cua so hep (popover Aa)", () => {
  for (const loc of ["en", "pt-PT"] as const) for (const w of [720, 860]) {
    test(`${loc} @${w}: Aa thay nhom, popover khong tran, khong che chu Ready, dong bang Escape/bam ngoai`, async ({ page, rlt }) => {
      await page.setViewportSize({ width: w, height: 700 });
      await rlt.open({ locale: loc });
      await seed(rlt);
      const aa = wide(page);
      await expect(aa).toBeVisible();
      await expect(page.locator(".tp-panel")).toBeHidden();
      const sb = await box(page, ".foot .sb"), foot = await box(page, ".foot"), aab = (await aa.boundingBox())!;
      expect(aab.x).toBeGreaterThanOrEqual(sb.x + sb.width);
      expect(aab.x + aab.width).toBeLessThanOrEqual(foot.x + foot.width + 0.5);
      expect(foot.height).toBeLessThanOrEqual(sb.height + 16 + 1.01);
      await aa.click();
      await expect(aa).toHaveAttribute("aria-expanded", "true");
      const panel = page.locator(".tp-panel");
      await expect(panel).toBeVisible();
      const pb = (await panel.boundingBox())!, vp = page.viewportSize()!;
      expect(pb.x).toBeGreaterThanOrEqual(0);
      expect(pb.x + pb.width).toBeLessThanOrEqual(vp.width);
      expect(pb.y).toBeGreaterThanOrEqual(0);
      // popover nam tren thanh, khong chong len chu Ready
      expect(pb.y + pb.height).toBeLessThanOrEqual(foot.y + 0.5);
      const sbAfter = await box(page, ".foot .sb");
      expect(sbAfter).toEqual(sb);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth)).toBe(true);
      await expect(page.locator(".tp-note")).toContainText(rlt.t("tp.scope"));
      await shot(page, `hep-${loc}-${w}-mo`);
      // thao tac trong popover ap ngay len transcript phia sau
      await rlt.btn("tp.size.increase", panel).click();
      expect(await css(page, ".ln .dst", "font-size")).toBe("17px");
      await page.keyboard.press("Escape");
      await expect(panel).toBeHidden();
      await expect(aa).toBeFocused();
      await aa.click();
      await expect(panel).toBeVisible();
      await page.locator(".trs").click({ position: { x: 5, y: 5 }, force: true });
      await expect(panel).toBeHidden();
      await shot(page, `hep-${loc}-${w}-dong`);
    });
  }

  test("nut dat lai khoi phuc mac dinh (co chu, mau, che do xem)", async ({ page, rlt }) => {
    await page.setViewportSize({ width: 720, height: 700 });
    await rlt.open();
    await seed(rlt);
    await wide(page).click();
    const panel = page.locator(".tp-panel");
    await rlt.btn("tp.size.increase", panel).click();
    await panel.getByRole("radiogroup", { name: rlt.t("tp.color.dst") }).getByRole("radio", { name: rlt.t("tp.color.purple") }).click();
    await panel.getByRole("switch", { name: rlt.t("tp.showsrc") }).uncheck();
    await rlt.btn("tp.reset", panel).click();
    expect(await stored(page)).toEqual({ size: 3, srcColor: "default", dstColor: "default", view: "both" });
    expect(await css(page, ".ln .dst", "font-size")).toBe("15px");
    await expect(page.locator(".ln .src").first()).toBeVisible();
  });
});
