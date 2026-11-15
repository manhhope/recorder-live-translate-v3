// Cau noi giua giao dien va cac lenh Tauri cua phien ghi va bien ban (M3). KHONG co ham nao tra khoa Soniox:
// `saveApiKey` chi gui khoa di; giao dien chi biet `hasApiKey` (co / chua co).
import { Channel, invoke } from "@tauri-apps/api/core";

export type RecordMode = "system" | "mic" | "mixed";
export type SessionStateName = "idle" | "recording" | "paused" | "stopping";
export type StatusCode = "permission" | "auth" | "quota" | "network" | "timeout";

export interface LineDto { captionLineId: number; seq: number; offsetMs: number; sourceLang: string; sourceText: string; translatedText: string | null }
export type { PartialDto } from "./live.ts";
import type { PartialDto } from "./live.ts";
export type UiEvent =
  | { kind: "state"; state: SessionStateName }
  | { kind: "line"; line: LineDto }
  | { kind: "partial"; partial: PartialDto | null }
  | { kind: "level"; level: number }
  | { kind: "status"; code: StatusCode | null }
  | { kind: "stopped"; sessionId: string; lines: number; durationMs: number; drained: boolean }
  /** Thong bao ma phien van chay: che do tron ha xuong chi he thong (khong co micro), hoac het quota (ghi cuc bo, khong dich). */
  | { kind: "notice"; code: "mixed_no_mic" | "translation_quota" };

export interface StartParams { targetLang: string; listenLangs: string[]; mode: RecordMode; title?: string | null }
export interface RecordPrefs { targetLang: string; listenLangs: string[]; mode: RecordMode; uiLanguage: string }
/** `mic`: notDetermined | denied | restricted | authorized. */
export interface PermissionsOut { mic: string; screen: string }
export type DriveConnState = "not_configured" | "disconnected" | "connected";
export interface DriveConnOut { state: DriveConnState; email: string | null; builtin: boolean }
export interface DriveSyncAllOut { total: number; synced: number; failed: number; waiting: number; oauthConnected: boolean }
export interface DriveDisconnectOut { revoked: boolean }

export interface SessionRow {
  sessionId: string; title: string; startedAt: string; durationMs: number | null; status: string;
  audioSourceMode: string; targetLang: string; lines: number; attachedImages: number; syncStatus: string;
}
export interface LineRow {
  captionLineId: number; seq: number; lineType: string; source: string; offsetMs: number;
  sourceLang: string | null; sourceText: string | null; translatedText: string | null; highlighted: boolean; note: string | null;
}
export interface SessionDetail { session: SessionRow; lines: LineRow[]; syncLastAt: string | null }
export interface HitRow { captionLineId: number; sessionId: string; offsetMs: number; snippet: string }
export interface NoteRow { sessionId: string; captionLineId: number | null; note: string | null }
export interface TermRow { id: number; termSource: string; termTarget: string | null; keepOriginal: boolean; sourceLang: string; targetLang: string }
export interface SearchOut { lines: HitRow[]; notes: NoteRow[]; terms: TermRow[] }
export interface DayRow { localDate: string; audioMs: number; estCostUsdMicros: number; calls: number }
export interface WeekOut { days: DayRow[]; totalAudioMs: number; totalEstCostUsdMicros: number; estimated: boolean; sessionsCount: number; failedStartCount: number }
export interface DriveOut { status: string; attempts: number; oauthConnected: boolean }
export interface ImportOut { totalRows: number; tables: [string, number][] }

export function newChannel(onEvent: (e: UiEvent) => void): Channel<UiEvent> {
  const ch = new Channel<UiEvent>();
  ch.onmessage = onEvent;
  return ch;
}

export const sessionStart = (params: StartParams, onEvent: Channel<UiEvent>) => invoke<string>("session_start", { params, onEvent });
export const sessionPause = () => invoke<void>("session_pause");
export const sessionResume = () => invoke<void>("session_resume");
export const sessionStop = () => invoke<void>("session_stop");
export const sessionNew = () => invoke<boolean>("session_new");
export const sessionState = () => invoke<[string, string | null]>("session_state");
export const consentGet = () => invoke<boolean>("consent_get");
export const consentAccept = () => invoke<void>("consent_accept");
export const saveApiKey = (value: string) => invoke<void>("soniox_save_api_key", { value });
export const hasApiKey = () => invoke<boolean>("soniox_has_api_key");
export const clearApiKey = () => invoke<void>("soniox_clear_api_key");
export const testConnection = () => invoke<void>("soniox_test_connection");
export const permissionsStatus = () => invoke<PermissionsOut>("permissions_status");
/** `granted` | `denied` | `needsrestart` | `unknown`. Do that SCShareableContent: chi goi khi nguoi dung bam Check again. */
export const screenPermissionCheck = (settingsOpened: boolean) => invoke<string>("screen_permission_check", { settingsOpened });
export const restartApp = () => invoke<void>("restart_app");
export const prefsGet = () => invoke<RecordPrefs>("record_prefs_get");
export const prefsSet = (prefs: RecordPrefs) => invoke<void>("record_prefs_set", { prefs });

export const sessionsList = (limit = 100, offset = 0) => invoke<SessionRow[]>("sessions_list", { limit, offset });
export const sessionGet = (sessionId: string) => invoke<SessionDetail>("session_get", { sessionId });
export const sessionRename = (sessionId: string, title: string) => invoke<void>("session_rename", { sessionId, title });
export const sessionDelete = (sessionId: string) => invoke<boolean>("session_delete", { sessionId });
export const recordsSearch = (query: string, sessionId?: string) => invoke<SearchOut>("records_search", { query, sessionId: sessionId ?? null });
export type ExportFormat = "md" | "txt" | "srt" | "docx";
export type ExportContent = "both" | "translation" | "source";
export interface ExportOut { path: string; missingImages: number; warning: string | null }
/** Mo hop thoai luu cua macOS (ten goi y theo phien, thu muc khoi dau = `dir` da nho hoac Documents) roi ghi file. `null` = nguoi dung huy (khong ghi, khong loi). `warning` = `export_image_missing`. */
export const sessionExport = (sessionId: string, format: ExportFormat, content: ExportContent, uiLang: string, dir: string | null) =>
  invoke<ExportOut | null>("session_export", { sessionId, format, content, uiLang, dir });
export const highlightToggle = (sessionId: string, captionLineId: number, note: string | null = null) =>
  invoke<boolean>("highlight_toggle", { sessionId, captionLineId, note });
export const glossaryList = () => invoke<TermRow[]>("glossary_list");
export const usageWeek = (date: string, provider: string | null = "soniox") => invoke<WeekOut>("usage_week", { date, provider });
export const usageExportCsv = () => invoke<string>("usage_export_csv");
export interface UsageDayOut { date: string; audioMs: number; costUsdMicros: number; apiCostUsdMicros: number; playgroundCostUsdMicros: number; requests: number }
export interface UsageModelOut { model: string; costUsdMicros: number; audioMs: number; requests: number }
/** `source`: soniox (API that, ngay UTC) | local (uoc tinh cuc bo, ngay gio may). `reason`: ly do roi ve cuc bo. */
export interface UsageSourceOut {
  source: "soniox" | "local"; basis: "utc" | "local"; reason: string | null; weekStart: string; weekEnd: string;
  days: UsageDayOut[]; totalCostUsdMicros: number; totalAudioMs: number; apiCostUsdMicros: number; playgroundCostUsdMicros: number;
  totalRequests: number; models: UsageModelOut[]; estimated: boolean; cached: boolean; fetchedAt: number; sessionsCount: number; failedStartCount: number;
}
export const usageWeekSource = (weeksAgo: number) => invoke<UsageSourceOut>("usage_week_source", { weeksAgo });
export const usageExportWeekCsv = (weeksAgo: number) => invoke<string>("usage_export_week_csv", { weeksAgo });
/** Ban dau cho tab: so Soniox da luu (cached=true, fetchedAt = luc lay that) hoac uoc tinh cuc bo (reason `no_cache`). Khong dung mang. */
export const usageWeekCached = (weeksAgo: number) => invoke<UsageSourceOut>("usage_week_cached", { weeksAgo });
export const usageClear = () => invoke<number>("usage_clear");
export const driveStatus = (sessionId: string) => invoke<[string, string | null]>("drive_status", { sessionId });
export const driveSync = (sessionId: string) => invoke<DriveOut>("drive_sync", { sessionId });
export type SettingsPane = "accessibility" | "microphone" | "screen_recording";
export const openSystemSettings = (pane: SettingsPane) => invoke<void>("open_system_settings", { pane });
export const micPermissionRequest = () => invoke<string>("mic_permission_request");
export const driveConnectionStatus = () => invoke<DriveConnOut>("drive_connection_status");
export const driveSaveClientConfig = (clientId: string, clientSecret: string | null = null) =>
  invoke<DriveConnOut>("drive_save_client_config", { clientId, clientSecret });
export const driveConnect = () => invoke<DriveConnOut>("drive_connect");
export const driveDisconnect = () => invoke<DriveDisconnectOut>("drive_disconnect");
export const importDotnetPick = () => invoke<ImportOut | null>("import_dotnet_pick");
export const driveConnectCancel = () => invoke<void>("drive_connect_cancel");
export const driveSyncNow = () => invoke<DriveSyncAllOut>("drive_sync_now");

/** Ma loi tra ve la chuoi ngan (vd `key_missing`); dua ve mot khoa i18n `err.*`, khong ro thi `err.generic`. */
export function errCode(e: unknown): string {
  return typeof e === "string" ? e : (e as { message?: string })?.message ?? "unknown";
}
export function usd(micros: number): number { return micros / 1_000_000; }
export function mondayOf(offsetWeeks: number, now = new Date()): string {
  const d = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate()));
  d.setUTCDate(d.getUTCDate() - offsetWeeks * 7);
  return d.toISOString().slice(0, 10);
}

/** Khoa i18n cho mot ma loi; ma la khong biet thi dung `err.generic`. Khoa i18n khong chua dau gach duoi. */
export function errKeyOf(code: string, known: (key: string) => boolean): string {
  const k = "err." + code.replace(/_/g, "");
  return known(k) ? k : "err.generic";
}
export const driveKeyOf = (status: string) => "ses.drive." + status.replace(/_/g, "");

// ---- Cai dat: vo ung dung, phim tat, phu de, thuat ngu, DeepL, quyen (M3 loi goi giao dien) ----
export type ShortcutAction = "quick_translate" | "inline_translate" | "ask_ai" | "region_capture";
export interface ShortcutStatus { action: ShortcutAction; accelerator: string; enabled: boolean; registered: boolean; error_key: string | null; error_detail: string | null }
export type DisplayMode = "bilingual" | "translation_only" | "source_only";
export interface SubtitleSettings { display_mode: DisplayMode; font_size_pt: number; source_color: string; translation_color: string; opacity_percent: number; hidden_from_capture: boolean }
export interface TermInput { termSource: string; termTarget: string | null; keepOriginal: boolean; sourceLang: string; targetLang: string }
export interface DeeplUsage { character_count: number; character_limit: number }

export const SHORTCUT_ACTIONS: ShortcutAction[] = ["quick_translate", "inline_translate", "ask_ai", "region_capture"];
/** Mac dinh cua Rust (`ShortcutAction::default_accelerator`); dung de nut "Dat lai". */
export const SHORTCUT_DEFAULTS: Record<ShortcutAction, string> = {
  quick_translate: "CommandOrControl+Shift+T", inline_translate: "CommandOrControl+Shift+I",
  ask_ai: "CommandOrControl+Shift+A", region_capture: "CommandOrControl+Shift+S",
};
/** Khoa i18n `set.sc.a.*` cua mot hanh dong (khoa i18n khong chua dau gach duoi). */
export const shortcutKeyOf = (a: ShortcutAction) => "set.sc.a." + a.replace(/_/g, "");

export const shellSetLocale = (locale: string) => invoke<void>("shell_set_locale", { locale });
/** Bao vo ung dung dang ghi hay khong: dieu khien khay he thong va che do giu may thuc. */
export const shellSetRecording = (recording: boolean) => invoke<void>("shell_set_recording", { recording });
export const shellSetKeepAwake = (enabled: boolean) => invoke<void>("shell_set_keep_awake", { enabled });
export const autostartGet = () => invoke<boolean>("autostart_get");
export const autostartSet = (enabled: boolean) => invoke<boolean>("autostart_set", { enabled });
export const shortcutsStatus = () => invoke<ShortcutStatus[]>("shortcuts_status");
export const shortcutRebind = (action: ShortcutAction, accelerator: string) => invoke<ShortcutStatus[]>("shortcut_rebind", { action, accelerator });
export const shortcutSetEnabled = (action: ShortcutAction, enabled: boolean) => invoke<ShortcutStatus[]>("shortcut_set_enabled", { action, enabled });

export const subtitleOpen = () => invoke<void>("subtitle_open");
export const subtitleClose = () => invoke<void>("subtitle_close");
export const subtitleLines = () => invoke<LineDto[]>("subtitle_lines");
export const subtitleGet = () => invoke<SubtitleSettings>("subtitle_get");
export const subtitleFontStep = (delta: number) => invoke<SubtitleSettings>("subtitle_font_step", { delta });
export const subtitleSetMode = (mode: DisplayMode) => invoke<SubtitleSettings>("subtitle_set_mode", { mode });
export const subtitleSetOpacity = (percent: number) => invoke<SubtitleSettings>("subtitle_set_opacity", { percent });
export const subtitleSetColor = (which: "source" | "translation", hex: string) => invoke<SubtitleSettings>("subtitle_set_color", { which, hex });
export const subtitleResetColors = () => invoke<SubtitleSettings>("subtitle_reset_colors");
export const subtitleSetHidden = (hidden: boolean) => invoke<SubtitleSettings>("subtitle_set_hidden", { hidden });
export const subtitleResetPosition = () => invoke<void>("subtitle_reset_position");

export interface CaptureRow {
  screenCaptureId: number; offsetMs: number; absPath: string; widthPx: number; heightPx: number; byteSize: number; isAttached: boolean; thumbDataUrl: string;
}
/** Anh chup vung cua mot phien, theo moc thoi gian tuong doi (ms). */
export const sessionCaptures = (sessionId: string) => invoke<CaptureRow[]>("session_captures", { sessionId });
export const sessionCaptureAttach = (sessionId: string, screenCaptureId: number, attached: boolean) =>
  invoke<boolean>("session_capture_attach", { sessionId, screenCaptureId, attached });
export const captureBegin = () => invoke<void>("capture_begin");
export const glossaryAdd = (term: TermInput) => invoke<number>("glossary_add", { term });
export const glossaryDelete = (id: number) => invoke<boolean>("glossary_delete", { id });
/** Tra ve [da nhap, trung, dong loi]. */
export const glossaryImportCsv = (csv: string, sourceLang: string, targetLang: string) =>
  invoke<[number, number, number]>("glossary_import_csv", { csv, sourceLang, targetLang });

export const deeplSaveKey = (value: string) => invoke<void>("deepl_save_key", { value });
export const deeplHasKey = () => invoke<boolean>("deepl_has_key");
export const deeplClearKey = () => invoke<void>("deepl_clear_key");
export const deeplTestConnection = () => invoke<DeeplUsage>("deepl_test_connection");
export const axStatus = () => invoke<boolean>("quick_ax_status");
export const coreVersion = () => invoke<string>("core_version");

/** Doi ngon ngu giao dien va bao vo ung dung (khay he thong) cung luc; vo loi thi giao dien van doi. */
export async function setUiLocale(apply: (l: "en" | "pt-PT") => Promise<unknown>, l: "en" | "pt-PT") {
  await apply(l);
  try { await shellSetLocale(l); } catch { /* vo chua san sang: giao dien da doi */ }
}

export const KEEP_AWAKE_KEY = "rlt.ui.keepAwake";
