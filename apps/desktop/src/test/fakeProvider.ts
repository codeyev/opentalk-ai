import { vi } from "vitest";
import type {
  KeyTarget,
  SpeechChoice,
  OllamaStatus,
  ProviderStatus,
  TranslationProviderStatus,
  WhisperModels,
} from "../providers/providerClient";

const OPENAI: ProviderStatus = {
  id: "openai",
  displayName: "OpenAI",
  location: "cloud",
  streamsPartialResults: true,
  detectsLanguage: false,
  languages: null,
  hasApiKey: false,
};

const OPENAI_TRANSLATION: TranslationProviderStatus = {
  id: "openai",
  displayName: "OpenAI",
  isCloud: true,
  acceptsArabicDialect: true,
  languages: null,
  hasApiKey: false,
};

const EMPTY_WHISPER: WhisperModels = {
  directory: "/app-data/models",
  installed: [],
  catalogue: [
    { id: "tiny", fileName: "ggml-tiny.bin", sizeBytes: 77_691_713 },
    { id: "base", fileName: "ggml-base.bin", sizeBytes: 147_951_465 },
  ],
};

/**
 * A stand-in for the provider commands, used by tests in place of
 * `providers/providerClient`. It keeps the "saved key" in memory only.
 */
export const fakeProvider = {
  /** `null` simulates a core that cannot be reached. */
  status: OPENAI as ProviderStatus | null,
  /** What the local Ollama server reports; `null` simulates a server that is not running. */
  ollama: null as OllamaStatus | null,
  /** Addresses of the user's own services that have a saved key. Each has its own. */
  customKeys: new Set<string>(),
  hasElevenLabsKey: false,
  /** The Whisper models folder as the core reports it. */
  whisper: EMPTY_WHISPER,
  saveApiKey: vi.fn<(target: KeyTarget, apiKey: string) => Promise<void>>(),
  deleteApiKey: vi.fn<(target: KeyTarget) => Promise<void>>(),
  downloadWhisperModel: vi.fn<(id: string, localOnly: boolean) => Promise<void>>(),
  /** Resolves to four silent bytes unless a test says otherwise. */
  synthesizeSpeech:
    vi.fn<
      (
        provider: SpeechChoice,
        text: string,
        language: string,
        localOnly: boolean,
      ) => Promise<ArrayBuffer>
    >(),

  reset({ hasApiKey = false } = {}) {
    this.status = { ...OPENAI, hasApiKey };
    this.ollama = null;
    this.customKeys = new Set();
    this.hasElevenLabsKey = false;
    this.synthesizeSpeech.mockReset().mockImplementation(() => Promise.resolve(new ArrayBuffer(4)));
    this.whisper = EMPTY_WHISPER;
    // Each service has its own key, as in the real credential store.
    const setKey = (target: KeyTarget, isSaved: boolean) => {
      if (target.owner === "openai") {
        this.status = { ...OPENAI, hasApiKey: isSaved };
      } else if (target.owner === "elevenlabs") {
        this.hasElevenLabsKey = isSaved;
      } else if (isSaved) {
        this.customKeys.add(target.baseUrl);
      } else {
        this.customKeys.delete(target.baseUrl);
      }
      return Promise.resolve();
    };
    this.saveApiKey.mockReset().mockImplementation((target) => setKey(target, true));
    this.deleteApiKey.mockReset().mockImplementation((target) => setKey(target, false));
    // A finished download shows up as an installed model.
    this.downloadWhisperModel.mockReset().mockImplementation((id) => {
      const model = this.whisper.catalogue.find((entry) => entry.id === id);
      if (model !== undefined) {
        this.whisper = { ...this.whisper, installed: [...this.whisper.installed, model] };
      }
      return Promise.resolve();
    });
  },
};

/** Rejects with a plain value, the way a failed command from the core does. */
function rejectWith(value: unknown): Promise<never> {
  // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
  return Promise.reject(value);
}

/** The module that replaces `providers/providerClient` in tests. */
export async function fakeProviderClientModule() {
  const actual = await vi.importActual<typeof import("../providers/providerClient")>(
    "../providers/providerClient",
  );
  return {
    ...actual,
    getTranscriptionProvider: () =>
      fakeProvider.status === null
        ? Promise.reject(new Error("no core"))
        : Promise.resolve(fakeProvider.status),
    // Every role uses the same provider, so they all share its saved key.
    getTranslationProvider: () =>
      fakeProvider.status === null
        ? Promise.reject(new Error("no core"))
        : Promise.resolve({ ...OPENAI_TRANSLATION, hasApiKey: fakeProvider.status.hasApiKey }),
    getReplyProvider: () =>
      fakeProvider.status === null
        ? Promise.reject(new Error("no core"))
        : Promise.resolve({
            id: "openai",
            displayName: "OpenAI",
            isCloud: true,
            acceptsArabicDialect: true,
            hasApiKey: fakeProvider.status.hasApiKey,
          }),
    getCustomProvider: (baseUrl: string) =>
      Promise.resolve({ hasApiKey: fakeProvider.customKeys.has(baseUrl) }),
    hasApiKey: (target: KeyTarget) =>
      Promise.resolve(
        target.owner === "openai"
          ? fakeProvider.status?.hasApiKey === true
          : target.owner === "elevenlabs"
            ? fakeProvider.hasElevenLabsKey
            : fakeProvider.customKeys.has(target.baseUrl),
      ),
    synthesizeSpeech: fakeProvider.synthesizeSpeech,
    getWhisperModels: () => Promise.resolve(fakeProvider.whisper),
    downloadWhisperModel: fakeProvider.downloadWhisperModel,
    cancelModelDownload: () => Promise.resolve(),
    subscribeToModelDownload: () => Promise.resolve(() => undefined),
    getOllamaStatus: () =>
      fakeProvider.ollama === null
        ? rejectWith({ code: "not_running" })
        : Promise.resolve(fakeProvider.ollama),
    saveApiKey: fakeProvider.saveApiKey,
    deleteApiKey: fakeProvider.deleteApiKey,
  };
}
