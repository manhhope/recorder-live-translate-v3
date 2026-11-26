// Chup anh ra soat UX (khong phai test san pham, mac dinh BO QUA). Chay: UX_SHOTS=1 [UX_SHOTS_DIR=img-sau] npx playwright test test/e2e/ux-shots.spec.ts
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";
import { mkdirSync, appendFileSync } from "node:fs";
import { resolve } from "node:path";

const OUT = process.env.UX_SHOTS_DIR ?? "img";
const ROOT = resolve(import.meta.dirname, "../../docs/ux-review", OUT);
const LOG = resolve(import.meta.dirname, "../../docs/ux-review", `shots-log-${OUT}.txt`);
test.skip(!process.env.UX_SHOTS, "Cong cu chup anh, chi chay khi dat UX_SHOTS=1");
const NOT_INST = { path: null, version: null, status: "not_installed" };
const MISSING = { claude_cli: NOT_INST, codex_cli: NOT_INST, antigravity_cli: NOT_INST };

async function shot(page: Page, dir: string, name: string) {
  mkdirSync(`${ROOT}/${dir}`, { recursive: true });
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: `${ROOT}/${dir}/${name}.png`, animations: "disabled", caret: "hide" });
  const ov = await page.evaluate(() => ({ x: document.documentElement.scrollWidth - window.innerWidth,
    clipped: [...document.querySelectorAll<HTMLElement>("button,span,b,label,small,h3,h4,option")].filter(e => e.offsetParent && e.scrollWidth > e.clientWidth + 1 && getComputedStyle(e).overflow !== "visible" && e.clientWidth > 0).map(e => (e.textContent || "").trim().slice(0, 40)).slice(0, 6) }));
  appendFileSync(LOG, `${dir}/${name}\toverflowX=${ov.x}\tclipped=${JSON.stringify(ov.clipped)}\n`);
}
const settingsTab = async (page: Page, rlt: any, key: string) => {
  await rlt.nav("nav.settings").click();
  await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t(key) }).click();
  await page.waitForTimeout(250);
};
const step = async (label: string, dir: string, fn: () => Promise<void>) => {
  try { await fn(); } catch (e) { appendFileSync(LOG, `FAIL ${dir}/${label}: ${String(e).split("\n")[0]}\n`); }
};

const COMBOS = [
  { loc: "en", scheme: "light" }, { loc: "pt-PT", scheme: "light" }, { loc: "en", scheme: "dark" }, { loc: "pt-PT", scheme: "dark" },
] as const;
const SIZES = [{ n: "", w: 1100, h: 760 }, { n: "small-", w: 720, h: 520 }] as const;

for (const c of COMBOS) for (const sz of SIZES) {
  if (sz.n && c.scheme === "dark") continue;
  const dir = `${c.loc}-${c.scheme}${sz.n ? "-720" : ""}`;
  const N = (s: string) => s;
  test.describe(dir, () => {
    test.use({ viewport: { width: sz.w, height: sz.h }, colorScheme: c.scheme });

    test("ghi", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc });
      await step("idle", dir, async () => shot(page, dir, "01-ghi-cho"));
      await step("start", dir, async () => {
        await rlt.startRecording();
        await rlt.emit(rlt.line(1, 1, "Let us review the roadmap for next quarter.", "Vamos rever o roteiro do próximo trimestre."));
        await rlt.emit(rlt.line(2, 2, "The pricing page ships on Friday, and the new onboarding flow goes out the week after that.", "A página de preços sai na sexta-feira, e o novo fluxo de integração sai na semana seguinte."));
        await rlt.emit(rlt.line(3, 3, "Any blockers from the design team?", null));
        await rlt.emit({ kind: "level", level: 0.4 });
        await page.waitForTimeout(300);
        await shot(page, dir, "02-ghi-dang-ghi");
      });
      await step("pause", dir, async () => { await rlt.btn("tool.pause").click(); await page.waitForTimeout(200); await shot(page, dir, "03-ghi-tam-dung"); await rlt.btn("tool.resume").click(); });
      for (const s of ["permission", "auth", "quota", "network", "rejected"]) {
        await step(s, dir, async () => { await rlt.emit({ kind: "status", code: s }); await page.waitForTimeout(150); await shot(page, dir, `04-ghi-loi-${s}`); });
      }
      await step("clear", dir, async () => rlt.emit({ kind: "status", code: null }));
      await step("stop", dir, async () => { await rlt.btn("tool.stop").click(); await page.waitForTimeout(400); await shot(page, dir, "05-ghi-da-dung"); });
      await step("view", dir, async () => { await page.getByRole("button", { name: rlt.t("rec.view.dst"), exact: true }).click(); await shot(page, dir, "06-ghi-chi-ban-dich"); });
    });

    test("ghi-truoc-khi-bat-dau", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, hasKey: false });
      await step("nokey", dir, async () => { await rlt.btn("tool.start").click(); await page.waitForTimeout(250); await shot(page, dir, "07-ghi-chua-co-khoa-soniox"); });
    });
    test("ghi-consent", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, consent: false });
      await step("consent", dir, async () => { await rlt.btn("tool.start").click(); await page.waitForTimeout(250); await shot(page, dir, "08-ghi-dong-y-gui-du-lieu"); });
    });
    test("ghi-khong-nguon", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc });
      await step("nosrc", dir, async () => {
        const g = page.getByRole("group", { name: rlt.t("cfg.audio.group") }).getByRole("button");
        await g.nth(0).click(); await g.nth(1).click(); await page.waitForTimeout(200); await shot(page, dir, "09-ghi-khong-chon-nguon");
      });
    });
    test("ghi-start-loi", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, fail: { session_start: "permission_mic" } });
      await step("startfail", dir, async () => { await rlt.btn("tool.start").click(); await page.waitForTimeout(300); await shot(page, dir, "10-ghi-bat-dau-that-bai-quyen-mic"); });
    });
    test("menu-ngon-ngu", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc });
      await step("lang", dir, async () => { await page.getByRole("combobox").or(page.locator(".sel")).first().click(); await page.waitForTimeout(250); await shot(page, dir, "11-ghi-menu-ngon-ngu"); });
    });

    test("first-run", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, whatsNewSeen: null, perms: { mic: "notDetermined", screen: "denied", ax: false } });
      await step("pw", dir, async () => { await page.waitForTimeout(500); await shot(page, dir, "12-lan-dau-cap-quyen"); });
      await step("pw-denied", dir, async () => { await rlt.page.evaluate(() => { (window as any).__rlt.setPerm({ mic: "denied" }); window.dispatchEvent(new Event("focus")); }); await page.waitForTimeout(300); await shot(page, dir, "13-cap-quyen-mic-bi-tu-choi"); });
    });
    test("whats-new", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, whatsNewSeen: "0.0.1" });
      await step("wn", dir, async () => { await page.waitForTimeout(400); await shot(page, dir, "14-whats-new"); });
    });

    test("phien", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc });
      await rlt.nav("nav.sessions").click();
      await step("list", dir, async () => { await expect(page.locator(".sesrow")).toHaveCount(3); await shot(page, dir, "20-phien-danh-sach"); });
      await step("search", dir, async () => { await page.getByRole("searchbox", { name: rlt.t("ses.search") }).fill("pricing"); await page.waitForTimeout(400); await shot(page, dir, "21-phien-tim-co-ket-qua"); });
      await step("search0", dir, async () => { await page.getByRole("searchbox", { name: rlt.t("ses.search") }).fill("zzzz"); await page.waitForTimeout(400); await shot(page, dir, "22-phien-tim-khong-thay"); await page.getByRole("searchbox", { name: rlt.t("ses.search") }).fill(""); });
      await step("delete", dir, async () => { await page.locator(".sesrow").nth(1).getByRole("button", { name: rlt.t("common.delete") }).click(); await page.waitForTimeout(200); await shot(page, dir, "23-phien-xac-nhan-xoa"); await page.keyboard.press("Escape"); });
      await step("detail", dir, async () => { await page.reload(); await rlt.nav("nav.sessions").click(); await page.getByRole("button", { name: "Weekly product meeting" }).click(); await page.waitForTimeout(300); await shot(page, dir, "24-phien-chi-tiet"); });
      await step("export", dir, async () => { await page.getByRole("button", { name: rlt.t("exp.open") }).click(); await page.waitForTimeout(250); await shot(page, dir, "25-phien-xuat-file"); });
    });
    test("phien-rong-va-loi", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, sessions: [], details: {} });
      await rlt.nav("nav.sessions").click();
      await step("empty", dir, async () => { await page.waitForTimeout(300); await shot(page, dir, "26-phien-rong"); });
    });
    test("phien-khong-dong", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc });
      await rlt.nav("nav.sessions").click();
      await step("nolines", dir, async () => { await page.getByRole("button", { name: "Retro draft" }).click(); await page.waitForTimeout(300); await shot(page, dir, "27-phien-loi-khong-co-dong"); });
    });
    test("phien-load-loi", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, fail: { records_list: "storage", sessions_list: "storage", session_list: "storage" } });
      await rlt.nav("nav.sessions").click();
      await step("loaderr", dir, async () => { await page.waitForTimeout(400); await shot(page, dir, "28-phien-loi-tai-danh-sach"); });
    });

    for (const [i, t] of ["general", "translate", "shortcuts", "glossary", "subtitle", "sonioxusage", "drive", "permissions"].entries()) {
      test("cai-dat-" + t, async ({ page, rlt }) => {
        await rlt.open({ locale: c.loc, aiKeys: { gemini: true, openai: false }, drive: { state: "disconnected", builtin: true } });
        await step(t, dir, async () => { await settingsTab(page, rlt, "set.tab." + t); await shot(page, dir, `3${i}-cai-dat-${t}`); });
      });
    }

    test("hoi-ai-tab", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc });
      await step("detect", dir, async () => { await settingsTab(page, rlt, "set.tab.translate"); await shot(page, dir, "40-ai-tab-cli-phat-hien"); await page.getByTestId("cli-card-antigravity_cli").scrollIntoViewIfNeeded(); await shot(page, dir, "41-ai-tab-agy-chua-cai"); });
      await step("full", dir, async () => { await page.screenshot({ path: `${ROOT}/${dir}/42-ai-tab-toan-trang.png`, fullPage: true }); });
      await step("claude-login", dir, async () => {
        const cl = page.getByTestId("cli-card-claude_cli"); await cl.scrollIntoViewIfNeeded();
        await cl.getByRole("button", { name: rlt.t("set.login.btn"), exact: true }).click(); await page.waitForTimeout(600); await shot(page, dir, "43-ai-tab-claude-dang-nhap-link");
        await cl.getByRole("button", { name: rlt.t("set.login.cancel") }).click(); await page.waitForTimeout(300); await shot(page, dir, "44-ai-tab-claude-da-huy");
      });
      await step("codex-login", dir, async () => {
        const cx = page.getByTestId("cli-card-codex_cli"); await cx.scrollIntoViewIfNeeded();
        await cx.getByRole("button", { name: rlt.t("set.login.btn"), exact: true }).click(); await page.waitForTimeout(600); await shot(page, dir, "45-ai-tab-codex-ma-thiet-bi");
      });
      await step("adv", dir, async () => { await page.getByTestId("cli-card-claude_cli").getByText(rlt.t("set.cli.adv")).click(); await page.waitForTimeout(200); await shot(page, dir, "46-ai-tab-nang-cao-mo"); });
    });
    test("hoi-ai-tab-cai-cli", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, cli: MISSING } as never);
      await step("miss", dir, async () => { await settingsTab(page, rlt, "set.tab.translate"); await page.screenshot({ path: `${ROOT}/${dir}/47-ai-tab-3-cli-chua-cai-toan-trang.png`, fullPage: true }); await shot(page, dir, "47b-ai-tab-3-cli-chua-cai"); });
      await step("confirm", dir, async () => { const x = page.getByTestId("cli-card-codex_cli"); await x.scrollIntoViewIfNeeded(); await x.getByTestId("install-run-codex_cli").click(); await page.waitForTimeout(300); await shot(page, dir, "48-ai-tab-cai-codex-xac-nhan"); });
      await step("running", dir, async () => { await page.getByTestId("cli-card-codex_cli").getByTestId("install-confirm-ok-codex_cli").click(); await page.waitForTimeout(500); await shot(page, dir, "49-ai-tab-cai-codex-dang-chay"); });
    });
    test("hoi-ai-tab-cai-loi", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, cli: MISSING, installFail: true } as never);
      await step("fail", dir, async () => {
        await settingsTab(page, rlt, "set.tab.translate");
        const x = page.getByTestId("cli-card-codex_cli"); await x.scrollIntoViewIfNeeded();
        await x.getByTestId("install-run-codex_cli").click(); await x.getByTestId("install-confirm-ok-codex_cli").click(); await page.waitForTimeout(3500); await shot(page, dir, "4a-ai-tab-cai-codex-that-bai");
      });
    });
    test("hoi-ai-tab-npm-thieu", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, cli: MISSING, installNpmMissing: true } as never);
      await step("npm", dir, async () => {
        await settingsTab(page, rlt, "set.tab.translate");
        const x = page.getByTestId("cli-card-codex_cli"); await x.scrollIntoViewIfNeeded();
        await x.getByTestId("install-run-codex_cli").click(); await x.getByTestId("install-confirm-ok-codex_cli").click(); await page.waitForTimeout(2500); await shot(page, dir, "4b-ai-tab-cai-codex-thieu-npm");
      });
    });
    test("hoi-ai-tab-san-sang", async ({ page, rlt }) => {
      const R = { path: "/x/bin", version: "1.0.0", status: "ready" };
      await rlt.open({ locale: c.loc, cli: { claude_cli: R, codex_cli: R, antigravity_cli: R } } as never);
      await step("ready", dir, async () => { await settingsTab(page, rlt, "set.tab.translate"); await page.screenshot({ path: `${ROOT}/${dir}/4c-ai-tab-ca-3-san-sang-toan-trang.png`, fullPage: true }); });
    });
    test("hoi-ai-tab-chua-dang-nhap-api", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, aiKeys: { gemini: false, openai: false } });
      await step("nokeys", dir, async () => { await settingsTab(page, rlt, "set.tab.translate"); await shot(page, dir, "4d-ai-tab-chua-co-khoa-api"); });
    });

    test("hoi-ai-man", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc });
      await rlt.nav("nav.askai").click();
      await step("nocfg", dir, async () => { await page.waitForTimeout(400); await shot(page, dir, "50-hoi-ai-chua-cau-hinh"); });
    });
    test("hoi-ai-man-cau-hinh", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, aiKeys: { gemini: true, openai: false } });
      await rlt.nav("nav.askai").click();
      await step("cfg", dir, async () => { await page.waitForTimeout(300); await shot(page, dir, "51-hoi-ai-dong-y-lan-dau"); });
      await step("agree", dir, async () => { await rlt.btn("ask.consent.ok").click(); await page.waitForTimeout(250); await shot(page, dir, "52-hoi-ai-san-sang"); await rlt.btn("ask.chip.summary").click(); await page.waitForTimeout(1200); await shot(page, dir, "53-hoi-ai-co-tra-loi"); });
    });

    test("cap-nhat", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, update: { version: "9.9.9", notes: "Faster export and fixes." } });
      await step("banner", dir, async () => { await page.waitForTimeout(400); await shot(page, dir, "60-cap-nhat-banner"); });
      await step("card", dir, async () => { await settingsTab(page, rlt, "set.tab.permissions"); await page.getByTestId("update-check").click(); await page.waitForTimeout(500); await shot(page, dir, "61-cap-nhat-co-ban-moi"); await page.getByTestId("update-download").click(); await page.waitForTimeout(800); await shot(page, dir, "62-cap-nhat-da-tai"); });
    });
    test("cap-nhat-loi", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, fail: { update_check: "network" } });
      await step("err", dir, async () => { await settingsTab(page, rlt, "set.tab.permissions"); await page.getByTestId("update-check").click(); await page.waitForTimeout(500); await shot(page, dir, "63-cap-nhat-loi-mang"); });
    });
    test("usage-loi", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, usage: { liveFail: "network" } });
      await step("usage", dir, async () => { await settingsTab(page, rlt, "set.tab.sonioxusage"); await page.waitForTimeout(600); await shot(page, dir, "64-usage-loi-mang-khong-cache"); });
    });
    test("drive", async ({ page, rlt }) => {
      await rlt.open({ locale: c.loc, drive: { state: "connected", email: "owner@example.com", builtin: true } });
      await step("conn", dir, async () => { await settingsTab(page, rlt, "set.tab.drive"); await page.waitForTimeout(400); await shot(page, dir, "65-drive-da-ket-noi"); });
    });

    test("popup-phu-de", async ({ page, rlt }) => {
      await page.setViewportSize({ width: 640, height: 160 });
      await rlt.open({ locale: c.loc, subtitleLines: [{ captionLineId: 1, seq: 1, offsetMs: 1000, sourceLang: "en", sourceText: "Let us review the roadmap.", translatedText: "Vamos rever o roteiro." }] });
      await step("sub", dir, async () => { await page.goto("/subtitle.html"); await page.waitForTimeout(500); await shot(page, dir, "70-phu-de-noi"); });
    });
    test("quick", async ({ page, rlt }) => {
      await page.setViewportSize({ width: 520, height: 360 });
      await rlt.open({ locale: c.loc });
      await step("quick", dir, async () => { await page.goto("/quick.html"); await page.waitForTimeout(500); await shot(page, dir, "71-quick-cua-so-nho"); });
    });
  });
}
