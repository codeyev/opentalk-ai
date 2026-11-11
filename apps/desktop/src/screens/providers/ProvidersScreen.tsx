import { useI18n } from "../../i18n/I18nContext";
import { useCustomProvider } from "../../providers/useCustomProvider";
import { useReplyProvider } from "../../providers/useReplyProvider";
import { useTranscriptionProvider } from "../../providers/useTranscriptionProvider";
import { useSettings } from "../../settings/SettingsContext";
import { CustomProviderSettings } from "./CustomProviderSettings";
import { LlmFeatureSettings } from "./LlmFeatureSettings";
import { LocalAiSettings } from "./LocalAiSettings";
import { LocalOnlySetting } from "./LocalOnlySetting";
import { SpeechSettings } from "./SpeechSettings";
import { TranscriptionSettings } from "./TranscriptionSettings";
import { WhisperSettings } from "./WhisperSettings";

/** Which provider does each job, what it receives, and the switches that turn each job on. */
export function ProvidersScreen() {
  const { t } = useI18n();
  const { settings } = useSettings();
  const transcription = useTranscriptionProvider();
  const openai = useReplyProvider();
  const custom = useCustomProvider(settings.customBaseUrl);
  const keys = {
    openai: openai.provider,
    customHasApiKey: custom.provider?.hasApiKey === true,
  };

  // Transcription and the OpenAI text features read the same saved key.
  const reloadOpenAi = async () => {
    await Promise.all([transcription.reload(), openai.reload()]);
  };

  return (
    <div className="form">
      <p className="intro">{t("providers.intro")}</p>
      <LocalOnlySetting />
      <TranscriptionSettings openai={transcription.provider} onKeyChange={reloadOpenAi} />
      <LlmFeatureSettings feature="translation" keys={keys} />
      <LlmFeatureSettings feature="replies" keys={keys} />
      <CustomProviderSettings hasApiKey={keys.customHasApiKey} onKeyChange={custom.reload} />
      <WhisperSettings />
      <LocalAiSettings />
      <SpeechSettings />
    </div>
  );
}
