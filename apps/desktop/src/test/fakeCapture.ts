import { vi } from "vitest";
import type {
  CaptureConfig,
  CaptureEventHandlers,
  MicrophoneInfo,
  SessionFeatures,
} from "../capture/captureClient";

/**
 * A stand-in for the capture core, used by tests in place of
 * `capture/captureClient`. Tests drive it directly: they decide whether
 * starting succeeds and when level or interruption events arrive.
 */
export const fakeCapture = {
  microphones: [] as MicrophoneInfo[],
  startCapture: vi.fn<(config: CaptureConfig, features: SessionFeatures) => Promise<void>>(),
  refreshReplies: vi.fn<() => Promise<void>>(),
  stopCapture: vi.fn<() => Promise<void>>(),
  /** Whether the core is already capturing when the app loads. */
  isCapturing: false,
  handlers: null as CaptureEventHandlers | null,

  reset() {
    this.microphones = [];
    this.handlers = null;
    this.isCapturing = false;
    this.startCapture.mockReset().mockResolvedValue(undefined);
    this.stopCapture.mockReset().mockResolvedValue(undefined);
    this.refreshReplies.mockReset().mockResolvedValue(undefined);
  },

  /** The event handlers the app subscribed with. */
  events(): CaptureEventHandlers {
    if (this.handlers === null) {
      throw new Error("The app has not subscribed to capture events yet.");
    }
    return this.handlers;
  },
};

/** The module that replaces `capture/captureClient` in tests. */
export async function fakeCaptureClientModule() {
  const actual = await vi.importActual<typeof import("../capture/captureClient")>(
    "../capture/captureClient",
  );
  return {
    ...actual,
    listMicrophones: () => Promise.resolve(fakeCapture.microphones),
    startCapture: fakeCapture.startCapture,
    stopCapture: fakeCapture.stopCapture,
    isCapturing: () => Promise.resolve(fakeCapture.isCapturing),
    refreshReplies: fakeCapture.refreshReplies,
    subscribeToCapture: (handlers: CaptureEventHandlers) => {
      fakeCapture.handlers = handlers;
      return Promise.resolve(() => undefined);
    },
  };
}
