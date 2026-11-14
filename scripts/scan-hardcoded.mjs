#!/usr/bin/env node
// Quet chuoi cung hien thi cho nguoi dung trong src/ (AC-49.2). Thoat 1 neu tim thay.
// Tim: (1) van ban trong thanh .svelte ngoai {...}, <script>, <style>; (2) thuoc tinh chu (title, aria-label, placeholder, alt, label) co gia tri chu;
// (3) ky tu tieng Viet trong ma nguon ngoai src/locales. Bo qua src/locales va src/i18n/.
import fs from "node:fs";
import path from "node:path";

const root = process.argv[2] ?? "src";
const VI = /[ĂăƠơƯưĐđẠ-ỹ]/;
const ATTRS = ["title", "aria-label", "placeholder", "alt", "label", "aria-description"];
const findings = [];

function walk(dir) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) { if (!["locales", "i18n"].includes(e.name)) walk(p); }
    else if (/\.(svelte|ts)$/.test(e.name)) scan(p);
  }
}

function lineOf(src, idx) { return src.slice(0, idx).split("\n").length; }

function scan(file) {
  const src = fs.readFileSync(file, "utf8");
  src.split("\n").forEach((l, i) => { if (VI.test(l)) findings.push(`${file}:${i + 1}: ky tu tieng Viet: ${l.trim()}`); });
  if (!file.endsWith(".svelte")) return;
  const markup = src.replace(/<script[\s\S]*?<\/script>/g, m => " ".repeat(m.length)).replace(/<style[\s\S]*?<\/style>/g, m => " ".repeat(m.length)).replace(/<!--[\s\S]*?-->/g, m => " ".repeat(m.length));
  const noExpr = markup.replace(/\{[^{}]*\}/g, m => " ".repeat(m.length));
  for (const m of noExpr.matchAll(/>([^<>]+)</g)) {
    const t = m[1].trim();
    if (/\p{L}/u.test(t)) findings.push(`${file}:${lineOf(src, m.index + 1)}: van ban cung: "${t}"`);
  }
  for (const a of ATTRS) {
    for (const m of noExpr.matchAll(new RegExp(`\\s${a}="([^"]*)"`, "g"))) {
      if (/\p{L}/u.test(m[1])) findings.push(`${file}:${lineOf(src, m.index)}: thuoc tinh ${a} cung: "${m[1]}"`);
    }
  }
}

if (!fs.existsSync(root)) { console.error(`khong thay thu muc ${root}`); process.exit(2); }
walk(root);
const nFiles = (function count(d){return fs.readdirSync(d,{withFileTypes:true}).reduce((n,e)=>n+(e.isDirectory()?(["locales","i18n"].includes(e.name)?0:count(path.join(d,e.name))):(/\.(svelte|ts)$/.test(e.name)?1:0)),0)})(root);
console.log(`Quet ${nFiles} tep trong ${root}/ (bo qua locales, i18n)`);
for (const f of findings) console.log(f);
console.log(`So chuoi cung tim thay: ${findings.length}`);
process.exit(findings.length ? 1 : 0);
