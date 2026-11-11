import type { PcmPlayer } from "./pcmPlayer";
import type { SpeechSynthesizer } from "./speechSynthesizer";

/**
 * How many texts may wait to be spoken. When speech arrives faster than
 * it can be read aloud, the oldest waiting text is skipped, so what is
 * heard never falls far behind the conversation.
 */
export const MAX_WAITING_TEXTS = 3;

interface Utterance {
  text: string;
  language: string;
}

interface ProviderSpeechOptions {
  /** Asks the provider for the audio of one text. */
  synthesize: (text: string, language: string) => Promise<ArrayBuffer>;
  player: PcmPlayer;
  /** Called with whatever `synthesize` rejected with. */
  onFailure: (failure: unknown) => void;
  /** Called when a text was spoken, so an earlier failure can be cleared. */
  onSuccess: () => void;
}

/**
 * A synthesizer that sends each text to a speech provider and plays the
 * audio that comes back, one text at a time.
 */
export function createProviderSpeechSynthesizer({
  synthesize,
  player,
  onFailure,
  onSuccess,
}: ProviderSpeechOptions): SpeechSynthesizer {
  let waiting: Utterance[] = [];
  let isBusy = false;
  // Counts up on every stop, so audio requested before it is not played after.
  let generation = 0;

  const run = async () => {
    isBusy = true;
    for (let next = waiting.shift(); next !== undefined; next = waiting.shift()) {
      const startedIn = generation;
      try {
        const audio = await synthesize(next.text, next.language);
        if (startedIn === generation) {
          await player.play(audio);
          onSuccess();
        }
      } catch (failure) {
        if (startedIn === generation) {
          onFailure(failure);
        }
      }
    }
    isBusy = false;
  };

  const enqueue = (utterance: Utterance) => {
    waiting = [...waiting, utterance].slice(-MAX_WAITING_TEXTS);
    if (!isBusy) {
      void run();
    }
  };

  const stop = () => {
    generation += 1;
    waiting = [];
    player.stop();
  };

  return {
    // A provider does not say in advance which languages its voice speaks.
    canSpeak: () => true,
    speak: (text, language) => {
      stop();
      enqueue({ text, language });
    },
    speakNext: (text, language) => {
      enqueue({ text, language });
    },
    stop,
    onVoicesChanged: () => () => undefined,
  };
}
