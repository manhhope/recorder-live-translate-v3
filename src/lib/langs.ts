// Danh sach ngon ngu hien thi trong bo chon. Ten hien thi di qua khoa i18n `lang.*`.
export type Lang = { code: string; flag: string; key: string; popular: boolean };

export const LANGS: Lang[] = [
  { code: "en", flag: "🇬🇧", key: "lang.en", popular: true },
  { code: "ja", flag: "🇯🇵", key: "lang.ja", popular: true },
  { code: "ko", flag: "🇰🇷", key: "lang.ko", popular: true },
  { code: "zh", flag: "🇨🇳", key: "lang.zh", popular: true },
  { code: "vi", flag: "🇻🇳", key: "lang.vi", popular: true },
  { code: "ar", flag: "🇸🇦", key: "lang.ar", popular: false },
  { code: "pt-PT", flag: "🇵🇹", key: "lang.pt", popular: false },
  { code: "de", flag: "🇩🇪", key: "lang.de", popular: false },
  { code: "hi", flag: "🇮🇳", key: "lang.hi", popular: false },
  { code: "id", flag: "🇮🇩", key: "lang.id", popular: false },
  { code: "fr", flag: "🇫🇷", key: "lang.fr", popular: false },
  { code: "es", flag: "🇪🇸", key: "lang.es", popular: false },
  { code: "th", flag: "🇹🇭", key: "lang.th", popular: false },
];

export const DEFAULT_SRC = "en";
export const DEFAULT_DST = "pt-PT";
export const AUTO = "auto";
