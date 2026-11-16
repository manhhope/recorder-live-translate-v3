// Popup chon ngon ngu (nguon/dich) va cac popup khac dung chung action `dismiss` (src/lib/dismiss.ts):
// pointerdown ngoai thi dong, trong thi giu, Esc dong, mo popup khac thi cai cu dong, dang ghi thi khoa.
import { test, expect } from "./fixtures.ts";

test.describe("Popup dong khi bam ra ngoai", () => {
  test("click ngoai dong, click trong va nut mo giu dung, Esc dong va tra focus", async ({ page, rlt }) => {
    await rlt.open();
    const src = page.locator(".cfg .sel").nth(0);
    const menu = page.getByRole("listbox");
    await expect(src).toHaveAttribute("aria-expanded", "false");

    await src.click();
    await expect(menu).toBeVisible();
    await expect(src).toHaveAttribute("aria-expanded", "true");

    // Click trong popup (o tim kiem, tieu de nhom) khong dong.
    await menu.getByRole("textbox").click();
    await expect(menu).toBeVisible();
    await menu.locator(".mg").first().click();
    await expect(menu).toBeVisible();

    // Click ngoai dong.
    await page.locator(".foot").first().click({ position: { x: 5, y: 5 } }).catch(() => page.mouse.click(5, 5));
    await expect(menu).toHaveCount(0);
    await expect(src).toHaveAttribute("aria-expanded", "false");

    // Nut mo bam lai: dong (toggle) roi mo, khong bi dong sai.
    await src.click();
    await expect(menu).toBeVisible();
    await src.click();
    await expect(menu).toHaveCount(0);

    // Esc dong va tra focus ve nut mo; chon mot ngon ngu thi dong.
    await src.click();
    await expect(menu).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(menu).toHaveCount(0);
    await expect(src).toBeFocused();
    await src.click();
    await menu.getByRole("option").nth(1).click();
    await expect(menu).toHaveCount(0);
  });

  test("mo popup khac thi popup cu dong (nguon <-> dich)", async ({ page, rlt }) => {
    await rlt.open();
    const src = page.locator(".cfg .sel").nth(0), dst = page.locator(".cfg .sel").nth(1);
    await src.click();
    await expect(page.getByRole("listbox")).toHaveCount(1);
    await dst.click();
    await expect(page.getByRole("listbox")).toHaveCount(1);
    await expect(src).toHaveAttribute("aria-expanded", "false");
    await expect(dst).toHaveAttribute("aria-expanded", "true");
    await src.click();
    await expect(src).toHaveAttribute("aria-expanded", "true");
    await expect(dst).toHaveAttribute("aria-expanded", "false");
  });

  test("popup Aa (man hep, khi dang ghi): click ngoai dong, click trong giu, Esc dong va tra focus", async ({ page, rlt }) => {
    await page.setViewportSize({ width: 720, height: 700 });
    await rlt.open();
    await rlt.startRecording();
    const aa = page.locator(".tp-aa"), panel = page.locator("#tp-panel");
    await aa.click();
    await expect(aa).toHaveAttribute("aria-expanded", "true");
    await panel.locator(".tp-row").first().click();
    await expect(aa).toHaveAttribute("aria-expanded", "true");
    await page.mouse.click(5, 5);
    await expect(aa).toHaveAttribute("aria-expanded", "false");
    await aa.click();
    await expect(aa).toHaveAttribute("aria-expanded", "true");
    await page.keyboard.press("Escape");
    await expect(aa).toHaveAttribute("aria-expanded", "false");
    await expect(aa).toBeFocused();
  });

  test("dang ghi: phan khoa ngon ngu khong mo, popup dang mo dong khi bat dau ghi", async ({ page, rlt }) => {
    await rlt.open();
    const src = page.locator(".cfg .sel").nth(0);
    await src.click();
    await expect(page.getByRole("listbox")).toBeVisible();
    await rlt.btn("tool.start").click();   // bam ngoai popup (nut Bat dau) -> dong, roi vao trang thai ghi
    await expect(rlt.btn("tool.stop")).toBeVisible();
    await expect(page.getByRole("listbox")).toHaveCount(0);
    await expect(page.locator(".cfg")).toHaveClass(/lock/);
    await expect(src).toBeDisabled();
    await src.click({ force: true });
    await expect(page.getByRole("listbox")).toHaveCount(0);
    await page.keyboard.press("Escape");
    await expect(page.getByRole("listbox")).toHaveCount(0);
  });
});
