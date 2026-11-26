// Tom tat cuoc ghi (US-63, US-64): kieu du lieu, lenh Tauri va ham thuan (kiem bang node --test).
import { Channel, invoke } from "@tauri-apps/api/core";
import type { Provider } from "./api.ts";

export interface SummaryPlan {
  total_chars: number; sent_chars: number; input_cut: boolean; dropped_head: boolean; cap_chars: number;
  chunks: number; calls: number; translate: boolean;
}
export interface SummaryParts { summary: string; key_points: string[]; action_items: string[] }
export interface SummaryProgress { stage: "map" | "reduce" | "translate"; index: number; total: number }
export interface SummaryOutcome {
  status: "completed" | "failed" | "cancelled"; error_code: string | null; failed_at: number | null;
  parts: SummaryParts | null; translated: SummaryParts | null; plan: SummaryPlan;
}
export type TargetLang = "en" | "pt-PT";

export const summaryPlan = (sessionId: string, translate: boolean) =>
  invoke<SummaryPlan>("ai_summary_plan", { sessionId, translate });

export function summarize(sessionId: string, provider: Provider | null, uiLang: TargetLang, translateTo: TargetLang | null, onProgress: (p: SummaryProgress) => void) {
  const ch = new Channel<SummaryProgress>();
  ch.onmessage = onProgress;
  return invoke<SummaryOutcome>("ai_summarize", { sessionId, provider, uiLang, translateTo, onProgress: ch });
}

// Ngon ngu dich: tom tat viet theo ngon ngu giao dien, ban dich sang ngon ngu con lai (en <-> pt-PT).
export const translateTarget = (ui: TargetLang): TargetLang => (ui === "pt-PT" ? "en" : "pt-PT");

export const progressKey = (stage: SummaryProgress["stage"]): string =>
  stage === "map" ? "sum.progress.map" : stage === "reduce" ? "sum.progress.reduce" : "sum.progress.translate";

// Thong bao khi dau vao bi cat (AC-63.3 va tran 200.000 ky tu): khoa i18n + gia tri, hoac null.
export function cutNotices(p: SummaryPlan): { key: string; values: Record<string, number> }[] {
  const out: { key: string; values: Record<string, number> }[] = [];
  if (p.input_cut) out.push({ key: "sum.cut.input", values: { total: p.total_chars, cap: p.cap_chars } });
  if (p.dropped_head) out.push({ key: "sum.cut.chunks", values: { n: p.chunks } });
  return out;
}

// Ket qua da dung khi chi hoan thanh mot phan: co "parts" thi hien, ke ca khi dich loi.
export const hasResult = (o: SummaryOutcome | null): boolean => !!o && !!o.parts;
