import { useCallback, useEffect, useState } from "react";
import { useI18n } from "../../i18n/I18nContext";
import {
  getOllamaStatus,
  type OllamaErrorCode,
  type OllamaStatus,
  toOllamaErrorCode,
} from "../../providers/providerClient";
import { MAX_ENDPOINT_LENGTH } from "../../settings/settings";
import { useSettings } from "../../settings/SettingsContext";

type Check =
  | { state: "checking" }
  | { state: "running"; status: OllamaStatus }
  | { state: "failed"; reason: OllamaErrorCode };

const BYTES_PER_GIGABYTE = 1_000_000_000;

function sizeInGigabytes(bytes: number): string {
  return `${(bytes / BYTES_PER_GIGABYTE).toFixed(1)} GB`;
}

/**
 * The local Ollama server: its address, whether it is running, and which
 * of its installed models to use. Models are only listed, never downloaded.
 */
export function LocalAiSettings() {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();
  const [check, setCheck] = useState<Check>({ state: "checking" });
  const endpoint = settings.ollamaEndpoint;

  const runCheck = useCallback(async (address: string): Promise<Check> => {
    try {
      return { state: "running", status: await getOllamaStatus(address) };
    } catch (error) {
      return { state: "failed", reason: toOllamaErrorCode(error) };
    }
  }, []);

  // Check once when the screen opens; after that only when the user asks.
  useEffect(() => {
    let isCurrent = true;
    void runCheck(endpoint).then((result) => {
      if (isCurrent) {
        setCheck(result);
      }
    });
    return () => {
      isCurrent = false;
    };
    // The address is read once on purpose: typing in the field must not send requests.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [runCheck]);

  const checkNow = async () => {
    setCheck({ state: "checking" });
    setCheck(await runCheck(endpoint));
  };

  const models = check.state === "running" ? check.status.models : [];

  return (
    <section className="panel" aria-labelledby="local-ai-title">
      <h2 id="local-ai-title">{t("providers.local.title")}</h2>
      <p>
        <strong>Ollama</strong> · {t("providers.location.local")}
      </p>
      <p className="note">{t("providers.local.description")}</p>

      <form
        className="field"
        onSubmit={(event) => {
          event.preventDefault();
          void checkNow();
        }}
      >
        <label htmlFor="ollama-endpoint">{t("providers.local.endpoint")}</label>
        <div className="button-row">
          <input
            id="ollama-endpoint"
            type="text"
            className="input input-key"
            spellCheck={false}
            maxLength={MAX_ENDPOINT_LENGTH}
            value={endpoint}
            aria-describedby="ollama-endpoint-hint"
            onChange={(event) => {
              updateSettings({ ollamaEndpoint: event.target.value });
            }}
          />
          <button type="submit" className="button" disabled={check.state === "checking"}>
            {t("providers.local.check")}
          </button>
        </div>
        <p id="ollama-endpoint-hint" className="note">
          {t("providers.local.endpointHint")}
        </p>
      </form>

      <p role="status" className={check.state === "failed" ? "error-text" : "note"}>
        {check.state === "checking" && t("providers.local.checking")}
        {check.state === "running" &&
          t("providers.local.running", { version: check.status.version })}
        {check.state === "failed" && t(`providers.local.error.${check.reason}`)}
      </p>

      {check.state === "running" && models.length === 0 && (
        <p className="warning">{t("providers.local.noModels")}</p>
      )}
      {models.length > 0 && (
        <div className="field">
          <label htmlFor="ollama-model">{t("providers.local.model")}</label>
          <select
            id="ollama-model"
            className="input"
            value={settings.ollamaModel}
            aria-describedby="ollama-model-hint"
            onChange={(event) => {
              updateSettings({ ollamaModel: event.target.value });
            }}
          >
            <option value="">{t("providers.local.chooseModel")}</option>
            {models.map((model) => (
              <option key={model.name} value={model.name}>
                {model.name} · {sizeInGigabytes(model.sizeBytes)}
              </option>
            ))}
          </select>
          <p id="ollama-model-hint" className="note">
            {t("providers.local.resources")}
          </p>
        </div>
      )}
    </section>
  );
}
