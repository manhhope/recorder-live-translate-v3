// Cac loi vao Hoi hoi AI (M3-loi-vao): nut o man Ghi (dang ghi va sau khi ghi), chat trong chi tiet bien ban, thanh ben,
// phim tat/deeplink ask_ai, va truong hop chua co khoa AI. Lop Tauri la gia (tauri-mock.ts).
import { test, expect, esc, type Rlt } from "./fixtures.ts";

const withAi = { aiKeys: { gemini: true, openai: false } };

/** Dong y gui du lieu cho nha cung cap (lan dau), roi cac chip moi bam duoc. */
async function agree(rlt: Rlt) {
  await rlt.btn("ask.consent.ok").click();
  await expect(rlt.btn("ask.chip.summary")).toBeEnabled();
}
const askBtn = (rlt: Rlt) => rlt.page.locator(".tr2").getByRole("button", { name: rlt.t("nav.askai") });

test.describe("Loi vao Hoi AI", () => {
  test("man Ghi: nut Hoi AI hien luc nghi va luc dang ghi; luc dang ghi bam la mo dung phien dang chay", async ({ page, rlt }) => {
    await rlt.open(withAi);
    await expect(askBtn(rlt)).toBeVisible();
    await expect(askBtn(rlt)).toBeEnabled();

    await rlt.startRecording();
    await expect(askBtn(rlt)).toBeVisible();
    await askBtn(rlt).click();
    await expect(page.getByRole("region", { name: rlt.t("ask.input") })).toBeVisible();
    await agree(rlt);
    await rlt.btn("ask.chip.explain").click();
    await expect.poll(async () => page.evaluate(() => (window as any).__rlt.aiAsks?.at(-1)?.session_id)).toBe("sess-live");
  });

  test("man Ghi: sau khi ghi xong nut Hoi AI van con va mo dung phien vua ghi", async ({ page, rlt }) => {
    await rlt.open(withAi);
    await rlt.startRecording();
    await rlt.btn("tool.stop").click();
    await expect(rlt.btn("tool.start")).toBeVisible();
    await expect(askBtn(rlt)).toBeVisible();
    await askBtn(rlt).click();
    await agree(rlt);
    await rlt.btn("ask.chip.summary").click();
    await expect.poll(async () => page.evaluate(() => (window as any).__rlt.aiAsks?.at(-1)?.session_id)).toBe("sess-live");
  });

  test("man Ghi chua tung ghi: nut Hoi AI mo man Hoi AI voi phien moi nhat", async ({ page, rlt }) => {
    await rlt.open(withAi);
    await askBtn(rlt).click();
    await expect(page.getByRole("combobox", { name: rlt.t("ask.screen.pick") })).toHaveValue("s-weekly");
    await expect(page.getByRole("region", { name: rlt.t("ask.input") })).toBeVisible();
  });

  test("chi tiet bien ban: chat nhieu luot gan bien ban, co chip Explain, Summarise, Action items", async ({ page, rlt }) => {
    await rlt.open(withAi);
    await rlt.nav("nav.sessions").click();
    await page.locator(".sesrow button").first().click();
    await expect(page.getByRole("region", { name: rlt.t("ask.input") })).toHaveCount(0);
    await rlt.btn("nav.askai", page.locator(".tr2")).click();
    const chat = page.getByRole("region", { name: rlt.t("ask.input") });
    await expect(chat).toBeVisible();
    for (const k of ["ask.chip.explain", "ask.chip.summary", "ask.chip.todo"]) await expect(rlt.btn(k, chat)).toBeVisible();
    await agree(rlt);

    const box = chat.getByRole("textbox", { name: rlt.t("ask.input") });
    await box.fill("What ships on Friday?");
    await rlt.btn("common.send", chat).click();
    await expect(chat.locator(".msg.assistant").last()).toContainText("turn 1");
    await box.fill("And who owns it?");
    await rlt.btn("common.send", chat).click();
    await expect(chat.locator(".msg.assistant").last()).toContainText("turn 2");
    await rlt.btn("ask.chip.todo", chat).click();
    await expect(chat.locator(".msg.assistant")).toHaveCount(3);

    const asks = await page.evaluate(() => (window as any).__rlt.aiAsks);
    expect(asks.map((a: any) => a.session_id)).toEqual(["s-weekly", "s-weekly", "s-weekly"]);
    expect(asks[1].conversation_id).toBe(1); // luot 2 cung mot cuoc tro chuyen
    expect(asks[2].chip).toBe("action_items");
  });

  test("thanh ben: muc Hoi AI co phim tat canh nhan va mo man Hoi AI", async ({ page, rlt }) => {
    await rlt.open(withAi);
    const item = rlt.nav("nav.askai");
    await expect(item).toBeVisible();
    await expect(item.locator("kbd")).toHaveText("⇧⌘A");
    await item.click();
    await expect(item).toHaveAttribute("aria-current", "page");
    await expect(page.getByRole("region", { name: rlt.t("ask.input") })).toBeVisible();
  });

  test("phim tat: su kien shell://deeplink {action: ask_ai} mo man Hoi AI; hanh dong khac thi bo qua", async ({ page, rlt }) => {
    await rlt.open(withAi);
    await expect.poll(async () => (await rlt.calls("plugin:event|listen")).map(c => c.args.event)).toContain("shell://deeplink");
    await page.evaluate(() => (window as any).__rlt.emitEvent("shell://deeplink", { action: "settings", tab: "general" }));
    await expect(rlt.nav("nav.record")).toHaveAttribute("aria-current", "page");
    await page.evaluate(() => (window as any).__rlt.emitEvent("shell://deeplink", { action: "ask_ai" }));
    await expect(rlt.nav("nav.askai")).toHaveAttribute("aria-current", "page");
    await expect(page.getByRole("region", { name: rlt.t("ask.input") })).toBeVisible();
  });

  test("chua co khoa AI: huong dan ro va nut nhay toi Cai dat, muc Dich va AI (khong man trong)", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.nav("nav.askai").click();
    const chat = page.getByRole("region", { name: rlt.t("ask.input") });
    await expect(chat.getByText(rlt.t("ask.nocfg.title"))).toBeVisible();
    await expect(chat.getByText(rlt.t("ask.nocfg.hint"))).toBeVisible();
    await rlt.btn("ask.open.ai", chat).click();
    await expect(rlt.nav("nav.settings")).toHaveAttribute("aria-current", "page");
    await expect(page.getByRole("button", { name: new RegExp("^" + esc(rlt.t("set.tab.translate"))) })).toHaveAttribute("aria-current", "page");
    await expect(page.getByRole("heading", { name: rlt.t("set.tab.translate") })).toBeVisible();
  });

  test("chua co bien ban nao: man Hoi AI co huong dan va nut ve man Ghi", async ({ page, rlt }) => {
    await rlt.open({ sessions: [], details: {} });
    await rlt.nav("nav.askai").click();
    await expect(page.getByTestId("ask-empty")).toContainText(rlt.t("ask.screen.nosession.title"));
    await rlt.btn("ask.screen.gorecord").click();
    await expect(rlt.btn("tool.start")).toBeVisible();
  });

  test("pt-PT: moi loi vao dung chuoi tieng Bo Dao Nha, khong co khoa i18n thieu", async ({ page, rlt }) => {
    await rlt.open({ ...withAi, locale: "pt-PT" });
    await expect(askBtn(rlt)).toHaveText(/Perguntar à IA/);
    await expect(rlt.nav("nav.askai")).toContainText("Perguntar à IA");
    await askBtn(rlt).click();
    await expect(page.getByRole("button", { name: rlt.t("ask.chip.todo") })).toBeVisible();
  });
});
