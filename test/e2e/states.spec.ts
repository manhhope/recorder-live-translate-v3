import { test, expect, esc } from "./fixtures.ts";

// Bon trang thai loi theo thu tu uu tien cua thiet ke: Thieu quyen > Khoa sai > Het quota > Mat mang.
const STATES = [
  { code: "permission", key: "status.perm", err: "err.permission" },
  { code: "auth", key: "status.badkey", err: "err.auth" },
  { code: "quota", key: "status.quota", err: "err.quota" },
  { code: "network", key: "status.offline", err: "err.network" },
] as const;

test.describe("Trang thai loi tren thanh trang thai khi dang ghi", () => {
  for (const s of STATES) {
    test(`${s.code}: hien nhan "${s.key}" va het khi trang thai ve null`, async ({ page, rlt }) => {
      await rlt.open();
      await rlt.startRecording();
      const foot = page.locator(".foot .sb");
      await rlt.emit({ kind: "status", code: s.code });
      await expect(foot).toContainText(rlt.t(s.key));
      await expect(foot).not.toHaveClass(/ok/);
      await rlt.emit({ kind: "status", code: null });
      await expect(foot).not.toContainText(rlt.t(s.key));
    });
  }

  test("thieu quyen: nguon He thong chuyen sang trang thai khong co tin hieu", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await rlt.emit({ kind: "status", code: "permission" });
    await expect(page.locator(".foot .sb")).toContainText(rlt.t("status.perm"));
    // Nut nguon he thong dung nhan trang thai "none" (x), khong con la "quiet".
    const sys = page.getByRole("group", { name: rlt.t("cfg.audio.group") }).getByRole("button").first();
    await expect(sys).toHaveAttribute("aria-label", new RegExp(esc(rlt.t("src.state.none"))));
  });
});

test.describe("Bat dau that bai: thong bao loi ro rang, khong mat du lieu", () => {
  const FAILS = [
    { code: "auth", err: "err.auth" },
    { code: "quota", err: "err.quota" },
    { code: "network", err: "err.network" },
    { code: "permission", err: "err.permission" },
  ] as const;
  for (const f of FAILS) {
    test(`session_start tra "${f.code}"`, async ({ page, rlt }) => {
      await rlt.open({ fail: { session_start: f.code } });
      await rlt.btn("tool.start").click();
      await expect(page.getByRole("alert")).toContainText(rlt.t(f.err));
      // Van o trang thai cho: nut Bat dau dung lai duoc, khong co nut Dung.
      await expect(rlt.btn("tool.start")).toBeEnabled();
      await expect(rlt.btn("tool.stop")).toHaveCount(0);
    });
  }

  test("ma loi la thi dung thong bao chung", async ({ page, rlt }) => {
    // Loi san pham P-02: errKeyOf doi khoa bang $_() nen moi ma loi la sinh ra canh bao "[i18n] missing key".
    rlt.allowConsole.push("[i18n] missing key: err.somethingnew");
    await rlt.open({ fail: { session_start: "something_new" } });
    await rlt.btn("tool.start").click();
    await expect(page.getByRole("alert")).toContainText(rlt.t("err.generic"));
  });

  test("sau khi loi, bo loi roi bat dau lai thanh cong va xoa thong bao", async ({ page, rlt }) => {
    await rlt.open({ fail: { session_start: "network" } });
    await rlt.btn("tool.start").click();
    await expect(page.getByRole("alert")).toContainText(rlt.t("err.network"));
    await rlt.setFail("session_start", null);
    await rlt.btn("tool.start").click();
    await expect(rlt.btn("tool.stop")).toBeVisible();
    await expect(page.getByRole("alert")).toHaveCount(0);
  });
});

test.describe("Trang thai loi o Phien va Muc dung", () => {
  test("danh sach phien loi mang: hien loi va Thu lai khoi phuc", async ({ page, rlt }) => {
    await rlt.open({ fail: { sessions_list: "network" } });
    await rlt.nav("nav.sessions").click();
    await expect(page.getByText(rlt.t("err.network"))).toBeVisible();
    await rlt.setFail("sessions_list", null);
    await page.getByRole("button", { name: rlt.t("common.retry") }).click();
    await expect(page.getByText("Weekly product meeting")).toBeVisible();
  });

  test("Muc dung loi: Thu lai khoi phuc", async ({ page, rlt }) => {
    await rlt.open({ fail: { usage_week_source: "store_error" } });
    await rlt.nav("nav.settings").click();
    await page.getByRole("button", { name: rlt.t("set.tab.sonioxusage") }).click();
    await expect(page.getByText(rlt.t("err.storeerror"))).toBeVisible();
    await rlt.setFail("usage_week_source", null);
    await page.getByRole("button", { name: rlt.t("common.retry") }).click();
    await expect(page.getByRole("heading", { name: rlt.t("set.tab.usagelogs") })).toBeVisible();
  });

  test("Thu ket noi Soniox: khoa sai, het quota, mat mang", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.nav("nav.settings").click();
    await page.getByRole("button", { name: rlt.t("set.tab.sonioxusage") }).click();
    for (const [code, key] of [["auth", "err.auth"], ["quota", "err.quota"], ["network", "err.network"]] as const) {
      await rlt.setFail("soniox_test_connection", code);
      await page.getByRole("button", { name: rlt.t("set.test") }).click();
      await expect(page.locator(".bd.rec:not([data-testid])")).toContainText(rlt.t(key));
    }
    await rlt.setFail("soniox_test_connection", null);
    await page.getByRole("button", { name: rlt.t("set.test") }).click();
    await expect(page.locator(".bd.ok:not([data-testid])")).toContainText(rlt.t("set.soniox.ok"));
  });
});
