// Cai dat > Google Drive: nut Connect, cac trang thai cho/huy/da ket noi/ngat/sync ngay va cac loi.
// Lop Tauri la gia (khong goi Google that): phan thuc te (trinh duyet that, Keychain that, Client ID that) xem
// docs/drive-connect-bao-cao.md muc "Chi thu duoc tren may that".
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";

async function openDrive(page: Page, rlt: any) {
  await rlt.nav("nav.settings").click();
  await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.drive") }).click();
  await expect(page.getByRole("heading", { name: rlt.t("set.drive.title") })).toBeVisible();
}
const finish = (page: Page, r: string) => page.evaluate(x => (window as any).__rlt.driveFinish(x), r);

test.describe("Cai dat: Google Drive (nut Connect)", () => {
  test("chua co Client ID: nut Connect hien nhung khoa, co giai thich va nut Mo Nang cao mo o nhap", async ({ page, rlt }) => {
    await rlt.open({ drive: { state: "not_configured" } });
    await openDrive(page, rlt);
    const connect = rlt.btn("set.drive.connect");
    await expect(connect).toBeVisible();
    await expect(connect).toBeDisabled();
    await expect(page.getByText(rlt.t("set.drive.notconfigured.hint"))).toBeVisible();
    // O Client ID nam trong Nang cao thu gon.
    await expect(page.getByLabel(rlt.t("set.drive.clientid"))).toHaveCount(0);
    await rlt.btn("set.drive.openadvanced").click();
    const id = page.getByLabel(rlt.t("set.drive.clientid"));
    await expect(id).toBeVisible();
    await id.fill("  1234-abcd.apps.googleusercontent.com ");
    await rlt.btn("set.drive.saveconfig").click();
    await expect(rlt.btn("set.drive.connect")).toBeEnabled();
    const [c] = await rlt.calls("drive_save_client_config");
    expect(c.args.clientId).toBe("1234-abcd.apps.googleusercontent.com");
    // Khong goi lenh ket noi khi chua bam.
    expect(await rlt.calls("drive_connect")).toHaveLength(0);
  });

  test("co Client ID dung san: Connect bat san, o Client ID van thu gon trong Nang cao", async ({ page, rlt }) => {
    await rlt.open({ drive: { state: "disconnected", builtin: true } });
    await openDrive(page, rlt);
    await expect(rlt.btn("set.drive.connect")).toBeEnabled();
    await expect(page.getByText(rlt.t("set.drive.connect.hint"))).toBeVisible();
    await expect(page.getByText(rlt.t("set.drive.notconfigured.hint"))).toHaveCount(0);
    await expect(page.getByLabel(rlt.t("set.drive.clientid"))).toHaveCount(0);
    const adv = rlt.btn("set.drive.advanced");
    await expect(adv).toHaveAttribute("aria-expanded", "false");
    await adv.click();
    await expect(adv).toHaveAttribute("aria-expanded", "true");
    await expect(page.getByLabel(rlt.t("set.drive.clientid"))).toBeVisible();
  });

  test("Connect -> Dang cho xac nhan -> Da ket noi kem email -> Sync ngay -> Ngat ket noi", async ({ page, rlt }) => {
    await rlt.open({ drive: { state: "disconnected", builtin: true } });
    await openDrive(page, rlt);
    await rlt.btn("set.drive.connect").click();
    await expect(page.getByText(rlt.t("set.drive.connecting"), { exact: false })).toBeVisible();
    await expect(rlt.btn("set.drive.cancel")).toBeVisible();
    await expect(rlt.btn("set.drive.connect")).toHaveCount(0);
    expect(await rlt.calls("drive_connect")).toHaveLength(1);

    await finish(page, "ok");
    await expect(page.locator(".bd.ok")).toContainText(rlt.t("set.drive.state.connected"));
    await expect(page.getByTestId("drive-account")).toContainText("chu@example.com");
    // Client ID / secret khong con hien khi da ket noi.
    await expect(page.getByLabel(rlt.t("set.drive.clientid"))).toHaveCount(0);

    await rlt.btn("set.drive.syncnow").click();
    await expect(page.getByText(rlt.t("set.drive.sync.done", { ok: 2 }))).toBeVisible();
    expect(await rlt.calls("drive_sync_now")).toHaveLength(1);

    await rlt.btn("set.drive.disconnect").click();
    await expect(page.getByText(rlt.t("set.drive.disconnected.ok"))).toBeVisible();
    await expect(rlt.btn("set.drive.connect")).toBeEnabled();
    await expect(page.getByTestId("drive-account")).toHaveCount(0);
  });

  test("bam Huy luc dang cho: ve trang thai san sang, bao da huy, khong bao loi", async ({ page, rlt }) => {
    await rlt.open({ drive: { state: "disconnected", builtin: true } });
    await openDrive(page, rlt);
    await rlt.btn("set.drive.connect").click();
    await rlt.btn("set.drive.cancel").click();
    expect(await rlt.calls("drive_connect_cancel")).toHaveLength(1);
    await expect(page.getByText(rlt.t("set.drive.cancelled"))).toBeVisible();
    await expect(rlt.btn("set.drive.connect")).toBeEnabled();
    await expect(page.locator(".bd.rec")).toHaveCount(0);
  });

  for (const [code, key] of [
    ["drive_auth_timeout", "err.driveauthtimeout"],
    ["drive_auth_denied", "err.driveauthdenied"],
    ["drive_network", "err.drivenetwork"],
    ["drive_browser_failed", "err.drivebrowserfailed"],
    ["drive_no_refresh_token", "err.drivenorefreshtoken"],
    ["keychain_error", "err.keychainerror"],
  ] as const) {
    test(`loi ${code}: hien thong bao dung va cho ket noi lai`, async ({ page, rlt }) => {
      await rlt.open({ drive: { state: "disconnected", builtin: true } });
      await openDrive(page, rlt);
      await rlt.btn("set.drive.connect").click();
      await finish(page, code);
      await expect(page.locator(".bd.rec")).toContainText(rlt.t(key));
      await expect(rlt.btn("set.drive.connect")).toBeEnabled();
      await expect(page.getByText(rlt.t("set.drive.cancelled"))).toHaveCount(0);
    });
  }

  test("Sync ngay: mot phan loi / cho mang thi bao so lieu", async ({ page, rlt }) => {
    await rlt.open({ drive: { state: "connected", email: "chu@example.com" }, driveSyncAll: { total: 4, synced: 2, failed: 1, waiting: 1, oauthConnected: true } });
    await openDrive(page, rlt);
    await rlt.btn("set.drive.syncnow").click();
    await expect(page.getByText(rlt.t("set.drive.sync.partial", { ok: 2, failed: 1, waiting: 1 }))).toBeVisible();
  });

  test("Sync ngay khi khong co gi de gui", async ({ page, rlt }) => {
    await rlt.open({ drive: { state: "connected", email: null }, driveSyncAll: { total: 0, synced: 0, failed: 0, waiting: 0, oauthConnected: true } });
    await openDrive(page, rlt);
    await expect(page.getByTestId("drive-account")).toContainText(rlt.t("set.drive.account.unknown"));
    await rlt.btn("set.drive.syncnow").click();
    await expect(page.getByText(rlt.t("set.drive.sync.nothing"))).toBeVisible();
  });

  test("pt-PT: cac chuoi moi hien dung ngon ngu", async ({ page, rlt }) => {
    await rlt.open({ locale: "pt-PT", drive: { state: "not_configured" } });
    await openDrive(page, rlt);
    await expect(rlt.btn("set.drive.connect")).toBeDisabled();
    await expect(page.getByText(rlt.t("set.drive.notconfigured.hint", {}, "pt-PT"))).toBeVisible();
    await expect(rlt.btn("set.drive.openadvanced")).toBeVisible();
  });
});
