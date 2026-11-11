import { useState } from "react";
import { useI18n } from "../i18n/I18nContext";
import { useSettings } from "../settings/SettingsContext";

const PRINCIPLES = ["capture", "audio", "cloud", "autosend"] as const;

export function PrivacyScreen() {
  const { t } = useI18n();
  const { resetSettings } = useSettings();
  const [wasReset, setWasReset] = useState(false);

  return (
    <div className="form">
      <p className="intro">{t("privacy.intro")}</p>
      <ul className="principles">
        {PRINCIPLES.map((principle) => (
          <li key={principle}>{t(`privacy.principle.${principle}`)}</li>
        ))}
      </ul>
      <p className="notice">{t("privacy.buildStatus")}</p>

      <section className="panel" aria-labelledby="history-title">
        <h2 id="history-title">{t("privacy.history.title")}</h2>
        <label className="choice">
          <input type="checkbox" disabled aria-describedby="history-note" />
          {t("privacy.history.toggle")}
        </label>
        <p id="history-note" className="note">
          {t("privacy.history.unavailable")}
        </p>
      </section>

      <section className="panel" aria-labelledby="stored-title">
        <h2 id="stored-title">{t("privacy.stored.title")}</h2>
        <p>{t("privacy.stored.body")}</p>
        <div className="button-row">
          <button
            type="button"
            className="button"
            onClick={() => {
              resetSettings();
              setWasReset(true);
            }}
          >
            {t("privacy.reset")}
          </button>
          <span role="status" className="copy-status">
            {wasReset && t("privacy.resetDone")}
          </span>
        </div>
      </section>
    </div>
  );
}
