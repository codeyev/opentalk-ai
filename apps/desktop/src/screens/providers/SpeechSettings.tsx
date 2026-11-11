import { useCallback } from "react";
import { RadioGroup } from "../../components/RadioGroup";
import { useI18n } from "../../i18n/I18nContext";
import { languageDisplayName } from "../../lib/language";
import { isAcceptableServiceUrl } from "../../lib/url";
import { hasApiKey } from "../../providers/providerClient";
import { useCustomProvider } from "../../providers/useCustomProvider";
import { useProviderStatus } from "../../providers/useProviderStatus";
import {
  DEFAULT_ELEVENLABS_MODEL,
  MAX_ENDPOINT_LENGTH,
  SPEECH_PROVIDERS,
} from "../../settings/settings";
import { useSettings } from "../../settings/SettingsContext";
import { useSpeechOutput } from "../../speech/SpeechContext";
import { isCloudSpeech, speechProviderName } from "../../speech/speechProviders";
import { ApiKeyField } from "./ApiKeyField";

interface TextSettingProps {
  id: string;
  label: string;
  hint: string;
  value: string;
  placeholder?: string;
  isInvalid?: boolean;
  onChange: (value: string) => void;
}

/** One labelled text setting with a hint underneath. */
function TextSetting({
  id,
  label,
  hint,
  value,
  placeholder,
  isInvalid,
  onChange,
}: TextSettingProps) {
  return (
    <div className="field">
      <label htmlFor={id}>{label}</label>
      <input
        id={id}
        type="text"
        className="input"
        spellCheck={false}
        maxLength={MAX_ENDPOINT_LENGTH}
        placeholder={placeholder}
        value={value}
        aria-invalid={isInvalid === true}
        aria-describedby={`${id}-hint`}
        onChange={(event) => {
          onChange(event.target.value);
        }}
      />
      <p id={`${id}-hint`} className={isInvalid === true ? "note note-error" : "note"}>
        {hint}
      </p>
    </div>
  );
}

/** The ElevenLabs voice, model and API key. */
function ElevenLabsSettings() {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();
  const fetchKeyStatus = useCallback(() => hasApiKey({ owner: "elevenlabs" }), []);
  const key = useProviderStatus(fetchKeyStatus);

  return (
    <>
      <TextSetting
        id="elevenlabs-voice"
        label={t("providers.speech.elevenlabs.voice")}
        hint={t("providers.speech.elevenlabs.voiceHint")}
        value={settings.elevenlabsVoice}
        onChange={(elevenlabsVoice) => {
          updateSettings({ elevenlabsVoice });
        }}
      />
      <TextSetting
        id="elevenlabs-model"
        label={t("providers.speech.model")}
        hint={t("providers.speech.elevenlabs.modelHint")}
        placeholder={DEFAULT_ELEVENLABS_MODEL}
        value={settings.elevenlabsModel}
        onChange={(elevenlabsModel) => {
          updateSettings({ elevenlabsModel });
        }}
      />
      <ApiKeyField
        target={{ owner: "elevenlabs" }}
        hasApiKey={key.provider === true}
        onChange={key.reload}
      />
    </>
  );
}

/** The address, model, voice and API key of an OpenAI-style speech service. */
function CompatibleSpeechSettings() {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();
  const baseUrl = settings.speechBaseUrl;
  const isAddressValid = isAcceptableServiceUrl(baseUrl);
  const key = useCustomProvider(baseUrl);

  return (
    <>
      <TextSetting
        id="speech-base-url"
        label={t("providers.custom.baseUrl")}
        hint={t(
          isAddressValid ? "providers.speech.baseUrlHint" : "providers.custom.baseUrlInvalid",
        )}
        isInvalid={!isAddressValid}
        value={baseUrl}
        onChange={(speechBaseUrl) => {
          updateSettings({ speechBaseUrl });
        }}
      />
      <TextSetting
        id="speech-model"
        label={t("providers.speech.model")}
        hint={t("providers.speech.compatible.modelHint")}
        value={settings.speechModel}
        onChange={(speechModel) => {
          updateSettings({ speechModel });
        }}
      />
      <TextSetting
        id="speech-voice"
        label={t("providers.speech.compatible.voice")}
        hint={t("providers.speech.compatible.voiceHint")}
        value={settings.speechVoice}
        onChange={(speechVoice) => {
          updateSettings({ speechVoice });
        }}
      />
      {isAddressValid && (
        <ApiKeyField
          // Each service has its own key, so the field starts afresh for a new address.
          key={baseUrl}
          target={{ owner: "custom", baseUrl }}
          hasApiKey={key.provider?.hasApiKey === true}
          onChange={key.reload}
        />
      )}
    </>
  );
}

/**
 * Speech output: which voice reads text aloud, and the two switches that
 * make the app speak — a Speak button on replies, and reading the other
 * side's translations aloud as they arrive.
 */
export function SpeechSettings() {
  const { t, locale } = useI18n();
  const { settings, updateSettings } = useSettings();
  const { voice, choice } = useSpeechOutput();
  const isAvailable = voice !== null;
  const isCloud = isCloudSpeech(choice);
  const isBlocked = settings.localOnly && isCloud;
  const providerName = speechProviderName(choice) ?? t("providers.speech.systemVoices");

  const description = () => {
    if (choice === null) {
      return t("providers.speech.description");
    }
    return t(isCloud ? "providers.speech.cloudDescription" : "providers.speech.localDescription", {
      provider: providerName,
    });
  };

  const unavailableReason = () => {
    if (isAvailable) {
      return null;
    }
    return t(isBlocked ? "providers.localOnly.blocked" : "providers.speech.unavailable");
  };

  const targetLanguage = settings.targetLanguage;
  const lacksTargetVoice = settings.speakTranslations && voice?.canSpeak(targetLanguage) === false;

  return (
    <section className="panel" aria-labelledby="speech-title">
      <h2 id="speech-title">{t("providers.speech")}</h2>
      <RadioGroup
        legend={t("providers.speech.provider")}
        options={SPEECH_PROVIDERS}
        value={settings.speechProvider}
        disabledOptions={settings.localOnly ? ["elevenlabs"] : []}
        onChange={(speechProvider) => {
          updateSettings({ speechProvider });
        }}
        optionLabel={(option) => t(`providers.speech.provider.${option}`)}
      />
      <p className="notice">{description()}</p>
      {choice === null && <p className="note">{t("providers.speech.voices")}</p>}
      {settings.speechProvider === "elevenlabs" && <ElevenLabsSettings />}
      {settings.speechProvider === "compatible" && <CompatibleSpeechSettings />}

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
        <label className="choice">
          <input
            type="checkbox"
            checked={settings.speakTranslations && isAvailable}
            disabled={!isAvailable}
            aria-describedby={isAvailable ? "speak-translations-description" : "speech-unavailable"}
            onChange={(event) => {
              updateSettings({ speakTranslations: event.target.checked });
            }}
          />
          {t("providers.speech.translations.enable")}
        </label>
        <p id="speak-translations-description" className="note">
          {t("providers.speech.translations.description")}
        </p>
        {!isAvailable && (
          <p id="speech-unavailable" className="note">
            {unavailableReason()}
          </p>
        )}
        {settings.speakTranslations && isAvailable && !settings.translationEnabled && (
          <p className="warning">{t("providers.speech.translations.needsTranslation")}</p>
        )}
        {lacksTargetVoice && (
          <p className="warning">
            {t("providers.speech.translations.noVoice", {
              language: languageDisplayName(targetLanguage, locale),
            })}
          </p>
        )}
      </div>
    </section>
  );
}
