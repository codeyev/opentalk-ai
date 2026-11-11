/**
 * Vendor-neutral conversation types shared by the UI.
 * They mirror the core domain types described in docs/ARCHITECTURE.md.
 */

export type SessionState = "idle" | "listening" | "processing" | "error";

export type AudioSource = "microphone" | "system";

export interface TranscriptSegment {
  id: string;
  source: AudioSource;
  /** Offset from the start of the session, in milliseconds. */
  startMs: number;
  text: string;
  /** BCP 47 language tag of the spoken text, or `null` if it is not known. */
  language: string | null;
  /** Partial segments may still change; final segments will not. */
  isFinal: boolean;
}

/** Where the translation of one transcript segment stands. */
export type TranslationState =
  | { status: "pending" }
  | {
      status: "done";
      text: string;
      targetLanguage: string;
      /** Numbers, links and glossary terms that may not have been carried over. */
      unpreserved: readonly string[];
    }
  | { status: "failed"; reason: ProviderFailure };

export const PROVIDER_FAILURES = [
  "missing_api_key",
  "authentication_failed",
  "rate_limited",
  "unsupported_language",
  "connection_failed",
  "timeout",
  "malformed_response",
  "refused",
  "model_unavailable",
  "overloaded",
  "provider",
] as const;

export type ProviderFailure = (typeof PROVIDER_FAILURES)[number] | "unknown";

/** Translation states by the id of the segment they belong to. */
export type Translations = Readonly<Record<string, TranslationState>>;

export type ReplyStyle = "professional" | "friendly" | "concise";

export interface ReplySuggestion {
  style: ReplyStyle;
  text: string;
  /** The reply's meaning in the user's own language, when one was requested. */
  meaning: string | null;
  /** The reply asks a question because a fact it would need was not available. */
  asksForMissingFacts: boolean;
}

/** The reply suggestions and how current they are. */
export interface Replies {
  /**
   * `pending` means newer suggestions are on their way and the ones listed
   * are out of date; `failed` means the last attempt did not work.
   */
  status: "idle" | "pending" | "ready" | "failed";
  suggestions: readonly ReplySuggestion[];
  /** Why the last attempt failed; `null` unless `status` is `failed`. */
  failure: ProviderFailure | null;
  /** Counts up with every new set of suggestions. */
  generation: number;
}

export const NO_REPLIES: Replies = {
  status: "idle",
  suggestions: [],
  failure: null,
  generation: 0,
};

export interface Conversation {
  segments: readonly TranscriptSegment[];
  translations: Translations;
  replies: Replies;
}

export const EMPTY_CONVERSATION: Conversation = {
  segments: [],
  translations: {},
  replies: NO_REPLIES,
};
