/** Arabic diacritics: the short-vowel marks, tanween, shadda, sukun and the dagger alif. */
const TASHKEEL = /[ً-ْٰ]/;

/** Whether `text` contains Arabic diacritics (tashkeel). */
export function hasTashkeel(text: string): boolean {
  return TASHKEEL.test(text);
}

/** Whether a BCP 47 tag names Arabic, in any regional form. */
export function isArabic(tag: string): boolean {
  return tag.split("-")[0]?.toLowerCase() === "ar";
}
