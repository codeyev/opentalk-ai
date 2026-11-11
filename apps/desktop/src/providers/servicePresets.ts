/**
 * Well-known services that offer an OpenAI-compatible API. A preset only
 * fills in the address; any other compatible service can be entered by
 * hand. The model names are examples from each service's documentation,
 * shown as a hint, and may go out of date.
 */
export interface ServicePreset {
  name: string;
  baseUrl: string;
  /** An example model name, shown as a hint in the model field. */
  modelExample: string;
}

export const SERVICE_PRESETS: readonly ServicePreset[] = [
  {
    name: "OpenRouter",
    baseUrl: "https://openrouter.ai/api/v1",
    modelExample: "openai/gpt-4o-mini",
  },
  { name: "OpenAI", baseUrl: "https://api.openai.com/v1", modelExample: "gpt-4o-mini" },
  {
    name: "Google Gemini",
    baseUrl: "https://generativelanguage.googleapis.com/v1beta/openai",
    modelExample: "gemini-3.8-flash",
  },
  {
    name: "Anthropic Claude",
    baseUrl: "https://api.anthropic.com/v1",
    modelExample: "claude-haiku-5-5",
  },
  { name: "DeepSeek", baseUrl: "https://api.deepseek.com", modelExample: "deepseek-flash" },
  { name: "Groq", baseUrl: "https://api.groq.com/openai/v1", modelExample: "openai/gpt-oss-20b" },
];

/** The preset whose address is `baseUrl`, ignoring a trailing slash. */
export function presetFor(baseUrl: string): ServicePreset | undefined {
  const normalized = baseUrl.trim().replace(/\/+$/, "");
  return SERVICE_PRESETS.find((preset) => preset.baseUrl === normalized);
}
