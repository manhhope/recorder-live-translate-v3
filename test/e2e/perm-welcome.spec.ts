// Welcome: cap quyen. Hien o lan mo app khi con quyen bat buoc chua cap, tu cap nhat khi lay lai tieu diem, Continue/Skip/Dung hien lai,
// thu tu voi What's New, nhac dung quyen thieu khi bam Bat dau (toi da mot lan moi quyen moi lan mo app), en va pt-PT, cua so nho.
// Lop Tauri la gia: quyen macOS duoc mo phong qua seed `perms` va __rlt.setPerm; phan chi thu duoc tren may that: xem docs/onboarding-quyen-bao-cao.md.
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";
import { mkdirSync } from "node:fs";

const SHOT = process.env.RLT_SHOT_DIR;
const shot = async (page: Page, name: string) => { if (SHOT) { mkdirSync(SHOT, { recursive: true }); await page.screenshot({ path: `${SHOT}/${name}.png` }); } };
const dlg = (page: Page) => page.getByRole("dialog");
const row = (page: Page, k: "mic" | "screen" | "ax") => page.getByTestId("pw-" + k);
const NONE = { mic: "notDetermined", screen: "denied", ax: false } as const;
/** Mo phong "nguoi dung doi quyen trong Cai dat He thong roi quay lai app": doi quyen gia roi phat su kien focus. */
const back = (page: Page, p: Record<string, unknown>) => page.evaluate(v => { (window as any).__rlt.setPerm(v); window.dispatchEvent(new Event("focus")); }, p);
/** Doi buoc kiem quyen luc khoi dong (App.svelte, requestAnimationFrame -> evaluatePermWelcome) doc xong ca micro lan Accessibility. Neu test doi quyen truoc do thi lan kiem dau thay thieu quyen va tu mo Welcome, che nut Bat dau. */
const startupPermsRead = async (page: Page) => {
  await expect.poll(() => calls(page, "permissions_status")).toBeGreaterThan(0);
  await expect.poll(() => calls(page, "quick_ax_status")).toBeGreaterThan(0);
  await page.evaluate(() => new Promise<void>(r => setTimeout(r, 50)));
};
const calls = (page: Page, cmd: string) => page.evaluate(c => (window as any).__rlt.calls.filter((x: any) => x.cmd === c).length as number, cmd);

for (const loc of ["en", "pt-PT"] as const) {
  test(`lan mo dau thieu quyen: hien Welcome voi ba quyen, ly do, trang thai, Continue bam duoc ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc, perms: NONE });
    const d = dlg(page);
    await expect(d).toBeVisible();
    await expect(d).toContainText(rlt.t("pw.title"));
    for (const k of ["mic", "screen", "ax"] as const) {
      await expect(row(page, k)).toContainText(rlt.t(`pw.${k}.name`));
      await expect(row(page, k)).toContainText(rlt.t(`pw.${k}.why`));
      await expect(row(page, k)).toContainText(rlt.t("pw.s.notasked"));
    }
    await expect(row(page, "ax")).toContainText(rlt.t("pw.optional"));
    await expect(row(page, "mic")).not.toContainText(rlt.t("pw.optional"));
    await expect(rlt.btn("pw.continue", d)).toBeEnabled();
    await expect(rlt.btn("pw.continue", d)).toBeFocused();
    await expect(rlt.btn("pw.skip", d)).toBeEnabled();
    await expect(d.getByLabel(rlt.t("pw.dontshow"))).not.toBeChecked();
    await shot(page, `perm-welcome-${loc}`);
    await rlt.btn("pw.continue", d).click();
    await expect(d).toHaveCount(0);
  });
}

test("trang thai Allowed / Not asked / Denied hien dung theo tung quyen", async ({ page, rlt }) => {
  await rlt.open({ perms: { mic: "denied", screen: "denied", ax: true } });
  await expect(row(page, "mic")).toContainText(rlt.t("pw.s.denied"));
  await expect(row(page, "mic")).toContainText(rlt.t("pw.mic.denied"));
  await expect(row(page, "screen")).toContainText(rlt.t("pw.s.notasked")); // Ghi man hinh khong phan biet chua hoi va tu choi cho den khi mo Cai dat
  await expect(row(page, "ax")).toContainText(rlt.t("pw.s.allowed"));
  await expect(row(page, "ax").getByRole("button")).toHaveCount(0);
  await shot(page, "perm-welcome-states-en");
});

test("Microphone: Allow goi hop thoai xin quyen he thong roi doi thanh Allowed", async ({ page, rlt }) => {
  await rlt.open({ perms: { mic: "notDetermined", screen: "granted", ax: true, micAnswer: "authorized" } });
  await expect(row(page, "mic")).toContainText(rlt.t("pw.s.notasked"));
  await rlt.btn("pw.mic.allow", row(page, "mic")).click();
  await expect(row(page, "mic")).toContainText(rlt.t("pw.s.allowed"));
  expect(await calls(page, "mic_permission_request")).toBe(1);
  expect(await calls(page, "open_system_settings")).toBe(0);
});

test("Microphone bi tu choi tu hop thoai: thanh Denied, nut mo Cai dat dung muc microphone", async ({ page, rlt }) => {
  await rlt.open({ perms: { mic: "notDetermined", screen: "granted", ax: true, micAnswer: "denied" } });
  await rlt.btn("pw.mic.allow", row(page, "mic")).click();
  await expect(row(page, "mic")).toContainText(rlt.t("pw.s.denied"));
  await rlt.btn("set.perm.opensys", row(page, "mic")).click();
  expect(await page.evaluate(() => (window as any).__rlt.calls.filter((c: any) => c.cmd === "open_system_settings").map((c: any) => c.args.pane))).toEqual(["microphone"]);
});

test("Screen Recording: mo dung muc, quay lai app van thieu thi bao can khoi dong lai, nut Restart goi restart_app, khong bat hinh", async ({ page, rlt }) => {
  await rlt.open({ perms: { mic: "authorized", screen: "denied", screenProbe: "needsrestart", ax: true } });
  const r = row(page, "screen");
  await expect(r).toContainText(rlt.t("pw.s.notasked"));
  expect(await calls(page, "screen_permission_check")).toBe(0); // chua mo Cai dat thi khong do
  await rlt.btn("set.perm.opensys", r).click();
  expect(await page.evaluate(() => (window as any).__rlt.calls.filter((c: any) => c.cmd === "open_system_settings").map((c: any) => c.args.pane))).toEqual(["screen_recording"]);
  await back(page, {}); // da bat cong tac nhung tien trinh chua nhan
  await expect(r).toContainText(rlt.t("pw.s.needsrestart"));
  await expect(r).toContainText(rlt.t("pw.screen.restart"));
  expect(await calls(page, "screen_permission_check")).toBeGreaterThan(0);
  await rlt.btn("set.perm.restart", r).click();
  expect(await calls(page, "restart_app")).toBe(1);
  // Khong co lenh bat hinh hay chup that nao duoc goi (fixture cung bat lenh la).
  expect(await page.evaluate(() => (window as any).__rlt.calls.map((c: any) => c.cmd).filter((c: string) => /capture|snip|region/.test(c)))).toEqual([]);
  await shot(page, "perm-welcome-needsrestart-en");
});

test("Screen Recording: sau khi mo Cai dat ma van tu choi thi la Denied", async ({ page, rlt }) => {
  await rlt.open({ perms: { mic: "authorized", screen: "denied", screenProbe: "denied", ax: true } });
  await rlt.btn("set.perm.opensys", row(page, "screen")).click();
  await back(page, {});
  await expect(row(page, "screen")).toContainText(rlt.t("pw.s.denied"));
});

test("tu cap nhat khi app lay lai tieu diem (khong can bam gi)", async ({ page, rlt }) => {
  await rlt.open({ perms: NONE });
  await expect(row(page, "mic")).toContainText(rlt.t("pw.s.notasked"));
  await back(page, { mic: "authorized", screen: "granted", ax: true });
  for (const k of ["mic", "screen", "ax"] as const) await expect(row(page, k)).toContainText(rlt.t("pw.s.allowed"));
  await expect(rlt.btn("pw.continue", dlg(page))).toBeEnabled();
});

test("Accessibility: nut mo dung muc accessibility", async ({ page, rlt }) => {
  await rlt.open({ perms: { mic: "authorized", screen: "granted", ax: false }, whatsNewSeen: "999.0.0" });
  await expect(dlg(page)).toHaveCount(0); // chi thieu quyen tuy chon thi khong tu hien
  await rlt.nav("nav.settings").click();
  await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.permissions") }).click();
  await page.getByTestId("perm-welcome-open").click();
  await expect(row(page, "ax")).toContainText(rlt.t("pw.s.notasked"));
  await rlt.btn("set.perm.opensys", row(page, "ax")).click();
  expect(await page.evaluate(() => (window as any).__rlt.calls.filter((c: any) => c.cmd === "open_system_settings").map((c: any) => c.args.pane))).toEqual(["accessibility"]);
});

test("du quyen bat buoc: khong hien Welcome o lan mo app", async ({ page, rlt }) => {
  await rlt.open({ perms: { mic: "authorized", screen: "granted", ax: false } });
  await page.waitForTimeout(1000);
  await expect(dlg(page)).toHaveCount(0);
});

test("da chon Dung hien lai: khong hien du con thieu quyen", async ({ page, rlt }) => {
  await rlt.open({ perms: NONE, permWelcomeOff: true });
  await page.waitForTimeout(1000);
  await expect(dlg(page)).toHaveCount(0);
});

test("Dung hien lai + Continue: luu lai, mo lai app khong hien; Skip for now khong luu nen lan sau van hien", async ({ page, rlt }) => {
  await rlt.open({ perms: NONE });
  await dlg(page).getByLabel(rlt.t("pw.dontshow")).check();
  await rlt.btn("pw.continue", dlg(page)).click();
  await expect(dlg(page)).toHaveCount(0);
  expect(await page.evaluate(() => localStorage.getItem("rlt.ui.permWelcomeOff"))).toBe("1");
  await page.reload();
  await expect(rlt.nav("nav.record")).toBeVisible();
  await page.waitForTimeout(1000);
  await expect(dlg(page)).toHaveCount(0);
  // Bo o: xoa khoa, lan sau hien lai
  await page.evaluate(() => localStorage.removeItem("rlt.ui.permWelcomeOff"));
  await page.reload();
  await expect(dlg(page)).toBeVisible();
  await rlt.btn("pw.skip", dlg(page)).click();
  await expect(dlg(page)).toHaveCount(0);
  expect(await page.evaluate(() => localStorage.getItem("rlt.ui.permWelcomeOff"))).toBeNull();
  await page.reload();
  await expect(dlg(page)).toBeVisible();
});

test("Esc va X dong Welcome", async ({ page, rlt }) => {
  await rlt.open({ perms: NONE });
  await page.keyboard.press("Escape");
  await expect(dlg(page)).toHaveCount(0);
  await page.reload();
  await expect(dlg(page)).toBeVisible();
  await dlg(page).getByRole("button", { name: rlt.t("pw.close") }).click();
  await expect(dlg(page)).toHaveCount(0);
});

test("Tab xoay vong trong hop thoai; bam ra ngoai khong dong", async ({ page, rlt }) => {
  await rlt.open({ perms: NONE });
  await page.mouse.click(5, 5);
  await expect(dlg(page)).toBeVisible();
  await page.keyboard.press("Tab");
  await expect(dlg(page).getByRole("button", { name: rlt.t("pw.close") })).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(rlt.btn("pw.continue", dlg(page))).toBeFocused();
});

test("thu tu voi What's New: Welcome truoc, What's New sau khi dong, khong bao gio chong hai hop thoai", async ({ page, rlt }) => {
  await rlt.open({ perms: NONE, whatsNewSeen: "0.0.1" });
  await expect(dlg(page)).toHaveCount(1);
  await expect(dlg(page)).toContainText(rlt.t("pw.title"));
  await page.waitForTimeout(1500);
  await expect(dlg(page)).toHaveCount(1); // What's New van cho
  await expect(dlg(page)).toContainText(rlt.t("pw.title"));
  await rlt.btn("pw.continue", dlg(page)).click();
  await expect(dlg(page)).toHaveCount(1);
  await expect(dlg(page)).toContainText(rlt.t("wn.title"));
  await shot(page, "perm-welcome-then-whatsnew-en");
});

test("Skip for now cung nhuong cho What's New; du quyen thi chi co What's New", async ({ page, rlt }) => {
  await rlt.open({ perms: NONE, whatsNewSeen: "0.0.1" });
  await rlt.btn("pw.skip", dlg(page)).click();
  await expect(dlg(page)).toContainText(rlt.t("wn.title"));
});

test("du quyen: chi hien What's New nhu cu", async ({ page, rlt }) => {
  await rlt.open({ whatsNewSeen: "0.0.1" });
  await expect(dlg(page)).toContainText(rlt.t("wn.title"));
  await expect(dlg(page)).toHaveCount(1);
});

test("bam Bat dau thieu quyen man hinh: nhac dung quyen thieu mot lan, lan sau chi con thong bao loi va nut xem lai", async ({ page, rlt }) => {
  await rlt.open({ fail: { session_start: "permission_screen" } });
  await expect(dlg(page)).toHaveCount(0);
  await back(page, { screen: "denied" }); // quyen bi thu hoi sau khi app da mo
  await rlt.btn("tool.start").click();
  await expect(dlg(page)).toBeVisible();
  await expect(row(page, "screen")).toHaveClass(/hot/);
  await expect(row(page, "screen")).toContainText(rlt.t("pw.missing"));
  await expect(row(page, "mic")).not.toHaveClass(/hot/);
  await rlt.btn("pw.skip", dlg(page)).click();
  await expect(page.getByRole("alert")).toContainText(rlt.t("err.permissionscreen"));
  await rlt.btn("tool.start").click(); // lan hai: khong nhac nua
  await page.waitForTimeout(500);
  await expect(dlg(page)).toHaveCount(0);
  await expect(page.getByRole("alert")).toContainText(rlt.t("err.permissionscreen"));
  await page.getByTestId("perm-review").click(); // mo lai tay van duoc
  await expect(row(page, "screen")).toHaveClass(/hot/);
});

test("bam Bat dau thieu micro: nhac dung quyen micro; thieu quyen khac van nhac rieng", async ({ page, rlt }) => {
  await rlt.open({ fail: { session_start: "permission_mic" } });
  await startupPermsRead(page);
  await back(page, { mic: "denied" });
  await rlt.btn("tool.start").click();
  await expect(row(page, "mic")).toHaveClass(/hot/);
  await expect(row(page, "screen")).not.toHaveClass(/hot/);
  await page.keyboard.press("Escape");
  await rlt.setFail("session_start", "permission_screen");
  await rlt.btn("tool.start").click();
  await expect(row(page, "screen")).toHaveClass(/hot/); // micro da nhac, Ghi man hinh la quyen khac nen duoc nhac
});

test("da hien Welcome luc mo app roi Skip: bam Bat dau khong nhac lai quyen do (thong nhat voi mot lan moi quyen)", async ({ page, rlt }) => {
  await rlt.open({ perms: NONE, fail: { session_start: "permission_screen" } });
  await rlt.btn("pw.skip", dlg(page)).click();
  await rlt.btn("tool.start").click();
  await page.waitForTimeout(500);
  await expect(dlg(page)).toHaveCount(0);
  await expect(page.getByRole("alert")).toContainText(rlt.t("err.permissionscreen"));
});

test("Dung hien lai thi bam Bat dau thieu quyen khong mo Welcome, van co thong bao loi", async ({ page, rlt }) => {
  await rlt.open({ permWelcomeOff: true, fail: { session_start: "permission_mic" } });
  await rlt.btn("tool.start").click();
  await page.waitForTimeout(500);
  await expect(dlg(page)).toHaveCount(0);
  await expect(page.getByRole("alert")).toContainText(rlt.t("err.permissionmic"));
});

for (const loc of ["en", "pt-PT"] as const) {
  test(`mo lai thu cong tu Cai dat > Quyen khi du quyen: ba quyen deu Allowed ${loc}`, async ({ page, rlt }) => {
    await rlt.open({ locale: loc });
    await rlt.nav("nav.settings").click();
    await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t("set.tab.permissions") }).click();
    await page.getByTestId("perm-welcome-open").click();
    await expect(dlg(page)).toContainText(rlt.t("pw.title"));
    for (const k of ["mic", "screen", "ax"] as const) await expect(row(page, k)).toContainText(rlt.t("pw.s.allowed"));
    await expect(rlt.btn("pw.continue", dlg(page))).toBeEnabled();
    await page.keyboard.press("Escape");
    await expect(dlg(page)).toHaveCount(0);
  });
}

for (const loc of ["en", "pt-PT"] as const) {
  test(`cua so nho 720x520: noi dung cuon, Continue, Skip va X khong bi cat ${loc}`, async ({ page, rlt }) => {
    await page.setViewportSize({ width: 720, height: 520 });
    await rlt.open({ locale: loc, perms: { mic: "denied", screen: "denied", screenProbe: "needsrestart", ax: false } });
    const d = dlg(page);
    await expect(d).toBeVisible();
    await rlt.btn("set.perm.opensys", row(page, "screen")).click();
    await back(page, {});
    for (const b of [rlt.btn("pw.continue", d), rlt.btn("pw.skip", d), d.getByRole("button", { name: rlt.t("pw.close") }), d.getByLabel(rlt.t("pw.dontshow"))]) {
      const r = (await b.boundingBox())!;
      expect(r.y).toBeGreaterThanOrEqual(0);
      expect(r.y + r.height).toBeLessThanOrEqual(520);
      expect(r.x).toBeGreaterThanOrEqual(0);
      expect(r.x + r.width).toBeLessThanOrEqual(720);
    }
    const m = await d.locator(".wn-b").evaluate(e => ({ ov: getComputedStyle(e).overflowY, sw: e.scrollWidth, cw: e.clientWidth }));
    expect(m.ov).toBe("auto");
    expect(m.sw).toBeLessThanOrEqual(m.cw + 1); // khong tran ngang
    const box = (await d.boundingBox())!;
    expect(box.y + box.height).toBeLessThanOrEqual(520);
    await shot(page, `perm-welcome-small-${loc}`);
  });
}
