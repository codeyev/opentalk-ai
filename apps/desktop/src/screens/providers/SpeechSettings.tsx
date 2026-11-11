import { useI18n } from "../../i18n/I18nContext";
import { useSettings } from "../../settings/SettingsContext";
import { useSpeech } from "../../speech/useSpeech";

/** Speech output: what it does, and the switch that adds a Speak button to replies. */
export function SpeechSettings() {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();
  const { isAvailable } = useSpeech();

  return (
    <section className="panel" aria-labelledby="speech-title">
      <h2 id="speech-title">{t("providers.speech")}</h2>
      <p>
        <strong>{t("providers.speech.systemVoices")}</strong> · {t("providers.location.local")}
      </p>
      <ul className="principles">
        <li>{t("providers.speech.description")}</li>
        <li>{t("providers.speech.voices")}</li>
      </ul>
      <div className="field">
        <label className="choice">
          <input
            type="checkbox"
            checked={settings.speechEnabled && isAvailable}
            disabled={!isAvailable}
            aria-describedby={isAvailable ? undefined : "speech-unavailable"}
            onChange={(event) => {
              updateSettings({ speechEnabled: event.target.checked });
            }}
          />
          {t("providers.speech.enable")}
        </label>
        {!isAvailable && (
          <p id="speech-unavailable" className="note">
            {t("providers.speech.unavailable")}
          </p>
        )}
      </div>
    </section>
  );
}
