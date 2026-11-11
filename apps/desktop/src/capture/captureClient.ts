/**
 * Typed access to the Rust capture commands and events. This is the only
 * file in the capture feature that talks to Tauri; everything else works
 * with the plain types exported here.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  type AudioSource,
  type ReplySuggestion,
  type TranscriptSegment,
  PROVIDER_FAILURES,
  type ProviderFailure,
  type TranslationState,
} from "../domain/conversation";

export interface MicrophoneInfo {
  id: string;
  name: string;
  isDefault: boolean;
}

export interface CaptureConfig {
  microphone: boolean;
  /** `null` selects the system's default microphone. */
  microphoneId: string | null;
  systemAudio: boolean;
}

/** Loudness from 0 to 1. */
export interface Level {
  peak: number;
  rms: number;
}

export interface LevelsUpdate {
  /** `null` when the source is not being captured. */
  microphone: Level | null;
  system: Level | null;
  droppedFrames: number;
}

/** The provider the user chose for transcription. */
export type TranscriptionProviderChoice = { id: "openai" } | { id: "whisper"; model: string };

/** What to transcribe with; `null` language lets the provider decide. */
export interface TranscriptionRequest {
  provider: TranscriptionProviderChoice;
  language: string | null;
}

/** The provider the user chose for translation or for reply suggestions. */
export type ProviderChoice =
  | { id: "openai" }
  | { id: "custom"; baseUrl: string; model: string }
  | { id: "ollama"; endpoint: string; model: string };

/** How Arabic output should be written. */
export interface ArabicStyle {
  /** Name of the dialect to write in; `null` for Modern Standard Arabic. */
  dialect: string | null;
  tashkeel: "off" | "partial" | "full";
}

/** What to translate the transcript into, and the terms to keep consistent. */
export interface TranslationRequest {
  provider: ProviderChoice;
  targetLanguage: string;
  glossary: { term: string; preferred: string }[];
  arabic: ArabicStyle;
}

/** New progress on the translation of one segment. */
export interface TranslationEvent {
  segmentId: string;
  state: TranslationState;
}

/** How reply suggestions should be written. */
export interface ReplyRequest {
  provider: ProviderChoice;
  replyLanguage: string;
  /** Language to show each reply's meaning in; `null` for no meaning. */
  meaningLanguage: string | null;
  businessContext: string;
  arabic: ArabicStyle;
}

/**
 * What to do with a session's audio beyond capturing it. `null` leaves a
 * feature off. Translation and replies need transcription.
 */
export interface SessionFeatures {
  transcription: TranscriptionRequest | null;
  translation: TranslationRequest | null;
  replies: ReplyRequest | null;
  /** Local-only mode: the core refuses to start if any feature would use a cloud service. */
  localOnly: boolean;
}

/** A change to the reply suggestions. Mirrors the Rust `ReplyEvent`. */
export type ReplyEvent =
  | { status: "pending" }
  | { status: "ready"; suggestions: ReplySuggestion[] }
  | { status: "failed"; reason: ProviderFailure };

/** A change to the live transcript. Mirrors the Rust `TranscriptEvent`. */
export type TranscriptEvent =
  | { kind: "segment"; segment: TranscriptSegment }
  | { kind: "discarded"; id: string }
  | { kind: "failed"; error: CaptureError };

export const CAPTURE_ERROR_CODES = [
  // Audio capture
  "no_source_selected",
  "invalid_microphone_selection",
  "already_running",
  "permission_denied",
  "device_unavailable",
  "unsupported",
  "interrupted",
  "backend",
  // Transcription
  "missing_api_key",
  "authentication_failed",
  "rate_limited",
  "unsupported_language",
  "connection_failed",
  "connection_lost",
  "timeout",
  "malformed_response",
  "model_unavailable",
  "provider",
  // Session rules
  "local_only",
] as const;

export type CaptureErrorCode = (typeof CAPTURE_ERROR_CODES)[number] | "unknown";

/**
 * Why capture or transcription failed or stopped. Mirrors the Rust
 * `CaptureError` and `TranscriptionError`, which share one set of codes.
 */
export interface CaptureError {
  code: CaptureErrorCode;
  /** The source the error is about, when it concerns a single one. */
  audioSource: AudioSource | null;
}

const UNKNOWN_ERROR: CaptureError = { code: "unknown", audioSource: null };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/** Converts whatever a command rejected with into a `CaptureError`. */
export function toCaptureError(value: unknown): CaptureError {
  if (!isRecord(value)) {
    return UNKNOWN_ERROR;
  }
  const code = CAPTURE_ERROR_CODES.find((known) => known === value.code);
  if (code === undefined) {
    return UNKNOWN_ERROR;
  }
  const source = value.audioSource;
  return { code, audioSource: source === "microphone" || source === "system" ? source : null };
}

function isMicrophoneInfo(value: unknown): value is MicrophoneInfo {
  return (
    isRecord(value) &&
    typeof value.id === "string" &&
    typeof value.name === "string" &&
    typeof value.isDefault === "boolean"
  );
}

function toLevel(value: unknown): Level | null {
  return isRecord(value) && typeof value.peak === "number" && typeof value.rms === "number"
    ? { peak: value.peak, rms: value.rms }
    : null;
}

function toLevelsUpdate(value: unknown): LevelsUpdate {
  const fields = isRecord(value) ? value : {};
  return {
    microphone: toLevel(fields.microphone),
    system: toLevel(fields.system),
    droppedFrames: typeof fields.droppedFrames === "number" ? fields.droppedFrames : 0,
  };
}

function toSegment(value: unknown): TranscriptSegment | null {
  if (!isRecord(value)) {
    return null;
  }
  const { id, source, startMs, text, language, isFinal } = value;
  const isValid =
    typeof id === "string" &&
    (source === "microphone" || source === "system") &&
    typeof startMs === "number" &&
    typeof text === "string" &&
    (typeof language === "string" || language === null) &&
    typeof isFinal === "boolean";
  return isValid ? { id, source, startMs, text, language, isFinal } : null;
}

/** Validates a transcript event from the core; returns `null` if it is malformed. */
export function toTranscriptEvent(value: unknown): TranscriptEvent | null {
  if (!isRecord(value)) {
    return null;
  }
  switch (value.kind) {
    case "segment": {
      const segment = toSegment(value.segment);
      return segment === null ? null : { kind: "segment", segment };
    }
    case "discarded":
      return typeof value.id === "string" ? { kind: "discarded", id: value.id } : null;
    case "failed":
      return { kind: "failed", error: toCaptureError(value.error) };
    default:
      return null;
  }
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}

function toTranslationState(event: Record<string, unknown>): TranslationState | null {
  switch (event.status) {
    case "pending":
      return { status: "pending" };
    case "done": {
      const translation = event.translation;
      if (
        !isRecord(translation) ||
        typeof translation.text !== "string" ||
        typeof translation.targetLanguage !== "string"
      ) {
        return null;
      }
      return {
        status: "done",
        text: translation.text,
        targetLanguage: translation.targetLanguage,
        unpreserved: isStringArray(translation.unpreserved) ? translation.unpreserved : [],
      };
    }
    case "failed":
      return { status: "failed", reason: toProviderFailure(event.error) };
    default:
      return null;
  }
}

/** Validates a translation event from the core; returns `null` if it is malformed. */
export function toTranslationEvent(value: unknown): TranslationEvent | null {
  if (!isRecord(value) || typeof value.segmentId !== "string") {
    return null;
  }
  const state = toTranslationState(value);
  return state === null ? null : { segmentId: value.segmentId, state };
}

function toProviderFailure(error: unknown): ProviderFailure {
  const code = isRecord(error) ? error.code : undefined;
  return PROVIDER_FAILURES.find((known) => known === code) ?? "unknown";
}

function toReplySuggestion(value: unknown): ReplySuggestion | null {
  if (!isRecord(value)) {
    return null;
  }
  const { style, text, meaning, asksForMissingFacts } = value;
  const isValid =
    (style === "professional" || style === "friendly" || style === "concise") &&
    typeof text === "string" &&
    (typeof meaning === "string" || meaning === null) &&
    typeof asksForMissingFacts === "boolean";
  return isValid ? { style, text, meaning, asksForMissingFacts } : null;
}

/** Validates a reply event from the core; returns `null` if it is malformed. */
export function toReplyEvent(value: unknown): ReplyEvent | null {
  if (!isRecord(value)) {
    return null;
  }
  switch (value.status) {
    case "pending":
      return { status: "pending" };
    case "ready": {
      const suggestions = Array.isArray(value.suggestions)
        ? value.suggestions.map(toReplySuggestion).filter((suggestion) => suggestion !== null)
        : [];
      return { status: "ready", suggestions };
    }
    case "failed":
      return { status: "failed", reason: toProviderFailure(value.error) };
    default:
      return null;
  }
}

export async function listMicrophones(): Promise<MicrophoneInfo[]> {
  const reply: unknown = await invoke("list_microphones");
  return Array.isArray(reply) ? reply.filter(isMicrophoneInfo) : [];
}

/**
 * Starts capture with the given features.
 * Rejects with the core's error value; pass it to `toCaptureError`.
 */
export async function startCapture(
  config: CaptureConfig,
  features: SessionFeatures,
): Promise<void> {
  await invoke("start_capture", { config, features });
}

/** Asks for fresh reply suggestions for the running session. */
export async function refreshReplies(): Promise<void> {
  await invoke("refresh_replies");
}

export async function stopCapture(): Promise<void> {
  await invoke("stop_capture");
}

/** Whether the core is capturing audio right now. */
export async function isCapturing(): Promise<boolean> {
  return (await invoke("is_capturing")) === true;
}

export interface CaptureEventHandlers {
  onLevels: (update: LevelsUpdate) => void;
  onInterrupted: (error: CaptureError) => void;
  onTranscript: (event: TranscriptEvent) => void;
  onTranslation: (event: TranslationEvent) => void;
  onReplies: (event: ReplyEvent) => void;
}

/** Subscribes to capture events. Resolves to a function that unsubscribes. */
export async function subscribeToCapture(handlers: CaptureEventHandlers): Promise<() => void> {
  const unlistenLevels = await listen("capture-levels", (event) => {
    handlers.onLevels(toLevelsUpdate(event.payload));
  });
  const unlistenInterrupted = await listen("capture-interrupted", (event) => {
    handlers.onInterrupted(toCaptureError(event.payload));
  });
  const unlistenTranscript = await listen("transcript-event", (event) => {
    const transcriptEvent = toTranscriptEvent(event.payload);
    if (transcriptEvent !== null) {
      handlers.onTranscript(transcriptEvent);
    }
  });
  const unlistenTranslation = await listen("translation-event", (event) => {
    const translationEvent = toTranslationEvent(event.payload);
    if (translationEvent !== null) {
      handlers.onTranslation(translationEvent);
    }
  });
  const unlistenReplies = await listen("reply-event", (event) => {
    const replyEvent = toReplyEvent(event.payload);
    if (replyEvent !== null) {
      handlers.onReplies(replyEvent);
    }
  });
  return () => {
    unlistenLevels();
    unlistenInterrupted();
    unlistenTranscript();
    unlistenTranslation();
    unlistenReplies();
  };
}
