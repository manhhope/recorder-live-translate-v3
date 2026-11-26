// US-63, US-64: Tom tat cuoc ghi + dich tom tat. Lop Tauri la gia (tauri-mock.ts), nen day KHONG goi nha cung cap that.
import { test, expect, type Rlt } from "./fixtures.ts";
import type { Page } from "@playwright/test";

const sumBtn = (rlt: Rlt) => rlt.btn("ses.summarize", rlt.page.locator(".tr2"));
const cfg = (page: Page, v: Record<string, unknown>) => page.evaluate(x => { (window as any).__rlt.sum = Object.assign((window as any).__rlt.sum ?? {}, x); }, v);

async function openDetail(rlt: Rlt, loc: "en" | "pt-PT", aiKeys: Record<string, boolean> = { gemini: true, openai: false }) {
  await rlt.open({ locale: loc, aiKeys });
  await rlt.nav("nav.sessions").click();
  await rlt.page.locator(".sesrow button").first().click();
}
// Lan dau dung mot nha cung cap: hop thoai dong y cua Hoi AI chan chay tom tat cho toi khi bam Dong y (AC-63.6).
async function agree(rlt: Rlt) {
  const card = rlt.page.getByTestId("sum-card");
  await expect(rlt.btn("sum.run", card)).toBeDisabled();
  await rlt.btn("ask.consent.ok").click();
  await expect(rlt.btn("sum.run", card)).toBeEnabled();
}

for (const loc of ["en", "pt-PT"] as const) {
  test.describe(`Tom tat cuoc ghi (${loc})`, () => {
    test("nut Tom tat canh nut Hoi AI; the hien uoc tinh truoc khi chay, chay ra du 3 phan bang van ban thuan", async ({ page, rlt }) => {
      await openDetail(rlt, loc);
      await expect(rlt.btn("nav.askai", page.locator(".tr2"))).toBeVisible();
      await sumBtn(rlt).click();
      const card = page.getByTestId("sum-card");
      await expect(card).toBeVisible();
      await expect(page.getByTestId("sum-estimate")).toContainText(loc === "en" ? "12,000" : "12");
      await expect(page.getByTestId("sum-estimate")).toContainText("1");
      expect(await rlt.calls("ai_summarize")).toHaveLength(0); // chua bam chay thi chua goi nha cung cap
      await agree(rlt);
      await rlt.btn("sum.run", card).click();
      await expect(page.getByTestId("sum-result")).toBeVisible();
      await expect(page.getByTestId("sum-result").getByRole("heading", { name: rlt.t("sum.h.summary") })).toBeVisible();
      await expect(page.getByTestId("sum-result").getByRole("heading", { name: rlt.t("sum.h.points") })).toBeVisible();
      await expect(page.getByTestId("sum-result").getByRole("heading", { name: rlt.t("sum.h.actions") })).toBeVisible();
      await expect(page.getByTestId("sum-points").locator("li")).toHaveCount(2);
      await expect(page.getByTestId("sum-actions").locator("li")).toHaveCount(1);
      // Van ban do mo hinh tra ve duoc hien nhu chu, khong thanh the HTML.
      await expect(page.getByTestId("sum-summary")).toContainText("<b>not bold</b>");
      await expect(card.locator("b", { hasText: "not bold" })).toHaveCount(0);
      await expect(page.getByTestId("sum-translated")).toHaveCount(0);
    });

    test("tran 200.000 ky tu va cat cac doan dau: co thong bao ro", async ({ page, rlt }) => {
      await openDetail(rlt, loc);
      await cfg(page, { plan: { total_chars: 450_000, sent_chars: 200_000, input_cut: true, chunks: 9, calls: 10, dropped_head: true } });
      await sumBtn(rlt).click();
      await expect(page.getByTestId("sum.cut.input")).toBeVisible();
      await expect(page.getByTestId("sum.cut.chunks")).toBeVisible();
    });

    test("huy giua chung: nut Huy dung lenh, khong hien ket qua gia, ghi ro da huy", async ({ page, rlt }) => {
      await openDetail(rlt, loc);
      await cfg(page, { hold: true });
      await sumBtn(rlt).click();
      const card = page.getByTestId("sum-card");
      await agree(rlt);
      await rlt.btn("sum.run", card).click();
      await expect(page.getByRole("status").filter({ hasText: /1/ }).first()).toBeVisible();
      await page.getByRole("button", { name: rlt.t("common.cancel"), exact: true }).click();
      await expect(card.getByRole("alert")).toContainText(rlt.t("sum.cancelled"));
      await expect(page.getByTestId("sum-result")).toHaveCount(0);
      expect((await rlt.calls("ai_cancel")).length).toBeGreaterThan(0);
      await expect(rlt.btn("sum.run", card)).toBeEnabled();
    });

    test("loi giua chung: bao ma loi va chunk bi loi, khong co ket qua gia", async ({ page, rlt }) => {
      await openDetail(rlt, loc);
      await cfg(page, { fail: "provider_unavailable", failAt: 3 });
      await sumBtn(rlt).click();
      const card = page.getByTestId("sum-card");
      await agree(rlt);
      await rlt.btn("sum.run", card).click();
      await expect(card.getByRole("alert")).toContainText(rlt.t("sum.partial", { i: 3 }));
      await expect(page.getByTestId("sum-result")).toHaveCount(0);
    });

    test("dich tom tat: tick o dich, ban dich co nhan do AI dich va khong dung ban dich cua Soniox", async ({ page, rlt }) => {
      await openDetail(rlt, loc);
      await sumBtn(rlt).click();
      const card = page.getByTestId("sum-card");
      const target = loc === "en" ? "pt-PT" : "en";
      await card.getByLabel(rlt.t(loc === "en" ? "sum.translate.pt" : "sum.translate.en")).check();
      await expect(page.getByTestId("sum-estimate")).toContainText("2"); // them mot luot goi
      await agree(rlt);
      await rlt.btn("sum.run", card).click();
      await expect(page.getByTestId("sum-translated")).toBeVisible();
      await expect(page.getByTestId("sum-ai-label")).toHaveText(rlt.t("sum.translated.label"));
      await expect(page.getByTestId("sum-translated").getByRole("heading", { level: 4 })).toHaveText(rlt.t(loc === "en" ? "sum.translated.title.pt" : "sum.translated.title.en"));
      const runs = await page.evaluate(() => (window as any).__rlt.sumRuns);
      expect(runs[0].translate_to).toBe(target);
      expect(runs[0].ui_lang).toBe(loc);
    });
  });
}

test("dung nha cung cap CLI: phai co dong y (hop thoai dong y co san) truoc khi chay duoc", async ({ page, rlt }) => {
  await rlt.open({ locale: "en", aiKeys: { gemini: false, openai: false } });
  await rlt.nav("nav.sessions").click();
  await page.locator(".sesrow button").first().click();
  await sumBtn(rlt).click();
  await page.getByLabel(rlt.t("ask.model.pick")).selectOption("claude_cli");
  const dialog = page.getByRole("alertdialog", { name: rlt.t("ai.consent.cli.title") });
  await expect(dialog).toBeVisible();
  await expect(rlt.btn("sum.run", page.getByTestId("sum-card"))).toBeDisabled();
  await dialog.getByLabel(rlt.t("ai.consent.cli.check")).check();
  await rlt.btn("ask.consent.ok").click();
  await expect(rlt.btn("sum.run", page.getByTestId("sum-card"))).toBeEnabled();
});
