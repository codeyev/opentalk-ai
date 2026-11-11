import { RadioGroup } from "../../components/RadioGroup";
import { useI18n } from "../../i18n/I18nContext";
import type { ProviderStatus } from "../../providers/providerClient";
import {
  describeTranscriptionProviders,
  effectiveTranscriptionProvider,
} from "../../providers/transcriptionProviders";
import { TRANSCRIPTION_PROVIDERS } from "../../settings/settings";
import { useSettings } from "../../settings/SettingsContext";
import { ApiKeyField } from "./ApiKeyField";

interface TranscriptionSettingsProps {
  /** The cloud provider; `null` if the core cannot be reached. */
  openai: ProviderStatus | null;
  /** Called after the saved key changed, so every provider's status can be reloaded. */
  onKeyChange: () => Promise<void>;
}

/**
 * Settings for transcription: which provider turns speech into text, what
 * that provider receives, and the switch that turns transcription on.
 */
export function TranscriptionSettings({ openai, onKeyChange }: TranscriptionSettingsProps) {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();
  const providers = describeTranscriptionProviders(settings, openai);
  const chosen = effectiveTranscriptionProvider(settings);
  const provider = providers[chosen];
  const canEnable = provider.choice !== null;

  return (
    <section className="panel" aria-labelledby="transcription-provider-title">
      <h2 id="transcription-provider-title">{t("providers.transcription")}</h2>
      <RadioGroup
        legend={t("providers.choice.label")}
        options={TRANSCRIPTION_PROVIDERS}
        value={chosen}
        disabledOptions={settings.localOnly ? ["openai"] : []}
        onChange={(transcriptionProvider) => {
          updateSettings({ transcriptionProvider });
        }}
        optionLabel={(id) =>
          t(providers[id].isCloud ? "providers.choice.cloud" : "providers.choice.local", {
            provider: providers[id].name,
          })
        }
      />

      {chosen === "whisper" ? (
        <>
          <p className="notice">
            {settings.whisperModel === ""
              ? t("providers.whisper.needsModel")
              : t("providers.whisper.processing", { model: settings.whisperModel })}
          </p>
          <ul className="principles">
            <li>{t("providers.whisper.finalOnly")}</li>
            <li>{t("providers.whisper.languages")}</li>
          </ul>
        </>
      ) : (
        <>
          <p className="notice">{t("providers.disclosure", { provider: provider.name })}</p>
          {openai === null ? (
            <p className="note">{t("providers.unavailable")}</p>
          ) : (
            <>
              <ul className="principles">
                <li>
                  {t(
                    openai.streamsPartialResults
                      ? "providers.capability.partial"
                      : "providers.capability.noPartial",
                  )}
                </li>
                <li>
                  {t(
                    openai.detectsLanguage
                      ? "providers.capability.languageDetection"
                      : "providers.capability.noLanguageDetection",
                  )}
                </li>
                <li>
                  {openai.languages === null
                    ? t("providers.capability.languagesUnknown")
                    : t("providers.capability.languages", {
                        languages: openai.languages.join(", "),
                      })}
                </li>
              </ul>
              <ApiKeyField
                target={{ owner: "openai" }}
                hasApiKey={openai.hasApiKey}
                onChange={onKeyChange}
              />
            </>
          )}
        </>
      )}

      <div className="field">
        <label className="choice">
          <input
            type="checkbox"
            checked={settings.transcriptionEnabled && canEnable}
            disabled={!canEnable}
            aria-describedby={canEnable ? undefined : "transcription-blocked"}
            onChange={(event) => {
              updateSettings({ transcriptionEnabled: event.target.checked });
            }}
          />
          {t(chosen === "whisper" ? "providers.whisper.enable" : "providers.enable", {
            provider: provider.name,
          })}
        </label>
        {!canEnable && (
          <p id="transcription-blocked" className="note">
            {t(chosen === "whisper" ? "providers.whisper.needsModel" : "providers.enableNeedsKey")}
          </p>
        )}
      </div>
    </section>
  );
}
