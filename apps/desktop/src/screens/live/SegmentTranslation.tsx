import { CopyButton } from "../../components/CopyButton";
import { DiacriticsNote } from "../../components/DiacriticsNote";
import { Icon } from "../../components/Icon";
import type { TranslationState } from "../../domain/conversation";
import { useI18n } from "../../i18n/I18nContext";
import { languageDisplayName } from "../../lib/language";

/**
 * The translation of one segment, or what is happening instead of it.
 * Renders nothing for a segment that is not being translated.
 */
export function SegmentTranslation({ state }: { state: TranslationState | undefined }) {
  const { t, locale } = useI18n();
  if (state === undefined) {
    return null;
  }
  return (
    <div className="translation" data-status={state.status}>
      <Icon name="languages" />
      <div>
        <p className="translation-label">
          {t("live.translation.title")}
          {state.status === "done" && ` · ${languageDisplayName(state.targetLanguage, locale)}`}
        </p>
        {state.status === "pending" && (
          <p className="translation-pending">{t("live.translation.pending")}</p>
        )}
        {state.status === "failed" && (
          <p className="translation-failed">
            {t("live.translation.failed", { reason: t(`live.reason.${state.reason}`) })}
          </p>
        )}
        {state.status === "done" && (
          <>
            <p className="translation-text" lang={state.targetLanguage} dir="auto">
              {state.text}
            </p>
            <DiacriticsNote text={state.text} />
            {state.unpreserved.length > 0 && (
              <p className="warning">
                {t("live.translation.check", { items: state.unpreserved.join(", ") })}
              </p>
            )}
            <div className="reply-actions">
              <CopyButton text={state.text} />
            </div>
          </>
        )}
      </div>
    </div>
  );
}
