// Du lieu mau co dinh cho lop Tauri gia (cung hinh dang voi cac DTO trong src/lib/api.ts).
const row = (sessionId: string, title: string, startedAt: string, durationMs: number | null, status: string, lines: number, syncStatus = "not_connected") =>
  ({ sessionId, title, startedAt, durationMs, status, audioSourceMode: "mixed", targetLang: "pt-PT", lines, attachedImages: 0, syncStatus });

export const SESSIONS = [
  row("s-weekly", "Weekly product meeting", "2026-11-12T09:00:00Z", 3_725_000, "completed", 3),
  row("s-customer", "Customer call", "2026-11-11T14:30:00Z", 1_260_000, "completed", 1),
  row("s-retro", "Retro draft", "2026-11-10T08:15:00Z", null, "error", 0),
];

const ln = (id: number, seq: number, offsetMs: number, src: string, dst: string | null, extra: Record<string, unknown> = {}) =>
  ({ captionLineId: id, seq, lineType: "caption", source: "system", offsetMs, sourceLang: "en", sourceText: src, translatedText: dst, highlighted: false, note: null, ...extra });

export const DETAILS: Record<string, unknown> = {
  "s-weekly": {
    session: SESSIONS[0], syncLastAt: null,
    lines: [
      ln(1, 1, 3000, "Let us review the roadmap for next quarter.", "Vamos rever o roteiro do próximo trimestre."),
      { captionLineId: 2, seq: 2, lineType: "gap", source: "system", offsetMs: 21_000, sourceLang: null, sourceText: null, translatedText: null, highlighted: false, note: null },
      ln(3, 3, 32_000, "The pricing page ships on Friday.", "A página de preços sai na sexta-feira.", { source: "mic" }),
    ],
  },
  "s-customer": {
    session: SESSIONS[1], syncLastAt: null,
    lines: [ln(10, 1, 5000, "Thanks for calling, how can I help?", "Obrigado por ligar, como posso ajudar?")],
  },
  "s-retro": { session: SESSIONS[2], syncLastAt: null, lines: [] },
};
