import type { MessageKey } from "../i18n/messages/en";
import type { CaptureError } from "./captureClient";

/** Picks the message that explains a capture or transcription error to the user. */
export function captureErrorMessageKey(error: CaptureError): MessageKey {
  const source = error.audioSource ?? "microphone";
  switch (error.code) {
    case "permission_denied":
      return `live.error.permission_denied.${source}`;
    case "device_unavailable":
      return `live.error.device_unavailable.${source}`;
    default:
      return `live.error.${error.code}`;
  }
}
