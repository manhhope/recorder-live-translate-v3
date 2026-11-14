//! Port Rust của bộ ghép đoạn bản .NET (M1): `SegmentTracker`, bộ tách câu và `TranscriptLineMerger`.
//!
//! Không dùng regex: lookbehind/lookahead của .NET (`(?<=[.!?…])(?:\s+|(?=\p{Lu}))`) được thay
//! bằng bộ quét tay trong [`sentence`]. Đồng hồ được tiêm từ ngoài (`now_ms` là tham số) nên test xác định.

pub mod merger;
pub mod offset;
pub mod sentence;
pub mod token;
pub mod tracker;

pub use merger::{
    count_words, is_mid_word, MergeMode, MergeUpdate, TranscriptFragment, TranscriptLine,
    TranscriptLineMerger, LINE_TYPE_SPEECH,
};
pub use offset::DropCompensator;
pub use sentence::sentences;
pub use token::{SttToken, TranslationStatus, END_TEXT};
pub use tracker::{CompletedSegment, SegmentTracker};
