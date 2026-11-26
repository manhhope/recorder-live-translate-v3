// Cau noi giua giao dien Hoi AI va lenh Tauri (M3-07, M3-08). KHONG co ham nao doc khoa API:
// `setApiKey` chi gui khoa di, khong lay lai. Moi DTO chi co `has_key`.
import { Channel, invoke } from "@tauri-apps/api/core";

export type Provider = "gemini" | "openai" | "ollama" | "compat" | "claude_cli" | "codex_cli" | "antigravity_cli";
export type ChipKind = "summary" | "explain" | "action_items";

export interface ProviderSettings { provider: Provider; model_name: string; base_url: string | null; has_key: boolean; consented: boolean }
export interface ChipPrompt { chip: ChipKind; prompt: string | null }
export interface AiSettings {
  default_provider: Provider | null; context_max_chars: number; context_max_turns: number;
  providers: ProviderSettings[]; chip_prompts: ChipPrompt[];
}
export interface AiSettingsInput {
  default_provider: Provider | null; context_max_chars: number; context_max_turns: number;
  providers: { provider: Provider; model_name: string | null; base_url: string | null }[];
  chip_prompts: ChipPrompt[];
}
export interface Conversation { conversation_id: number; session_id: string; provider: string; model_name: string | null; title: string | null; updated_at: string }
export interface Message {
  message_id: number; conversation_id: number; role: "user" | "assistant"; reply_to_message_id: number | null;
  chip_kind: ChipKind | null; content: string; status: "streaming" | "completed" | "failed";
  model_name: string | null; error_code: string | null; context_truncated: boolean | null;
}
export interface StreamUpdate { assistant_message_id: number; text: string }
export interface AskOutcome { conversation_id: number; user_message_id: number; assistant_message_id: number; status: "completed" | "failed"; error_code: string | null; text: string }
export interface AskParams { session_id: string; conversation_id: number | null; text: string | null; chip: ChipKind | null; provider: Provider | null; ui_lang: string }

// Danh sach DAY DU cac gia tri co the co. Nha cung cap nao that su hien/chon duoc do BACKEND quyet (`AiSettings.providers`):
// `codex_cli` BAT mac dinh tu 3.0.6 (tinh nang Cargo `codex-cli`); van loc qua `enabledProviders` de build tat co van dung.
export const PROVIDERS: Provider[] = ["gemini", "openai", "ollama", "compat", "claude_cli", "codex_cli", "antigravity_cli"];
export const CLI_PROVIDERS: Provider[] = ["claude_cli", "codex_cli", "antigravity_cli"];
// Nha cung cap chay qua CLI da dang nhap: khong co khoa, gui ban ghi ra dich vu cua nha cung cap CLI (US-58..US-62).
export const isCli = (p: Provider | null | undefined): boolean => p === "claude_cli" || p === "codex_cli" || p === "antigravity_cli";
// Chi giu nha cung cap ma backend tra ve trong cai dat (nguon su that duy nhat cho viec bat/tat).
export const enabledProviders = (list: Provider[], s: { providers: { provider: Provider }[] } | null | undefined): Provider[] =>
  s ? list.filter(p => s.providers.some(x => x.provider === p)) : list;
export const COMPAT_DEFAULT_URL = "http://localhost:1234/v1";

// Kiem URL nhu phia Rust (`rlt_ai::compat::validate_base_url`, AC-57.1): http/https, khong user:pw, query, fragment, khoang trang.
export function validCompatUrl(raw: string): boolean {
  const t = raw.trim();
  if (!t || t.length > 500 || /\s/.test(t) || !/^https?:\/\//i.test(t)) return false;
  try {
    const u = new URL(t);
    return !!u.hostname && !u.username && !u.password && !u.search && !u.hash && !t.includes("?") && !t.includes("#");
  } catch { return false; }
}

export const getSettings = () => invoke<AiSettings>("ai_get_settings");
export const saveSettings = (input: AiSettingsInput) => invoke<void>("ai_save_settings", { input });
export const setApiKey = (provider: Provider, value: string) => invoke<void>("ai_set_api_key", { provider, value });
export const clearApiKey = (provider: Provider) => invoke<void>("ai_clear_api_key", { provider });
export const acceptConsent = (provider: Provider) => invoke<void>("ai_accept_consent", { provider });
export const testConnection = (provider: Provider) => invoke<void>("ai_test_connection", { provider });
// Danh sach model cua endpoint tuong thich (AC-57.4). Chi tra ten model.
export const listModels = (provider: Provider) => invoke<string[]>("ai_list_models", { provider });
export const listConversations = (sessionId: string) => invoke<Conversation[]>("ai_list_conversations", { sessionId });
export const listMessages = (conversationId: number) => invoke<Message[]>("ai_list_messages", { conversationId });
export const clearHistory = (conversationId: number) => invoke<number>("ai_clear_history", { conversationId });
export const deleteConversation = (conversationId: number) => invoke<boolean>("ai_delete_conversation", { conversationId });
export const cancelAsk = () => invoke<void>("ai_cancel");

export function ask(params: AskParams, onUpdate: (u: StreamUpdate) => void) {
  const ch = new Channel<StreamUpdate>();
  ch.onmessage = onUpdate;
  return invoke<AskOutcome>("ai_ask", { params, onUpdate: ch });
}
export function retry(assistantMessageId: number, provider: Provider | null, onUpdate: (u: StreamUpdate) => void) {
  const ch = new Channel<StreamUpdate>();
  ch.onmessage = onUpdate;
  return invoke<AskOutcome>("ai_retry", { assistantMessageId, provider, onUpdate: ch });
}

// Ma loi tu Rust -> khoa i18n. Ma la chuoi on dinh, khong bao gio chua khoa.
const ERR_KEY: Record<string, string> = {
  provider_unauthorized: "ai.err.unauthorized",
  provider_rate_limited: "ai.err.rate",
  provider_unreachable: "ai.err.unreachable",
  provider_bad_response: "ai.err.badresp",
  provider_model_not_found: "ai.err.model",
  provider_blocked: "ai.err.blocked",
  provider_quota: "ai.err.quota",
  provider_region: "ai.err.region",
  provider_rejected: "ai.err.rejected",
  provider_no_key: "ai.err.nokey",
  provider_not_configured: "ai.err.notconfigured",
  consent_required: "ai.err.consent",
  provider_cancelled: "ai.err.cancelled",
  invalid_input: "ai.err.input",
  store_error: "ai.err.store",
  cli_not_installed: "ai.err.cli.notinstalled",
  cli_not_logged_in: "ai.err.cli.notloggedin",
  cli_login_expired: "ai.err.cli.expired",
  cli_quota_exhausted: "ai.err.cli.quota",
  cli_timeout: "ai.err.cli.timeout",
  cli_unknown: "ai.err.cli.unknown",
};
// Loi cua CLI: ban Codex co chuoi rieng (".codex"), de thong bao goi dung ten chuong trinh.
export const errKey = (code: string | null | undefined, provider?: Provider | null): string => {
  if (code === "provider_unreachable" && provider === "compat") return "ai.err.unreachable.compat";
  const k = (code && ERR_KEY[code]) || "ai.err.generic";
  return k.startsWith("ai.err.cli.") ? k + cliSuffix(provider) : k;
};
export const providerLabelKey = (p: Provider) => `ai.provider.${p.replace("_", "")}`;
// Chuoi giao dien cua CLI theo nha cung cap: "claude_cli" -> "...cli", "codex_cli" -> "...cli.codex".
export const cliKey = (base: string, p: Provider | null | undefined) => base + cliSuffix(p);
function cliSuffix(p: Provider | null | undefined): string {
  return p === "codex_cli" ? ".codex" : p === "antigravity_cli" ? ".agy" : "";
}

// Dang nhap CLI trong app (3.0.6). Chi Claude (va Codex khi bat tinh nang) co nut dang nhap; agy khong co.
// Backend giu link va ma; giao dien chi hien link (de mo/chep), nhap ma dan, va khong bao gio ghi nhat ky.
export type LoginPhase = "starting" | "waiting_user" | "code_sent" | "verifying" | "done" | "failed" | "cancelled";
export interface LoginSnapshot {
  phase: LoginPhase; url: string | null; device_code: string | null; error: string | null; needs_code: boolean; remaining_secs: number;
}
export const loginStart = (provider: Provider) => invoke<void>("ai_login_start", { provider });
export const loginCode = (value: string) => invoke<void>("ai_login_code", { value });
export const loginPoll = () => invoke<LoginSnapshot | null>("ai_login_poll");
export const loginCancel = () => invoke<void>("ai_login_cancel");
export const loginOpen = () => invoke<void>("ai_login_open");
export const canLogin = (p: Provider): boolean => p === "claude_cli" || p === "codex_cli";

// Tu do CLI khi mo tab AI (3.0.6, luong giong Javis): duong dan tim thay, dong phien ban, trang thai dang nhap.
// Khong co token: trang thai chi tu MA THOAT cua lenh kiem (`claude auth status`, `codex login status`, `agy models`).
export type CliStatus = "ready" | "not_installed" | "not_logged_in" | "unknown";
export interface CliInfo { path: string | null; version: string | null; status: CliStatus }
export const cliStatus = (provider: Provider) => invoke<CliInfo>("ai_cli_status", { provider });
// Khoa i18n cua dong trang thai: "set.cli.st.ready", "set.cli.st.notloggedin"... (khoa khong dung dau gach duoi).
export const cliStatusKey = (st: CliStatus | "checking") => `set.cli.st.${st.replace(/_/g, "")}`;
// Nut Sign in la hanh dong CHINH khi CLI da cai ma chua dang nhap (hoac chua ro); da dang nhap thi chi la nut phu.
export const signInPrimary = (p: Provider, info: CliInfo | null | undefined): boolean =>
  canLogin(p) && !!info && info.status !== "ready" && info.status !== "not_installed";
export const loginActive = (s: LoginSnapshot | null): boolean =>
  !!s && ["starting", "waiting_user", "code_sent", "verifying"].includes(s.phase);
// Ma loi cua dang nhap -> khoa i18n.
const LOGIN_ERR: Record<string, string> = {
  cli_not_installed: "ai.err.cli.notinstalled", login_no_link: "set.login.err.nolink", login_expired: "set.login.err.expired",
  login_timeout: "set.login.err.timeout", login_not_saved: "set.login.err.notsaved", login_already_running: "set.login.err.busy",
  login_invalid_code: "set.login.err.code", login_unsupported: "set.login.err.unsupported", login_spawn_failed: "set.login.err.spawn",
};
export const loginErrKey = (code: string | null | undefined, provider?: Provider | null): string => {
  const k = (code && LOGIN_ERR[code]) || "set.login.err.unknown";
  return k.startsWith("ai.err.cli.") ? k + cliSuffix(provider) : k;
};
