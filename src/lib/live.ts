// Chu tam (token non-final cua Soniox) dung chung cho cua so chinh va cua so phu de. Logic thuan, co test.
// Backend gui moi lan MOT ban thay the day du cua chu tam; dong chot (`line`) den truoc `partial` cua cung lo token.

export interface PartialDto {
  source: "mic" | "system";
  offsetMs: number;
  sourceLang: string;
  sourceText: string;
  translatedText: string | null;
}

export interface LiveText {
  source: string;
  translation: string;
  origin: "mic" | "system";
  lang: string;
  offsetMs: number;
}

/** `null` = xoa chu tam. Ban moi THAY ban cu (khong noi them), nen chu dang hien doi tai cho. */
export function fromPartial(p: PartialDto | null): LiveText | null {
  if (!p || !p.sourceText.trim()) return null;
  return {
    source: p.sourceText,
    translation: p.translatedText?.trim() ?? "",
    origin: p.source,
    lang: p.sourceLang,
    offsetMs: p.offsetMs,
  };
}

const norm = (s: string) => s.replace(/\s+/g, " ").trim();

function dropPrefix(text: string, prefix: string): string | null {
  const t = norm(text), p = norm(prefix);
  if (!p || !t.startsWith(p)) return null;
  const rest = t.slice(p.length);
  // Chi cat o ranh gioi tu, khong cat giua tu.
  return rest === "" || rest.startsWith(" ") ? rest.trim() : null;
}

/**
 * Mot dong chot den: cau da chot khong con la chu tam. Chu tam dang gom ca cau do (cau cho dich) cong cau dang noi
 * tiep thi chi bo phan da chot; chu tam la cau khac thi giu nguyen. Nho vay khong co khoang hai ban cua cung mot cau.
 */
export function afterFinal(live: LiveText | null, finalSource: string, finalTranslation: string | null): LiveText | null {
  if (!live) return null;
  const rest = dropPrefix(live.source, finalSource);
  if (rest === null) return live;
  if (rest === "") return null;
  const tr = finalTranslation ? dropPrefix(live.translation, finalTranslation) : null;
  return { ...live, source: rest, translation: tr ?? live.translation };
}
