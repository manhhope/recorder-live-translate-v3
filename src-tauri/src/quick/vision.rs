//! OCR bang framework Vision (VNRecognizeTextRequest) qua objc2 `msg_send!`, khong can crate binding. Chay tren may,
//! khong gui anh di dau (AC-42.4). CHUA DO tren may that. Goi tu luong nen (khong phai luong chinh).

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{msg_send, ClassType};
use objc2_foundation::{NSArray, NSData, NSDictionary, NSError, NSString};
use rlt_quick::ocr::{OcrEngine, OcrError, OcrImage};

#[link(name = "Vision", kind = "framework")]
extern "C" {}

pub struct VisionOcr;

impl OcrEngine for VisionOcr {
    fn recognize(&self, image: &OcrImage, langs: &[String]) -> Result<String, OcrError> {
        if image.bytes.is_empty() {
            return Err(OcrError::BadImage);
        }
        unsafe {
            let req_cls = AnyClass::get(c"VNRecognizeTextRequest").ok_or(OcrError::Unsupported)?;
            let handler_cls = AnyClass::get(c"VNImageRequestHandler").ok_or(OcrError::Unsupported)?;
            let req: Retained<AnyObject> = msg_send![msg_send![req_cls, alloc], init];
            let _: () = msg_send![&*req, setRecognitionLevel: 0isize]; // accurate
            let _: () = msg_send![&*req, setUsesLanguageCorrection: true];
            if !langs.is_empty() {
                let l: Vec<Retained<NSString>> = langs.iter().map(|s| NSString::from_str(s)).collect();
                let arr = NSArray::from_retained_slice(&l);
                let _: () = msg_send![&*req, setRecognitionLanguages: &*arr];
            }
            let data = NSData::with_bytes(&image.bytes);
            let opts: Retained<NSDictionary<NSString, AnyObject>> = NSDictionary::new();
            let handler: Retained<AnyObject> = msg_send![msg_send![handler_cls, alloc], initWithData: &*data, options: &*opts];
            let reqs = NSArray::from_slice(&[&*req]);
            let mut err: Option<Retained<NSError>> = None;
            let ok: bool = msg_send![&*handler, performRequests: &*reqs, error: &mut err];
            if !ok {
                return Err(if err.is_some() { OcrError::Engine("vision_failed".into()) } else { OcrError::BadImage });
            }
            let results: Option<Retained<NSArray<AnyObject>>> = msg_send![&*req, results];
            let mut lines = Vec::new();
            for obs in results.iter().flat_map(|a| a.iter()) {
                let cands: Retained<NSArray<AnyObject>> = msg_send![&*obs, topCandidates: 1usize];
                if let Some(c) = cands.iter().next() {
                    let s: Retained<NSString> = msg_send![&*c, string];
                    lines.push(s.to_string());
                }
            }
            Ok(lines.join("\n"))
        }
    }
}

#[allow(dead_code)]
fn _t() -> &'static AnyClass {
    NSString::class()
}
