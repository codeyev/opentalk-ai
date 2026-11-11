import { type ReactNode, useEffect, useMemo, useState } from "react";
import {
  type SpeechChoice,
  type SpeechErrorCode,
  synthesizeSpeech,
  toSpeechErrorCode,
} from "../providers/providerClient";
import { useSettings } from "../settings/SettingsContext";
import { createPcmPlayer } from "./pcmPlayer";
import { createProviderSpeechSynthesizer } from "./providerSpeechSynthesizer";
import { SpeechContext } from "./SpeechContext";
import { isCloudSpeech, speechChoiceOf } from "./speechProviders";
import { createSystemSpeechSynthesizer } from "./speechSynthesizer";

/** A failure, and the provider settings it happened with. */
interface Failure {
  choice: SpeechChoice;
  code: SpeechErrorCode;
}

/** Sets up the voice chosen in the settings and shares it with the screens. */
export function SpeechProvider({ children }: { children: ReactNode }) {
  const { settings } = useSettings();
  const { localOnly } = settings;
  const { speechProvider, elevenlabsVoice, elevenlabsModel, speechBaseUrl, speechModel } = settings;
  const { speechVoice } = settings;
  const choice = useMemo(
    () =>
      speechChoiceOf({
        speechProvider,
        elevenlabsVoice,
        elevenlabsModel,
        speechBaseUrl,
        speechModel,
        speechVoice,
      }),
    [speechProvider, elevenlabsVoice, elevenlabsModel, speechBaseUrl, speechModel, speechVoice],
  );

  const systemVoices = useMemo(() => createSystemSpeechSynthesizer(), []);
  const player = useMemo(() => createPcmPlayer(), []);
  const [failure, setFailure] = useState<Failure | null>(null);

  const voice = useMemo(() => {
    if (choice === null) {
      return systemVoices;
    }
    // The core refuses a cloud voice in local-only mode as well.
    if (player === null || (localOnly && isCloudSpeech(choice))) {
      return null;
    }
    return createProviderSpeechSynthesizer({
      synthesize: (text, language) => synthesizeSpeech(choice, text, language, localOnly),
      player,
      onFailure: (reason) => {
        setFailure({ choice, code: toSpeechErrorCode(reason) });
      },
      onSuccess: () => {
        setFailure(null);
      },
    });
  }, [choice, systemVoices, player, localOnly]);

  // A voice that is replaced or no longer shown must fall silent.
  useEffect(
    () => () => {
      voice?.stop();
    },
    [voice],
  );

  // System voices load after the page does; re-render when they arrive.
  const [, setVoiceChanges] = useState(0);
  useEffect(
    () =>
      systemVoices?.onVoicesChanged(() => {
        setVoiceChanges((count) => count + 1);
      }),
    [systemVoices],
  );

  const output = useMemo(
    () => ({
      voice,
      choice,
      // A failure says nothing about settings that were changed since.
      failure: failure?.choice === choice ? failure.code : null,
    }),
    [voice, choice, failure],
  );
  return <SpeechContext value={output}>{children}</SpeechContext>;
}
