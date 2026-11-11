/**
 * Reading text aloud. The app only ever speaks when the user presses a
 * Speak button; nothing here starts on its own.
 *
 * `SpeechSynthesizer` is the interface the UI depends on. The one
 * implementation uses the voices installed on the computer, through the
 * webview's speech API, so no text leaves the machine. A cloud voice could
 * be added later as another implementation of the same interface.
 */
export interface SpeechSynthesizer {
  /** Whether a voice is installed for the language of a BCP 47 tag. */
  canSpeak(language: string): boolean;
  /** Reads `text` aloud in `language`, replacing anything still being spoken. */
  speak(text: string, language: string): void;
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

  return {
    canSpeak: (language) => voiceFor(language) !== undefined,
    speak: (text, language) => {
      const voice = voiceFor(language);
      if (voice === undefined) {
        return;
      }
      synthesis.cancel();
      const utterance = new SpeechSynthesisUtterance(text);
      utterance.voice = voice;
      utterance.lang = voice.lang;
      synthesis.speak(utterance);
    },
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
