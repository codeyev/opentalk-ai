import { useSettings } from "../settings/SettingsContext";
import { useSpeechOutput } from "./SpeechContext";
import type { SpeechSynthesizer } from "./speechSynthesizer";

export interface Speech {
  /** Whether the chosen voice can be used at all, regardless of the setting. */
  isAvailable: boolean;
  /** The synthesizer for Speak buttons, or `null` when they are unavailable or turned off. */
  synthesizer: SpeechSynthesizer | null;
}

/** Gives access to speech output for Speak buttons, honouring the user's setting. */
export function useSpeech(): Speech {
  const { speechEnabled } = useSettings().settings;
  const { voice } = useSpeechOutput();
  return {
    isAvailable: voice !== null,
    synthesizer: speechEnabled ? voice : null,
  };
}
