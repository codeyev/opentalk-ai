import { useEffect, useRef } from "react";
import { useCaptureSession } from "../capture/CaptureContext";
import { useSettings } from "../settings/SettingsContext";
import { useSpeechOutput } from "./SpeechContext";

/**
 * Reads the translation of what the other side says aloud, as each one
 * arrives, while the user has "speak translations" on and capture is
 * running. Renders nothing.
 *
 * Only system audio is spoken: that is the other side of the
 * conversation. The user's own words are never read back to them.
 */
export function SpokenTranslations() {
  const { speakTranslations } = useSettings().settings;
  const { segments, translations, state } = useCaptureSession();
  const { voice } = useSpeechOutput();
  const isOn = speakTranslations && voice !== null && state === "listening";
  /** Segments already dealt with, so that each is spoken at most once. */
  const handled = useRef(new Set<string>());

  useEffect(() => {
    if (segments.length === 0) {
      handled.current.clear();
    }
    for (const segment of segments) {
      const translation = translations[segment.id];
      if (
        segment.source !== "system" ||
        translation?.status !== "done" ||
        handled.current.has(segment.id)
      ) {
        continue;
      }
      // Marked even while off, so turning the setting on later does not
      // read out everything said so far.
      handled.current.add(segment.id);
      if (isOn) {
        voice.speakNext(translation.text, translation.targetLanguage);
      }
    }
  }, [segments, translations, isOn, voice]);

  // Stopping capture, or turning the setting off, ends the reading at once.
  const wasOn = useRef(false);
  useEffect(() => {
    if (wasOn.current && !isOn) {
      voice?.stop();
    }
    wasOn.current = isOn;
  }, [isOn, voice]);

  return null;
}
