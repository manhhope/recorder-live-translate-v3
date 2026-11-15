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
  // Ban chup dong chu cua popup phu de (lenh subtitle_lines): cac dong da co truoc khi popup mo.
  subtitleLines?: unknown[];
  // Anh chup theo phien (lenh session_captures): sessionId -> CaptureRow[]. Mac dinh khong co anh.
  captures?: Record<string, unknown[]>;
  // So anh docx khong doc duoc ma session_export bao ve (mac dinh 0: khong canh bao).
  exportMissing?: number;
  // Usage Logs (stale-while-revalidate). cached = ban Soniox da luu (lenh usage_week_cached); live = so Soniox ma
  // usage_week_source tra ve; liveFail = ly do du phong cuc bo (vd "network") thay cho live; hold = giu usage_week_source
  // cho den khi test goi __rlt.releaseUsage(). Moi thoi diem la epoch giay. Mac dinh: khong cache, nguon tra du phong "no_key".
  usage?: { cached?: { costUsdMicros: number; fetchedAt: number }; live?: { costUsdMicros: number; fetchedAt: number }; liveFail?: string; hold?: boolean };
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
    captures: clone(seed.captures ?? {}) as Record<string, any[]>,
    aiKeys: { ...seed.aiKeys },
    ai: {
      default_provider: null, context_max_chars: 60000, context_max_turns: 10,
      models: { gemini: "gemini-2.5-flash", openai: "gpt-4.1-mini", ollama: "qwen2.5:1.5b" },
      chips: { summary: null, explain: null, action_items: null },
    },
    usageCleared: false,
    aiConsent: {} as Record<string, boolean>,
    convs: [] as any[],
    msgs: {} as Record<number, any[]>,
    listeners: [] as { event: string; id: number; cb: number }[],
    nextConv: 1,
    nextMsg: 1,
    exportMissing: seed.exportMissing ?? 0,
  };
  const rlt: any = { calls: [] as { cmd: string; args: any }[], unhandled: [] as string[], state, fail: { ...seed.fail } };
  w.__rlt = rlt;
  rlt.subtitleLines = clone(seed.subtitleLines ?? []);

  rlt.usage = clone(seed.usage ?? {});
  rlt.emit = (event: unknown) => {
    const ev = event as any;
    if (ev && ev.kind === "line") { state.lineIds.add(ev.line.captionLineId); state.lineCount = state.lineIds.size; }
    const cb = callbacks.get(state.channelId);
    if (!cb) throw new Error("khong co Channel dang mo");
    cb({ index: state.channelIndex++, message: event });
  };
  // Su kien Tauri (listen): giong `plugin:event|listen`; test phat bang __rlt.emitEvent(ten, payload).
  rlt.emitEvent = (event: string, payload: unknown) => {
    for (const l of state.listeners.filter((x: any) => x.event === event)) callbacks.get(l.cb)?.({ event, id: l.id, payload });
  };
  w.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() { /* khong can */ } };
  const emitState = (s: string) => { state.run = s; rlt.emit({ kind: "state", state: s }); };

  const aiSettings = () => ({
    default_provider: state.ai.default_provider,
    context_max_chars: state.ai.context_max_chars,
    context_max_turns: state.ai.context_max_turns,
    providers: (["gemini", "openai", "ollama"] as const).map(p => ({
      provider: p, model_name: state.ai.models[p], has_key: p === "ollama" ? false : !!state.aiKeys[p],
      base_url: p === "ollama" ? "http://localhost:11434" : null, consented: !!state.aiConsent[p],
    })),
    chip_prompts: (["summary", "explain", "action_items"] as const).map(c => ({ chip: c, prompt: state.ai.chips[c] })),
  });

  const lineText = (l: any) => `${l.sourceText ?? ""} ${l.translatedText ?? ""}`.toLowerCase();

  const handlers: Record<string, (a: any) => unknown> = {
  const localWeek = (wk: number, reason: string) => {
    const ms = wk === 0 && !state.usageCleared ? [0, 1_800_000, 3_600_000, 900_000, 5_400_000, 0, 0] : [0, 0, 0, 0, 0, 0, 0];
    const total = ms.reduce((x, y) => x + y, 0);
    return {
      source: "local", basis: "local", reason, weekStart: "2026-11-09", weekEnd: "2026-11-15",
      days: ms.map((audioMs, i) => ({ date: "2026-11-" + String(9 + i).padStart(2, "0"), audioMs, costUsdMicros: Math.round(audioMs * 0.07), apiCostUsdMicros: 0, playgroundCostUsdMicros: 0, requests: audioMs ? 1 : 0 })),
      totalCostUsdMicros: Math.round(total * 0.07), totalAudioMs: total, apiCostUsdMicros: 0, playgroundCostUsdMicros: 0, totalRequests: total ? 4 : 0,
      models: [], estimated: true, cached: false, fetchedAt: 0, sessionsCount: total ? 4 : 0, failedStartCount: 0,
    };
  };
  const sonioxWeek = (cost: number, fetchedAt: number, cached: boolean) => {
    const ms = [0, 1_200_000, 2_400_000, 600_000, 3_600_000, 0, 0];
    return {
      source: "soniox", basis: "utc", reason: null, weekStart: "2026-11-09", weekEnd: "2026-11-15",
      days: ms.map((audioMs, i) => ({ date: "2026-11-" + String(9 + i).padStart(2, "0"), audioMs, costUsdMicros: Math.round(cost / 4), apiCostUsdMicros: Math.round(cost / 4), playgroundCostUsdMicros: 0, requests: audioMs ? 1 : 0 })),
      totalCostUsdMicros: cost, totalAudioMs: ms.reduce((x, y) => x + y, 0), apiCostUsdMicros: cost, playgroundCostUsdMicros: 0, totalRequests: 4,
      models: [{ model: "stt-rt-v4", costUsdMicros: cost, audioMs: 7_800_000, requests: 4 }], estimated: false, cached, fetchedAt, sessionsCount: 0, failedStartCount: 0,
    };
  };

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
    // Hop dong: Rust `session_captures(session_id) -> Result<Vec<CaptureRow>, String>` (src-tauri/src/capture/mod.rs),
    // api.ts `sessionCaptures(sessionId) -> CaptureRow[]`. Phien test khong co anh chup nen tra mang rong (hop le), khong bia dong anh.
    session_captures: (a) => clone(state.captures[a.sessionId] ?? []),
    // Hop dong: Rust `session_capture_attach(session_id, screen_capture_id, attached) -> Result<bool, String>`.
    session_capture_attach: (a) => {
      const c = (state.captures[a.sessionId] ?? []).find((x: any) => x.screenCaptureId === a.screenCaptureId);
      if (!c) throw "not_found";
      c.isAttached = a.attached;
      return a.attached;
    },
    // Hop dong: Rust `export_default_dir() -> Option<String>`, `export_pick_dir() -> Option<String>` (None = huy),
    // `session_export(session_id, format, content, ui_lang, dir) -> Result<{path, missingImages, warning}, String>`.
    export_default_dir: () => "/Users/test/Documents",
    export_pick_dir: () => state.pickDir === undefined ? "/Users/test/Exports" : state.pickDir,
    session_export: (a) => ({
      path: `${a.dir ?? "/Users/test/Documents"}/${a.sessionId}.${a.format}`,
      missingImages: state.exportMissing ?? 0,
      warning: state.exportMissing ? "export_image_missing" : null,
    }),
    highlight_toggle: (a) => {
      const l = state.details[a.sessionId].lines.find((x: any) => x.captionLineId === a.captionLineId);
      l.highlighted = !l.highlighted;
      return l.highlighted;
    },
    glossary_list: () => [],
    drive_status: () => ["not_connected", null],
    shell_set_locale: () => undefined,
    shell_set_recording: () => undefined,
    shell_set_keep_awake: () => undefined,
    autostart_get: () => false,
    core_version: () => "0.1.0",
    quick_ax_status: () => true,
    shortcuts_status: () => ["quick_translate", "inline_translate", "ask_ai", "region_capture"].map((action, i) => ({ action, accelerator: ["CmdOrCtrl+Shift+T", "CmdOrCtrl+Shift+I", "CmdOrCtrl+Shift+A", "CmdOrCtrl+Shift+R"][i], enabled: true, registered: true, error_key: null, error_detail: null })),
    subtitle_font_step: () => undefined,
    subtitle_set_opacity: () => undefined,
    subtitle_lines: () => clone(rlt.subtitleLines ?? []),
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
    usage_week_cached: (a) => {
      const c = rlt.usage.cached;
      return c && !state.usageCleared ? sonioxWeek(c.costUsdMicros, c.fetchedAt, true) : localWeek(a.weeksAgo as number, "no_cache");
    },
    usage_week_source: async (a) => {
      if (rlt.usage.hold) await new Promise<void>(r => { rlt.releaseUsage = () => { rlt.usage.hold = false; r(); }; });
      const u = rlt.usage;
      if (u.liveFail) return localWeek(a.weeksAgo as number, u.liveFail);
      if (u.live) return sonioxWeek(u.live.costUsdMicros, u.live.fetchedAt, false);
      return localWeek(a.weeksAgo as number, "no_key");
    },
    usage_export_week_csv: () => "source,day_basis,date,audio_seconds,requests,cost_usd,api_cost_usd,playground_cost_usd\nlocal_estimate,local,2026-11-09,1800,1,0.126,0,0\n",
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
    "plugin:event|listen": (a) => { const id = state.listeners.length + 1; state.listeners.push({ event: a.event, id, cb: a.handler }); return id; },
    "plugin:window|start_dragging": () => undefined,
    "plugin:event|unlisten": (a) => { state.listeners = state.listeners.filter((l: any) => !(l.event === a.event && l.id === a.eventId)); },
    ai_accept_consent: (a) => { state.aiConsent[a.provider] = true; },
    ai_list_conversations: (a) => state.convs.filter((c: any) => c.session_id === a.sessionId).map(clone),
    ai_list_messages: (a) => clone(state.msgs[a.conversationId] ?? []),
    ai_clear_history: (a) => { const n = (state.msgs[a.conversationId] ?? []).length; state.msgs[a.conversationId] = []; return n; },
    ai_delete_conversation: (a) => { state.convs = state.convs.filter((c: any) => c.conversation_id !== a.conversationId); return true; },
    ai_cancel: () => undefined,
    ai_ask: (a) => {
      const p = a.params;
      let conv = state.convs.find((c: any) => c.conversation_id === p.conversation_id);
      if (!conv) {
        conv = { conversation_id: state.nextConv++, session_id: p.session_id, provider: p.provider ?? "gemini", model_name: null, title: p.text ?? p.chip, updated_at: "2026-11-14T10:00:00Z" };
        state.convs.push(conv); state.msgs[conv.conversation_id] = [];
      }
      const list = state.msgs[conv.conversation_id];
      const mk = (role: string, content: string, chip: string | null, reply: number | null) => ({
        message_id: state.nextMsg++, conversation_id: conv.conversation_id, role, reply_to_message_id: reply, chip_kind: chip,
        content, status: "completed", model_name: null, error_code: null, context_truncated: null,
      });
      const u = mk("user", p.text ?? p.chip, p.chip, null);
      const reply = "Reply to: " + (p.text ?? p.chip) + " (turn " + (list.filter((m: any) => m.role === "user").length + 1) + ")";
      const ai = mk("assistant", reply, null, u.message_id);
      list.push(u, ai);
      rlt.aiAsks = (rlt.aiAsks ?? []).concat([{ session_id: p.session_id, text: p.text, chip: p.chip, conversation_id: p.conversation_id }]);
      const ch = a.onUpdate?.id;
      if (ch !== undefined) callbacks.get(ch)?.({ index: 0, message: { assistant_message_id: ai.message_id, text: reply } });
      return { conversation_id: conv.conversation_id, user_message_id: u.message_id, assistant_message_id: ai.message_id, status: "completed", error_code: null, text: reply };
    },
  };

  w.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
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
