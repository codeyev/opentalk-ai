import { useI18n } from "../../i18n/I18nContext";
import { isAcceptableServiceUrl } from "../../lib/url";
import { presetFor, SERVICE_PRESETS } from "../../providers/servicePresets";
import { MAX_ENDPOINT_LENGTH } from "../../settings/settings";
import { useSettings } from "../../settings/SettingsContext";
import { ApiKeyField } from "./ApiKeyField";

/** `<select>` value for an address that is not one of the presets. */
const OTHER_ADDRESS = "";

interface CustomProviderSettingsProps {
  /** Whether a key is saved for the service at the current address. */
  hasApiKey: boolean;
  /** Called after the saved key changed, so its status can be reloaded. */
  onKeyChange: () => Promise<void>;
}

/**
 * The user's own choice of AI service: any service with an
 * OpenAI-compatible API, such as OpenRouter, Gemini, Claude or DeepSeek.
 * Holds its address, the model to use, and the API key for it.
 */
export function CustomProviderSettings({ hasApiKey, onKeyChange }: CustomProviderSettingsProps) {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();
  const baseUrl = settings.customBaseUrl;
  const isAddressValid = isAcceptableServiceUrl(baseUrl);
  const preset = presetFor(baseUrl);

  return (
    <section className="panel" aria-labelledby="custom-provider-title">
      <h2 id="custom-provider-title">{t("providers.custom.title")}</h2>
      <p className="note">{t("providers.custom.description")}</p>
      <p className="notice">{t("providers.custom.policies")}</p>

      <div className="field">
        <label htmlFor="custom-service">{t("providers.custom.service")}</label>
        <select
          id="custom-service"
          className="input"
          value={preset?.baseUrl ?? OTHER_ADDRESS}
          onChange={(event) => {
            // A model name from one service means nothing to another.
            updateSettings({ customBaseUrl: event.target.value, customModel: "" });
          }}
        >
          {SERVICE_PRESETS.map((service) => (
            <option key={service.baseUrl} value={service.baseUrl}>
              {service.name}
            </option>
          ))}
          <option value={OTHER_ADDRESS}>{t("providers.custom.other")}</option>
        </select>
      </div>

      <div className="field">
        <label htmlFor="custom-base-url">{t("providers.custom.baseUrl")}</label>
        <input
          id="custom-base-url"
          type="text"
          className="input"
          spellCheck={false}
          maxLength={MAX_ENDPOINT_LENGTH}
          value={baseUrl}
          aria-invalid={!isAddressValid}
          aria-describedby="custom-base-url-hint"
          onChange={(event) => {
            updateSettings({ customBaseUrl: event.target.value });
          }}
        />
        <p id="custom-base-url-hint" className={isAddressValid ? "note" : "note note-error"}>
          {isAddressValid
            ? t("providers.custom.baseUrlHint")
            : t("providers.custom.baseUrlInvalid")}
        </p>
      </div>

      <div className="field">
        <label htmlFor="custom-model">{t("providers.custom.model")}</label>
        <input
          id="custom-model"
          type="text"
          className="input"
          spellCheck={false}
          maxLength={MAX_ENDPOINT_LENGTH}
          placeholder={preset?.modelExample}
          value={settings.customModel}
          aria-describedby="custom-model-hint"
          onChange={(event) => {
            updateSettings({ customModel: event.target.value });
          }}
        />
        <p id="custom-model-hint" className="note">
          {t("providers.custom.modelHint")}
        </p>
      </div>

      {isAddressValid && (
        <ApiKeyField
          // Each service has its own key, so the field starts afresh for a new address.
          key={baseUrl}
          target={{ owner: "custom", baseUrl }}
          hasApiKey={hasApiKey}
          onChange={onKeyChange}
        />
      )}
      <p className="note">{t("providers.custom.noTranscription")}</p>
    </section>
  );
}
