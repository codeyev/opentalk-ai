import { describe, expect, it } from "vitest";
import {
  DEFAULT_SETTINGS,
  MAX_BUSINESS_CONTEXT_LENGTH,
  MAX_GLOSSARY_ENTRIES,
  parseSettings,
} from "./settings";

describe("parseSettings", () => {
  it("returns the defaults for input that is not an object", () => {
    for (const input of [null, undefined, "text", 42, []]) {
      expect(parseSettings(input)).toEqual(DEFAULT_SETTINGS);
    }
  });

  it("keeps valid values", () => {
    const stored = {
      ...DEFAULT_SETTINGS,
      theme: "dark",
      uiLocale: "az",
      rtlPreview: true,
      sourceLanguage: "ru",
      targetLanguage: "ar-EG",
      tashkeel: "full",
      glossary: [{ id: "1", term: "invoice", preferred: "hesab-faktura" }],
    };
    expect(parseSettings(stored)).toEqual(stored);
  });

  it("replaces invalid values with their defaults", () => {
    const parsed = parseSettings({
      theme: "neon",
      uiLocale: "xx",
      rtlPreview: "yes",
      sourceLanguage: "not a language",
      targetLanguage: 7,
      tashkeel: null,
      glossary: "none",
    });
    expect(parsed).toEqual(DEFAULT_SETTINGS);
  });

  it("canonicalises language tags", () => {
    expect(parseSettings({ targetLanguage: "AR-eg" }).targetLanguage).toBe("ar-EG");
  });

  it("truncates oversized text and glossary lists", () => {
    const entries = Array.from({ length: MAX_GLOSSARY_ENTRIES + 5 }, (_, index) => ({
      id: String(index),
      term: "term",
      preferred: "translation",
    }));
    const parsed = parseSettings({
      businessContext: "x".repeat(MAX_BUSINESS_CONTEXT_LENGTH + 1),
      glossary: entries,
    });
    expect(parsed.businessContext).toHaveLength(MAX_BUSINESS_CONTEXT_LENGTH);
    expect(parsed.glossary).toHaveLength(MAX_GLOSSARY_ENTRIES);
  });

  it("drops glossary entries without an id", () => {
    const parsed = parseSettings({ glossary: [{ term: "a" }, { id: "", term: "b" }, "junk"] });
    expect(parsed.glossary).toEqual([]);
  });
});
