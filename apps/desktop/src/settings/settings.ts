/**
 * User preferences kept by the UI. None of these are secrets; API keys must
 * never be added here (see docs/SECURITY.md).
 */

import { canonicalLanguageTag } from "../lib/language";

export const THEMES = ["system", "light", "dark"] as const;
export type Theme = (typeof THEMES)[number];

export const UI_LOCALES = ["en", "az", "ru"] as const;
export type UiLocale = (typeof UI_LOCALES)[number];

export const ARABIC_VARIETIES = ["msa", "dialect"] as const;
export type ArabicVariety = (typeof ARABIC_VARIETIES)[number];

/** Providers that can transcribe speech. */
export const TRANSCRIPTION_PROVIDERS = ["openai", "whisper"] as const;
export type TranscriptionProviderId = (typeof TRANSCRIPTION_PROVIDERS)[number];

/** Providers that can translate and suggest replies. */
export const LLM_PROVIDERS = ["openai", "custom", "ollama"] as const;
export type LlmProviderId = (typeof LLM_PROVIDERS)[number];

/**
 * What reads text aloud: the voices installed on this computer, ElevenLabs,
 * or any service with an OpenAI-style speech endpoint.
 */
export const SPEECH_PROVIDERS = ["system", "elevenlabs", "compatible"] as const;
export type SpeechProviderId = (typeof SPEECH_PROVIDERS)[number];

/** An ElevenLabs model that speaks the most languages, Azerbaijani among them. */
export const DEFAULT_ELEVENLABS_MODEL = "eleven_v3";
/** An example of a service with an OpenAI-style speech endpoint. */
export const DEFAULT_SPEECH_BASE_URL = "https://api.openai.com/v1";

export const DEFAULT_OLLAMA_ENDPOINT = "http://localhost:11434";
/** An example of an OpenAI-compatible service address. */
export const DEFAULT_CUSTOM_BASE_URL = "https://openrouter.ai/api/v1";

export const TASHKEEL_LEVELS = ["off", "partial", "full"] as const;
export type Tashkeel = (typeof TASHKEEL_LEVELS)[number];

/** Spoken-language value meaning "let the provider detect it". */
export const AUTO_LANGUAGE = "auto";

export const MAX_BUSINESS_CONTEXT_LENGTH = 4000;
export const MAX_GLOSSARY_ENTRIES = 100;
export const MAX_SHORT_TEXT_LENGTH = 120;
export const MAX_ENDPOINT_LENGTH = 200;

export interface GlossaryEntry {
  id: string;
  term: string;
  preferred: string;
}

export interface Settings {
  theme: Theme;
  uiLocale: UiLocale;
  /** Lays the whole interface out right-to-left, for RTL testing. */
  rtlPreview: boolean;
  /** `AUTO_LANGUAGE` or a BCP 47 tag. */
  sourceLanguage: string;
  targetLanguage: string;
  replyLanguage: string;
  arabicVariety: ArabicVariety;
  arabicDialectLabel: string;
  tashkeel: Tashkeel;
  businessContext: string;
  glossary: GlossaryEntry[];
  /** Whether captured speech is sent to the transcription provider. Off by default. */
  transcriptionEnabled: boolean;
  /** Whether transcript text is sent to the translation provider. Off by default. */
  translationEnabled: boolean;
  /** Whether the conversation is sent to the reply provider. Off by default. */
  repliesEnabled: boolean;
  /** Whether reply suggestions get a Speak button. Off by default. */
  speechEnabled: boolean;
  /**
   * Whether translations of what the other side says are read aloud as
   * they arrive. Off by default.
   */
  speakTranslations: boolean;
  speechProvider: SpeechProviderId;
  /** Identifier of the ElevenLabs voice to use, or empty if none was entered. */
  elevenlabsVoice: string;
  elevenlabsModel: string;
  /** Base URL of a service with an OpenAI-style speech endpoint. */
  speechBaseUrl: string;
  /** Speech model and voice names on that service, or empty if none were entered. */
  speechModel: string;
  speechVoice: string;
  /** Never use a cloud provider. Off by default. */
  localOnly: boolean;
  transcriptionProvider: TranscriptionProviderId;
  /** File name of the Whisper model to use, or empty if none was chosen. */
  whisperModel: string;
  translationProvider: LlmProviderId;
  repliesProvider: LlmProviderId;
  /** Address of the Ollama server on this computer. */
  ollamaEndpoint: string;
  /** Name of the local model to use, or empty if none was chosen. */
  ollamaModel: string;
  /** Base URL of the user's own OpenAI-compatible service, such as OpenRouter. */
  customBaseUrl: string;
  /** Model name on that service, or empty if none was entered. */
  customModel: string;
}

export const DEFAULT_SETTINGS: Settings = {
  theme: "system",
  uiLocale: "en",
  rtlPreview: false,
  sourceLanguage: AUTO_LANGUAGE,
  targetLanguage: "en",
  replyLanguage: "en",
  arabicVariety: "msa",
  arabicDialectLabel: "",
  tashkeel: "off",
  businessContext: "",
  glossary: [],
  transcriptionEnabled: false,
  translationEnabled: false,
  repliesEnabled: false,
  speechEnabled: false,
  speakTranslations: false,
  speechProvider: "system",
  elevenlabsVoice: "",
  elevenlabsModel: DEFAULT_ELEVENLABS_MODEL,
  speechBaseUrl: DEFAULT_SPEECH_BASE_URL,
  speechModel: "",
  speechVoice: "",
  localOnly: false,
  transcriptionProvider: "openai",
  whisperModel: "",
  translationProvider: "openai",
  repliesProvider: "openai",
  ollamaEndpoint: DEFAULT_OLLAMA_ENDPOINT,
  ollamaModel: "",
  customBaseUrl: DEFAULT_CUSTOM_BASE_URL,
  customModel: "",
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function oneOf<T extends string>(allowed: readonly T[], value: unknown, fallback: T): T {
  return allowed.find((option) => option === value) ?? fallback;
}

function boundedText(value: unknown, maxLength: number, fallback: string): string {
  return typeof value === "string" ? value.slice(0, maxLength) : fallback;
}

function languageTag(value: unknown, fallback: string): string {
  return typeof value === "string" ? (canonicalLanguageTag(value) ?? fallback) : fallback;
}

function glossaryEntries(value: unknown): GlossaryEntry[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value
    .filter(isRecord)
    .filter((entry) => typeof entry.id === "string" && entry.id !== "")
    .slice(0, MAX_GLOSSARY_ENTRIES)
    .map((entry) => ({
      id: entry.id as string,
      term: boundedText(entry.term, MAX_SHORT_TEXT_LENGTH, ""),
      preferred: boundedText(entry.preferred, MAX_SHORT_TEXT_LENGTH, ""),
    }));
}

/**
 * Builds valid settings from untrusted input (for example stored JSON).
 * Any missing, malformed or oversized field falls back to its default.
 */
export function parseSettings(input: unknown): Settings {
  if (!isRecord(input)) {
    return DEFAULT_SETTINGS;
  }
  const defaults = DEFAULT_SETTINGS;
  return {
    theme: oneOf(THEMES, input.theme, defaults.theme),
    uiLocale: oneOf(UI_LOCALES, input.uiLocale, defaults.uiLocale),
    rtlPreview: input.rtlPreview === true,
    sourceLanguage:
      input.sourceLanguage === AUTO_LANGUAGE
        ? AUTO_LANGUAGE
        : languageTag(input.sourceLanguage, defaults.sourceLanguage),
    targetLanguage: languageTag(input.targetLanguage, defaults.targetLanguage),
    replyLanguage: languageTag(input.replyLanguage, defaults.replyLanguage),
    arabicVariety: oneOf(ARABIC_VARIETIES, input.arabicVariety, defaults.arabicVariety),
    arabicDialectLabel: boundedText(input.arabicDialectLabel, MAX_SHORT_TEXT_LENGTH, ""),
    tashkeel: oneOf(TASHKEEL_LEVELS, input.tashkeel, defaults.tashkeel),
    businessContext: boundedText(input.businessContext, MAX_BUSINESS_CONTEXT_LENGTH, ""),
    glossary: glossaryEntries(input.glossary),
    transcriptionEnabled: input.transcriptionEnabled === true,
    translationEnabled: input.translationEnabled === true,
    repliesEnabled: input.repliesEnabled === true,
    speechEnabled: input.speechEnabled === true,
    speakTranslations: input.speakTranslations === true,
    speechProvider: oneOf(SPEECH_PROVIDERS, input.speechProvider, defaults.speechProvider),
    elevenlabsVoice: boundedText(input.elevenlabsVoice, MAX_ENDPOINT_LENGTH, ""),
    elevenlabsModel: boundedText(
      input.elevenlabsModel,
      MAX_ENDPOINT_LENGTH,
      defaults.elevenlabsModel,
    ),
    speechBaseUrl: boundedText(input.speechBaseUrl, MAX_ENDPOINT_LENGTH, defaults.speechBaseUrl),
    speechModel: boundedText(input.speechModel, MAX_ENDPOINT_LENGTH, ""),
    speechVoice: boundedText(input.speechVoice, MAX_ENDPOINT_LENGTH, ""),
    localOnly: input.localOnly === true,
    transcriptionProvider: oneOf(
      TRANSCRIPTION_PROVIDERS,
      input.transcriptionProvider,
      defaults.transcriptionProvider,
    ),
    whisperModel: boundedText(input.whisperModel, MAX_ENDPOINT_LENGTH, ""),
    translationProvider: oneOf(
      LLM_PROVIDERS,
      input.translationProvider,
      defaults.translationProvider,
    ),
    repliesProvider: oneOf(LLM_PROVIDERS, input.repliesProvider, defaults.repliesProvider),
    ollamaEndpoint: boundedText(input.ollamaEndpoint, MAX_ENDPOINT_LENGTH, defaults.ollamaEndpoint),
    ollamaModel: boundedText(input.ollamaModel, MAX_ENDPOINT_LENGTH, ""),
    customBaseUrl: boundedText(input.customBaseUrl, MAX_ENDPOINT_LENGTH, defaults.customBaseUrl),
    customModel: boundedText(input.customModel, MAX_ENDPOINT_LENGTH, ""),
  };
}
