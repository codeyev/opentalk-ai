import { afterEach, describe, expect, it, vi } from "vitest";
import { installFakeSpeech } from "../test/fakeSpeech";
import { createSystemSpeechSynthesizer } from "./speechSynthesizer";

describe("createSystemSpeechSynthesizer", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("is unavailable where the webview has no speech output", () => {
    expect(createSystemSpeechSynthesizer()).toBeNull();
  });

  it("can speak only languages that have an installed voice", () => {
    installFakeSpeech(["en-US", "ar_SA"]);
    const synthesizer = createSystemSpeechSynthesizer();
    expect(synthesizer?.canSpeak("en")).toBe(true);
    expect(synthesizer?.canSpeak("en-GB")).toBe(true);
    expect(synthesizer?.canSpeak("ar-EG")).toBe(true);
    expect(synthesizer?.canSpeak("az")).toBe(false);
  });

  it("speaks with the matching voice, replacing anything still being spoken", () => {
    const speech = installFakeSpeech(["ar-SA"]);
    createSystemSpeechSynthesizer()?.speak("مرحبا", "ar");

    expect(speech.cancel).toHaveBeenCalledOnce();
    expect(speech.speak).toHaveBeenCalledOnce();
    const utterance = speech.speak.mock.calls[0]?.[0];
    expect(utterance?.text).toBe("مرحبا");
    expect(utterance?.lang).toBe("ar-SA");
  });

  it("stays silent for a language without a voice", () => {
    const speech = installFakeSpeech(["en-US"]);
    createSystemSpeechSynthesizer()?.speak("Salam", "az");
    expect(speech.speak).not.toHaveBeenCalled();
  });

  it("stops on request", () => {
    const speech = installFakeSpeech(["en-US"]);
    createSystemSpeechSynthesizer()?.stop();
    expect(speech.cancel).toHaveBeenCalledOnce();
  });

  it("unsubscribes from voice changes", () => {
    const speech = installFakeSpeech(["en-US"]);
    const unsubscribe = createSystemSpeechSynthesizer()?.onVoicesChanged(() => undefined);
    expect(speech.listenerCount()).toBe(1);
    unsubscribe?.();
    expect(speech.listenerCount()).toBe(0);
  });

  it("queues a text behind what is being spoken when asked to speak it next", () => {
    const speech = installFakeSpeech(["en-US"]);
    const synthesizer = createSystemSpeechSynthesizer();
    synthesizer?.speakNext("First.", "en");
    synthesizer?.speakNext("Second.", "en");
    // No voice for this language, so nothing is queued.
    synthesizer?.speakNext("Salam.", "az");

    expect(speech.cancel).not.toHaveBeenCalled();
    expect(speech.speak.mock.calls.map(([utterance]) => utterance.text)).toEqual([
      "First.",
      "Second.",
    ]);
  });
});
