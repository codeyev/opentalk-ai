import { useI18n } from "../../i18n/I18nContext";
import { useSettings } from "../../settings/SettingsContext";

/** The switch that rules out every cloud provider. */
export function LocalOnlySetting() {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();

  return (
    <section className="panel" aria-labelledby="local-only-title">
      <h2 id="local-only-title">{t("providers.localOnly.title")}</h2>
      <label className="choice">
        <input
          type="checkbox"
          checked={settings.localOnly}
          aria-describedby="local-only-description"
          onChange={(event) => {
            updateSettings({ localOnly: event.target.checked });
          }}
        />
        {t("providers.localOnly.toggle")}
      </label>
      <p id="local-only-description" className="note">
        {t("providers.localOnly.description")}
      </p>
      {settings.localOnly && <p className="warning">{t("providers.localOnly.noTranscription")}</p>}
    </section>
  );
}
