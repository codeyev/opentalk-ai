import { vi } from "vitest";

/**
 * Installs a stand-in for the browser's speech API, which the test
 * environment does not have. Returns the spies and a function that removes it.
 */
export function installFakeSpeech(voiceLanguages: string[]) {
  const speak = vi.fn<(utterance: { text: string; lang: string }) => void>();
  const cancel = vi.fn();
  const listeners = new Set<() => void>();

  class FakeUtterance {
    voice: unknown = null;
    lang = "";
    constructor(public text: string) {}
  }

  vi.stubGlobal("SpeechSynthesisUtterance", FakeUtterance);
  vi.stubGlobal("speechSynthesis", {
    getVoices: () => voiceLanguages.map((lang) => ({ lang })),
    speak,
    cancel,
    addEventListener: (_type: string, listener: () => void) => listeners.add(listener),
    removeEventListener: (_type: string, listener: () => void) => listeners.delete(listener),
  });

  return {
    speak,
    cancel,
    /** How many components are waiting to hear about new voices. */
    listenerCount: () => listeners.size,
    uninstall: () => {
      vi.unstubAllGlobals();
    },
  };
}
