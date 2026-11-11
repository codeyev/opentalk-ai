import { useEffect, useMemo, useState } from "react";
import { useSettings } from "../settings/SettingsContext";
import { createSystemSpeechSynthesizer, type SpeechSynthesizer } from "./speechSynthesizer";

export interface Speech {
  /** Whether this system can speak at all, regardless of the setting. */
  isAvailable: boolean;
  /** The synthesizer to use, or `null` when speech is unavailable or turned off. */
  synthesizer: SpeechSynthesizer | null;
}

/** Gives access to speech output, honouring the user's setting. */
export function useSpeech(): Speech {
  const { speechEnabled } = useSettings().settings;
  const synthesizer = useMemo(() => createSystemSpeechSynthesizer(), []);
  // Voices load after the page does; re-render when they arrive.
  const [, setVoiceChanges] = useState(0);

  useEffect(
    () =>
      synthesizer?.onVoicesChanged(() => {
        setVoiceChanges((count) => count + 1);
      }),
    [synthesizer],
  );

  return {
    isAvailable: synthesizer !== null,
    synthesizer: speechEnabled ? synthesizer : null,
  };
}
