import { test, expect } from "./fixtures.ts";

test.describe("Doi ngon ngu giao dien en <-> pt-PT", () => {
  test("lan dau mo la pt-PT, doi sang en roi quay lai, nho lua chon sau khi tai lai", async ({ page, rlt }) => {
    await rlt.open({ locale: "pt-PT" });
    const toggle = page.getByRole("button", { name: rlt.t("ui.lang.to.en", {}, "pt-PT") });
    await expect(toggle).toHaveText(/PT/);
    await expect(rlt.btn("tool.start")).toHaveText(new RegExp(rlt.t("tool.start", {}, "pt-PT")));
    await expect(page.getByText(rlt.t("rec.empty.title", {}, "pt-PT"))).toBeVisible();
    await expect(rlt.nav("nav.record")).toContainText(rlt.t("nav.record", {}, "pt-PT"));
    await expect(page.locator("html")).toHaveAttribute("lang", /pt/);

    await toggle.click();
    await expect(page.getByText(rlt.t("rec.empty.title", {}, "en"))).toBeVisible();
    await expect(page.getByRole("button", { name: rlt.t("ui.lang.to.pt", {}, "en") })).toHaveText(/EN/);
    await expect(page.locator("html")).toHaveAttribute("lang", "en");
    expect(await page.evaluate(() => localStorage.getItem("rlt.ui.locale"))).toBe("en");

    await page.reload();
    await expect(page.getByText(rlt.t("rec.empty.title", {}, "en"))).toBeVisible();

    await page.getByRole("button", { name: rlt.t("ui.lang.to.pt", {}, "en") }).click();
    await expect(page.getByText(rlt.t("rec.empty.title", {}, "pt-PT"))).toBeVisible();
  });

  test("doi ngon ngu o moi man: Ghi, Phien, Cai dat (Usage + AI) deu doi het chu", async ({ page, rlt }) => {
    await rlt.open({ locale: "en" });
    const switchTo = async (to: "en" | "pt-PT") => {
      const from = to === "en" ? "pt-PT" : "en";
      await page.getByRole("button", { name: rlt.t(from === "en" ? "ui.lang.to.pt" : "ui.lang.to.en", {}, from) }).click();
      rlt.locale = to;
      await expect(page.getByRole("button", { name: rlt.t(to === "en" ? "ui.lang.to.pt" : "ui.lang.to.en", {}, to) })).toBeVisible();
    };
    await switchTo("pt-PT");

    await rlt.nav("nav.sessions").click();
    await expect(page.getByRole("searchbox", { name: rlt.t("ses.search") })).toBeVisible();
    await expect(page.locator(".sesrow .bd.ok").first()).toContainText(rlt.t("ses.saved"));
    await expect(page.locator(".sesrow").first()).toContainText(rlt.t("ses.lines", { n: 3 }));

    await rlt.nav("nav.settings").click();
    await page.getByRole("button", { name: rlt.t("set.tab.sonioxusage") }).click();
    await expect(page.getByText(rlt.t("set.soniox.title"))).toBeVisible();
    await expect(page.getByRole("heading", { name: rlt.t("set.tab.usagelogs") })).toBeVisible();
    await page.getByRole("button", { name: rlt.t("set.tab.translate") }).click();
    await expect(page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true })).toBeVisible();

    await switchTo("en");
    await expect(page.getByRole("button", { name: rlt.t("ai.settings.save"), exact: true })).toBeVisible();
    await rlt.nav("nav.record").click();
    await expect(rlt.btn("tool.start")).toBeVisible();
  });

  test("moi khoa giao dien dung o man Ghi cua pt-PT khong bi nguyen khoa (khong thieu khoa)", async ({ page, rlt }) => {
    await rlt.open({ locale: "pt-PT" });
    await rlt.startRecording();
    await rlt.emit({ kind: "status", code: "network" });
    await expect(page.locator(".foot .sb")).toContainText(rlt.t("status.offline", {}, "pt-PT"));
    await page.getByRole("button", { name: rlt.t("tool.pause", {}, "pt-PT") }).click();
    await expect(page.getByRole("button", { name: rlt.t("tool.resume", {}, "pt-PT") })).toBeVisible();
    // Khong khoa nao hien ra dang "ten.khoa.tho" tren man hinh.
    const body = await page.locator("body").innerText();
    expect(body).not.toMatch(/\b(?:tool|status|rec|cfg|nav|ses|set|src|ln)\.[a-z0-9.]+\b/);
  });

  test("thong bao loi theo ngon ngu pt-PT", async ({ page, rlt }) => {
    await rlt.open({ locale: "pt-PT", fail: { session_start: "auth" } });
    await rlt.btn("tool.start").click();
    await expect(page.getByRole("alert")).toContainText(rlt.t("err.auth", {}, "pt-PT"));
  });
});
