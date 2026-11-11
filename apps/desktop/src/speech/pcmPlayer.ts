/** Plays the audio a speech provider returns. */
export interface PcmPlayer {
  /**
   * Plays mono, 16-bit little-endian PCM at 24 kHz. Resolves when it has
   * been played to the end, or was stopped.
   */
  play(pcm: ArrayBuffer): Promise<void>;
  /** Ends what is playing now. */
  stop(): void;
}

/** Sample rate of the audio the core delivers; see `crates/speech`. */
const SAMPLE_RATE_HZ = 24_000;
const BYTES_PER_SAMPLE = 2;
/** The largest magnitude of a 16-bit sample. */
const FULL_SCALE = 32_768;

/** Converts 16-bit little-endian PCM to samples between -1 and 1. */
export function pcmToSamples(pcm: ArrayBuffer): Float32Array<ArrayBuffer> {
  const view = new DataView(pcm);
  const samples = new Float32Array(Math.floor(pcm.byteLength / BYTES_PER_SAMPLE));
  for (let index = 0; index < samples.length; index += 1) {
    samples[index] = view.getInt16(index * BYTES_PER_SAMPLE, true) / FULL_SCALE;
  }
  return samples;
}

/** A player that uses the webview's audio output, or `null` where it has none. */
export function createPcmPlayer(): PcmPlayer | null {
  if (!("AudioContext" in window)) {
    return null;
  }
  // Created on first use: a webview may refuse audio before the user has acted.
  let context: AudioContext | null = null;
  let playing: AudioBufferSourceNode | null = null;

  return {
    play: async (pcm) => {
      const samples = pcmToSamples(pcm);
      if (samples.length === 0) {
        return;
      }
      context ??= new AudioContext();
      await context.resume();
      const buffer = context.createBuffer(1, samples.length, SAMPLE_RATE_HZ);
      buffer.copyToChannel(samples, 0);
      const source = context.createBufferSource();
      source.buffer = buffer;
      source.connect(context.destination);
      playing = source;
      await new Promise<void>((resolve) => {
        // Fires both at the natural end and after `stop()`.
        source.addEventListener("ended", () => {
          resolve();
        });
        source.start();
      });
      if (playing === source) {
        playing = null;
      }
    },
    stop: () => {
      playing?.stop();
      playing = null;
    },
  };
}
