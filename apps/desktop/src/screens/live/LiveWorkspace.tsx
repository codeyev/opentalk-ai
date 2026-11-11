import { useState } from "react";
import { useCaptureSession } from "../../capture/CaptureContext";
import type { CaptureConfig } from "../../capture/captureClient";
import { captureErrorMessageKey } from "../../capture/captureErrorMessage";
import { DEMO_CONVERSATION } from "../../demo/demoConversation";
import { useI18n } from "../../i18n/I18nContext";
import { languageDisplayName } from "../../lib/language";
import type { SpeechErrorCode } from "../../providers/providerClient";
import { useSessionPlan } from "../../providers/useSessionPlan";
import { AUTO_LANGUAGE } from "../../settings/settings";
import { useSettings } from "../../settings/SettingsContext";
import { useSpeechOutput } from "../../speech/SpeechContext";
import { isCloudSpeech, speechProviderName } from "../../speech/speechProviders";
import { AudioControls } from "./AudioControls";
import { CaptureStatus } from "./CaptureStatus";
import { ConversationPanel } from "./ConversationPanel";
import { FeatureStatus } from "./FeatureStatus";
import { ReplySuggestions } from "./ReplySuggestions";

/** The message that says why a translation could not be read aloud. */
function speechFailureKey(failure: SpeechErrorCode) {
  return failure === "invalid_settings" || failure === "local_only"
    ? (`live.speech.reason.${failure}` as const)
    : (`live.reason.${failure}` as const);
}

/** Languages of the demo data's replies and their meanings. */
const DEMO_LANGUAGES = { reply: "en", meaning: "az" };

/**
 * The main screen. The transcript, translations and reply suggestions come
 * from the capture session; demo data is only available while capture is
 * stopped, so it can never be mistaken for a live session.
 */
export function LiveWorkspace() {
  const { t, locale } = useI18n();
  const { settings } = useSettings();
  const capture = useCaptureSession();
  const plan = useSessionPlan();
  const { transcriber, translator, replier, meaningLanguage } = plan;
  const speech = useSpeechOutput();
  const speaksTranslations = settings.speakTranslations && speech.voice !== null;

  const isCapturing = capture.state === "listening";
  const [wantsDemo, setWantsDemo] = useState(false);
  const isDemo = wantsDemo && !isCapturing;
  const conversation = isDemo ? DEMO_CONVERSATION : capture;

  const start = (config: CaptureConfig) => {
    capture.start(config, plan.features);
  };

  const spokenLanguage =
    settings.sourceLanguage === AUTO_LANGUAGE
      ? t("live.languages.automatic")
      : languageDisplayName(settings.sourceLanguage, locale);

  return (
    <div className="live">
      <header className="page-header">
        <div>
          <h1>{t("nav.live")}</h1>
          <p className="subtitle">{t("live.subtitle")}</p>
        </div>
        <div className="page-header-aside">
          <p className="language-pair">
            <span className="visually-hidden">{t("languages.source")}: </span>
            <span className="chip">{spokenLanguage}</span>
            <span aria-hidden="true">→</span>
            <span className="visually-hidden">{t("languages.target")}: </span>
            <span className="chip">{languageDisplayName(settings.targetLanguage, locale)}</span>
          </p>
          <label className="choice">
            <input
              type="checkbox"
              checked={isDemo}
              disabled={isCapturing}
              aria-describedby="demo-hint"
              onChange={(event) => {
                setWantsDemo(event.target.checked);
              }}
            />
            {t("live.demo.toggle")}
          </label>
          <p id="demo-hint" className="visually-hidden">
            {t("live.demo.hint")}
          </p>
        </div>
      </header>

      {isDemo && (
        <p role="status" className="demo-banner">
          {t("live.demo.banner")}
        </p>
      )}

      <div className="live-columns">
        <div className="live-main">
          <CaptureStatus capture={capture} />
          {capture.transcriptionError !== null && (
            <p role="alert" className="error-text">
              {t(captureErrorMessageKey(capture.transcriptionError))}{" "}
              {t("live.transcription.stopped")}
            </p>
          )}
          {speaksTranslations && speech.failure !== null && (
            <p role="alert" className="error-text">
              {t("live.speech.failed", { reason: t(speechFailureKey(speech.failure)) })}
            </p>
          )}
          <ConversationPanel conversation={conversation} isDemo={isDemo} />
        </div>
        <ReplySuggestions
          replies={conversation.replies}
          replyLanguage={isDemo ? DEMO_LANGUAGES.reply : settings.replyLanguage}
          meaningLanguage={isDemo ? DEMO_LANGUAGES.meaning : meaningLanguage}
          isDemo={isDemo}
          onRefresh={isCapturing && replier !== null ? capture.refreshReplies : null}
        />
      </div>

      <ul className="feature-status">
        <FeatureStatus isOn={transcriber !== null}>
          {transcriber !== null
            ? t(transcriber.isCloud ? "live.transcription.on" : "live.transcription.onLocal", {
                provider: transcriber.name,
              })
            : t(settings.localOnly ? "live.transcription.localOnly" : "live.transcription.off")}
        </FeatureStatus>
        <FeatureStatus isOn={translator !== null}>
          {translator === null
            ? t("live.translating.off")
            : t(translator.isCloud ? "live.translating.on" : "live.translating.onLocal", {
                language: languageDisplayName(settings.targetLanguage, locale),
                provider: translator.name,
              })}
        </FeatureStatus>
        <FeatureStatus isOn={replier !== null}>
          {replier === null
            ? t("live.replies.off")
            : t(replier.isCloud ? "live.replies.on" : "live.replies.onLocal", {
                provider: replier.name,
              })}
        </FeatureStatus>
        {speaksTranslations && (
          <FeatureStatus isOn={translator !== null}>
            {translator === null
              ? t("live.speech.needsTranslation")
              : t(isCloudSpeech(speech.choice) ? "live.speech.onCloud" : "live.speech.on", {
                  provider: speechProviderName(speech.choice) ?? t("providers.speech.systemVoices"),
                })}
          </FeatureStatus>
        )}
      </ul>

      <AudioControls capture={capture} onStart={start} />
    </div>
  );
}
