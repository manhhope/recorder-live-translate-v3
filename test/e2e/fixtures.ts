// Fixture dung chung: dung trang voi lop Tauri gia, doc chuoi giao dien tu src/locales (khong chep tay), va
// tu bat loi: moi loi trang (pageerror), moi canh bao i18n thieu khoa, moi lenh Tauri khong co trong lop gia.
import { test as base, expect, type Locator, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { installTauriMock, type MockSeed } from "./tauri-mock.ts";
import { DETAILS, SESSIONS } from "./data.ts";

const LOCALES = {
  en: JSON.parse(readFileSync(resolve(import.meta.dirname, "../../src/locales/en.json"), "utf8")) as Record<string, string>,
  "pt-PT": JSON.parse(readFileSync(resolve(import.meta.dirname, "../../src/locales/pt-PT.json"), "utf8")) as Record<string, string>,
};
export type Loc = keyof typeof LOCALES;

export const esc = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

export class Rlt {
  consoleProblems: string[] = [];
  /** Canh bao console duoc phep (da ghi thanh loi san pham trong bao cao), khop theo chuoi con. */
  allowConsole: string[] = [];
  pageErrors: string[] = [];
  locale: Loc = "en";
  constructor(readonly page: Page) {
    page.on("pageerror", e => this.pageErrors.push(String(e)));
    page.on("console", m => {
      const t = m.text();
      if (m.type() === "error" || t.includes("[i18n]")) this.consoleProblems.push(`${m.type()}: ${t}`);
    });
  }

  /** Chuoi giao dien theo khoa; `{n}` duoc thay bang values. */
  t(key: string, values: Record<string, string | number> = {}, loc: Loc = this.locale): string {
    const raw = LOCALES[loc][key];
    if (raw === undefined) throw new Error(`khong co khoa i18n ${key} (${loc})`);
    return raw.replace(/\{(\w+)\}/g, (_m, k) => String(values[k] ?? `{${k}}`));
  }

  async open(over: Partial<MockSeed> = {}, path = "/index.html") {
    const seed: MockSeed = {
      locale: "en", hasKey: true, consent: true, aiKeys: { gemini: false, openai: false },
      sessions: SESSIONS, details: DETAILS, fail: {}, ...over,
    };
    this.locale = seed.locale;
    await this.page.addInitScript(installTauriMock, seed);
    await this.page.goto(path);
    await expect(this.page.getByRole("navigation", { name: this.t("nav.tools") })).toBeVisible();
    // Man Ghi da nap xong khi nut Bat dau het bi khoa (prefs/khoa/dong y da tra ve).
    await expect(this.btn("tool.start")).toBeEnabled();
  }

  /** Nut theo khoa i18n; ten truy cap co them phim tat/icon nen khop theo tien to. */
  btn(key: string, scope: Page | Locator = this.page): Locator {
    return scope.getByRole("button", { name: new RegExp("^" + esc(this.t(key))) });
  }
  nav(key: string): Locator {
    return this.page.getByRole("navigation", { name: this.t("nav.tools") }).getByRole("button", { name: new RegExp("^" + esc(this.t(key))) });
  }

  calls(cmd: string): Promise<{ cmd: string; args: any }[]> {
    return this.page.evaluate(c => (window as any).__rlt.calls.filter((x: any) => x.cmd === c), cmd);
  }
  emit(event: unknown) { return this.page.evaluate(e => (window as any).__rlt.emit(e), event); }
  setFail(cmd: string, code: string | null) {
    return this.page.evaluate(([c, v]) => { const f = (window as any).__rlt.fail; if (v) f[c as string] = v; else delete f[c as string]; }, [cmd, code]);
  }
  /** ms de lenh tra cham; -1 = treo han (khong bao gio tra ve); null = xoa. */
  setDelay(cmd: string, ms: number | null) {
    return this.page.evaluate(([c, v]) => { const d = (window as any).__rlt.delay; if (v === null) delete d[c as string]; else d[c as string] = v; }, [cmd, ms]);
  }
  unhandled(): Promise<string[]> { return this.page.evaluate(() => (window as any).__rlt.unhandled); }

  /** Bam Bat dau va doi phien that su chay (huy hieu Dang ghi). */
  async startRecording() {
    await this.btn("tool.start").click();
    await expect(this.btn("tool.stop")).toBeVisible();
    await expect(this.page.locator(".bd.rec")).toContainText(this.t("tool.badge.rec"));
  }
  line(id: number, seq: number, src: string, dst: string | null, offsetMs = seq * 4000) {
    return { kind: "line", line: { captionLineId: id, seq, offsetMs, sourceLang: "en", sourceText: src, translatedText: dst } };
  }
}

export const test = base.extend<{ rlt: Rlt }>({
  rlt: async ({ page }, use) => {
    const rlt = new Rlt(page);
    await use(rlt);
    // Sau moi test: khong loi trang, khong khoa i18n thieu, khong lenh Tauri la.
    expect(rlt.pageErrors, "loi chua bat trong trang").toEqual([]);
    expect(rlt.consoleProblems.filter(p => !rlt.allowConsole.some(a => p.includes(a))), "console error hoac canh bao i18n").toEqual([]);
    if (page.url().startsWith("http")) expect(await rlt.unhandled(), "lenh Tauri khong co trong lop gia").toEqual([]);
  },
});
export { expect };
