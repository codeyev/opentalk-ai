import { useState } from "react";
import type { CaptureConfig } from "../../capture/captureClient";
import type { Capture } from "../../capture/useCapture";
import { Icon } from "../../components/Icon";
import { useI18n } from "../../i18n/I18nContext";

/** `<select>` value meaning "use the system's default microphone". */
const DEFAULT_MICROPHONE = "";

interface AudioControlsProps {
  capture: Capture;
  /** Called when the user presses Start, with the sources they chose. */
  onStart: (config: CaptureConfig) => void;
}

/**
 * The control bar: which sources to capture, and the single button that
 * starts and stops capture. Sources can only be changed while stopped.
 */
export function AudioControls({ capture, onStart }: AudioControlsProps) {
  const { t } = useI18n();
  const [useMicrophone, setUseMicrophone] = useState(true);
  const [microphoneId, setMicrophoneId] = useState(DEFAULT_MICROPHONE);
  const [useSystemAudio, setUseSystemAudio] = useState(false);

  const isCapturing = capture.state === "listening";
  const hasSource = useMicrophone || useSystemAudio;

  const start = () => {
    onStart({
      microphone: useMicrophone,
      microphoneId: microphoneId === DEFAULT_MICROPHONE ? null : microphoneId,
      systemAudio: useSystemAudio,
    });
  };

  return (
    <section className="control-bar" aria-labelledby="audio-title">
      <h2 id="audio-title" className="visually-hidden">
        {t("live.audio.title")}
      </h2>

      <div className="source">
        <label className="choice">
          <input
            type="checkbox"
            checked={useMicrophone}
            disabled={isCapturing}
            onChange={(event) => {
              setUseMicrophone(event.target.checked);
            }}
          />
          <Icon name="microphone" />
          {t("live.audio.useMicrophone")}
        </label>
        <label className="visually-hidden" htmlFor="microphone">
          {t("live.audio.microphone")}
        </label>
        <select
          id="microphone"
          className="input"
          value={microphoneId}
          disabled={isCapturing || !useMicrophone}
          onChange={(event) => {
            setMicrophoneId(event.target.value);
          }}
        >
          <option value={DEFAULT_MICROPHONE}>{t("live.audio.defaultMicrophone")}</option>
          {capture.microphones.map((microphone) => (
            <option key={microphone.id} value={microphone.id}>
              {microphone.name}
            </option>
          ))}
        </select>
      </div>

      <div className="source">
        <label className="choice">
          <input
            type="checkbox"
            checked={useSystemAudio}
            disabled={isCapturing}
            onChange={(event) => {
              setUseSystemAudio(event.target.checked);
            }}
          />
          <Icon name="monitor" />
          {t("live.audio.system")}
        </label>
      </div>

      {isCapturing ? (
        <button type="button" className="button button-stop" onClick={capture.stop}>
          <Icon name="stop" />
          {t("live.audio.stop")}
        </button>
      ) : (
        <button
          type="button"
          className="button button-primary button-start"
          disabled={!hasSource || capture.isStarting}
          onClick={start}
        >
          <Icon name="play" />
          {t(capture.isStarting ? "live.audio.starting" : "live.audio.start")}
        </button>
      )}

      <p className="note control-note">{t("live.audio.notStored")}</p>
    </section>
  );
}
