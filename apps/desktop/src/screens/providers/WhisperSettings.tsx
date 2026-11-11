import { useCallback, useEffect, useState } from "react";
import { useI18n } from "../../i18n/I18nContext";
import {
  cancelModelDownload,
  type CatalogueModel,
  type DownloadErrorCode,
  downloadWhisperModel,
  getWhisperModels,
  subscribeToModelDownload,
  toDownloadErrorCode,
  type WhisperModels,
} from "../../providers/providerClient";
import { useSettings } from "../../settings/SettingsContext";

const BYTES_PER_MEGABYTE = 1_000_000;

function sizeLabel(bytes: number): string {
  const megabytes = bytes / BYTES_PER_MEGABYTE;
  return megabytes >= 1000
    ? `${(megabytes / 1000).toFixed(1)} GB`
    : `${String(Math.round(megabytes))} MB`;
}

/** A model being downloaded, and how far it has got. */
interface Download {
  model: CatalogueModel;
  receivedBytes: number;
}

/**
 * The Whisper models for local transcription: which are installed, which
 * one to use, and downloading more. A model is only downloaded when the
 * user presses its Download button.
 */
export function WhisperSettings() {
  const { t } = useI18n();
  const { settings, updateSettings } = useSettings();
  const [models, setModels] = useState<WhisperModels | null>(null);
  const [download, setDownload] = useState<Download | null>(null);
  const [error, setError] = useState<DownloadErrorCode | null>(null);

  const loadModels = useCallback(() => getWhisperModels().then(setModels, () => undefined), []);

  useEffect(() => {
    void loadModels();
  }, [loadModels]);

  useEffect(() => {
    let isCurrent = true;
    let unsubscribe: (() => void) | undefined;
    subscribeToModelDownload((receivedBytes) => {
      setDownload((current) => (current === null ? null : { ...current, receivedBytes }));
    }).then(
      (stopListening) => {
        if (isCurrent) {
          unsubscribe = stopListening;
        } else {
          stopListening();
        }
      },
      () => undefined,
    );
    return () => {
      isCurrent = false;
      unsubscribe?.();
    };
  }, []);

  const startDownload = async (model: CatalogueModel) => {
    setError(null);
    setDownload({ model, receivedBytes: 0 });
    try {
      await downloadWhisperModel(model.id, settings.localOnly);
      await loadModels();
      // A freshly downloaded model is the obvious one to use if none was chosen.
      if (settings.whisperModel === "") {
        updateSettings({ whisperModel: model.fileName });
      }
    } catch (failure) {
      setError(toDownloadErrorCode(failure));
    }
    setDownload(null);
  };

  const installed = models?.installed ?? [];
  const isInstalled = (model: CatalogueModel) =>
    installed.some((file) => file.fileName === model.fileName);

  return (
    <section className="panel" aria-labelledby="whisper-title">
      <h2 id="whisper-title">{t("providers.whisper.title")}</h2>
      <p>
        <strong>Whisper</strong> · {t("providers.location.local")}
      </p>
      <p className="note">{t("providers.whisper.description")}</p>

      {models === null ? (
        <p className="note">{t("providers.whisper.error.unknown")}</p>
      ) : (
        <>
          {installed.length === 0 ? (
            <p className="warning">{t("providers.whisper.noModels")}</p>
          ) : (
            <div className="field">
              <label htmlFor="whisper-model">{t("providers.whisper.model")}</label>
              <select
                id="whisper-model"
                className="input"
                value={settings.whisperModel}
                onChange={(event) => {
                  updateSettings({ whisperModel: event.target.value });
                }}
              >
                <option value="">{t("providers.whisper.chooseModel")}</option>
                {installed.map((file) => (
                  <option key={file.fileName} value={file.fileName}>
                    {file.fileName} · {sizeLabel(file.sizeBytes)}
                  </option>
                ))}
              </select>
            </div>
          )}

          <div className="field">
            <h3>{t("providers.whisper.download")}</h3>
            <p className="note">{t("providers.whisper.downloadNote")}</p>
            {settings.localOnly && (
              <p className="warning">{t("providers.whisper.error.local_only")}</p>
            )}
            <ul className="model-list">
              {models.catalogue.map((model) => (
                <li key={model.id}>
                  {isInstalled(model) ? (
                    <span className="note">
                      {t("providers.whisper.installed", { model: model.id })}
                    </span>
                  ) : (
                    <button
                      type="button"
                      className="button"
                      disabled={download !== null || settings.localOnly}
                      onClick={() => {
                        void startDownload(model);
                      }}
                    >
                      {t("providers.whisper.downloadButton", {
                        model: model.id,
                        size: sizeLabel(model.sizeBytes),
                      })}
                    </button>
                  )}
                </li>
              ))}
            </ul>
          </div>

          {download !== null && (
            <div className="button-row">
              <p role="status">
                {t("providers.whisper.downloading", {
                  model: download.model.id,
                  percent: Math.floor((download.receivedBytes / download.model.sizeBytes) * 100),
                })}
              </p>
              <button
                type="button"
                className="button"
                onClick={() => {
                  void cancelModelDownload();
                }}
              >
                {t("providers.whisper.cancel")}
              </button>
            </div>
          )}
          {error !== null && (
            <p role="alert" className="error-text">
              {t(`providers.whisper.error.${error}`)}
            </p>
          )}
          <p className="note">{t("providers.whisper.folder", { folder: models.directory })}</p>
        </>
      )}
      <p className="note">{t("providers.whisper.license")}</p>
    </section>
  );
}
