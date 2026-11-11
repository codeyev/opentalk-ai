import { describe, expect, it } from "vitest";
import { UI_LOCALES } from "../settings/settings";
import { az } from "./messages/az";
import { en, type MessageKey } from "./messages/en";
import { ru } from "./messages/ru";
import { createTranslate } from "./translate";

const placeholders = (text: string) => [...text.matchAll(/\{(\w+)\}/g)].map((match) => match[1]);

describe("translations", () => {
  it("fills placeholders", () => {
    const t = createTranslate("en");
    expect(t("context.glossary.remove", { number: 2 })).toBe("Remove term 2");
  });

  it("leaves an unknown placeholder untouched", () => {
    const t = createTranslate("en");
    expect(t("context.glossary.remove", { other: 1 })).toBe("Remove term {number}");
  });

  it("translates into every interface language", () => {
    for (const locale of UI_LOCALES) {
      expect(createTranslate(locale)("nav.live")).not.toBe("");
    }
  });

  it("uses the same placeholders in every language", () => {
    for (const key of Object.keys(en) as MessageKey[]) {
      const expected = placeholders(en[key]).sort();
      expect(placeholders(az[key]).sort(), `az: ${key}`).toEqual(expected);
      expect(placeholders(ru[key]).sort(), `ru: ${key}`).toEqual(expected);
    }
  });
});
