// Cau noi giua giao dien Hoi AI va lenh Tauri (M3-07, M3-08). KHONG co ham nao doc khoa API:
// `setApiKey` chi gui khoa di, khong lay lai. Moi DTO chi co `has_key`.
import { Channel, invoke } from "@tauri-apps/api/core";

export type Provider = "gemini" | "openai" | "ollama" | "compat" | "claude_cli";
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

// Codex bi tat cho toi khi spike xac minh co dong lenh (AC-60.2): backend khong tra ra, nen khong co o day.
export const PROVIDERS: Provider[] = ["gemini", "openai", "ollama", "compat", "claude_cli"];
// Nha cung cap chay qua CLI da dang nhap: khong co khoa, gui ban ghi ra dich vu cua nha cung cap CLI (US-58..US-62).
export const isCli = (p: Provider | null | undefined): boolean => p === "claude_cli";
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
export const errKey = (code: string | null | undefined, provider?: Provider | null): string =>
  code === "provider_unreachable" && provider === "compat" ? "ai.err.unreachable.compat" : (code && ERR_KEY[code]) || "ai.err.generic";
export const providerLabelKey = (p: Provider) => `ai.provider.${p.replace("_", "")}`;
