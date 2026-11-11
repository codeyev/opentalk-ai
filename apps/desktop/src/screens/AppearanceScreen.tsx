import { RadioGroup } from "../components/RadioGroup";
import { useI18n } from "../i18n/I18nContext";
import { LOCALE_NAMES } from "../i18n/translate";
import { THEMES, UI_LOCALES } from "../settings/settings";
import { useSettings } from "../settings/SettingsContext";

export function AppearanceScreen() {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();

  return (
    <div className="form">
      <RadioGroup
        legend={t("appearance.theme")}
        options={THEMES}
        value={settings.theme}
        onChange={(theme) => {
          updateSettings({ theme });
        }}
        optionLabel={(theme) => t(`appearance.theme.${theme}`)}
      />
      <RadioGroup
        legend={t("appearance.language")}
        options={UI_LOCALES}
        value={settings.uiLocale}
        onChange={(uiLocale) => {
          updateSettings({ uiLocale });
        }}
        optionLabel={(locale) => LOCALE_NAMES[locale]}
      />
    </div>
  );
}
