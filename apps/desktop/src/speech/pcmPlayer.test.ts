import { describe, expect, it } from "vitest";
import { createPcmPlayer, pcmToSamples } from "./pcmPlayer";

describe("pcmToSamples", () => {
  it("reads 16-bit little-endian samples as values between -1 and 1", () => {
    // 0, the largest positive sample, the largest negative sample.
    const pcm = new Uint8Array([0x00, 0x00, 0xff, 0x7f, 0x00, 0x80]).buffer;
    const samples = pcmToSamples(pcm);
    expect(samples[0]).toBe(0);
    expect(samples[1]).toBeCloseTo(1, 4);
    expect(samples[2]).toBe(-1);
  });

  it("ignores a trailing half sample", () => {
    expect(pcmToSamples(new Uint8Array([0, 0, 1]).buffer)).toHaveLength(1);
    expect(pcmToSamples(new ArrayBuffer(0))).toHaveLength(0);
  });
});

describe("createPcmPlayer", () => {
  it("is unavailable where the webview has no audio output", () => {
    expect(createPcmPlayer()).toBeNull();
  });
});
