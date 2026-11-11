import type { ProviderChoice } from "../capture/captureClient";
import { hostOf, isAcceptableServiceUrl, isLoopbackUrl } from "../lib/url";
import type { LlmProviderId, Settings } from "../settings/settings";

const OLLAMA_NAME = "Ollama";
/** Shown for OpenAI while the core has not described its provider. */
const OPENAI_NAME = "OpenAI";

/** One of the providers that can translate and suggest replies. */
export interface LlmProvider {
  /** Name shown to the user; for the custom service, its host name. */
  name: string;
  /** Whether using it sends data to another machine. */
  isCloud: boolean;
  /** What to send to the core to use it, or `null` if it is not set up yet. */
  choice: ProviderChoice | null;
}

/** What the core reported about the providers that need an API key. */
export interface KeyStatus {
  /** OpenAI's name and key status; `null` if the core cannot be reached. */
  openai: { displayName: string; hasApiKey: boolean } | null;
  customHasApiKey: boolean;
}

/**
 * Describes every provider a language-model feature can use, and whether
 * each is ready, from the settings and the saved-key status.
 */
export function describeLlmProviders(
  settings: Settings,
  keys: KeyStatus,
): Record<LlmProviderId, LlmProvider> {
  const { customBaseUrl, customModel, ollamaEndpoint, ollamaModel } = settings;
  const customIsLocal = isLoopbackUrl(customBaseUrl);
  // A service on this computer may not need a key at all.
  const customIsReady =
    isAcceptableServiceUrl(customBaseUrl) &&
    customModel.trim() !== "" &&
    (keys.customHasApiKey || customIsLocal);

  return {
    openai: {
      name: keys.openai?.displayName ?? OPENAI_NAME,
      isCloud: true,
      choice: keys.openai?.hasApiKey ? { id: "openai" } : null,
    },
    custom: {
      name: hostOf(customBaseUrl) ?? "",
      isCloud: !customIsLocal,
      choice: customIsReady
        ? { id: "custom", baseUrl: customBaseUrl.trim(), model: customModel.trim() }
        : null,
    },
    ollama: {
      name: OLLAMA_NAME,
      isCloud: false,
      choice:
        ollamaModel === "" ? null : { id: "ollama", endpoint: ollamaEndpoint, model: ollamaModel },
    },
  };
}

/**
 * The provider a feature will actually use. Local-only mode replaces a
 * cloud choice with the local model, so nothing can reach a cloud service.
 */
export function effectiveProvider(
  settings: Settings,
  providers: Record<LlmProviderId, LlmProvider>,
  chosen: LlmProviderId,
): LlmProviderId {
  return settings.localOnly && providers[chosen].isCloud ? "ollama" : chosen;
}
