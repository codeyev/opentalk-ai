import { describe, expect, it, vi } from "vitest";
import type { PcmPlayer } from "./pcmPlayer";
import { createProviderSpeechSynthesizer, MAX_WAITING_TEXTS } from "./providerSpeechSynthesizer";

/** Lets every promise that is already settled run its continuation. */
async function settle() {
  for (let turn = 0; turn < 10; turn += 1) {
    await Promise.resolve();
  }
}

/** A synthesizer whose provider and player are controlled by the test. */
function setUp() {
  /** Finishes the playback that is running now. */
  let finishPlayback: () => void = () => undefined;
  const play = vi.fn(
    () =>
      new Promise<void>((resolve) => {
        finishPlayback = resolve;
      }),
  );
  const stop = vi.fn(() => {
    finishPlayback();
  });
  const player: PcmPlayer = { play, stop };
  const synthesize = vi.fn((text: string) =>
    Promise.resolve(new TextEncoder().encode(text).buffer),
  );
  const onFailure = vi.fn();
  const onSuccess = vi.fn();
  const synthesizer = createProviderSpeechSynthesizer({ synthesize, player, onFailure, onSuccess });
  const requested = () => synthesize.mock.calls.map(([text]) => text);
  return {
    synthesizer,
    play,
    stop,
    synthesize,
    onFailure,
    onSuccess,
    requested,
    finishPlayback: async () => {
      finishPlayback();
      await settle();
    },
  };
}

describe("createProviderSpeechSynthesizer", () => {
  it("speaks queued texts one at a time, in order", async () => {
    const { synthesizer, play, requested, finishPlayback, onSuccess } = setUp();
    synthesizer.speakNext("one", "az");
    synthesizer.speakNext("two", "az");
    await settle();

    // The second text is not even requested until the first has been played.
    expect(requested()).toEqual(["one"]);
    expect(play).toHaveBeenCalledOnce();

    await finishPlayback();
    expect(requested()).toEqual(["one", "two"]);
    expect(onSuccess).toHaveBeenCalledOnce();
  });

  it("skips the oldest waiting texts when speech falls behind", async () => {
    const { synthesizer, requested, finishPlayback } = setUp();
    const texts = ["first", "a", "b", "c", "d"];
    for (const text of texts) {
      synthesizer.speakNext(text, "az");
    }
    for (let played = 0; played <= MAX_WAITING_TEXTS; played += 1) {
      await settle();
      await finishPlayback();
    }

    expect(requested()).toEqual(["first", "b", "c", "d"]);
  });

  it("replaces everything when asked to speak now", async () => {
    const { synthesizer, stop, requested } = setUp();
    synthesizer.speakNext("one", "az");
    synthesizer.speakNext("two", "az");
    await settle();

    synthesizer.speak("now", "en");
    await settle();

    expect(stop).toHaveBeenCalled();
    expect(requested()).toEqual(["one", "now"]);
  });

  it("does not play audio that arrives after it was stopped", async () => {
    const { synthesizer, synthesize, play, onFailure } = setUp();
    let deliver: (audio: ArrayBuffer) => void = () => undefined;
    synthesize.mockImplementationOnce(
      () =>
        new Promise<ArrayBuffer>((resolve) => {
          deliver = resolve;
        }),
    );
    synthesizer.speakNext("late", "az");
    await settle();

    synthesizer.stop();
    deliver(new ArrayBuffer(4));
    await settle();

    expect(play).not.toHaveBeenCalled();
    expect(onFailure).not.toHaveBeenCalled();
  });

  it("reports a failure and carries on with the next text", async () => {
    const { synthesizer, synthesize, onFailure, requested } = setUp();
    const reason = { code: "rate_limited" };
    // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
    synthesize.mockImplementationOnce(() => Promise.reject(reason));
    synthesizer.speakNext("one", "az");
    synthesizer.speakNext("two", "az");
    await settle();

    expect(onFailure).toHaveBeenCalledExactlyOnceWith(reason);
    expect(requested()).toEqual(["one", "two"]);
  });
});
