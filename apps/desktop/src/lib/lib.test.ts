import { describe, expect, it } from "vitest";
import { canonicalLanguageTag, languageDisplayName } from "./language";
import { formatOffset } from "./time";

describe("canonicalLanguageTag", () => {
  it("canonicalises valid tags", () => {
    expect(canonicalLanguageTag(" az ")).toBe("az");
    expect(canonicalLanguageTag("AR-eg")).toBe("ar-EG");
  });

  it("returns null for invalid or empty input", () => {
    expect(canonicalLanguageTag("")).toBeNull();
    expect(canonicalLanguageTag("not a language")).toBeNull();
  });
});

describe("languageDisplayName", () => {
  it("names a language in the display language", () => {
    expect(languageDisplayName("ru", "en")).toBe("Russian");
  });
});

describe("formatOffset", () => {
  it("formats milliseconds as minutes and seconds", () => {
    expect(formatOffset(0)).toBe("00:00");
    expect(formatOffset(4_000)).toBe("00:04");
    expect(formatOffset(125_900)).toBe("02:05");
  });

  it("never goes below zero", () => {
    expect(formatOffset(-500)).toBe("00:00");
  });
});
