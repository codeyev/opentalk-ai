import type { UiLocale } from "../settings/settings";
import { az } from "./messages/az";
import { en, type MessageKey, type Messages } from "./messages/en";
import { ru } from "./messages/ru";

export type MessageParams = Record<string, string | number>;

export type Translate = (key: MessageKey, params?: MessageParams) => string;

const MESSAGES: Record<UiLocale, Messages> = { en, az, ru };

/** The language's own name, shown in the interface-language picker. */
export const LOCALE_NAMES: Record<UiLocale, string> = {
  en: "English",
  az: "Azərbaycanca",
  ru: "Русский",
};

/** Replaces each `{name}` placeholder in `template` with the matching parameter. */
function fillPlaceholders(template: string, params: MessageParams): string {
  return template.replace(/\{(\w+)\}/g, (placeholder, name: string) =>
    name in params ? String(params[name]) : placeholder,
  );
}

export function createTranslate(locale: UiLocale): Translate {
  const messages = MESSAGES[locale];
  return (key, params) => {
    const template = messages[key];
    return params === undefined ? template : fillPlaceholders(template, params);
  };
}
