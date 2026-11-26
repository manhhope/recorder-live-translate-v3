// U-04 vong focus, U-08 nhan nguon am, U-09 mau phu de co ten + dau tich, U-18 nut EN/PT va sang/toi co ten hanh dong.
import { test, expect } from "./fixtures.ts";

for (const loc of ["en", "pt-PT"] as const) {
  test(`U-04 vong focus ban phim: outline 2px ro rang ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc });
    await page.keyboard.press("Tab");
    const st = await page.evaluate(() => {
      const el = document.activeElement as HTMLElement;
      const c = getComputedStyle(el);
      return { tag: el.tagName, w: c.outlineWidth, style: c.outlineStyle, shadow: c.boxShadow };
    });
    expect(st.style).toBe("solid");
    expect(parseFloat(st.w)).toBeGreaterThanOrEqual(2);
    expect(st.shadow).not.toBe("none");
  });

  test(`U-08 nut nguon am co aria-label ngan va nhan hien thi o cua so rong ${loc}`, async ({ page, rlt }) => {
    await page.setViewportSize({ width: 1280, height: 760 });
    await rlt.open({ locale: loc });
    const group = page.getByRole("group", { name: rlt.t("cfg.audio.group") });
    const sys = group.getByRole("button", { name: new RegExp("^" + rlt.t("src.label.sys")) });
    const mic = group.getByRole("button", { name: new RegExp("^" + rlt.t("src.label.mic")) });
    await expect(sys).toHaveAccessibleName(rlt.t("src.label.sys") + ": " + rlt.t("src.state.idle"));
    await expect(mic).toHaveAccessibleName(rlt.t("src.label.mic") + ": " + rlt.t("src.state.idle"));
    await expect(sys.locator(".sl")).toBeVisible();
    await expect(sys.locator(".sl")).toHaveText(rlt.t("src.label.sys"));
    // Tooltip dai van con (mo ta cach bat/tat) nhung khong phai ten truy cap.
    expect((await sys.getAttribute("data-tip"))!.length).toBeGreaterThan(40);
  });

  test(`U-08 cua so hep an nhan, chi con aria-label ${loc}`, async ({ page, rlt }) => {
    await page.setViewportSize({ width: 900, height: 600 });
    await rlt.open({ locale: loc });
    const sys = page.getByRole("group", { name: rlt.t("cfg.audio.group") }).getByRole("button").first();
    await expect(sys.locator(".sl")).toBeHidden();
    await expect(sys).toHaveAccessibleName(new RegExp("^" + rlt.t("src.label.sys")));
  });

  test(`U-09 12 o mau phu de co ten truy cap va dau tich o o dang chon ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc });
    if (await page.locator(".tp-aa").isVisible()) await page.locator(".tp-aa").click(); // cua so hep: popover Aa
    const row = page.getByRole("radiogroup", { name: rlt.t("tp.color.src") });
    const radios = row.getByRole("radio");
    await expect(radios).toHaveCount(6);
    for (const id of ["default", "blue", "green", "amber", "red", "purple"]) await expect(row.getByRole("radio", { name: rlt.t("tp.color." + id) })).toBeVisible();
    const on = row.getByRole("radio", { checked: true });
    await expect(on).toHaveCount(1);
    expect(await on.evaluate(e => getComputedStyle(e, "::after").content)).toContain("✓");
    const off = row.getByRole("radio", { checked: false }).first();
    expect(await off.evaluate(e => getComputedStyle(e, "::after").content)).toBe("none");
    // Hai hang (nguon + dich) = 12 o.
    await expect(page.locator(".tp-panel .sw")).toHaveCount(12);
  });

  test(`U-18 nut ngon ngu va giao dien co ten la hanh dong ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc });
    const nav = page.getByRole("navigation", { name: rlt.t("nav.tools") });
    const langKey = loc === "en" ? "ui.lang.to.pt" : "ui.lang.to.en";
    await expect(nav.getByRole("button", { name: rlt.t(langKey) })).toBeVisible();
    const themeBtns = nav.getByRole("button", { name: new RegExp(`^(${rlt.t("theme.to.dark")}|${rlt.t("theme.to.light")})$`) });
    await expect(themeBtns).toHaveCount(1);
    const before = await themeBtns.getAttribute("aria-label");
    await themeBtns.click();
    const after = await nav.getByRole("button", { name: new RegExp(`^(${rlt.t("theme.to.dark")}|${rlt.t("theme.to.light")})$`) }).getAttribute("aria-label");
    expect(after).not.toBe(before);
  });
}
