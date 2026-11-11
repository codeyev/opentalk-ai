import { useI18n } from "../i18n/I18nContext";
import {
  type GlossaryEntry,
  MAX_BUSINESS_CONTEXT_LENGTH,
  MAX_GLOSSARY_ENTRIES,
  MAX_SHORT_TEXT_LENGTH,
} from "../settings/settings";
import { useSettings } from "../settings/SettingsContext";

export function BusinessContextScreen() {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();
  const { glossary } = settings;
  const isGlossaryFull = glossary.length >= MAX_GLOSSARY_ENTRIES;

  const addEntry = () => {
    const entry: GlossaryEntry = { id: crypto.randomUUID(), term: "", preferred: "" };
    updateSettings({ glossary: [...glossary, entry] });
  };

  const changeEntry = (id: string, changes: Partial<GlossaryEntry>) => {
    updateSettings({
      glossary: glossary.map((entry) => (entry.id === id ? { ...entry, ...changes } : entry)),
    });
  };

  const removeEntry = (id: string) => {
    updateSettings({ glossary: glossary.filter((entry) => entry.id !== id) });
  };

  return (
    <div className="form">
      <p className="intro">{t("context.intro")}</p>

      <div className="field">
        <label htmlFor="business-profile">{t("context.profile")}</label>
        <textarea
          id="business-profile"
          className="input"
          rows={8}
          maxLength={MAX_BUSINESS_CONTEXT_LENGTH}
          value={settings.businessContext}
          aria-describedby="business-profile-hint"
          onChange={(event) => {
            updateSettings({ businessContext: event.target.value });
          }}
        />
        <p id="business-profile-hint" className="note">
          {t("context.profileHint", { max: MAX_BUSINESS_CONTEXT_LENGTH })}
        </p>
      </div>

      <section className="panel" aria-labelledby="glossary-title">
        <h2 id="glossary-title">{t("context.glossary.title")}</h2>
        <p className="note">{t("context.glossary.intro")}</p>
        {glossary.length === 0 ? (
          <p className="empty">{t("context.glossary.empty")}</p>
        ) : (
          <ol className="glossary">
            {glossary.map((entry, index) => {
              const number = index + 1;
              return (
                <li key={entry.id}>
                  <fieldset className="glossary-entry">
                    <legend className="visually-hidden">
                      {t("context.glossary.entry", { number })}
                    </legend>
                    <label className="field">
                      {t("context.glossary.term")}
                      <input
                        type="text"
                        className="input"
                        maxLength={MAX_SHORT_TEXT_LENGTH}
                        value={entry.term}
                        onChange={(event) => {
                          changeEntry(entry.id, { term: event.target.value });
                        }}
                      />
                    </label>
                    <label className="field">
                      {t("context.glossary.preferred")}
                      <input
                        type="text"
                        className="input"
                        dir="auto"
                        maxLength={MAX_SHORT_TEXT_LENGTH}
                        value={entry.preferred}
                        onChange={(event) => {
                          changeEntry(entry.id, { preferred: event.target.value });
                        }}
                      />
                    </label>
                    <button
                      type="button"
                      className="button"
                      onClick={() => {
                        removeEntry(entry.id);
                      }}
                    >
                      {t("context.glossary.remove", { number })}
                    </button>
                  </fieldset>
                </li>
              );
            })}
          </ol>
        )}
        <button type="button" className="button" disabled={isGlossaryFull} onClick={addEntry}>
          {t("context.glossary.add")}
        </button>
        {isGlossaryFull && (
          <p className="note">{t("context.glossary.limit", { max: MAX_GLOSSARY_ENTRIES })}</p>
        )}
      </section>
    </div>
  );
}
