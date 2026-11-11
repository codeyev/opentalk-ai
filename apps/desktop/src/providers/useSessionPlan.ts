import type { ArabicStyle, ProviderChoice, SessionFeatures } from "../capture/captureClient";
import { sameLanguage } from "../lib/language";
import { AUTO_LANGUAGE, type LlmProviderId } from "../settings/settings";
import { useSettings } from "../settings/SettingsContext";
import { describeLlmProviders, effectiveProvider } from "./llmProviders";
import {
  describeTranscriptionProviders,
  effectiveTranscriptionProvider,
} from "./transcriptionProviders";
import { useCustomProvider } from "./useCustomProvider";
import { useReplyProvider } from "./useReplyProvider";
import { useTranscriptionProvider } from "./useTranscriptionProvider";

/** A provider that will be used in the next session. */
export interface PlannedProvider {
  name: string;
  /** Whether using it sends data to another machine. */
  isCloud: boolean;
  choice: ProviderChoice;
}

/** What the next capture session will do with its audio, given the settings. */
export interface SessionPlan {
  /** `null` for each feature that is off or cannot run. */
  transcriber: { name: string; isCloud: boolean } | null;
  translator: PlannedProvider | null;
  replier: PlannedProvider | null;
  /** Language reply meanings are shown in; `null` when replies are already in the user's language. */
  meaningLanguage: string | null;
  /** The request to send to the core when the user presses Start. */
  features: SessionFeatures;
}

/**
 * Works out which features the next session will use. A feature runs only
 * if the user turned it on and its provider is ready; translation and
 * replies also need a transcript; local-only mode rules out cloud providers.
 */
export function useSessionPlan(): SessionPlan {
  const { settings } = useSettings();
  const transcription = useTranscriptionProvider().provider;
  const openai = useReplyProvider().provider;
  const customHasApiKey = useCustomProvider(settings.customBaseUrl).provider?.hasApiKey === true;
  const providers = describeLlmProviders(settings, { openai, customHasApiKey });

  /** The provider chosen for a feature, if it is set up. */
  const planned = (chosen: LlmProviderId): PlannedProvider | null => {
    const { name, isCloud, choice } = providers[effectiveProvider(settings, providers, chosen)];
    return choice === null ? null : { name, isCloud, choice };
  };

  const transcriptionProvider = describeTranscriptionProviders(settings, transcription)[
    effectiveTranscriptionProvider(settings)
  ];
  const transcriber =
    settings.transcriptionEnabled && transcriptionProvider.choice !== null
      ? { name: transcriptionProvider.name, isCloud: transcriptionProvider.isCloud }
      : null;
  const translator =
    transcriber !== null && settings.translationEnabled
      ? planned(settings.translationProvider)
      : null;
  const replier =
    transcriber !== null && settings.repliesEnabled ? planned(settings.repliesProvider) : null;

  const meaningLanguage = sameLanguage(settings.replyLanguage, settings.targetLanguage)
    ? null
    : settings.targetLanguage;
  const arabic: ArabicStyle = {
    dialect:
      settings.arabicVariety === "dialect" && settings.arabicDialectLabel.trim() !== ""
        ? settings.arabicDialectLabel.trim()
        : null,
    tashkeel: settings.tashkeel,
  };

  return {
    transcriber,
    translator,
    replier,
    meaningLanguage,
    features: {
      localOnly: settings.localOnly,
      transcription:
        transcriber === null || transcriptionProvider.choice === null
          ? null
          : {
              provider: transcriptionProvider.choice,
              language: settings.sourceLanguage === AUTO_LANGUAGE ? null : settings.sourceLanguage,
            },
      translation:
        translator === null
          ? null
          : {
              provider: translator.choice,
              targetLanguage: settings.targetLanguage,
              glossary: settings.glossary.map(({ term, preferred }) => ({ term, preferred })),
              arabic,
            },
      replies:
        replier === null
          ? null
          : {
              provider: replier.choice,
              replyLanguage: settings.replyLanguage,
              meaningLanguage,
              businessContext: settings.businessContext,
              arabic,
            },
    },
  };
}
