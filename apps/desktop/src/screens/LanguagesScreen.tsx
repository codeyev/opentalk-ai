import { useId } from "react";
import { LanguageField } from "../components/LanguageField";
import { RadioGroup } from "../components/RadioGroup";
import { useI18n } from "../i18n/I18nContext";
import { useReplyProvider } from "../providers/useReplyProvider";
import { useTranslationProvider } from "../providers/useTranslationProvider";
import {
  ARABIC_VARIETIES,
  AUTO_LANGUAGE,
  MAX_SHORT_TEXT_LENGTH,
  TASHKEEL_LEVELS,
} from "../settings/settings";
import { useSettings } from "../settings/SettingsContext";

/** Language chosen when the user turns automatic detection off. */
const DEFAULT_SOURCE_LANGUAGE = "en";

export function LanguagesScreen() {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();
  const dialectId = useId();
  const isAutoSource = settings.sourceLanguage === AUTO_LANGUAGE;

  // A dialect can only be chosen if a provider that writes Arabic accepts one.
  const translation = useTranslationProvider().provider;
  const replies = useReplyProvider().provider;
  const acceptsDialect =
    translation?.acceptsArabicDialect === true || replies?.acceptsArabicDialect === true;
  const variety = acceptsDialect ? settings.arabicVariety : "msa";

  return (
    <div className="form">
      <p className="intro">{t("languages.intro")}</p>

      <label className="choice">
        <input
          type="checkbox"
          checked={isAutoSource}
          onChange={(event) => {
            updateSettings({
              sourceLanguage: event.target.checked ? AUTO_LANGUAGE : DEFAULT_SOURCE_LANGUAGE,
            });
          }}
        />
        {t("languages.source")}: {t("languages.sourceAuto")}
      </label>
      {!isAutoSource && (
        <LanguageField
          label={t("languages.source")}
          value={settings.sourceLanguage}
          onChange={(sourceLanguage) => {
            updateSettings({ sourceLanguage });
          }}
        />
      )}
      <LanguageField
        label={t("languages.target")}
        value={settings.targetLanguage}
        onChange={(targetLanguage) => {
          updateSettings({ targetLanguage });
        }}
      />
      <LanguageField
        label={t("languages.reply")}
        value={settings.replyLanguage}
        onChange={(replyLanguage) => {
          updateSettings({ replyLanguage });
        }}
      />

      <section className="panel form" aria-labelledby="arabic-title">
        <h2 id="arabic-title">{t("languages.arabic.title")}</h2>
        <p className="note">{t("languages.arabic.scope")}</p>
        <RadioGroup
          legend={t("languages.arabic.variety")}
          options={ARABIC_VARIETIES}
          value={variety}
          disabledOptions={acceptsDialect ? [] : ["dialect"]}
          onChange={(arabicVariety) => {
            updateSettings({ arabicVariety });
          }}
          optionLabel={(variety) => t(`languages.arabic.variety.${variety}`)}
        />
        {!acceptsDialect && <p className="note">{t("languages.arabic.dialectUnavailable")}</p>}
        {variety === "dialect" && (
          <div className="field">
            <label htmlFor={dialectId}>{t("languages.arabic.dialectLabel")}</label>
            <input
              id={dialectId}
              type="text"
              className="input"
              maxLength={MAX_SHORT_TEXT_LENGTH}
              value={settings.arabicDialectLabel}
              aria-describedby={`${dialectId}-hint`}
              onChange={(event) => {
                updateSettings({ arabicDialectLabel: event.target.value });
              }}
            />
            <p id={`${dialectId}-hint`} className="note">
              {t("languages.arabic.dialectHint")} {t("languages.arabic.dialectUnverified")}
            </p>
          </div>
        )}
        <RadioGroup
          legend={t("languages.arabic.tashkeel")}
          options={TASHKEEL_LEVELS}
          value={settings.tashkeel}
          onChange={(tashkeel) => {
            updateSettings({ tashkeel });
          }}
          optionLabel={(level) => t(`languages.arabic.tashkeel.${level}`)}
        />
        <p className="note">{t("languages.arabic.tashkeelWarning")}</p>
      </section>
    </div>
  );
}
