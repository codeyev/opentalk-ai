import { createContext, useContext } from "react";
import type { UiLocale } from "../settings/settings";
import type { Translate } from "./translate";

export interface I18nContextValue {
  locale: UiLocale;
  t: Translate;
}

export const I18nContext = createContext<I18nContextValue | null>(null);

export function useI18n(): I18nContextValue {
  const value = useContext(I18nContext);
  if (value === null) {
    throw new Error("useI18n must be used inside an I18nProvider.");
  }
  return value;
}
