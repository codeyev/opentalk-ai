import { useCaptureSession } from "../capture/CaptureContext";
import { useElapsedSeconds } from "../capture/useElapsedSeconds";
import { useI18n } from "../i18n/I18nContext";
import { formatOffset } from "../lib/time";

/** The session's state and running time, always visible in the sidebar. */
export function SessionCard() {
  const { t } = useI18n();
  const { state, startedAt } = useCaptureSession();
  const elapsedSeconds = useElapsedSeconds(startedAt);

  return (
    <dl className="session-card">
      <div>
        <dt className="visually-hidden">{t("live.state.label")}</dt>
        <dd className="session-state" data-state={state}>
          <span className="state-dot" aria-hidden="true" />
          {t(`live.state.${state}`)}
        </dd>
      </div>
      <div>
        <dt className="visually-hidden">{t("live.elapsed.label")}</dt>
        {elapsedSeconds === null ? (
          <dd className="session-time">
            <span aria-hidden="true">--:--</span>
            <span className="visually-hidden">{t("live.elapsed.notRunning")}</span>
          </dd>
        ) : (
          <dd className="session-time">{formatOffset(elapsedSeconds * 1000)}</dd>
        )}
      </div>
    </dl>
  );
}
