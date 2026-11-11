import { describe, expect, it } from "vitest";
import { hasTashkeel, isArabic } from "./arabic";

describe("hasTashkeel", () => {
  it("detects Arabic diacritics", () => {
    expect(hasTashkeel("مَرْحَبًا")).toBe(true);
  });

  it("is false for Arabic without diacritics and for other scripts", () => {
    expect(hasTashkeel("مرحبا")).toBe(false);
    expect(hasTashkeel("GitHub 1,200 USD")).toBe(false);
    // Hamza and madda are parts of letters, not vowel marks.
    expect(hasTashkeel("آمن أحمد")).toBe(false);
  });
});

describe("isArabic", () => {
  it("recognizes Arabic in any regional form", () => {
    expect(isArabic("ar")).toBe(true);
    expect(isArabic("ar-EG")).toBe(true);
    expect(isArabic("AR-sa")).toBe(true);
    expect(isArabic("fa")).toBe(false);
    expect(isArabic("az")).toBe(false);
  });
});
