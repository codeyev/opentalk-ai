/** Returns the canonical BCP 47 form of `input`, or null if it is not a valid tag. */
export function canonicalLanguageTag(input: string): string | null {
  const trimmed = input.trim();
  if (trimmed === "") {
    return null;
  }
  try {
    return Intl.getCanonicalLocales(trimmed)[0] ?? null;
  } catch {
    return null;
  }
}

/** Human-readable name of a language tag, written in the `displayLocale` language. */
export function languageDisplayName(tag: string, displayLocale: string): string {
  try {
    return new Intl.DisplayNames([displayLocale], { type: "language" }).of(tag) ?? tag;
  } catch {
    return tag;
  }
}

/** Whether two tags name the same language, ignoring region: `en` and `en-GB` do. */
export function sameLanguage(first: string, second: string): boolean {
  const language = (tag: string) => tag.split("-")[0]?.toLowerCase();
  return language(first) === language(second);
}
