import { captureErrorMessageKey } from "../../capture/captureErrorMessage";
import type { Capture } from "../../capture/useCapture";
import { useI18n } from "../../i18n/I18nContext";
import { LevelMeter } from "./LevelMeter";

/**
 * The strip above the conversation: whether audio is being captured right
 * now, how loud each source is, and any capture error.
 */
export function CaptureStatus({ capture }: { capture: Capture }) {
  const { t } = useI18n();
  const isCapturing = capture.state === "listening";

  return (
    <div className="capture-status" data-capturing={isCapturing}>
      {isCapturing ? (
        <>
          <p role="status" className="capture-indicator">
            <span className="capture-dot" aria-hidden="true" />
            {t("live.audio.capturing")}
          </p>
          <div className="levels">
            {capture.levels?.microphone && (
              <LevelMeter
                label={t("live.audio.level.microphone")}
                icon="microphone"
                level={capture.levels.microphone}
              />
            )}
            {capture.levels?.system && (
              <LevelMeter
                label={t("live.audio.level.system")}
                icon="monitor"
                level={capture.levels.system}
              />
            )}
          </div>
        </>
      ) : capture.isStarting ? (
        <p role="status" className="note">
          {t("live.audio.startingNote")}
        </p>
      ) : (
        <p className="note">{t("live.audio.consent")}</p>
      )}
      {capture.error !== null && (
        <p role="alert" className="error-text">
          {t(captureErrorMessageKey(capture.error))}
        </p>
      )}
    </div>
  );
}
