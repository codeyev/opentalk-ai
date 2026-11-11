import type { TranscriptionProviderChoice } from "../capture/captureClient";
import type { Settings, TranscriptionProviderId } from "../settings/settings";

const WHISPER_NAME = "Whisper";
/** Shown for OpenAI while the core has not described its provider. */
const OPENAI_NAME = "OpenAI";

/** One of the providers that can transcribe speech. */
export interface TranscriptionProvider {
  name: string;
  /** Whether using it sends audio to another machine. */
  isCloud: boolean;
  /** What to send to the core to use it, or `null` if it is not set up yet. */
  choice: TranscriptionProviderChoice | null;
}

/** Describes both transcription providers and whether each is ready. */
export function describeTranscriptionProviders(
  settings: Settings,
  openai: { displayName: string; hasApiKey: boolean } | null,
): Record<TranscriptionProviderId, TranscriptionProvider> {
  return {
    openai: {
      name: openai?.displayName ?? OPENAI_NAME,
      isCloud: true,
      choice: openai?.hasApiKey ? { id: "openai" } : null,
    },
    whisper: {
      name: WHISPER_NAME,
      isCloud: false,
      choice: settings.whisperModel === "" ? null : { id: "whisper", model: settings.whisperModel },
    },
  };
}

/** The provider that will actually transcribe: local-only mode replaces the cloud one with Whisper. */
export function effectiveTranscriptionProvider(settings: Settings): TranscriptionProviderId {
  return settings.localOnly ? "whisper" : settings.transcriptionProvider;
}
