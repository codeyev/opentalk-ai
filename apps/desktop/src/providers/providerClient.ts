/**
 * Typed access to the Rust commands for the transcription provider and its
 * API key. The key only ever travels from the UI to the core: it can be
 * saved and removed here, but never read back.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface ProviderStatus {
  id: string;
  displayName: string;
  location: "local" | "cloud";
  streamsPartialResults: boolean;
  detectsLanguage: boolean;
  /** `null` when the provider does not publish a list of supported languages. */
  languages: string[] | null;
  hasApiKey: boolean;
}

/** The translation provider as described by the core. */
export interface TranslationProviderStatus {
  id: string;
  displayName: string;
  isCloud: boolean;
  /** Whether the provider can be asked for a named Arabic dialect. */
  acceptsArabicDialect: boolean;
  /** `null` when the provider does not publish a list of supported languages. */
  languages: string[] | null;
  hasApiKey: boolean;
}

/** The reply provider as described by the core. */
export interface ReplyProviderStatus {
  id: string;
  displayName: string;
  isCloud: boolean;
  /** Whether the provider can be asked for a named Arabic dialect. */
  acceptsArabicDialect: boolean;
  hasApiKey: boolean;
}

/** A model installed in the local Ollama server. */
export interface LocalModel {
  name: string;
  sizeBytes: number;
}

/** What a running local Ollama server reports about itself. */
export interface OllamaStatus {
  version: string;
  models: LocalModel[];
}

export const OLLAMA_ERROR_CODES = [
  "not_local",
  "not_running",
  "timeout",
  "malformed_response",
  "server",
] as const;

export type OllamaErrorCode = (typeof OLLAMA_ERROR_CODES)[number] | "unknown";

export type KeyErrorCode = "invalid_key" | "store_unavailable";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function toProviderStatus(value: unknown): ProviderStatus {
  if (!isRecord(value)) {
    throw new Error("Unexpected provider description from the application core.");
  }
  const { id, displayName, location, streamsPartialResults, detectsLanguage, languages } = value;
  if (
    typeof id !== "string" ||
    typeof displayName !== "string" ||
    (location !== "local" && location !== "cloud") ||
    typeof streamsPartialResults !== "boolean" ||
    typeof detectsLanguage !== "boolean" ||
    typeof value.hasApiKey !== "boolean"
  ) {
    throw new Error("Unexpected provider description from the application core.");
  }
  return {
    id,
    displayName,
    location,
    streamsPartialResults,
    detectsLanguage,
    languages: isStringArray(languages) ? languages : null,
    hasApiKey: value.hasApiKey,
  };
}

/** Converts whatever a key command rejected with into a known error code. */
export function toKeyErrorCode(value: unknown): KeyErrorCode {
  return isRecord(value) && value.code === "invalid_key" ? "invalid_key" : "store_unavailable";
}

export async function getTranscriptionProvider(): Promise<ProviderStatus> {
  return toProviderStatus(await invoke("transcription_provider"));
}

export async function getTranslationProvider(): Promise<TranslationProviderStatus> {
  const value: unknown = await invoke("translation_provider");
  if (
    !isRecord(value) ||
    typeof value.id !== "string" ||
    typeof value.displayName !== "string" ||
    typeof value.isCloud !== "boolean" ||
    typeof value.hasApiKey !== "boolean"
  ) {
    throw new Error("Unexpected provider description from the application core.");
  }
  return {
    id: value.id,
    displayName: value.displayName,
    isCloud: value.isCloud,
    acceptsArabicDialect: value.acceptsArabicDialect === true,
    languages: isStringArray(value.languages) ? value.languages : null,
    hasApiKey: value.hasApiKey,
  };
}

export async function getReplyProvider(): Promise<ReplyProviderStatus> {
  const value: unknown = await invoke("reply_provider");
  if (
    !isRecord(value) ||
    typeof value.id !== "string" ||
    typeof value.displayName !== "string" ||
    typeof value.isCloud !== "boolean" ||
    typeof value.hasApiKey !== "boolean"
  ) {
    throw new Error("Unexpected provider description from the application core.");
  }
  const { id, displayName, isCloud, hasApiKey } = value;
  const acceptsArabicDialect = value.acceptsArabicDialect === true;
  return { id, displayName, isCloud, acceptsArabicDialect, hasApiKey };
}

/** Converts whatever the status check rejected with into a known error code. */
export function toOllamaErrorCode(value: unknown): OllamaErrorCode {
  const code = isRecord(value) ? value.code : undefined;
  return OLLAMA_ERROR_CODES.find((known) => known === code) ?? "unknown";
}

function toLocalModel(value: unknown): LocalModel | null {
  return isRecord(value) && typeof value.name === "string" && typeof value.sizeBytes === "number"
    ? { name: value.name, sizeBytes: value.sizeBytes }
    : null;
}

/**
 * Asks the Ollama server at `endpoint` for its version and installed models.
 * Rejects with the core's error value; pass it to `toOllamaErrorCode`.
 */
export async function getOllamaStatus(endpoint: string): Promise<OllamaStatus> {
  const value: unknown = await invoke("ollama_status", { endpoint });
  if (!isRecord(value) || typeof value.version !== "string" || !Array.isArray(value.models)) {
    throw new Error("Unexpected status from the application core.");
  }
  return {
    version: value.version,
    models: value.models.map(toLocalModel).filter((model) => model !== null),
  };
}

/** The services an API key can be saved for. */
export type KeyOwner = "openai" | "custom";

/**
 * Says which service a key belongs to. The user's own service is
 * identified by its address, so each service keeps a key of its own.
 */
export type KeyTarget = { owner: "openai" } | { owner: "custom"; baseUrl: string };

function keyArguments(target: KeyTarget) {
  return {
    provider: target.owner,
    baseUrl: target.owner === "custom" ? target.baseUrl : null,
  };
}

/** Whether a key is saved for the OpenAI-compatible service at `baseUrl`. */
export async function getCustomProvider(baseUrl: string): Promise<{ hasApiKey: boolean }> {
  const value: unknown = await invoke("custom_provider", { baseUrl });
  return { hasApiKey: isRecord(value) && value.hasApiKey === true };
}

/**
 * Saves the key for one service in the system credential store.
 * Rejects with the core's error value; pass it to `toKeyErrorCode`.
 */
export async function saveApiKey(target: KeyTarget, apiKey: string): Promise<void> {
  await invoke("save_api_key", { ...keyArguments(target), apiKey });
}

export async function deleteApiKey(target: KeyTarget): Promise<void> {
  await invoke("delete_api_key", keyArguments(target));
}

/** A Whisper model file in the app's models folder. */
export interface InstalledModel {
  fileName: string;
  sizeBytes: number;
}

/** A Whisper model the app can download on request. */
export interface CatalogueModel {
  id: string;
  fileName: string;
  sizeBytes: number;
}

export interface WhisperModels {
  /** The models folder, where the user can also put model files of their own. */
  directory: string;
  installed: InstalledModel[];
  catalogue: CatalogueModel[];
}

export const DOWNLOAD_ERROR_CODES = [
  "local_only",
  "unknown_model",
  "busy",
  "storage_failed",
  "connection_failed",
  "cancelled",
  "verification_failed",
] as const;

export type DownloadErrorCode = (typeof DOWNLOAD_ERROR_CODES)[number] | "unknown";

/** Converts whatever a download rejected with into a known error code. */
export function toDownloadErrorCode(value: unknown): DownloadErrorCode {
  const code = isRecord(value) ? value.code : undefined;
  return DOWNLOAD_ERROR_CODES.find((known) => known === code) ?? "unknown";
}

function toModelFile(value: unknown): InstalledModel | null {
  return isRecord(value) &&
    typeof value.fileName === "string" &&
    typeof value.sizeBytes === "number"
    ? { fileName: value.fileName, sizeBytes: value.sizeBytes }
    : null;
}

function toCatalogueModel(value: unknown): CatalogueModel | null {
  const file = toModelFile(value);
  return file !== null && isRecord(value) && typeof value.id === "string"
    ? { ...file, id: value.id }
    : null;
}

/** Lists the installed Whisper models and the ones that can be downloaded. */
export async function getWhisperModels(): Promise<WhisperModels> {
  const value: unknown = await invoke("whisper_models");
  if (
    !isRecord(value) ||
    typeof value.directory !== "string" ||
    !Array.isArray(value.installed) ||
    !Array.isArray(value.catalogue)
  ) {
    throw new Error("Unexpected model list from the application core.");
  }
  return {
    directory: value.directory,
    installed: value.installed.map(toModelFile).filter((model) => model !== null),
    catalogue: value.catalogue.map(toCatalogueModel).filter((model) => model !== null),
  };
}

/**
 * Downloads a catalogue model. Only ever called because the user pressed
 * Download. Rejects with the core's error value; pass it to `toDownloadErrorCode`.
 */
export async function downloadWhisperModel(id: string, localOnly: boolean): Promise<void> {
  await invoke("download_whisper_model", { id, localOnly });
}

export async function cancelModelDownload(): Promise<void> {
  await invoke("cancel_model_download");
}

/** Calls `onProgress` with the bytes received while a model downloads. Resolves to a function that unsubscribes. */
export async function subscribeToModelDownload(
  onProgress: (receivedBytes: number) => void,
): Promise<() => void> {
  return listen("model-download", (event) => {
    const payload: unknown = event.payload;
    if (isRecord(payload) && typeof payload.receivedBytes === "number") {
      onProgress(payload.receivedBytes);
    }
  });
}
