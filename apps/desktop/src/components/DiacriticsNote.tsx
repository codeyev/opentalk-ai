import { useI18n } from "../i18n/I18nContext";
import { hasTashkeel } from "../lib/arabic";

/**
 * Tells the user that the diacritics in a generated Arabic text came from
 * a model. Renders nothing for text without diacritics.
 */
export function DiacriticsNote({ text }: { text: string }) {
  const { t } = useI18n();
  return hasTashkeel(text) ? <p className="note">{t("live.arabic.generatedDiacritics")}</p> : null;
}
