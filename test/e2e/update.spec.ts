// US-33 cap nhat tu dong: ban moi (kiem, tai co tien do, cai va khoi dong lai), khong co ban moi, chu ky sai, dang ghi am, loi mang, cong tat.
// Lop Tauri la gia (update_check/update_download/update_install trong tauri-mock.ts); viec that su kiem chu ky, tai, cai chi thu duoc tren may that:
// xem docs/us33-updater-bao-cao.md.
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";

const aboutTab = (page: Page, rlt: any) => page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.permissions") }).click();
const openAbout = async (page: Page, rlt: any) => { await rlt.nav("nav.settings").click(); await aboutTab(page, rlt); };
const calls = (page: Page, cmd: string) => page.evaluate(c => (window as any).__rlt.calls.filter((x: any) => x.cmd === c).length as number, cmd);
const NEW = { version: "9.9.9", notes: "Faster export and a fix for the subtitle window." };
const card = (page: Page) => page.getByTestId("update-card");

for (const loc of ["en", "pt-PT"] as const) {
  test(`ban moi: hien phien ban hien tai va moi, ghi chu, tai co tien do, Cai va khoi dong lai ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc, update: NEW });
    await openAbout(page, rlt);
    await expect(card(page)).toContainText(rlt.t("set.upd.title"));
    await expect(page.getByTestId("update-current")).not.toBeEmpty();
    await expect(page.getByTestId("update-auto")).toBeChecked(); // mac dinh bat (data-dictionary)
    await page.getByTestId("update-check").click();
    await expect(page.getByTestId("update-new")).toHaveText("9.9.9");
    await expect(page.getByTestId("update-notes")).toContainText(NEW.notes);
    await page.getByTestId("update-download").click();
    await expect(page.getByTestId("update-ready")).toContainText("9.9.9");
    await expect(page.getByTestId("update-install")).toBeEnabled();
    expect(await calls(page, "update_download")).toBe(1);
    expect(await calls(page, "update_install")).toBe(0); // chua bam Cai thi khong cai
    await page.getByTestId("update-install").click();
    await expect.poll(() => calls(page, "update_install")).toBe(1);
  });
}

test("khong co ban moi: bao da moi nhat, khong co nut tai", async ({ page, rlt }) => {
  await rlt.open();
  await openAbout(page, rlt);
  await page.getByTestId("update-check").click();
  await expect(page.getByTestId("update-uptodate")).toContainText(rlt.t("set.upd.uptodate"));
  await expect(page.getByTestId("update-download")).toHaveCount(0);
  await expect(page.getByTestId("update-new")).toHaveCount(0);
  await expect(page.getByTestId("update-error")).toHaveCount(0);
});

test("chu ky sai khi tai: bao loi, KHONG cho cai, nut Thu lai van co", async ({ page, rlt }) => {
  await rlt.open({ update: NEW });
  await openAbout(page, rlt);
  await page.getByTestId("update-check").click();
  await page.evaluate(() => { (window as any).__rlt.fail.update_download = "update_signature"; });
  await page.getByTestId("update-download").click();
  await expect(page.getByTestId("update-error")).toContainText(rlt.t("err.updatesignature"));
  await expect(page.getByTestId("update-install")).toHaveCount(0);
  await expect(page.getByTestId("update-retry")).toBeVisible();
  expect(await calls(page, "update_install")).toBe(0);
  await expect(page.getByTestId("update-banner")).toHaveCount(0);
});

test("chu ky sai ngay khi kiem: bao loi chu ky", async ({ page, rlt }) => {
  await rlt.open({ update: NEW, fail: { update_check: "update_signature" } });
  await openAbout(page, rlt);
  await page.getByTestId("update-check").click();
  await expect(page.getByTestId("update-error")).toContainText(rlt.t("err.updatesignature"));
  await expect(page.getByTestId("update-download")).toHaveCount(0);
});

test("loi mang khi kiem: bao loi, Thu lai kiem lai va thanh cong khi mang tro lai", async ({ page, rlt }) => {
  await rlt.open({ update: NEW, fail: { update_check: "update_network" } });
  await openAbout(page, rlt);
  await page.getByTestId("update-check").click();
  await expect(page.getByTestId("update-error")).toContainText(rlt.t("err.updatenetwork"));
  await page.evaluate(() => { delete (window as any).__rlt.fail.update_check; });
  await page.getByTestId("update-retry").click();
  await expect(page.getByTestId("update-new")).toHaveText("9.9.9");
});

test("dang ghi am: tai duoc nhung Cai bi khoa va hien ly do; Dung xong thi Cai duoc", async ({ page, rlt }) => {
  await rlt.open({ update: NEW });
  await rlt.startRecording();
  await openAbout(page, rlt);
  await page.getByTestId("update-check").click();
  await page.getByTestId("update-download").click();
  await expect(page.getByTestId("update-ready")).toBeVisible();
  await expect(page.getByTestId("update-install")).toBeDisabled();
  await expect(page.getByTestId("update-deferred")).toContainText(rlt.t("set.upd.deferred"));
  expect(await calls(page, "update_install")).toBe(0);
  await rlt.nav("nav.record").click();
  await rlt.btn("tool.stop").click();
  await expect(rlt.btn("tool.start")).toBeVisible();
  await rlt.nav("nav.settings").click();
  await expect(page.getByTestId("update-install")).toBeEnabled();
  await expect(page.getByTestId("update-deferred")).toHaveCount(0);
  await page.getByTestId("update-install").click();
  await expect.poll(() => calls(page, "update_install")).toBe(1);
});

test("Rust tu choi cai khi dang ghi (update_recording): bao loi, khong mat ban da tai", async ({ page, rlt }) => {
  await rlt.open({ update: NEW, fail: { update_install: "update_recording" } });
  await openAbout(page, rlt);
  await page.getByTestId("update-check").click();
  await page.getByTestId("update-download").click();
  await page.getByTestId("update-install").click();
  await expect(page.getByTestId("update-error")).toContainText(rlt.t("err.updaterecording"));
  await expect(page.getByTestId("update-install")).toBeVisible();
});

test("tat tu dong kiem: luu lai sau khi tai lai, khong goi update_check neu khong bam Check", async ({ page, rlt }) => {
  await rlt.open({ update: NEW });
  await openAbout(page, rlt);
  await page.getByTestId("update-auto").uncheck();
  expect(await page.evaluate(() => localStorage.getItem("rlt.ui.autoUpdate"))).toBe("0");
  await page.reload();
  await expect(rlt.btn("tool.start")).toBeEnabled();
  await openAbout(page, rlt);
  await expect(page.getByTestId("update-auto")).not.toBeChecked();
  await page.waitForTimeout(500);
  expect(await calls(page, "update_check")).toBe(0);
  // nguon su that la cot DB: da goi update_auto_set(false) va doc lai bang update_auto_get sau khi tai lai
  expect(await calls(page, "update_auto_set")).toBe(0); // trang moi sau reload (nhat ky goi bi xoa)
  expect(await calls(page, "update_auto_get")).toBeGreaterThan(0);
});

test("banner nhac khi ban tai xong va dang o man khac; Mai sau an banner", async ({ page, rlt }) => {
  await rlt.open({ update: NEW });
  await openAbout(page, rlt);
  await page.getByTestId("update-check").click();
  await page.getByTestId("update-download").click();
  await expect(page.getByTestId("update-ready")).toBeVisible();
  await rlt.nav("nav.record").click();
  await expect(page.getByTestId("update-banner")).toContainText("9.9.9");
  await page.getByTestId("update-banner-later").click();
  await expect(page.getByTestId("update-banner")).toHaveCount(0);
});
