// Khong tran ngang o bo rong toi thieu: moi tab Cai dat + Ghi + Phien, 720/800/1000, en va pt-PT.
import { test, expect } from "./fixtures.ts";
import type { Page } from "@playwright/test";
import { mkdirSync } from "node:fs";

const WIDTHS = [720, 800, 1000];
const LOCALES = ["en", "pt-PT"] as const;
const TABS = ["set.tab.general", "set.tab.translate", "set.tab.shortcuts", "set.tab.glossary", "set.tab.subtitle", "set.tab.sonioxusage", "set.tab.drive", "set.tab.permissions"];
const SHOT = process.env.RLT_SHOT_DIR;

/** Phan tu nao rong hon khung cua no (de bao loi doc duoc). */
async function overflow(page: Page) {
  return page.evaluate(() => {
    const d = document.documentElement;
    const bad: string[] = [];
    for (const sel of ["html", "body", ".app", ".main", ".sbody"]) {
      const e = sel === "html" ? d : document.querySelector(sel);
      if (e && e.scrollWidth > e.clientWidth) bad.push(`${sel} ${e.scrollWidth}>${e.clientWidth}`);
    }
    const vw = d.clientWidth;
    for (const e of Array.from(document.querySelectorAll(".card, .card *"))) {
      const r = e.getBoundingClientRect();
      if (r.width && r.right > vw + 0.5) { bad.push(`${e.tagName}.${(e as HTMLElement).className} right=${Math.round(r.right)}>${vw}`); if (bad.length > 8) break; }
    }
    return bad;
  });
}

for (const loc of LOCALES) for (const w of WIDTHS) {
  test(`khong tran ngang ${loc} @${w}`, async ({ page, rlt }) => {
    await page.setViewportSize({ width: w, height: 700 });
    await rlt.open({ locale: loc });
    const shot = async (name: string) => { if (SHOT) { mkdirSync(SHOT, { recursive: true }); await page.screenshot({ path: `${SHOT}/${name}-${loc}-${w}.png` }); } };
    const failures: string[] = [];
    const check = async (name: string) => { const b = await overflow(page); await shot(name); if (b.length) failures.push(`${name}: ${b.join("; ")}`); };

    await check("record");
    await rlt.nav("nav.sessions").click();
    await check("sessions");
    await rlt.nav("nav.settings").click();
    for (const key of TABS) {
      await page.getByRole("navigation", { name: rlt.t("nav.settings") }).getByRole("button", { name: rlt.t(key) }).click();
      await check(key.replace("set.tab.", "tab-"));
    }
    expect(failures).toEqual([]);
  });
}
