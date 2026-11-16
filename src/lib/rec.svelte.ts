// Phien ghi DANG CHAY song o day (module toan cuc), khong phai trong component Record. Truoc day state/lines/live/
// level/status nam trong Record.svelte va kenh su kien (Channel) gan voi onEvent cua no; chuyen Cai dat/Phien thi Record
// bi go, dong ho dung, mat phu de va nut Dung/Tam dung roi vao trang thai lech voi backend (loi win-test e1079c0).
import * as api from "./api.ts";
import type { LineDto, SessionStateName, StatusCode } from "./api.ts";
import { app } from "./app.svelte.ts";
import { afterFinal, fromPartial, type LiveText } from "./live.ts";

export const rec = $state({
  state: "idle" as SessionStateName,
  lines: [] as LineDto[],
  live: null as LiveText | null,
  level: 0,
  status: null as StatusCode | null,
  statusDetail: null as string | null,
  notices: [] as string[],
  lastStop: null as { lines: number; drained: boolean } | null,
  sessionId: null as string | null,
});

let timer: ReturnType<typeof setInterval> | undefined;
// Giu tham chieu kenh de no song suot phien (khong phu thuoc vong doi component).
let channel: ReturnType<typeof api.newChannel> | null = null;

export function setState(s: SessionStateName) {
  rec.state = s;
  app.recording = s !== "idle";
  void api.shellSetRecording(s === "recording").catch(() => {});
  clearInterval(timer);
  timer = undefined;
  if (s === "recording") timer = setInterval(() => app.seconds++, 1000);
}

/** Xoa man hinh phien cu va dat lai cac gia tri theo phien (nut Bat dau, Phien moi). */
export function resetForNewSession() {
  rec.lines = []; rec.live = null; rec.level = 0; rec.status = null; rec.statusDetail = null; rec.notices = []; rec.lastStop = null;
}

export function upsertLine(l: LineDto) {
  const i = rec.lines.findIndex(x => x.captionLineId === l.captionLineId);
  if (i >= 0) rec.lines[i] = l; else rec.lines.push(l);
}

export function onEvent(e: api.UiEvent) {
  if (e.kind === "partial") rec.live = fromPartial(e.partial);
  else if (e.kind === "state") {
    if (e.state === "idle") rec.live = null;
    if (e.state === "recording" && rec.state === "idle") { app.seconds = 0; rec.level = 0; }
    setState(e.state);
    void api.sessionState().then(st => (rec.sessionId = st[1])).catch(() => {});
  } else if (e.kind === "line") {
    upsertLine(e.line);
    rec.live = afterFinal(rec.live, e.line.sourceText, e.line.translatedText);
  } else if (e.kind === "level") rec.level = e.level;
  else if (e.kind === "status") { rec.status = e.code; rec.statusDetail = e.code === "rejected" || e.code === "network" ? (e.detail ?? null) : null; }
  else if (e.kind === "notice") { if (!rec.notices.includes(e.code)) rec.notices.push(e.code); }
  else if (e.kind === "stopped") rec.lastStop = { lines: e.lines, drained: e.drained };
}

/** Kenh moi cho `session_start`; do store giu nen khong bi huy khi roi man Ghi. */
export function openChannel() {
  channel = api.newChannel(onEvent);
  return channel;
}

/**
 * Doi chieu voi backend: backend la nguon dung. Goi khi man Ghi gan lai va khi Dung/Tam dung bao `not_recording`.
 * - backend idle ma giao dien con tuong dang ghi (su kien idle roi mat) -> ve idle;
 * - backend dang ghi: lay trang thai that va nap lai dong chot (hop cac dong dang co, khong mat dong moi hon).
 */
export async function sync() {
  const [name, id] = await api.sessionState();
  rec.sessionId = id;
  if (name === "idle") {
    if (rec.state !== "idle") { rec.live = null; setState("idle"); }
    return;
  }
  if (rec.state !== name) setState(name as SessionStateName);
  else if (name === "recording" && timer === undefined) setState("recording");
  try { for (const l of await api.sessionLines()) upsertLine(l); rec.lines.sort((a, b) => a.seq - b.seq); } catch { /* backend cu: giu nguyen nhung gi da co */ }
}
