import { RadioGroup } from "../../components/RadioGroup";
import { useI18n } from "../../i18n/I18nContext";
import {
  describeLlmProviders,
  effectiveProvider,
  type KeyStatus,
} from "../../providers/llmProviders";
import { LLM_PROVIDERS, type LlmProviderId } from "../../settings/settings";
import { useSettings } from "../../settings/SettingsContext";

/** A feature that a language model performs on the transcript. */
type LlmFeature = "translation" | "replies";

interface LlmFeatureSettingsProps {
  feature: LlmFeature;
  /** Which providers have a saved API key. */
  keys: KeyStatus;
}

/** The settings keys that hold each feature's switch and provider choice. */
const SETTING_KEYS = {
  translation: { enabled: "translationEnabled", provider: "translationProvider" },
  replies: { enabled: "repliesEnabled", provider: "repliesProvider" },
} as const;

/**
 * Settings for translation or reply suggestions: which provider does it,
 * what that provider receives, and the switch that turns the feature on.
 */
export function LlmFeatureSettings({ feature, keys }: LlmFeatureSettingsProps) {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();
  const settingKeys = SETTING_KEYS[feature];
  const providers = describeLlmProviders(settings, keys);

  const chosen = effectiveProvider(settings, providers, settings[settingKeys.provider]);
  const provider = providers[chosen];
  const name = provider.name === "" ? t("providers.custom.unnamed") : provider.name;
  // The feature works on the transcript, so transcription must be on too.
  const canEnable = provider.choice !== null && settings.transcriptionEnabled;
  const titleId = `${feature}-provider-title`;
  const noteId = `${feature}-needs`;

  const optionLabel = (id: LlmProviderId) => {
    const option = providers[id];
    return t(option.isCloud ? "providers.choice.cloud" : "providers.choice.local", {
      provider: option.name === "" ? t("providers.custom.unnamed") : option.name,
    });
  };

  /** What the chosen provider receives, or what it still needs before it can be used. */
  const explanation = () => {
    if (chosen === "ollama") {
      return settings.ollamaModel === ""
        ? t("providers.local.needsModel")
        : t("providers.local.processing", { model: settings.ollamaModel });
    }
    if (chosen === "custom" && provider.choice === null) {
      return t("providers.custom.needsSetup");
    }
    return t(`providers.${feature}.disclosure`, { provider: name });
  };

  return (
    <section className="panel" aria-labelledby={titleId}>
      <h2 id={titleId}>{t(`providers.${feature}`)}</h2>
      <RadioGroup
        legend={t("providers.choice.label")}
        options={LLM_PROVIDERS}
        value={chosen}
        disabledOptions={
          settings.localOnly ? LLM_PROVIDERS.filter((id) => providers[id].isCloud) : []
        }
        onChange={(id) => {
          updateSettings({ [settingKeys.provider]: id });
        }}
        optionLabel={optionLabel}
      />

      <p className="notice">{explanation()}</p>
      <ul className="principles">
        {feature === "translation" ? (
          <>
            <li>{t("providers.translation.languagesUnknown")}</li>
            <li>{t("providers.translation.checks")}</li>
          </>
        ) : (
          <>
            <li>{t("providers.replies.rules")}</li>
            <li>{t("providers.replies.source")}</li>
          </>
        )}
      </ul>

      <div className="field">
        <label className="choice">
          <input
            type="checkbox"
            checked={settings[settingKeys.enabled] && canEnable}
            disabled={!canEnable}
            aria-describedby={canEnable ? undefined : noteId}
            onChange={(event) => {
              updateSettings({ [settingKeys.enabled]: event.target.checked });
            }}
          />
          {t(`providers.${feature}.enable`, { provider: name })}
        </label>
        {!canEnable && (
          <p id={noteId} className="note">
            {t(`providers.${feature}.needsTranscription`)}
          </p>
        )}
      </div>
    </section>
  );
}
