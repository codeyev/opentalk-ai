import { hostOf, isLoopbackUrl } from "../lib/url";
import type { SpeechChoice } from "../providers/providerClient";
import type { Settings } from "../settings/settings";

/** The settings that decide which voice is used. */
export type SpeechSettings = Pick<
  Settings,
  | "speechProvider"
  | "elevenlabsVoice"
  | "elevenlabsModel"
  | "speechBaseUrl"
  | "speechModel"
  | "speechVoice"
>;

/**
 * The speech provider in the core that the settings ask for, or `null`
 * for the system's own voices, which the UI uses directly.
 */
export function speechChoiceOf(settings: SpeechSettings): SpeechChoice | null {
  switch (settings.speechProvider) {
    case "system":
      return null;
    case "elevenlabs":
      return { id: "elevenlabs", voice: settings.elevenlabsVoice, model: settings.elevenlabsModel };
    case "compatible":
      return {
        id: "compatible",
        baseUrl: settings.speechBaseUrl,
        model: settings.speechModel,
        voice: settings.speechVoice,
      };
  }
}

/** Whether a speech provider sends the text to another machine. */
export function isCloudSpeech(choice: SpeechChoice | null): boolean {
  if (choice === null) {
    return false;
  }
  return choice.id === "elevenlabs" || !isLoopbackUrl(choice.baseUrl);
}

/** The provider's name for display, or `null` for the system's voices. */
export function speechProviderName(choice: SpeechChoice | null): string | null {
  if (choice === null) {
    return null;
  }
  return choice.id === "elevenlabs" ? "ElevenLabs" : (hostOf(choice.baseUrl) ?? choice.baseUrl);
}
