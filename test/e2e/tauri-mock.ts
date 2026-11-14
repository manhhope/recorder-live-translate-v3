// Lop gia cua Tauri cho test e2e. Dat o tang test, KHONG nam trong src/. Ham `installTauriMock` duoc tiem vao trang
// bang page.addInitScript truoc khi ung dung chay, nen phai tu chua (khong tham chieu bien ngoai).
//
// Mo phong dung hop dong that cua @tauri-apps/api/core:
//   - invoke(cmd, args) -> window.__TAURI_INTERNALS__.invoke
//   - Channel dung transformCallback; tin nhan gui xuong co dang { index, message } (dung thu tu), giong Rust.
// Moi lenh duoc ghi vao `__rlt.calls`; lenh la (khong co trong bang) bi tu choi va ghi vao `__rlt.unhandled` de test bat.

export interface MockSeed {
  locale: "en" | "pt-PT";
  hasKey: boolean;
  consent: boolean;
  aiKeys: { gemini: boolean; openai: boolean };
  sessions: unknown[];
  details: Record<string, unknown>;
  // Lenh -> ma loi se bi tu choi (chuoi ngan nhu Rust). Doi duoc trong luc chay bang __rlt.fail.
  fail: Record<string, string>;
}

export function installTauriMock(seed: MockSeed): void {
  const w = window as unknown as Record<string, any>;
  const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v));
  try {
    if (!localStorage.getItem("rlt.ui.locale")) localStorage.setItem("rlt.ui.locale", seed.locale);
  } catch { /* bo qua */ }

  const callbacks = new Map<number, (m: unknown) => void>();
  let nextCb = 1;
  const state: any = {
    hasKey: seed.hasKey,
    consent: seed.consent,
    run: "idle",
    channelId: null as number | null,
    channelIndex: 0,
    lineCount: 0,
    lineIds: new Set<number>(),
    sessions: clone(seed.sessions),
    details: clone(seed.details),
    aiKeys: { ...seed.aiKeys },
    ai: {
      default_provider: null, context_max_chars: 60000, context_max_turns: 10,
      models: { gemini: "gemini-2.5-flash", openai: "gpt-4.1-mini", ollama: "qwen2.5:1.5b" },
      chips: { summary: null, explain: null, action_items: null },
    },
    usageCleared: false,
  };
  const rlt: any = { calls: [] as { cmd: string; args: any }[], unhandled: [] as string[], state, fail: { ...seed.fail } };
  w.__rlt = rlt;

  rlt.emit = (event: unknown) => {
    const ev = event as any;
    if (ev && ev.kind === "line") { state.lineIds.add(ev.line.captionLineId); state.lineCount = state.lineIds.size; }
    const cb = callbacks.get(state.channelId);
    if (!cb) throw new Error("khong co Channel dang mo");
    cb({ index: state.channelIndex++, message: event });
  };
  const emitState = (s: string) => { state.run = s; rlt.emit({ kind: "state", state: s }); };

  const aiSettings = () => ({
    default_provider: state.ai.default_provider,
    context_max_chars: state.ai.context_max_chars,
    context_max_turns: state.ai.context_max_turns,
    providers: (["gemini", "openai", "ollama"] as const).map(p => ({
      provider: p, model_name: state.ai.models[p], has_key: p === "ollama" ? false : !!state.aiKeys[p],
      base_url: p === "ollama" ? "http://localhost:11434" : null, consented: false,
    })),
    chip_prompts: (["summary", "explain", "action_items"] as const).map(c => ({ chip: c, prompt: state.ai.chips[c] })),
  });

  const lineText = (l: any) => `${l.sourceText ?? ""} ${l.translatedText ?? ""}`.toLowerCase();

  const handlers: Record<string, (a: any) => unknown> = {
    consent_get: () => state.consent,
    consent_accept: () => { state.consent = true; },
    soniox_has_api_key: () => state.hasKey,
    soniox_save_api_key: () => { state.hasKey = true; },
    soniox_clear_api_key: () => { state.hasKey = false; },
    soniox_test_connection: () => undefined,
    permissions_status: () => ({ mic: "granted", screen: "granted" }),
    record_prefs_get: () => ({ targetLang: "pt-PT", listenLangs: ["en"], mode: "mixed", uiLanguage: "pt-PT" }),
    record_prefs_set: () => undefined,
    session_state: () => [state.run, state.run === "idle" ? null : "sess-live"],
    session_start: (a) => {
      if (!state.hasKey) throw "key_missing";
      if (!state.consent) throw "consent_required";
      if (state.run !== "idle") throw "already_recording";
      state.channelId = Number(String(a.onEvent.id ?? a.onEvent));
      state.channelIndex = 0; state.lineCount = 0; state.lineIds = new Set<number>();
      // Rust phat su kien sau khi lenh tra ve; hoan lai de giong thu tu that.
      queueMicrotask(() => emitState("recording"));
      return "sess-live";
    },
    session_pause: () => { if (state.run !== "recording") throw "invalid_state"; queueMicrotask(() => emitState("paused")); },
    session_resume: () => { if (state.run !== "paused") throw "invalid_state"; queueMicrotask(() => emitState("recording")); },
    session_stop: () => {
      if (state.run === "idle") throw "not_recording";
      queueMicrotask(() => {
        emitState("stopping");
        rlt.emit({ kind: "stopped", sessionId: "sess-live", lines: state.lineCount, durationMs: 62_000, drained: true });
        emitState("idle");
      });
    },
    session_new: () => { const was = state.run !== "idle"; if (was) queueMicrotask(() => emitState("idle")); return was; },

    sessions_list: () => clone(state.sessions),
    session_get: (a) => {
      const d = state.details[a.sessionId];
      if (!d) throw "not_found";
      return clone(d);
    },
    session_rename: (a) => {
      const row = state.sessions.find((s: any) => s.sessionId === a.sessionId);
      if (!row) throw "not_found";
      row.title = a.title;
      if (state.details[a.sessionId]) state.details[a.sessionId].session.title = a.title;
    },
    session_delete: (a) => { state.sessions = state.sessions.filter((s: any) => s.sessionId !== a.sessionId); delete state.details[a.sessionId]; return true; },
    records_search: (a) => {
      const q = String(a.query).toLowerCase();
      const lines: any[] = [];
      for (const [sid, d] of Object.entries<any>(state.details)) {
        for (const l of d.lines) {
          if (l.lineType !== "gap" && lineText(l).includes(q)) {
            lines.push({ captionLineId: l.captionLineId, sessionId: sid, offsetMs: l.offsetMs, snippet: l.translatedText ?? l.sourceText });
          }
        }
      }
      return { lines, notes: [], terms: [] };
    },
    session_export: (a) => `/tmp/${a.sessionId}.${a.format}`,
    highlight_toggle: (a) => {
      const l = state.details[a.sessionId].lines.find((x: any) => x.captionLineId === a.captionLineId);
      l.highlighted = !l.highlighted;
      return l.highlighted;
    },
    glossary_list: () => [],
    drive_status: () => ["not_connected", null],
    shell_set_locale: () => undefined,
    autostart_get: () => false,
    core_version: () => "0.1.0",
    quick_ax_status: () => true,
    shortcuts_status: () => ["quick_translate", "inline_translate", "ask_ai", "region_capture"].map((action, i) => ({ action, accelerator: ["CmdOrCtrl+Shift+T", "CmdOrCtrl+Shift+I", "CmdOrCtrl+Shift+A", "CmdOrCtrl+Shift+R"][i], enabled: true, registered: true, error_key: null, error_detail: null })),
    subtitle_get: () => ({ display_mode: "bilingual", font_size_pt: 24, source_color: "#FFFFFF", translation_color: "#FFD60A", opacity_percent: 80, hidden_from_capture: false }),
    drive_connection_status: () => ({ state: "disconnected" }),
    deepl_has_key: () => false,
    drive_sync: () => ({ status: "not_connected", attempts: 0, oauthConnected: false }),
    import_dotnet_pick: () => ({ totalRows: 12, tables: [["sessions", 12]] }),

    usage_week: (a) => {
      if (state.usageCleared || a.date !== rlt.thisMonday) {
        return { days: Array.from({ length: 7 }, (_v, i) => ({ localDate: a.date + "#" + i, audioMs: 0, estCostUsdMicros: 0, calls: 0 })), totalAudioMs: 0, totalEstCostUsdMicros: 0, estimated: true };
      }
      const ms = [0, 1_800_000, 3_600_000, 900_000, 5_400_000, 0, 0];
      return {
        days: ms.map((audioMs, i) => ({ localDate: a.date + "#" + i, audioMs, estCostUsdMicros: Math.round(audioMs * 0.00007 * 1000), calls: audioMs ? 1 : 0 })),
        totalAudioMs: ms.reduce((x, y) => x + y, 0),
        totalEstCostUsdMicros: Math.round(ms.reduce((x, y) => x + y, 0) * 0.07),
        estimated: true,
      };
    },
    usage_export_csv: () => "date,audio_ms,cost\n2026-11-09,1800000,0.126\n",
    usage_clear: () => { state.usageCleared = true; return 4; },

    ai_get_settings: () => aiSettings(),
    ai_save_settings: (a) => {
      const i = a.input;
      state.ai.default_provider = i.default_provider; state.ai.context_max_chars = i.context_max_chars; state.ai.context_max_turns = i.context_max_turns;
      for (const p of i.providers) if (p.model_name) state.ai.models[p.provider] = p.model_name;
      for (const c of i.chip_prompts) state.ai.chips[c.chip] = c.prompt;
    },
    ai_set_api_key: (a) => { state.aiKeys[a.provider] = true; },
    ai_clear_api_key: (a) => { state.aiKeys[a.provider] = false; },
    ai_test_connection: () => undefined,
  };

  w.__TAURI_INTERNALS__ = {
    transformCallback(cb: (m: unknown) => void) { const id = nextCb++; callbacks.set(id, cb); return id; },
    unregisterCallback(id: number) { callbacks.delete(id); },
    async invoke(cmd: string, args: any = {}) {
      rlt.calls.push({ cmd, args: clone(args ?? {}) });
      const h = handlers[cmd];
      if (!h) { rlt.unhandled.push(cmd); throw "mock_unhandled:" + cmd; }
      // Mot nhip bat dong bo that, nhu mot lenh IPC.
      await Promise.resolve();
      const code = rlt.fail[cmd];
      if (code) throw code;
      return h(args);
    },
  };
  // Thu Hai dau tuan hien tai theo cung cong thuc voi giao dien (UTC), de tuan 0 co du lieu.
  const d = new Date(); const utc = new Date(Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate()));
  rlt.thisMonday = utc.toISOString().slice(0, 10);
}
