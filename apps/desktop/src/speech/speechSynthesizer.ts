/**
 * Reading text aloud. The app speaks only what the user asked to hear: a
 * text whose Speak button they pressed, or translations while "speak
 * translations" is on.
 *
 * `SpeechSynthesizer` is the interface the UI depends on. One
 * implementation uses the voices installed on the computer, through the
 * webview's speech API, so no text leaves the machine. The other sends
 * the text to a speech provider (see `providerSpeechSynthesizer.ts`).
 */
export interface SpeechSynthesizer {
  /** Whether `language` (a BCP 47 tag) can be spoken, as far as is known. */
  canSpeak(language: string): boolean;
  /** Reads `text` aloud in `language`, replacing anything still being spoken. */
  speak(text: string, language: string): void;
  /** Reads `text` aloud once everything before it has been spoken. */
  speakNext(text: string, language: string): void;
  stop(): void;
  /** Calls `listener` when the set of installed voices changes. Returns a function that unsubscribes. */
  onVoicesChanged(listener: () => void): () => void;
}

function languageOf(tag: string): string | undefined {
  return tag.split(/[-_]/)[0]?.toLowerCase();
}

/**
 * A synthesizer backed by the system's voices, or `null` where the webview
 * offers no speech output.
 */
export function createSystemSpeechSynthesizer(): SpeechSynthesizer | null {
  if (!("speechSynthesis" in window) || !("SpeechSynthesisUtterance" in window)) {
    return null;
  }
  const synthesis = window.speechSynthesis;
  const voiceFor = (language: string) =>
    synthesis.getVoices().find((voice) => languageOf(voice.lang) === languageOf(language));

  // The speech API keeps its own queue: an utterance waits for the ones before it.
  const enqueue = (text: string, language: string) => {
    const voice = voiceFor(language);
    if (voice === undefined) {
      return;
    }
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.voice = voice;
    utterance.lang = voice.lang;
    synthesis.speak(utterance);
  };

  return {
    canSpeak: (language) => voiceFor(language) !== undefined,
    speak: (text, language) => {
      if (voiceFor(language) !== undefined) {
        synthesis.cancel();
        enqueue(text, language);
      }
    },
    speakNext: enqueue,
    stop: () => {
      synthesis.cancel();
    },
    onVoicesChanged: (listener) => {
      synthesis.addEventListener("voiceschanged", listener);
      return () => {
        synthesis.removeEventListener("voiceschanged", listener);
      };
    },
  };
}
