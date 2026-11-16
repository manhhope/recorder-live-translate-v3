// Loi win-test e1079c0: dang ghi ma chuyen sang Cai dat/Phien roi ve man Ghi thi mat phu de, dong ho dung,
// Stop/Pause khong bam duoc (err.notrecording). Phien dang chay phai song ngoai vong doi component Record.
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";

const clock = (page: Page) => page.locator(".tm").innerText();
const toSec = (s: string) => { const [h, m, x] = s.split(":").map(Number); return h * 3600 + m * 60 + x; };

test.describe("Chuyen man khi dang ghi", () => {
  test("Cai dat roi ve: phu de, chu tam, dong ho con nguyen", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await rlt.emit(rlt.line(1, 1, "Hello everyone", "Olá a todos"));
    await rlt.emit({ kind: "partial", partial: { source: "mic", offsetMs: 9000, sourceLang: "en", sourceText: "and now", translatedText: "e agora" } });
    await expect(page.locator(".ln:not(.live)")).toHaveCount(1);

    await rlt.nav("nav.settings").click();
    await rlt.nav("nav.record").click();

    await expect(page.locator(".ln:not(.live)")).toHaveCount(1);
    await expect(page.getByText("Olá a todos")).toBeVisible();
    await expect(page.locator(".ln.live")).toContainText("and now");
    await expect(rlt.btn("tool.stop")).toBeVisible();
  });

  test("dong ho tiep tuc chay sau khi ve, ke ca khi roi man nhieu giay", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await rlt.nav("nav.sessions").click();
    await page.waitForTimeout(2200); // dong ho van phai chay khi component Record da bi go
    await rlt.nav("nav.record").click();
    const t1 = toSec(await clock(page));
    expect(t1).toBeGreaterThanOrEqual(2);
    await page.waitForTimeout(1500);
    expect(toSec(await clock(page))).toBeGreaterThan(t1);
  });

  test("chuyen Cai dat/Phien nhieu lan lien tiep roi Pause, Resume, Stop deu dung phien dang chay", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await rlt.emit(rlt.line(1, 1, "One", "Um"));
    for (let i = 0; i < 4; i++) {
      await rlt.nav(i % 2 ? "nav.sessions" : "nav.settings").click();
      await rlt.emit(rlt.line(10 + i, 2 + i, `Line ${i}`, `Linha ${i}`)); // su kien den luc Record khong gan
      await rlt.nav("nav.record").click();
    }
    await expect(page.locator(".ln:not(.live)")).toHaveCount(5);

    await rlt.btn("tool.pause").click();
    await expect(rlt.btn("tool.resume")).toBeVisible();
    await expect(page.getByText(rlt.t("err.notrecording"))).toHaveCount(0);
    await rlt.nav("nav.settings").click();
    await rlt.nav("nav.record").click();
    await expect(rlt.btn("tool.resume")).toBeVisible();
    await rlt.btn("tool.resume").click();
    await expect(rlt.btn("tool.pause")).toBeVisible();

    await rlt.btn("tool.stop").click();
    await expect(rlt.btn("tool.start")).toBeVisible();
    await expect(page.getByText(rlt.t("err.notrecording"))).toHaveCount(0);
    await expect(page.getByText(rlt.t("rec.stopped", { n: 5 }))).toBeVisible();
    expect(await rlt.calls("session_stop")).toHaveLength(1);
  });

  test("phien ket thuc luc dang o man khac: ve lai thay Idle, khong ket o Recording", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await rlt.nav("nav.settings").click();
    await page.evaluate(() => { (window as any).__rlt.state.run = "idle"; }); // backend da dung
    await rlt.emit({ kind: "stopped", sessionId: "sess-live", lines: 0, durationMs: 1000, drained: true });
    await rlt.emit({ kind: "state", state: "idle" });
    await rlt.nav("nav.record").click();
    await expect(rlt.btn("tool.start")).toBeVisible();
    await expect(page.locator(".bd.rec")).toHaveCount(0);
  });

  test("backend bao not_recording trong khi giao dien tuong dang ghi: tu dong bo ve Idle, khong ket", async ({ page, rlt }) => {
    await rlt.open();
    await rlt.startRecording();
    await page.evaluate(() => { (window as any).__rlt.state.run = "idle"; }); // backend da dung nhung su kien idle khong toi
    await rlt.btn("tool.stop").click();
    await expect(rlt.btn("tool.start")).toBeVisible();
  });
});
