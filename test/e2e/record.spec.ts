import { test, expect } from "./fixtures.ts";

test.describe("Man Ghi: bat dau, tam dung, dung", () => {
  test("man hinh trong, bat dau gui dung tham so va vao trang thai Dang ghi", async ({ page, rlt }) => {
    await rlt.open();
    await expect(page.getByText(rlt.t("rec.empty.title"))).toBeVisible();
    await expect(rlt.btn("tool.stop")).toHaveCount(0);

    await rlt.startRecording();

    const [prefs] = await rlt.calls("record_prefs_set");
    expect(prefs.args.prefs).toMatchObject({ targetLang: "pt-PT", listenLangs: ["en"], mode: "mixed" });
    const [start] = await rlt.calls("session_start");
    expect(start.args.params).toEqual({ targetLang: "pt-PT", listenLangs: ["en"], mode: "mixed" });
    await expect(rlt.btn("tool.pause")).toBeVisible();
    await expect(page.locator(".tm")).toHaveText(/^\d\d:\d\d:\d\d$/);
    // Cau hinh bi khoa khi dang ghi.
    await expect(page.locator(".cfg")).toHaveClass(/lock/);
  });

  for (const [code, key] of [["mixed_no_mic", "rec.notice.mixednomic"], ["translation_quota", "rec.notice.translationquota"]] as const) {
    test(`thong bao ${code} hien ro cho nguoi dung va xoa khi bat dau phien moi`, async ({ page, rlt }) => {
      await rlt.open();
      await rlt.startRecording();
      await rlt.emit({ kind: "notice", code });
      await expect(page.getByRole("status").filter({ hasText: rlt.t(key) })).toBeVisible();
      await rlt.btn("tool.stop").click();
      await expect(page.getByRole("status").filter({ hasText: rlt.t(key) })).toBeVisible();
      await rlt.startRecording();
      await expect(page.getByText(rlt.t(key))).toHaveCount(0);
    });
  }

  test("tam dung roi tiep tuc", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();

    await rlt.btn("tool.pause").click();
    await expect(rlt.btn("tool.resume")).toBeVisible();
    await expect(page.locator(".foot .sb")).toContainText(rlt.t("status.paused"));
    expect(await rlt.calls("session_pause")).toHaveLength(1);

    await rlt.btn("tool.resume").click();
    await expect(rlt.btn("tool.pause")).toBeVisible();
    await expect(page.locator(".bd.rec")).toContainText(rlt.t("tool.badge.rec"));
    expect(await rlt.calls("session_resume")).toHaveLength(1);
  });

  test("dung ghi: ve man Bat dau, bao so dong da luu", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await rlt.emit(rlt.line(1, 1, "Hello everyone", "Olá a todos"));
    await rlt.emit(rlt.line(2, 2, "Let us begin", "Vamos começar"));
    await expect(page.locator(".ln:not(.live)")).toHaveCount(2);

    await rlt.btn("tool.stop").click();
    await expect(rlt.btn("tool.start")).toBeVisible();
    await expect(rlt.btn("tool.stop")).toHaveCount(0);
    await expect(page.getByText(rlt.t("rec.stopped", { n: 2 }))).toBeVisible();
    expect(await rlt.calls("session_stop")).toHaveLength(1);
    // Noi dung van con tren man hinh sau khi dung.
    await expect(page.locator(".ln")).toHaveCount(2);
  });

  test("dung khi chua het (drained=false) hien canh bao mat chu cuoi", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await rlt.emit({ kind: "stopped", sessionId: "sess-live", lines: 0, durationMs: 1000, drained: false });
    await expect(page.getByText(rlt.t("rec.stopped.partial"))).toBeVisible();
  });

  test("Phien moi khi dang ghi dung phien va xoa man hinh", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await rlt.emit(rlt.line(1, 1, "Hello", "Olá"));
    await expect(page.locator(".ln:not(.live)")).toHaveCount(1);
    await rlt.btn("tool.new").click();
    await expect(rlt.btn("tool.start")).toBeVisible();
    await expect(page.getByText(rlt.t("rec.empty.title"))).toBeVisible();
    expect(await rlt.calls("session_new")).toHaveLength(1);
  });

  test("tat ca hai nguon am thanh thi khong the bat dau", async ({ page, rlt }) => {
    await rlt.open();
    const group = page.getByRole("group", { name: rlt.t("cfg.audio.group") });
    for (const b of await group.getByRole("button").all()) await b.click();
    await expect(rlt.btn("tool.start")).toBeDisabled();
    await expect(page.getByText(rlt.t("rec.empty.nosrc"))).toBeVisible();
  });
});

test.describe("Man Ghi: hop thoai dieu kien truoc khi ghi", () => {
  test("chua co khoa Soniox: hoi khoa, luu xong thi tu bat dau", async ({ page, rlt }) => {
    await rlt.open({ hasKey: false });
    await expect(page.getByText(rlt.t("rec.key.missing"))).toBeVisible();
    await rlt.btn("tool.start").click();
    const dlg = page.getByRole("dialog", { name: rlt.t("rec.key.title") });
    await expect(dlg).toBeVisible();
    await expect(dlg.getByRole("button", { name: rlt.t("rec.key.save") })).toBeDisabled();
    await dlg.getByLabel(rlt.t("set.soniox.key")).fill("sk-test-123");
    await dlg.getByRole("button", { name: rlt.t("rec.key.save") }).click();
    await expect(rlt.btn("tool.stop")).toBeVisible();
    const [save] = await rlt.calls("soniox_save_api_key");
    expect(save.args.value).toBe("sk-test-123");
    // Giao dien khong doc lai khoa: o nhap bi xoa va hop thoai dong.
    await expect(dlg).toHaveCount(0);
  });

  test("chua dong y thong bao ghi am: hoi dong y roi bat dau", async ({ page, rlt }) => {
    await rlt.open({ consent: false });
    await rlt.btn("tool.start").click();
    const dlg = page.getByRole("dialog", { name: rlt.t("rec.consent.title") });
    await expect(dlg).toBeVisible();
    await dlg.getByRole("button", { name: rlt.t("rec.consent.accept") }).click();
    await expect(rlt.btn("tool.stop")).toBeVisible();
    expect(await rlt.calls("consent_accept")).toHaveLength(1);
  });

  test("huy hop thoai dong y thi khong ghi", async ({ page, rlt }) => {
    await rlt.open({ consent: false });
    await rlt.btn("tool.start").click();
    const dlg = page.getByRole("dialog", { name: rlt.t("rec.consent.title") });
    await dlg.getByRole("button", { name: rlt.t("common.cancel") }).click();
    await expect(dlg).toHaveCount(0);
    expect(await rlt.calls("session_start")).toHaveLength(0);
  });
});

test.describe("Chu chay truc tiep", () => {
  test("cau tam (chua dich) doi thanh ban dich khi cap nhat cung id, thu tu giu nguyen", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    const lines = page.locator(".ln:not(.live)");

    await rlt.emit(rlt.line(1, 1, "We should ship", null));
    await expect(lines).toHaveCount(1);
    await expect(lines.first()).toHaveClass(/prov/);
    await expect(lines.first().getByText(rlt.t("ln.provisional"))).toBeVisible();
    await expect(lines.first().locator(".dst")).toHaveCount(0);

    await rlt.emit(rlt.line(1, 1, "We should ship it on Friday.", "Devemos lançar na sexta-feira."));
    await expect(lines).toHaveCount(1); // cap nhat tai cho, khong them dong
    await expect(lines.first()).not.toHaveClass(/prov/);
    await expect(lines.first().locator(".src")).toHaveText("We should ship it on Friday.");
    await expect(lines.first().locator(".dst")).toHaveText("Devemos lançar na sexta-feira.");

    await rlt.emit(rlt.line(2, 2, "Agreed.", "De acordo."));
    await expect(lines).toHaveCount(2);
    await expect(lines.nth(1).locator(".dst")).toHaveText("De acordo.");
    // Dong "dang nghe" luon nam cuoi khi con ghi.
    await expect(page.locator(".ln.live")).toContainText(rlt.t("ln.listening"));
  });

  test("ba che do xem: ca hai, chi ban dich, chi nguon", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await rlt.emit(rlt.line(1, 1, "Good morning", "Bom dia"));
    const ln = page.locator(".ln:not(.live)").first();
    await expect(ln.locator(".src")).toBeVisible();
    await expect(ln.locator(".dst")).toBeVisible();

    await page.getByRole("button", { name: rlt.t("rec.view.dst") }).click();
    await expect(ln.locator(".src")).toHaveCount(0);
    await expect(ln.locator(".dst")).toBeVisible();

    await page.getByRole("button", { name: rlt.t("rec.view.src") }).click();
    await expect(ln.locator(".src")).toBeVisible();
    await expect(ln.locator(".dst")).toHaveCount(0);
  });

  test("tin hieu am luong dong huy hieu nguon (dang nghe / im lang)", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await rlt.emit({ kind: "level", level: 0 });
    await expect(page.locator(".foot .sb")).toContainText(rlt.t("status.silent"));
    await rlt.emit({ kind: "level", level: 0.4 });
    await expect(page.locator(".foot .sb")).toContainText(rlt.t("status.listening"));
  });
});
