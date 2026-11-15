// Usage Logs kieu stale-while-revalidate (US-32, AC-32.6, AC-32.10..12): 4 kich ban + xoa cache + cache rieng tung tuan.
// Lop invoke gia o tauri-mock.ts (seed.usage). So Soniox cu = $0.19 (188.100 micro), so moi = $0.25 (250.000 micro).
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";

const T_OLD = Math.floor(Date.UTC(2026, 10, 15, 9, 5) / 1000);
const T_NEW = Math.floor(Date.UTC(2026, 10, 15, 14, 42) / 1000);
const OLD = { costUsdMicros: 188_100, fetchedAt: T_OLD };
const NEW = { costUsdMicros: 250_000, fetchedAt: T_NEW };

async function openTab(page: Page, rlt: any) {
  await rlt.nav("nav.settings").click();
  await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.sonioxusage") }).click();
}
/** HH:mm theo gio may cua trinh duyet, cung cach giao dien tinh. */
const hhmm = (page: Page, ts: number) => page.evaluate(t => new Date(t * 1000).toLocaleTimeString("en", { hour: "2-digit", minute: "2-digit", hour12: false }), ts);
const label = async (page: Page, rlt: any, ts: number) => rlt.t("set.usage.updatedat", { time: await hhmm(page, ts) });

test.describe("Usage Logs: stale-while-revalidate", () => {
  test("1. co cache: hien ngay so cu kem 'Cap nhat luc', chi bao Dang cap nhat, khong skeleton, khong trong", async ({ page, rlt }) => {
    await rlt.open({ usage: { cached: OLD, live: NEW, hold: true } });
    await openTab(page, rlt);
    await expect(page.getByTestId("usage-total")).toContainText(/0[.,]19/);
    await expect(page.getByTestId("usage-updated")).toHaveText(await label(page, rlt, T_OLD));
    await expect(page.getByTestId("usage-source")).toContainText(rlt.t("set.usage.src.soniox"));
    await expect(page.getByTestId("usage-refreshing")).toHaveText(rlt.t("set.usage.refreshing"));
    await expect(page.getByTestId("usage-skeleton")).toHaveCount(0);
    await expect(page.getByTestId("usage-warning")).toHaveCount(0);
    // trong luc doi mang so cu van o do (khong nhap nhay trong): do lai nhieu lan
    for (let i = 0; i < 5; i++) { await expect(page.getByTestId("usage-total")).toContainText(/0[.,]19/); await page.waitForTimeout(40); }
    expect((await rlt.calls("usage_week_cached")).length).toBe(1);
    expect((await rlt.calls("usage_week_source")).length).toBe(1);
  });

  test("2. khong cache: skeleton kem so cuc bo du phong va nhan nguon ro rang", async ({ page, rlt }) => {
    await rlt.open({ usage: { live: NEW, hold: true } });
    await openTab(page, rlt);
    await expect(page.getByTestId("usage-skeleton")).toBeVisible();
    await expect(page.getByTestId("usage-source")).toContainText(rlt.t("set.usage.src.local"));
    await expect(page.getByTestId("usage-nocache")).toHaveText(rlt.t("set.usage.reason.nocache"));
    await expect(page.getByText(rlt.t("set.usage.audio", { h: 3, m: 15 }))).toBeVisible(); // so cuc bo 11.700.000 ms
    await expect(page.getByTestId("usage-refreshing")).toBeVisible();
    await expect(page.getByTestId("usage-updated")).toHaveCount(0);
    await page.evaluate(() => (window as any).__rlt.releaseUsage());
    await expect(page.getByTestId("usage-skeleton")).toHaveCount(0);
    await expect(page.getByTestId("usage-source")).toContainText(rlt.t("set.usage.src.soniox"));
    await expect(page.getByTestId("usage-total")).toContainText(/0[.,]25/);
  });

  test("3. loi mang: giu so cu, canh bao gon kem Thu lai, bam Thu lai thi cap nhat", async ({ page, rlt }) => {
    await rlt.open({ usage: { cached: OLD, liveFail: "network" } });
    await openTab(page, rlt);
    const warn = page.getByTestId("usage-warning");
    await expect(warn).toContainText(rlt.t("set.usage.reason.network"));
    await expect(warn.getByRole("button", { name: rlt.t("common.retry") })).toBeEnabled();
    await expect(page.getByTestId("usage-total")).toContainText(/0[.,]19/);
    await expect(page.getByTestId("usage-updated")).toHaveText(await label(page, rlt, T_OLD)); // gio khong doi
    await expect(page.getByTestId("usage-refreshing")).toHaveCount(0);
    await expect(page.getByTestId("usage-source")).toContainText(rlt.t("set.usage.src.soniox"));
    // het han khoa / Soniox loi cung giu so cu
    for (const reason of ["unauthorized", "server"]) {
      await page.evaluate(r => { (window as any).__rlt.usage.liveFail = r; }, reason);
      await warn.getByRole("button", { name: rlt.t("common.retry") }).click();
      await expect(warn).toContainText(rlt.t("set.usage.reason." + reason));
      await expect(page.getByTestId("usage-total")).toContainText(/0[.,]19/);
    }
    // Thu lai thanh cong
    await page.evaluate(n => { const u = (window as any).__rlt.usage; delete u.liveFail; u.live = n; }, NEW);
    await warn.getByRole("button", { name: rlt.t("common.retry") }).click();
    await expect(page.getByTestId("usage-warning")).toHaveCount(0);
    await expect(page.getByTestId("usage-total")).toContainText(/0[.,]25/);
  });

  test("4. tai xong: so moi thay so cu va nhan gio doi, chi bao bien mat", async ({ page, rlt }) => {
    await rlt.open({ usage: { cached: OLD, live: NEW, hold: true } });
    await openTab(page, rlt);
    await expect(page.getByTestId("usage-total")).toContainText(/0[.,]19/);
    await expect(page.getByTestId("usage-refreshing")).toBeVisible();
    await page.evaluate(() => (window as any).__rlt.releaseUsage());
    await expect(page.getByTestId("usage-total")).toContainText(/0[.,]25/);
    await expect(page.getByTestId("usage-updated")).toHaveText(await label(page, rlt, T_NEW));
    await expect(page.getByTestId("usage-refreshing")).toHaveCount(0);
    await expect(page.getByTestId("usage-warning")).toHaveCount(0);
  });

  test("moi tuan co cache rieng: doi tuan goi usage_week_cached theo so tuan", async ({ page, rlt }) => {
    await rlt.open({ usage: { cached: OLD, live: NEW } });
    await openTab(page, rlt);
    await expect(page.getByTestId("usage-total")).toContainText(/0[.,]25/);
    await page.getByRole("button", { name: rlt.t("set.week.2") }).click();
    await expect.poll(async () => (await rlt.calls("usage_week_cached")).map(c => c.args.weeksAgo)).toEqual([0, 2]);
    await expect.poll(async () => (await rlt.calls("usage_week_source")).map(c => c.args.weeksAgo)).toEqual([0, 2]);
  });

  test("Xoa du lieu muc dung xoa ca cache: mo lai chua co ban luu, hien skeleton + so cuc bo (AC-32.6, AC-32.12)", async ({ page, rlt }) => {
    await rlt.open({ usage: { cached: OLD, live: NEW, hold: true } });
    await openTab(page, rlt);
    await expect(page.getByTestId("usage-total")).toContainText(/0[.,]19/);
    await page.evaluate(() => (window as any).__rlt.releaseUsage());
    await expect(page.getByTestId("usage-refreshing")).toHaveCount(0);
    await page.getByRole("button", { name: rlt.t("set.usage.clear") }).click();
    await page.evaluate(() => { (window as any).__rlt.usage.hold = true; });
    await page.getByRole("button", { name: rlt.t("set.usage.clear.confirm") }).click();
    expect(await rlt.calls("usage_clear")).toHaveLength(1);
    await expect(page.getByTestId("usage-nocache")).toBeVisible(); // cache da bi xoa, dang cho Soniox
    await expect(page.getByTestId("usage-source")).toContainText(rlt.t("set.usage.src.local"));
    await expect(page.getByTestId("usage-clear-note")).toContainText(rlt.t("set.usage.clear.note"));
    await page.evaluate(() => (window as any).__rlt.releaseUsage());
    await expect(page.getByTestId("usage-total")).toContainText(/0[.,]25/); // so ben Soniox khong doi vi bi xoa cuc bo
  });

  test("pt-PT: nhan gio va chi bao co ban dich", async ({ page, rlt }) => {
    await rlt.open({ locale: "pt-PT", usage: { cached: OLD, live: NEW, hold: true } });
    await openTab(page, rlt);
    await expect(page.getByTestId("usage-refreshing")).toHaveText(rlt.t("set.usage.refreshing", {}, "pt-PT"));
    const t = await page.evaluate(ts => new Date(ts * 1000).toLocaleTimeString("pt-PT", { hour: "2-digit", minute: "2-digit", hour12: false }), T_OLD);
    await expect(page.getByTestId("usage-updated")).toHaveText(rlt.t("set.usage.updatedat", { time: t }, "pt-PT"));
  });
});
