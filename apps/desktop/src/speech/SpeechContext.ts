import { createContext, useContext } from "react";
import type { SpeechChoice, SpeechErrorCode } from "../providers/providerClient";
import type { SpeechSynthesizer } from "./speechSynthesizer";

export interface SpeechOutput {
  /**
   * The voice chosen in the settings, or `null` when it cannot be used:
   * the system offers no speech output, or local-only mode rules it out.
   */
  voice: SpeechSynthesizer | null;
  /** The speech provider behind the voice, or `null` for the system's voices. */
  choice: SpeechChoice | null;
  /** Why the provider last failed to speak, or `null` if it has not since it last spoke. */
  failure: SpeechErrorCode | null;
}

export const SpeechContext = createContext<SpeechOutput | null>(null);

/** The app's one speech output, shared so that only one text is spoken at a time. */
export function useSpeechOutput(): SpeechOutput {
  const output = useContext(SpeechContext);
  if (output === null) {
    throw new Error("useSpeechOutput must be used inside a SpeechProvider.");
  }
  return output;
}
