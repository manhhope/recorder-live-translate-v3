// Du lieu gia cho M2. Se thay bang Tauri command o M3. Van ban hien thi di qua khoa i18n `demo.*`.
export type Lang = { code: string; flag: string; key: string; popular: boolean };

export const LANGS: Lang[] = [
  { code: "en", flag: "🇬🇧", key: "lang.en", popular: true },
  { code: "ja", flag: "🇯🇵", key: "lang.ja", popular: true },
  { code: "ko", flag: "🇰🇷", key: "lang.ko", popular: true },
  { code: "zh", flag: "🇨🇳", key: "lang.zh", popular: true },
  { code: "vi", flag: "🇻🇳", key: "lang.vi", popular: true },
  { code: "ar", flag: "🇸🇦", key: "lang.ar", popular: false },
  { code: "pt-PT", flag: "🇵🇹", key: "lang.pt", popular: false },
  { code: "de", flag: "🇩🇪", key: "lang.de", popular: false },
  { code: "hi", flag: "🇮🇳", key: "lang.hi", popular: false },
  { code: "id", flag: "🇮🇩", key: "lang.id", popular: false },
  { code: "fr", flag: "🇫🇷", key: "lang.fr", popular: false },
  { code: "es", flag: "🇪🇸", key: "lang.es", popular: false },
  { code: "th", flag: "🇹🇭", key: "lang.th", popular: false },
];

export const DEFAULT_SRC = "en";
export const DEFAULT_DST = "pt-PT";
export const AUTO = "auto";

export type TranscriptLine = { ts: string; lang: string; src: string; dst: string; provisional?: boolean };
export const LINES: TranscriptLine[] = [
  { ts: "00:11:02", lang: "en", src: "demo.ln.src1", dst: "demo.ln.dst1" },
  { ts: "00:11:09", lang: "en", src: "demo.ln.src2", dst: "demo.ln.dst2" },
  { ts: "00:11:21", lang: "ja", src: "demo.ln.src3", dst: "demo.ln.dst3" },
  { ts: "00:12:40", lang: "en", src: "demo.ln.src4", dst: "demo.ln.dst4", provisional: true },
];

export type SessionStatus = "saved" | "saving" | "error";
export const SESSIONS = [
  { id: 1, title: "demo.ses.1", date: Date.UTC(2026, 10, 14), dur: "00:48:10", src: "en", dst: "pt", status: "saved" as SessionStatus },
  { id: 2, title: "demo.ses.2", date: Date.UTC(2026, 10, 13), dur: "00:32:44", src: "ja", dst: "pt", status: "saving" as SessionStatus },
  { id: 3, title: "demo.ses.3", date: Date.UTC(2026, 10, 12), dur: "01:05:02", src: "en", dst: "pt", status: "error" as SessionStatus },
];

export type Provider = { id: string; name: string; keyShown: string; model: string; hasKey: boolean; url?: string };
export const PROVIDERS: Provider[] = [
  { id: "gemini", name: "Gemini", keyShown: "AIza...c42e", model: "gemini-flash", hasKey: true },
  { id: "openai", name: "OpenAI", keyShown: "", model: "gpt-mini", hasKey: false },
];
export const OLLAMA = { url: "http://localhost:11434", model: "qwen2.5:1.5b" };
export const PROMPT_KEYS = ["set.ai.p1", "set.ai.p2", "set.ai.p3"] as const;
export const CHIP_LABELS = ["ask.chip.summary", "ask.chip.explain", "ask.chip.todo"] as const;

// Usage Logs Soniox: gio ghi am theo ngay (do cao cot 0..100), thu Hai den Chu nhat.
export const WEEK_DAYS = ["day.mon", "day.tue", "day.wed", "day.thu", "day.fri", "day.sat", "day.sun"] as const;
export const WEEKS: { bars: number[]; cost: number }[] = [
  { bars: [60, 80, 40, 100, 50, 0, 0], cost: 1.98 },
  { bars: [30, 55, 70, 20, 45, 10, 0], cost: 1.42 },
  { bars: [], cost: 0 },
  { bars: [], cost: 0 },
];
export const SONIOX_KEY_SHOWN = "sk-...a91f";
