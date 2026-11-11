import { type ReactNode, useMemo } from "react";
import { useSettings } from "../settings/SettingsContext";
import { I18nContext, type I18nContextValue } from "./I18nContext";
import { createTranslate } from "./translate";

/** Provides translations for the interface language chosen in settings. */
export function I18nProvider({ children }: { children: ReactNode }) {
  const locale = useSettings().settings.uiLocale;
  const value = useMemo<I18nContextValue>(() => ({ locale, t: createTranslate(locale) }), [locale]);
  return <I18nContext value={value}>{children}</I18nContext>;
}
