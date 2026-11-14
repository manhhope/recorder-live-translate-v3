// Sinh vector vàng 70 từ / nghỉ 10 giây bằng CHÍNH mã .NET (SegmentTracker, TranscriptLineMerger).
// Dùng: dotnet run -- <thư mục out2 của testdata> <thư mục crate rlt-segment>
using System.Globalization;
using System.Reflection;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.RegularExpressions;
using RecorderLiveTranslate.Core.Measurement;
using RecorderLiveTranslate.Core.Sessions;
using RecorderLiveTranslate.Core.Stt;

var out2 = args[0];
var crate = args[1];
const int MaxWords = 70; const double Pause = 10, MaxDur = 60;

var sentencesFn = typeof(SegmentTracker).GetMethod("Sentences", BindingFlags.NonPublic | BindingFlags.Static)!;
var names = new List<(string Kind, string Name)>();
string Id(string kind, string name) { var n = kind + "_" + names.Count(x => x.Kind == kind).ToString("D3") + "_" + Regex.Replace(name.ToLowerInvariant(), "[^a-z0-9]+", "_").Trim('_'); names.Add((kind, n)); return n; }

// ---------- bộ tách câu ----------
var sentenceInputs = new (string Name, string Text)[]
{
    ("en basic", "Hello there. How are you? I am fine! Thanks."),
    ("en no space capital", "End of line.Next sentence starts here."),
    ("en lower after dot", "version 3.5 is out. see notes."),
    ("en ellipsis", "Well… maybe. Or not..."),
    ("en ellipsis three dots", "Wait... Then go."),
    ("en multiple spaces", "One.   Two.\t\nThree."),
    ("en question bang", "Really?! Yes!"),
    ("en no terminator", "no punctuation at all"),
    ("en trailing space", "  Padded text. Here.  "),
    ("en abbreviations", "Dr. Smith went to Washington D.C. yesterday. It was cold."),
    ("en decimal", "The rate is 3.5 percent. Next."),
    ("pt-PT basic", "Bom dia a todos. Vamos começar a reunião? Sim!"),
    ("pt-PT accents capital", "Acabou.Último ponto. Ônibus parou."),
    ("pt-PT lowercase after", "isto acabou. ótimo."),
    ("vi basic", "Xin chào mọi người. Hôm nay chúng ta bàn về dự án. Được không?"),
    ("vi capital no space", "Xong rồi.Đúng vậy! Ăn cơm chưa?Rồi."),
    ("vi combining decomposed", "Đã xong.Ê kíp bắt đầu. Ổn."),
    ("vi lowercase after", "xong rồi. đúng vậy."),
    ("greek capital", "Τέλος.Αρχή νέα. ώρα."),
    ("cyrillic capital", "Конец.Начало. дальше?Да."),
    ("titlecase Lt", "ok.ǅx then. y"),
    ("roman numeral Other_Uppercase", "see part.Ⅰ and more. Ⅰ ok."),
    ("circled letter Other_Uppercase", "x.Ⓐ y.Ⓑ"),
    ("supplementary uppercase Deseret", "end.\U00010400bc then. \U00010400d"),
    ("digit after", "Done.1 more. 2 more."),
    ("only punctuation", "...?!"),
    ("empty", ""),
    ("whitespace only", "   "),
    ("nbsp separator", "A. B. C. D."),
    ("zero width after", "A.​B. C."),
    ("terminator inside quotes", "He said \"stop.\" Then left."),
    ("terminator close paren", "(done.) Next."),
    ("mixed lang", "Hello. Olá. Xin chào. Bonjour!"),
    ("long chain", string.Join(" ", Enumerable.Range(1, 12).Select(i => $"Câu số {i}."))),
};
var sentenceCases = new JsonArray();
var rustTests = new StringBuilder();
foreach (var (name, text) in sentenceInputs)
{
    var res = (List<string>)sentencesFn.Invoke(null, [text])!;
    var id = Id("sentence", name);
    sentenceCases.Add(new JsonObject { ["id"] = id, ["name"] = name, ["input"] = text, ["expected"] = new JsonArray(res.Select(s => (JsonNode?)JsonValue.Create(s)).ToArray()) });
    rustTests.AppendLine($"#[test]\nfn {id}() {{ run_sentence_case(\"{id}\"); }}\n");
}

// ---------- SegmentTracker ----------
JsonNode Tok(SttToken t) => new JsonObject
{
    ["text"] = t.Text, ["start_ms"] = t.StartMs, ["end_ms"] = t.EndMs, ["is_final"] = t.IsFinal,
    ["language"] = t.Language, ["status"] = t.TranslationStatus.ToString().ToLowerInvariant(),
};
JsonNode Seg(CompletedSegment s) => new JsonObject
{
    ["segment_index"] = s.SegmentIndex, ["source_lang"] = s.SourceLang, ["stream_end_ms"] = s.StreamEndMs,
    ["stt_final_at"] = s.SttFinalAt, ["translation_received_at"] = s.TranslationReceivedAt,
    ["source_text"] = s.SourceText, ["translated_text"] = s.TranslatedText, ["stream_start_ms"] = s.StreamStartMs,
};
SttToken Src(string text, long start, long end, string? lang = "en", bool fin = true) => new(text, start, end, fin, lang, TranslationStatus.Original, lang);
SttToken Tr(string text, string? lang = "pt", bool fin = true) => new(text, null, null, fin, lang, TranslationStatus.Translation, "en");
SttToken End() => new("<end>", null, null, true, null, TranslationStatus.None, null);

var trackerCases = new JsonArray();
void AddTracker(string name, List<(long Now, List<SttToken> Toks)> steps, bool flush)
{
    var t = new SegmentTracker();
    var batches = new JsonArray();
    var stepsJson = new JsonArray();
    foreach (var (now, toks) in steps)
    {
        var done = t.OnTokens(toks, now);
        stepsJson.Add(new JsonObject { ["now_ms"] = now, ["tokens"] = new JsonArray(toks.Select(Tok).ToArray()) });
        batches.Add(new JsonObject { ["segments"] = new JsonArray(done.Select(Seg).ToArray()), ["current_translation"] = t.CurrentTranslation });
    }
    var flushed = flush ? t.Flush() : [];
    var id = Id("tracker", name);
    trackerCases.Add(new JsonObject
    {
        ["id"] = id, ["name"] = name, ["steps"] = stepsJson, ["flush"] = flush,
        ["expected_batches"] = batches, ["expected_flush"] = new JsonArray(flushed.Select(Seg).ToArray()),
    });
    rustTests.AppendLine($"#[test]\nfn {id}() {{ run_tracker_case(\"{id}\"); }}\n");
}
List<(long, List<SttToken>)> One(params List<SttToken>[] batches) => batches.Select((b, i) => ((long)(1000 * (i + 1)), b)).ToList();

AddTracker("translation before end", One([Src(" Hello", 0, 400), Src(" world.", 400, 800), Tr("Olá mundo."), End()]), true);
AddTracker("translation after end then next", One(
    [Src(" Hello", 0, 400), End()], [Tr("Olá")], [Tr(" mundo.")], [Src(" Next", 1000, 1300)], [Tr("Seguinte"), End()]), true);
AddTracker("b15 no translation for first segment", One(
    [Src(" One.", 0, 300), End()], [Src(" Two.", 400, 800), Tr("Dois."), End()]), true);
AddTracker("ac24 4 held with extra sentences", One(
    [Src(" One.", 0, 300), End()], [Src(" Two.", 400, 800)], [Tr("Um."), Tr(" Dois."), End()]), true);
AddTracker("ac24 4 held no extra", One(
    [Src(" One.", 0, 300), End()], [Src(" Two.", 400, 800), Tr("Dois."), End()], [Src(" Three.", 900, 1200), Tr("Tres."), End()]), true);
AddTracker("held then flush", One([Src(" One.", 0, 300), End()], [Src(" Two.", 400, 800), Tr("Um. Dois.")]), true);
AddTracker("flush open current", One([Src(" Cut", 0, 200), Src(" off", 200, 400)]), true);
AddTracker("flush awaiting", One([Src(" Hi.", 0, 200), End(), Tr("Oi.")]), true);
AddTracker("non final ignored", One([Src(" draft", 0, 100, "en", false), Src(" final", 100, 300), Tr("x", "pt", false), Tr("final"), End()]), true);
AddTracker("empty end ignored", One([End(), End(), Src(" a", 0, 100), Tr("a"), End(), End()]), true);
AddTracker("language majority", One([Src(" a", 0, 100, "en"), Src(" b", 100, 200, "pt"), Src(" c", 200, 300, "pt"), Tr("t"), End()]), true);
AddTracker("language tie first wins", One([Src(" a", 0, 100, "pt"), Src(" b", 100, 200, "en"), Tr("t"), End()]), true);
AddTracker("language null", One([Src(" a", 0, 100, null), Tr("t"), End()]), true);
AddTracker("end ms only last non null", new() { (5, [new SttToken(" a", 10, 20, true, "en", TranslationStatus.Original, "en"), new SttToken(" b", 20, null, true, "en", TranslationStatus.Original, "en"), Tr("t"), End()]) }, true);
AddTracker("vi multi sentence held", One(
    [Src(" Xin chào.", 0, 500, "vi"), End()], [Src(" Hôm nay trời đẹp.", 600, 1500, "vi")],
    [Tr("Olá."), Tr(" Hoje está um dia bonito."), End()]), true);
AddTracker("vi held extra capital no space", One(
    [Src(" Một.", 0, 300, "vi"), End()], [Src(" Hai.", 400, 800, "vi")], [Tr("Um.Dois."), End()]), true);
AddTracker("two held in a row", One(
    [Src(" A.", 0, 100), End()], [Src(" B.", 200, 300)], [Tr("A. B."), End()], [Src(" C.", 400, 500)], [Tr("C2."), End()]), true);
AddTracker("no flush leaves open", One([Src(" a.", 0, 100), Tr("a."), End(), Src(" b", 200, 300)]), false);
AddTracker("current translation visible", One([Src(" a", 0, 100), Tr("par"), Tr("cial")]), true);

// Tạo ngẫu nhiên có seed, tất cả do .NET chạy thật.
string[] words = ["the", "meeting", "starts", "now", "please", "bom", "dia", "reunião", "xin", "chào", "dự", "án", "go", "ok."];
string[] ends = [".", "!", "?", "…", ""];
for (var seed = 1; seed <= 60; seed++)
{
    var rng = new Random(seed);
    var langs = new[] { "en", "pt", "vi" };
    var toks = new List<SttToken>();
    long ms = 0; var utter = rng.Next(1, 7);
    for (var u = 0; u < utter; u++)
    {
        var lang = langs[rng.Next(langs.Length)];
        void Trans()
        {
            var n = rng.Next(1, 4);
            for (var k = 0; k < n; k++)
            {
                var w = words[rng.Next(words.Length)];
                toks.Add(Tr((k == 0 ? "" : " ") + char.ToUpperInvariant(w[0]) + w[1..] + ends[rng.Next(ends.Length - 1)], "pt", rng.Next(10) != 0));
            }
        }
        var before = rng.Next(5) == 0;
        var nw = rng.Next(1, 8);
        for (var k = 0; k < nw; k++)
        {
            var w = " " + words[rng.Next(words.Length)] + (k == nw - 1 ? ends[rng.Next(ends.Length)] : "");
            toks.Add(Src(w, ms, ms + 200, lang, rng.Next(12) != 0)); ms += 250;
            if (rng.Next(6) == 0) Trans();
        }
        if (before) Trans();
        toks.Add(End());
        if (rng.Next(3) != 0) Trans();
        if (rng.Next(8) == 0) toks.Add(End());
    }
    var steps = new List<(long, List<SttToken>)>(); long now = 100;
    for (var i = 0; i < toks.Count;)
    {
        var take = Math.Min(toks.Count - i, rng.Next(1, 5));
        steps.Add((now, toks.GetRange(i, take))); i += take; now += rng.Next(50, 900);
    }
    AddTracker($"random seed {seed}", steps, rng.Next(4) != 0);
}

// ---------- TranscriptLineMerger 70/10 ----------
JsonNode Frag(TranscriptFragment f) => new JsonObject
{
    ["id"] = f.Id, ["seq"] = f.Seq, ["line_type"] = f.LineType, ["source"] = f.Source, ["offset_ms"] = f.OffsetMs,
    ["end_offset_ms"] = f.EndOffsetMs, ["source_lang"] = f.SourceLang, ["source_text"] = f.SourceText, ["translated_text"] = f.TranslatedText,
};
JsonNode Line(TranscriptLine l) => new JsonObject
{
    ["fragments"] = new JsonArray(l.Fragments.Select(Frag).ToArray()), ["source_text"] = l.SourceText, ["translated_text"] = l.TranslatedText,
};
var mergerCases = new JsonArray();
void AddMerger(string name, List<TranscriptFragment> frags)
{
    var lines = TranscriptLineMerger.MergeAll(frags, MaxWords, Pause, MaxDur, SegmentMergeModes.Sentence);
    var id = Id("merger", name);
    mergerCases.Add(new JsonObject { ["id"] = id, ["name"] = name, ["fragments"] = new JsonArray(frags.Select(Frag).ToArray()), ["expected_lines"] = new JsonArray(lines.Select(Line).ToArray()) });
    rustTests.AppendLine($"#[test]\nfn {id}() {{ run_merger_case(\"{id}\"); }}\n");
}
TranscriptFragment F(long id, long off, long? end, string? lang, string? src, string? tr, string type = "speech", string source = "system") =>
    new(id, (int)id + 1, type, source, off, end, lang, src, tr);

foreach (var name in new[] { "v2_ES2002a", "v2_TS3003a", "v2_IS1000a", "v2_vi_tin_tuc", "v2_vi_noi_dung_hon_hop" })
{
    var frags = new List<(int Idx, string Lang, string Src, string Tr)>();
    foreach (var line in File.ReadLines(Path.Combine(out2, name + ".log")))
    {
        var m = Regex.Match(line, @"^\[(\d+)\] ([a-z]+): (.*)$");
        if (m.Success) { frags.Add((int.Parse(m.Groups[1].Value, CultureInfo.InvariantCulture), m.Groups[2].Value, m.Groups[3].Value, "")); continue; }
        var t = Regex.Match(line, @"^\s+-> ?(.*)$");
        if (t.Success && frags.Count > 0) frags[^1] = frags[^1] with { Tr = t.Groups[1].Value };
    }
    var endsMap = new Dictionary<int, long>();
    var rows = File.ReadAllLines(Path.Combine(out2, name + ".csv"));
    var header = rows[0].Split(',');
    int iIdx = Array.IndexOf(header, "segment_index"), iEnd = Array.IndexOf(header, "t_speech_end");
    foreach (var row in rows.Skip(1)) { var c = row.Split(','); endsMap[int.Parse(c[iIdx], CultureInfo.InvariantCulture)] = long.Parse(c[iEnd], CultureInfo.InvariantCulture); }
    AddMerger("real log " + name, frags.Select(f => new TranscriptFragment(f.Idx, f.Idx + 1, "speech", "system", endsMap[f.Idx], endsMap[f.Idx], f.Lang, f.Src, f.Tr)).ToList());
}

AddMerger("mid word join", [F(0, 0, 500, "en", "forecast", "previsão"), F(1, 600, 900, "en", "ing.", "ao")]);
AddMerger("gap over pause opens new line", [F(0, 0, 500, "en", "Hello there.", "Olá."), F(1, 11_000, 11_500, "en", "Back again.", "De novo.")]);
AddMerger("gap exactly pause joins", [F(0, 0, 500, "en", "Hello there.", "Olá."), F(1, 10_500, 11_000, "en", "Back again.", "De novo.")]);
AddMerger("duration over cap opens new line", [F(0, 0, 30_000, "en", "Long one.", "A."), F(1, 31_000, 61_000, "en", "Second.", "B.")]);
AddMerger("different source", [F(0, 0, 500, "en", "A.", "a"), F(1, 600, 900, "en", "B.", "b", source: "mic")]);
AddMerger("different lang", [F(0, 0, 500, "en", "A.", "a"), F(1, 600, 900, "pt", "B.", "b")]);
AddMerger("gap line type", [F(0, 0, 500, "en", "A.", "a"), F(1, 600, 900, null, null, null, type: "gap"), F(2, 1000, 1200, "en", "C.", "c")]);
AddMerger("null and blank texts", [F(0, 0, null, "en", "  ", null), F(1, 100, null, "en", null, "x"), F(2, 200, 300, "en", " hi ", " oi ")]);
string Words(int n, string p) => string.Join(" ", Enumerable.Range(1, n).Select(i => p + i));
AddMerger("single fragment 150 words", [F(0, 0, 9000, "en", Words(150, "w"), Words(150, "t"))]);
AddMerger("single fragment 70 words exact", [F(0, 0, 9000, "en", Words(70, "w"), "ok")]);
AddMerger("single fragment 71 words", [F(0, 0, 9000, "en", Words(71, "w"), "ok")]);
AddMerger("two fragments 40 plus 40", [F(0, 0, 4000, "en", Words(40, "a"), "A"), F(1, 4100, 8000, "en", Words(40, "b"), "B")]);
AddMerger("two fragments 35 plus 35 fit", [F(0, 0, 4000, "en", Words(35, "a"), "A"), F(1, 4100, 8000, "en", Words(35, "b"), "B")]);
AddMerger("split with mid word at cap", [F(0, 0, 4000, "en", Words(69, "a") + " fore", "A"), F(1, 4100, 8000, "en", "casting more words.", "B")]);
AddMerger("pt PT text", [F(0, 0, 500, "pt", "Bom dia a todos.", "Good morning."), F(1, 700, 1500, "pt", "vamos começar.", "Let's start.")]);

for (var seed = 1; seed <= 25; seed++)
{
    var rng = new Random(1000 + seed);
    var frags = new List<TranscriptFragment>(); long off = 0;
    var n = rng.Next(3, 25);
    for (var i = 0; i < n; i++)
    {
        var nw = rng.Next(1, 60);
        var text = string.Join(" ", Enumerable.Range(0, nw).Select(_ => words[rng.Next(words.Length)]));
        if (rng.Next(6) == 0) text = text.ToLowerInvariant();
        var dur = rng.Next(200, 9000);
        var lang = rng.Next(10) == 0 ? "pt" : "en";
        frags.Add(F(i, off, rng.Next(5) == 0 ? null : off + dur, lang, text, rng.Next(8) == 0 ? null : "tr " + text, source: rng.Next(12) == 0 ? "mic" : "system"));
        off += dur + (rng.Next(4) == 0 ? rng.Next(8000, 14000) : rng.Next(0, 3000));
    }
    AddMerger($"random seed {seed}", frags);
}

var root = new JsonObject
{
    ["generator"] = "tools/golden-dotnet (chạy mã .NET thật: SegmentTracker, TranscriptLineMerger)",
    ["config"] = new JsonObject { ["max_words"] = MaxWords, ["pause_seconds"] = Pause, ["max_duration_seconds"] = MaxDur, ["merge_mode"] = "sentence" },
    ["sentences"] = sentenceCases, ["tracker"] = trackerCases, ["merger"] = mergerCases,
};
var opts = new JsonSerializerOptions { WriteIndented = true, Encoder = System.Text.Encodings.Web.JavaScriptEncoder.UnsafeRelaxedJsonEscaping };
File.WriteAllText(Path.Combine(crate, "fixtures", "golden-70-10.json"), root.ToJsonString(opts) + "\n", new UTF8Encoding(false));
File.WriteAllText(Path.Combine(crate, "tests", "golden_cases.rs"),
    "// SINH TỰ ĐỘNG bởi tools/golden-dotnet (mỗi vector vàng một #[test]). Không sửa tay.\nmod common;\nuse common::*;\n\n" + rustTests, new UTF8Encoding(false));
Console.WriteLine($"sentences={sentenceCases.Count} tracker={trackerCases.Count} merger={mergerCases.Count}");
