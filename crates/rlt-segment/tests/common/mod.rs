//! Đọc vector vàng (fixtures/golden-70-10.json, sinh từ mã .NET thật) và so khớp từng ca.
#![allow(dead_code)]

use rlt_segment::*;
use serde_json::Value;

fn golden() -> Value {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/golden-70-10.json");
    serde_json::from_str(&std::fs::read_to_string(p).expect("thiếu fixtures/golden-70-10.json")).unwrap()
}

fn case(kind: &str, id: &str) -> Value {
    golden()[kind].as_array().unwrap().iter().find(|c| c["id"] == id).unwrap_or_else(|| panic!("không có vector {id}")).clone()
}

fn s(v: &Value) -> Option<String> {
    v.as_str().map(str::to_string)
}
fn i(v: &Value) -> Option<i64> {
    v.as_i64()
}

pub fn run_sentence_case(id: &str) {
    let c = case("sentences", id);
    let expected: Vec<String> = c["expected"].as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_string()).collect();
    assert_eq!(sentences(c["input"].as_str().unwrap()), expected, "vector {id}: {:?}", c["input"]);
}

fn token(v: &Value) -> SttToken {
    SttToken {
        text: s(&v["text"]).unwrap(),
        start_ms: i(&v["start_ms"]),
        end_ms: i(&v["end_ms"]),
        is_final: v["is_final"].as_bool().unwrap(),
        language: s(&v["language"]),
        translation_status: match v["status"].as_str().unwrap() {
            "translation" => TranslationStatus::Translation,
            "original" => TranslationStatus::Original,
            _ => TranslationStatus::None,
        },
    }
}

fn seg_json(g: &CompletedSegment) -> Value {
    serde_json::json!({
        "segment_index": g.segment_index, "source_lang": g.source_lang, "stream_end_ms": g.stream_end_ms,
        "stt_final_at": g.stt_final_at, "translation_received_at": g.translation_received_at,
        "source_text": g.source_text, "translated_text": g.translated_text, "stream_start_ms": g.stream_start_ms,
    })
}

pub fn run_tracker_case(id: &str) {
    let c = case("tracker", id);
    let mut t = SegmentTracker::new();
    for (n, step) in c["steps"].as_array().unwrap().iter().enumerate() {
        let toks: Vec<SttToken> = step["tokens"].as_array().unwrap().iter().map(token).collect();
        let got: Vec<Value> = t.on_tokens(&toks, step["now_ms"].as_i64().unwrap()).iter().map(seg_json).collect();
        let exp = &c["expected_batches"][n];
        assert_eq!(Value::Array(got), exp["segments"], "vector {id}, lô {n}: đoạn đóng");
        assert_eq!(t.current_translation(), exp["current_translation"].as_str().unwrap(), "vector {id}, lô {n}: dịch tạm");
    }
    if c["flush"].as_bool().unwrap() {
        let got: Vec<Value> = t.flush().iter().map(seg_json).collect();
        assert_eq!(Value::Array(got), c["expected_flush"], "vector {id}: flush");
    }
}

fn frag(v: &Value) -> TranscriptFragment {
    TranscriptFragment {
        id: v["id"].as_i64().unwrap(),
        seq: v["seq"].as_i64().unwrap() as i32,
        line_type: s(&v["line_type"]).unwrap(),
        source: s(&v["source"]).unwrap(),
        offset_ms: v["offset_ms"].as_i64().unwrap(),
        end_offset_ms: i(&v["end_offset_ms"]),
        source_lang: s(&v["source_lang"]),
        source_text: s(&v["source_text"]),
        translated_text: s(&v["translated_text"]),
    }
}

fn frag_json(f: &TranscriptFragment) -> Value {
    serde_json::json!({
        "id": f.id, "seq": f.seq, "line_type": f.line_type, "source": f.source, "offset_ms": f.offset_ms,
        "end_offset_ms": f.end_offset_ms, "source_lang": f.source_lang, "source_text": f.source_text,
        "translated_text": f.translated_text,
    })
}

pub fn run_merger_case(id: &str) {
    let c = case("merger", id);
    let frags: Vec<TranscriptFragment> = c["fragments"].as_array().unwrap().iter().map(frag).collect();
    let cfg = &golden()["config"];
    let lines = TranscriptLineMerger::merge_all(
        frags,
        cfg["max_words"].as_u64().unwrap() as usize,
        cfg["pause_seconds"].as_f64().unwrap(),
        cfg["max_duration_seconds"].as_f64().unwrap(),
        MergeMode::Sentence,
    );
    let got: Vec<Value> = lines
        .iter()
        .map(|l| {
            serde_json::json!({
                "fragments": l.fragments.iter().map(frag_json).collect::<Vec<_>>(),
                "source_text": l.source_text, "translated_text": l.translated_text,
            })
        })
        .collect();
    let exp = c["expected_lines"].as_array().unwrap();
    assert_eq!(got.len(), exp.len(), "vector {id}: số dòng");
    for (n, (g, e)) in got.iter().zip(exp).enumerate() {
        assert_eq!(g, e, "vector {id}: dòng {n}");
    }
}
