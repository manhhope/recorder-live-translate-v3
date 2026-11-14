// Quy uoc khoa: `nhom.muc[.phu].loai`, chu thuong ASCII, cac doan cach nhau bang dau cham.
// Vi du: `nav.record`, `cfg.mode.one.tip`, `dlg.rec.title`, `common.cancel`.
//  - `.aria` = nhan cho VoiceOver, `.tip` = tooltip, `.hint` = chu goi y.
//  - nhom dung chung: `common.*`; theme: `theme.*`; ngon ngu: `lang.*`.
// Khoa la mot chuoi phang (khong long nhau) vi mot so khoa vua la la vua la cha (vd `lang.search`, `lang.search.q`).
// Thong diep dung cu phap ICU cua svelte-i18n: {ten} cho bien, khong ghep chuoi tay.
export const KEY_PATTERN = /^[a-z0-9]+(\.[A-Za-z0-9]+)+$/;

export function isValidKey(key: string): boolean {
  return KEY_PATTERN.test(key);
}
