// Chup anh man hinh M2 bang playwright-core (cai ngoai repo, vd /tmp/pw).
import { createRequire } from "node:module";
const require = createRequire(process.env.PW_DIR ? process.env.PW_DIR + "/" : "/tmp/pw/");
const { chromium } = require("playwright-core");
const exe = process.env.CHROME;
const url = process.env.URL ?? "http://localhost:4173/";
const out = "docs/anh/";
const b = await chromium.launch({ executablePath: exe });
const p = await b.newPage({ viewport: { width: 1280, height: 800 } });
await p.goto(url); await p.waitForSelector(".app");
const shot = (n) => p.screenshot({ path: out + n + ".png" });
await shot("m2-record-idle");
for (const lang of ["en", "pt"]) { /* ngon ngu mac dinh pt-PT, chup them ban en */ }
await p.locator(".btn.primary").first().click().catch(() => {});
await p.waitForTimeout(300); await shot("m2-record-recording");
await p.getByRole("button", { name: /2$|Sessions|Sessões/ }).first().click();
await p.waitForTimeout(200); await shot("m2-sessions");
await p.locator(".ni").nth(2).click(); await p.waitForTimeout(200); await shot("m2-settings-ai");
await p.locator(".sn").nth(1).click(); await p.waitForTimeout(200); await shot("m2-settings-usage");
await p.locator(".ntools .btn").first().click(); await p.waitForTimeout(200); await shot("m2-settings-usage-en");
await b.close();
