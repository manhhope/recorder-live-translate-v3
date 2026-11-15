// Dong phu cua mot phien trong danh sach/chi tiet Sessions: "<ngay> · <thoi luong> · <ngon ngu> · <N dong>[ · <N anh>]".
export type Translate = (key: string, values?: { n: number }) => string;

/** Nhan "N image(s)" theo so it/nhieu; 0 hoac am -> chuoi rong (khong hien gi). */
export function imagesLabel(n: number, t: Translate): string {
  if (!Number.isFinite(n) || n < 1) return "";
  return t(n === 1 ? "ses.images.one" : "ses.images.other", { n });
}

/** Ghep phan co san voi nhan anh; phien 0 anh thi giu nguyen. */
export function sessionSubline(base: string, attachedImages: number, t: Translate): string {
  const img = imagesLabel(attachedImages, t);
  return img ? `${base} · ${img}` : base;
}
